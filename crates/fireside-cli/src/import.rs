//! Markdown → protocol JSON import (`fireside import`), per ADR-006
//! (`.claude/adrs/adr-006-markdown-import.md`).
//!
//! Pure parsing: [`import`] performs no file I/O — `main.rs`'s
//! `Command::Import` handler owns reading the input and writing the
//! output. This keeps the conversion logic unit-testable directly against
//! an in-memory `&str`.

use std::fmt;
use std::ops::Range;

use fireside_core::{
    BranchOption, BranchPoint, ContentBlock, Graph, Node, Traversal, TraversalSpec,
};
use fireside_engine::{Diagnostic, Severity, validate};
use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::slugify;

/// Used when neither frontmatter nor the graph otherwise specifies a
/// protocol version.
const CURRENT_PROTOCOL_VERSION: &str = "0.1.0";

/// Why an import was refused. Every variant carries enough location
/// information for the presenter to find and fix the problem in their
/// source Markdown.
#[derive(Debug)]
pub enum ImportError {
    /// The document has no `##` headings at all.
    NoHeadings,
    /// A nested (multi-level) list was found.
    NestedList {
        /// 1-based line number of the nested item.
        line: usize,
    },
    /// A branch option's link didn't resolve to any node.
    UnresolvedBranchTarget {
        /// 1-based line number of the offending link.
        line: usize,
        /// The unresolved `#slug`.
        target: String,
        /// The section (node) the branch fence belongs to.
        section: String,
    },
    /// Content appeared after a `branch` fence within the same section.
    ContentAfterBranch {
        /// 1-based line number of the misplaced content.
        line: usize,
        /// The section (node) it appeared in.
        section: String,
    },
    /// A line inside a `branch` fence didn't parse as a prompt or an
    /// option.
    MalformedBranchLine {
        /// 1-based line number of the offending line.
        line: usize,
        /// The section (node) the branch fence belongs to.
        section: String,
    },
    /// The generated deck failed Layer-2 semantic validation.
    ValidationFailed(Vec<Diagnostic>),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoHeadings => write!(
                f,
                "no ## headings found — at least one is required to produce a deck"
            ),
            Self::NestedList { line } => write!(
                f,
                "line {line}: nested lists aren't supported by v1 Markdown import — flatten this list, or hand-edit the generated JSON afterward"
            ),
            Self::UnresolvedBranchTarget {
                line,
                target,
                section,
            } => write!(
                f,
                "line {line} (in \"{section}\"): branch link points to \"#{target}\", which doesn't match any ## heading in the document"
            ),
            Self::ContentAfterBranch { line, section } => write!(
                f,
                "line {line} (in \"{section}\"): content found after the branch declaration — a branch fence must be the last thing in its section"
            ),
            Self::MalformedBranchLine { line, section } => write!(
                f,
                "line {line} (in \"{section}\"): couldn't parse this as a branch option — expected `- [label](#target)`"
            ),
            Self::ValidationFailed(diags) => {
                writeln!(f, "the generated deck failed validation:")?;
                for (i, d) in diags.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "  {d}")?;
                }
                Ok(())
            }
        }
    }
}

/// Deck-level metadata parsed from optional leading frontmatter.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct Frontmatter {
    title: Option<String>,
    author: Option<String>,
    date: Option<String>,
    description: Option<String>,
    fireside_version: Option<String>,
}

/// Splits an optional `---`-delimited frontmatter block off the front of
/// `source`, hand-parsing flat `key: value` lines (no YAML crate needed —
/// research.md §4). Returns the parsed frontmatter (if any) and the
/// remaining source with the frontmatter block excluded.
fn split_frontmatter(source: &str) -> (Option<Frontmatter>, &str) {
    let Some(rest) = source.strip_prefix("---\n") else {
        return (None, source);
    };
    let Some(end) = rest.find("\n---") else {
        return (None, source);
    };
    let body = &rest[..end];
    let after_marker = &rest[end + 4..];
    let remaining = after_marker
        .strip_prefix('\n')
        .unwrap_or(after_marker.trim_start_matches(['\r', '\n']));

    let mut fm = Frontmatter::default();
    for line in body.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        match key.trim() {
            "title" => fm.title = Some(value.to_owned()),
            "author" => fm.author = Some(value.to_owned()),
            "date" => fm.date = Some(value.to_owned()),
            "description" => fm.description = Some(value.to_owned()),
            "fireside-version" | "fireside_version" => fm.fireside_version = Some(value.to_owned()),
            _ => {}
        }
    }
    (Some(fm), remaining)
}

/// The `#` (H1) heading text before the first `##`, if any — used as a
/// fallback deck title only when frontmatter didn't supply one (FR-007).
fn leading_h1(source: &str) -> Option<String> {
    let mut text = String::new();
    let mut in_h1 = false;
    for event in Parser::new_ext(source, Options::empty()) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1,
                ..
            }) => in_h1 = true,
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => break,
            Event::End(TagEnd::Heading(HeadingLevel::H1)) if in_h1 => {
                let trimmed = text.trim();
                return (!trimmed.is_empty()).then(|| trimmed.to_owned());
            }
            Event::Text(t) | Event::Code(t) if in_h1 => text.push_str(&t),
            Event::SoftBreak if in_h1 => text.push(' '),
            _ => {}
        }
    }
    None
}

/// 1-based line number containing byte offset `pos` in `source`.
fn line_at(source: &str, pos: usize) -> usize {
    source[..pos.min(source.len())].matches('\n').count() + 1
}

fn is_h2_start(event: &Event<'_>) -> bool {
    matches!(
        event,
        Event::Start(Tag::Heading {
            level: HeadingLevel::H2,
            ..
        })
    )
}

/// Given `events[i]` is a `Start(tag)`, returns the index just past its
/// matching `End`, correctly skipping arbitrarily nested children via a
/// depth stack — the shared primitive every other walker in this module
/// builds on.
fn skip_element(events: &[(Event<'_>, Range<usize>)], i: usize) -> usize {
    let Event::Start(tag) = &events[i].0 else {
        return i + 1;
    };
    let mut stack = vec![tag.to_end()];
    let mut j = i + 1;
    while j < events.len() && !stack.is_empty() {
        match &events[j].0 {
            Event::Start(t) => stack.push(t.to_end()),
            // pulldown-cmark guarantees balanced Start/End events, so the
            // top of the stack always matches; pop unconditionally rather
            // than asserting, so a future non-fatal mismatch can't panic.
            Event::End(_) => {
                stack.pop();
            }
            _ => {}
        }
        j += 1;
    }
    j
}

/// Concatenates `Text`/`Code` events (soft breaks become spaces, hard
/// breaks become newlines) between `events[i]` (a `Start`) and its
/// matching `End`, returning the text and the index just past that `End`.
/// Used for headings and list items, where the marker/prefix must not
/// leak into the extracted text (research.md §3).
fn concat_inner_text(events: &[(Event<'_>, Range<usize>)], i: usize) -> (String, usize) {
    let end = skip_element(events, i);
    let mut text = String::new();
    for (event, _) in &events[i + 1..end.saturating_sub(1)] {
        match event {
            Event::Text(t) | Event::Code(t) => text.push_str(t),
            Event::SoftBreak => text.push(' '),
            Event::HardBreak => text.push('\n'),
            _ => {}
        }
    }
    (text, end)
}

/// If the paragraph starting at `events[i]` contains exactly one image and
/// nothing else (the common "standalone image on its own line" shape),
/// returns the `Image` content block; otherwise `None`. Either way,
/// returns the index just past the paragraph's `End`.
fn try_paragraph_as_image(
    events: &[(Event<'_>, Range<usize>)],
    i: usize,
) -> (Option<ContentBlock>, usize) {
    let end = skip_element(events, i);
    let inner_start = i + 1;
    let inner_end = end.saturating_sub(1);
    if inner_end > inner_start
        && let Event::Start(Tag::Image {
            dest_url, title, ..
        }) = &events[inner_start].0
    {
        let src = dest_url.to_string();
        let caption = (!title.is_empty()).then(|| title.to_string());
        let mut alt = String::new();
        let mut k = inner_start + 1;
        while k < inner_end {
            match &events[k].0 {
                Event::Text(t) => alt.push_str(t),
                Event::End(TagEnd::Image) => {
                    k += 1;
                    break;
                }
                _ => return (None, end),
            }
            k += 1;
        }
        if k == inner_end {
            return (
                Some(ContentBlock::Image {
                    reveal: None,
                    src,
                    alt: (!alt.is_empty()).then_some(alt),
                    caption,
                    width: None,
                    height: None,
                }),
                end,
            );
        }
    }
    (None, end)
}

/// Walks a `List` starting at `events[i]`, returning its items (source
/// text per item, trimmed) and the index just past the list's `End`.
/// Detects nesting: a `List` found inside an `Item` is rejected (FR-012)
/// rather than silently flattened.
fn collect_list_items(
    events: &[(Event<'_>, Range<usize>)],
    i: usize,
    source: &str,
) -> Result<(Vec<String>, usize), ImportError> {
    let list_end = skip_element(events, i);
    let mut items = Vec::new();
    let mut j = i + 1;
    while j + 1 < list_end {
        if matches!(events[j].0, Event::Start(Tag::Item)) {
            let item_end = skip_element(events, j);
            for (event, range) in &events[j + 1..item_end.saturating_sub(1)] {
                if matches!(event, Event::Start(Tag::List(_))) {
                    return Err(ImportError::NestedList {
                        line: line_at(source, range.start),
                    });
                }
            }
            let (text, _) = concat_inner_text(events, j);
            items.push(text.trim().to_owned());
            j = item_end;
        } else {
            j += 1;
        }
    }
    Ok((items, list_end))
}

/// Intermediate form of a `- [label](#target)` \`key\` line, before its
/// target is resolved against known node ids.
struct BranchOptionSource {
    label: String,
    target_slug: String,
    key: Option<String>,
    line: usize,
}

/// Parses a `branch` fence's raw body into a prompt and ordered option
/// list (research.md §5): the first non-list line is the prompt, every
/// other non-blank line must be `- [label](#target)` with an optional
/// trailing `` `key` ``.
fn parse_branch_body(
    body: &str,
    fence_line: usize,
    section: &str,
) -> Result<(Option<String>, Vec<BranchOptionSource>), ImportError> {
    let mut prompt = None;
    let mut options = Vec::new();
    let mut first = true;
    for (offset, line) in body.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let line_no = fence_line + 1 + offset;
        if first && !trimmed.starts_with('-') {
            prompt = Some(trimmed.to_owned());
            first = false;
            continue;
        }
        first = false;
        options.push(parse_branch_option_line(trimmed, line_no, section)?);
    }
    if options.is_empty() {
        return Err(ImportError::MalformedBranchLine {
            line: fence_line,
            section: section.to_owned(),
        });
    }
    Ok((prompt, options))
}

fn parse_branch_option_line(
    line: &str,
    line_no: usize,
    section: &str,
) -> Result<BranchOptionSource, ImportError> {
    let malformed = || ImportError::MalformedBranchLine {
        line: line_no,
        section: section.to_owned(),
    };
    let rest = line.strip_prefix('-').ok_or_else(malformed)?.trim_start();
    let rest = rest.strip_prefix('[').ok_or_else(malformed)?;
    let (label, rest) = rest.split_once(']').ok_or_else(malformed)?;
    let rest = rest.strip_prefix('(').ok_or_else(malformed)?;
    let rest = rest.strip_prefix('#').ok_or_else(malformed)?;
    let (target, rest) = rest.split_once(')').ok_or_else(malformed)?;
    let key_part = rest.trim();
    let key = if key_part.is_empty() {
        None
    } else {
        let key = key_part
            .strip_prefix('`')
            .and_then(|s| s.strip_suffix('`'))
            .ok_or_else(malformed)?;
        Some(key.to_owned())
    };
    if label.is_empty() || target.is_empty() {
        return Err(malformed());
    }
    Ok(BranchOptionSource {
        label: label.to_owned(),
        target_slug: target.to_owned(),
        key,
        line: line_no,
    })
}

/// Resolves a parsed branch declaration's option targets against the
/// document's known node ids (built in the id-collection pass, so forward
/// references to later sections already work).
fn resolve_branch(
    prompt: Option<String>,
    sources: Vec<BranchOptionSource>,
    node_ids: &[(String, String)],
    section: &str,
) -> Result<BranchPoint, ImportError> {
    let mut options = Vec::with_capacity(sources.len());
    for src in sources {
        let target = node_ids
            .iter()
            .find(|(_, id)| *id == src.target_slug)
            .map(|(_, id)| id.clone())
            .ok_or_else(|| ImportError::UnresolvedBranchTarget {
                line: src.line,
                target: src.target_slug.clone(),
                section: section.to_owned(),
            })?;
        options.push(BranchOption {
            label: src.label,
            key: src.key,
            target,
            description: None,
        });
    }
    Ok(BranchPoint { prompt, options })
}

/// One `##`-delimited region of the source document, converted.
struct Section {
    heading_text: String,
    id: String,
    blocks: Vec<ContentBlock>,
    branch: Option<BranchPoint>,
}

/// First pass: walks every `##` heading in document order, slugifying and
/// deduplicating ids (FR-004, FR-005). Node ids from this pass are known
/// before any section's content is built, which is what lets a branch
/// fence reference a node appearing later in the document.
fn collect_node_ids(source: &str) -> Result<Vec<(String, String)>, ImportError> {
    let mut ids: Vec<(String, String)> = Vec::new();
    let mut in_h2 = false;
    let mut text = String::new();
    for event in Parser::new_ext(source, Options::empty()) {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                in_h2 = true;
                text.clear();
            }
            Event::End(TagEnd::Heading(HeadingLevel::H2)) => {
                in_h2 = false;
                let heading_text = text.trim().to_owned();
                let base = slugify(&heading_text);
                let id = unique_id(&base, &ids);
                ids.push((heading_text, id));
            }
            Event::Text(t) | Event::Code(t) if in_h2 => text.push_str(&t),
            Event::SoftBreak if in_h2 => text.push(' '),
            _ => {}
        }
    }
    if ids.is_empty() {
        return Err(ImportError::NoHeadings);
    }
    Ok(ids)
}

/// Appends `-2`, `-3`, ... to `base` until it no longer collides with an
/// existing id (FR-005).
fn unique_id(base: &str, existing: &[(String, String)]) -> String {
    if !existing.iter().any(|(_, id)| id == base) {
        return base.to_owned();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !existing.iter().any(|(_, id)| id == &candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// Second pass: builds each section's content blocks and resolves its
/// branch declaration (if any), using the ids `collect_node_ids` already
/// found.
fn parse_sections(source: &str, node_ids: &[(String, String)]) -> Result<Vec<Section>, ImportError> {
    let events: Vec<(Event<'_>, Range<usize>)> =
        Parser::new_ext(source, Options::empty()).into_offset_iter().collect();
    let mut sections = Vec::new();
    let mut node_index = 0usize;
    let mut i = 0usize;

    while i < events.len() && !is_h2_start(&events[i].0) {
        i += 1;
    }

    while i < events.len() {
        // Consume the H2 heading itself — its text/id already came from
        // collect_node_ids.
        i = skip_element(&events, i);
        let (heading_text, id) = node_ids[node_index].clone();
        node_index += 1;

        let mut blocks = Vec::new();
        let mut branch: Option<BranchPoint> = None;
        let mut branch_seen_at: Option<usize> = None;

        while i < events.len() && !is_h2_start(&events[i].0) {
            let (event, range) = &events[i];
            let start = range.start;
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let level_u8 = *level as u8;
                    let (text, next_i) = concat_inner_text(&events, i);
                    i = next_i;
                    if let Some(line) = branch_seen_at {
                        return Err(ImportError::ContentAfterBranch {
                            line,
                            section: heading_text,
                        });
                    }
                    blocks.push(ContentBlock::Heading {
                        reveal: None,
                        level: level_u8,
                        text: text.trim().to_owned(),
                    });
                }
                Event::Start(Tag::Paragraph) => {
                    let (image, next_i) = try_paragraph_as_image(&events, i);
                    if let Some(block) = image {
                        i = next_i;
                        if let Some(line) = branch_seen_at {
                            return Err(ImportError::ContentAfterBranch {
                                line,
                                section: heading_text,
                            });
                        }
                        blocks.push(block);
                        continue;
                    }
                    let text = source[range.clone()].trim().to_owned();
                    i = skip_element(&events, i);
                    if let Some(line) = branch_seen_at {
                        return Err(ImportError::ContentAfterBranch {
                            line,
                            section: heading_text,
                        });
                    }
                    blocks.push(ContentBlock::Text { reveal: None, body: text });
                }
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(info))) => {
                    let lang = info.to_string();
                    let fence_line = line_at(source, start);
                    let (body, next_i) = concat_inner_text(&events, i);
                    i = next_i;
                    if lang == "branch" {
                        let (prompt, sources) =
                            parse_branch_body(&body, fence_line, &heading_text)?;
                        branch = Some(resolve_branch(prompt, sources, node_ids, &heading_text)?);
                        branch_seen_at = Some(fence_line);
                        continue;
                    }
                    if let Some(line) = branch_seen_at {
                        return Err(ImportError::ContentAfterBranch {
                            line,
                            section: heading_text,
                        });
                    }
                    blocks.push(ContentBlock::Code {
                        reveal: None,
                        language: (!lang.is_empty()).then_some(lang),
                        source: body,
                        highlight_lines: None,
                        show_line_numbers: None,
                    });
                }
                Event::Start(Tag::List(start_num)) => {
                    let ordered = start_num.is_some();
                    let (items, next_i) = collect_list_items(&events, i, source)?;
                    i = next_i;
                    if let Some(line) = branch_seen_at {
                        return Err(ImportError::ContentAfterBranch {
                            line,
                            section: heading_text,
                        });
                    }
                    blocks.push(ContentBlock::List {
                        reveal: None,
                        ordered: Some(ordered),
                        items,
                    });
                }
                Event::Rule => {
                    i += 1;
                    if let Some(line) = branch_seen_at {
                        return Err(ImportError::ContentAfterBranch {
                            line,
                            section: heading_text,
                        });
                    }
                    blocks.push(ContentBlock::Divider { reveal: None });
                }
                _ => i += 1,
            }
        }

        sections.push(Section {
            heading_text,
            id,
            blocks,
            branch,
        });
    }

    Ok(sections)
}

/// Assembles the final `Graph`: frontmatter metadata plus one `Node` per
/// section, wired with linear (FR-020) or branch traversal.
fn build_graph(frontmatter: Frontmatter, sections: Vec<Section>) -> Graph {
    let ids: Vec<String> = sections.iter().map(|s| s.id.clone()).collect();
    let nodes = sections
        .into_iter()
        .enumerate()
        .map(|(idx, section)| {
            let traversal = match section.branch {
                Some(branch_point) => Some(TraversalSpec::Rules(Traversal {
                    next: None,
                    branch_point: Some(branch_point),
                })),
                None => ids.get(idx + 1).map(|next| TraversalSpec::Target(next.clone())),
            };
            Node {
                id: section.id,
                title: Some(section.heading_text),
                view_mode: None,
                transition: None,
                speaker_notes: None,
                traversal,
                content: section.blocks,
            }
        })
        .collect();

    Graph {
        fireside_version: Some(
            frontmatter
                .fireside_version
                .unwrap_or_else(|| CURRENT_PROTOCOL_VERSION.to_owned()),
        ),
        title: frontmatter.title,
        author: frontmatter.author,
        date: frontmatter.date,
        description: frontmatter.description,
        version: None,
        defaults: None,
        nodes,
    }
}

/// Parses `source` (Markdown) into a validated [`Graph`], or a specific,
/// located [`ImportError`]. Performs no file I/O.
///
/// # Errors
///
/// Returns [`ImportError`] for every case named in
/// `specs/003-markdown-import/contracts/cli-import.md`'s exit-behavior
/// table (no `##` headings, a nested list, an unresolved or malformed
/// branch fence, or a generated deck that fails validation).
#[must_use = "an import that isn't written anywhere was pointless"]
pub fn import(source: &str) -> Result<Graph, ImportError> {
    let (frontmatter, body) = split_frontmatter(source);
    let node_ids = collect_node_ids(body)?;
    let sections = parse_sections(body, &node_ids)?;

    let mut frontmatter = frontmatter.unwrap_or_default();
    if frontmatter.title.is_none() {
        frontmatter.title = leading_h1(body);
    }

    let graph = build_graph(frontmatter, sections);

    let errors: Vec<Diagnostic> = validate(&graph)
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    if !errors.is_empty() {
        return Err(ImportError::ValidationFailed(errors));
    }
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_extracts_title_and_author() {
        let src = "---\ntitle: My Talk\nauthor: Ada Lovelace\n---\n\n## Welcome\n\nHi.\n";
        let (fm, rest) = split_frontmatter(src);
        let fm = fm.expect("frontmatter present");
        assert_eq!(fm.title.as_deref(), Some("My Talk"));
        assert_eq!(fm.author.as_deref(), Some("Ada Lovelace"));
        assert!(rest.trim_start().starts_with("## Welcome"), "{rest:?}");
    }

    #[test]
    fn split_frontmatter_absent_returns_full_source_unchanged() {
        let src = "## Welcome\n\nHi.\n";
        let (fm, rest) = split_frontmatter(src);
        assert!(fm.is_none());
        assert_eq!(rest, src);
    }

    #[test]
    fn split_frontmatter_ignores_unrecognized_keys() {
        let src = "---\ntitle: My Talk\nunknown: whatever\n---\n\n## Welcome\n";
        let (fm, _) = split_frontmatter(src);
        let fm = fm.expect("frontmatter present");
        assert_eq!(fm.title.as_deref(), Some("My Talk"));
    }

    #[test]
    fn collect_node_ids_orders_and_dedupes() {
        let src = "## Welcome\n\n## The Code\n\n## Welcome\n";
        let ids = collect_node_ids(src).expect("three headings");
        assert_eq!(
            ids,
            vec![
                ("Welcome".to_owned(), "welcome".to_owned()),
                ("The Code".to_owned(), "the-code".to_owned()),
                ("Welcome".to_owned(), "welcome-2".to_owned()),
            ]
        );
    }

    #[test]
    fn collect_node_ids_requires_at_least_one_h2() {
        let err = collect_node_ids("# Just an H1\n\nNo sections here.\n")
            .expect_err("no ## headings");
        assert!(matches!(err, ImportError::NoHeadings));
    }

    const LINEAR: &str = "---\ntitle: My Talk\nauthor: Ada Lovelace\n---\n\n# My Talk\n\n## Welcome\n\nThanks for coming. Here's what we'll cover.\n\n- Point one\n- Point two\n\n## The Code\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\n## Thanks\n\nQuestions?\n";

    #[test]
    fn import_linear_deck_has_three_nodes_in_order_with_linear_traversal() {
        let graph = import(LINEAR).expect("linear deck imports cleanly");
        assert_eq!(graph.title.as_deref(), Some("My Talk"));
        assert_eq!(graph.author.as_deref(), Some("Ada Lovelace"));
        let ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
        assert_eq!(ids, vec!["welcome", "the-code", "thanks"]);

        assert_eq!(graph.nodes[0].next_target(), Some("the-code"));
        assert_eq!(graph.nodes[1].next_target(), Some("thanks"));
        assert!(graph.nodes[2].traversal.is_none(), "last node is terminal");

        match &graph.nodes[0].content[1] {
            ContentBlock::List { ordered, items, .. } => {
                assert_eq!(*ordered, Some(false));
                assert_eq!(items, &["Point one".to_owned(), "Point two".to_owned()]);
            }
            other => panic!("expected a list block, got {other:?}"),
        }
        match &graph.nodes[1].content[0] {
            ContentBlock::Code { language, source, .. } => {
                assert_eq!(language.as_deref(), Some("rust"));
                assert!(source.contains("println!"));
            }
            other => panic!("expected a code block, got {other:?}"),
        }
    }

    #[test]
    fn import_falls_back_to_h1_title_when_no_frontmatter_title() {
        let src = "# Fallback Title\n\n## Only\n\nHi.\n";
        let graph = import(src).expect("imports cleanly");
        assert_eq!(graph.title.as_deref(), Some("Fallback Title"));
    }

    #[test]
    fn import_rejects_a_document_with_no_h2_headings() {
        let err = import("Just a paragraph, no headings.\n").expect_err("no headings");
        assert!(matches!(err, ImportError::NoHeadings));
    }

    const BRANCHING: &str = "## Choose your path\n\n```branch\nWhat would you like to see?\n- [Explore the features](#core-features) `f`\n- [Watch a demo](#code-demo) `d`\n```\n\n## Core Features\n\nSome features.\n\n## Code Demo\n\n```rust\nfn demo() {}\n```\n";

    #[test]
    fn import_branching_deck_resolves_forward_references() {
        let graph = import(BRANCHING).expect("branching deck imports cleanly");
        let choose = graph.node("choose-your-path").expect("branch node");
        let bp = choose.branch_point().expect("branch point");
        assert_eq!(bp.prompt.as_deref(), Some("What would you like to see?"));
        assert_eq!(bp.options.len(), 2);
        assert_eq!(bp.options[0].label, "Explore the features");
        assert_eq!(bp.options[0].target, "core-features");
        assert_eq!(bp.options[1].key.as_deref(), Some("d"));
        assert_eq!(bp.options[1].target, "code-demo");
    }

    #[test]
    fn import_rejects_an_unresolved_branch_target() {
        let src = BRANCHING.replace("#code-demo", "#nonexistent");
        let err = import(&src).expect_err("unresolved target");
        match err {
            ImportError::UnresolvedBranchTarget { target, .. } => {
                assert_eq!(target, "nonexistent");
            }
            other => panic!("expected UnresolvedBranchTarget, got {other:?}"),
        }
    }

    #[test]
    fn import_rejects_content_after_a_branch_fence() {
        let src = "## Choose\n\n```branch\n- [A](#a)\n```\n\nMore text after the fence.\n\n## A\n\nHi.\n";
        let err = import(src).expect_err("content after branch fence");
        assert!(matches!(err, ImportError::ContentAfterBranch { .. }));
    }

    #[test]
    fn import_rejects_a_malformed_branch_line() {
        let src = "## Choose\n\n```branch\nnot a link at all\n```\n\n## Elsewhere\n\nHi.\n";
        let err = import(src).expect_err("malformed branch line");
        assert!(matches!(err, ImportError::MalformedBranchLine { .. }));
    }

    #[test]
    fn import_rejects_a_nested_list() {
        let src = "## Slide\n\n- Top item\n  - Nested item\n";
        let err = import(src).expect_err("nested list");
        assert!(matches!(err, ImportError::NestedList { .. }));
    }

    #[test]
    fn import_converts_a_standalone_image_and_a_divider() {
        let src = "## Slide\n\n![a diagram](diagram.png \"A caption\")\n\n---\n\nAfter the divider.\n";
        let graph = import(src).expect("imports cleanly");
        let blocks = &graph.nodes[0].content;
        match &blocks[0] {
            ContentBlock::Image { src, alt, caption, .. } => {
                assert_eq!(src, "diagram.png");
                assert_eq!(alt.as_deref(), Some("a diagram"));
                assert_eq!(caption.as_deref(), Some("A caption"));
            }
            other => panic!("expected an image block, got {other:?}"),
        }
        assert!(matches!(blocks[1], ContentBlock::Divider { .. }));
        assert!(matches!(blocks[2], ContentBlock::Text { .. }));
    }
}
