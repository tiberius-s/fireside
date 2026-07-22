//! `fireside new`: scaffolds a starter deck, either immediately (given a
//! name) or after asking a few quick questions.

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use fireside_core::{ContentBlock, Graph};

use crate::Template;
use crate::art::{DEFAULT_ART_WIDTH, render_text_banner};
use crate::slugify;
use crate::templates::{branching_template, linear_template, workshop_template};

pub(crate) fn new_deck(
    name: Option<String>,
    template: Option<Template>,
    author: Option<String>,
    banner: bool,
) -> Result<Option<PathBuf>> {
    let interactive = name.is_none();
    let (name, template, author, banner) = match name {
        Some(name) => (
            name,
            template.unwrap_or(Template::Branching),
            author,
            banner,
        ),
        None => interactive_new()?,
    };

    let slug = slugify(&name);
    if slug.is_empty() {
        bail!("please give the deck a name with at least one letter or digit");
    }
    let path = PathBuf::from(format!("{slug}.fireside.json"));
    if path.exists() {
        bail!("{} already exists — pick another name", path.display());
    }

    let mut graph = starter_deck(&name, template, author.as_deref())?;
    let banner_skipped = banner && !add_title_banner(&mut graph, &name);

    let json = graph
        .to_json_pretty()
        .context("could not serialize the starter deck")?;
    std::fs::write(&path, json + "\n")
        .with_context(|| format!("could not write {}", path.display()))?;

    println!("Created {}.", path.display());
    if banner_skipped {
        println!("Note: the title banner was too wide to fit and was skipped.");
    }
    println!("\nPresent it:   fireside {}", path.display());
    println!("Check it:     fireside validate {}", path.display());

    // Only the interactive wizard offers to launch straight into a
    // rehearsal (FR-010) — `fireside new <name>` stays script-friendly with
    // no extra prompt.
    if !interactive {
        return Ok(None);
    }
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let present_now = match prompt_line(&mut stdin, "\nPresent it now? [Y/n]: ")? {
        None => false,
        Some(s) if s.is_empty() => true,
        Some(s) => matches!(s.to_lowercase().as_str(), "y" | "yes"),
    };
    Ok(present_now.then_some(path))
}

/// Generates a FIGlet banner from `title` and prepends it as an
/// [`ContentBlock::AsciiArt`] block to the first node's content.
/// Returns `false` (a no-op, never an error) when the banner can't be
/// generated at all (e.g. a title with no recognized character) or
/// would exceed [`DEFAULT_ART_WIDTH`] — a starter deck must always
/// validate clean, so a banner that can't be made to fit is silently
/// skipped rather than failing deck creation over a decoration.
fn add_title_banner(graph: &mut Graph, title: &str) -> bool {
    let Ok(art) = render_text_banner(title) else {
        return false;
    };
    let widest = art.lines().map(str::len).max().unwrap_or(0);
    if widest > DEFAULT_ART_WIDTH as usize {
        return false;
    }
    let Some(first) = graph.nodes.first_mut() else {
        return false;
    };
    first.content.insert(
        0,
        ContentBlock::AsciiArt {
            reveal: None,
            art,
            alt: Some(title.to_owned()),
        },
    );
    true
}

/// Reads one line from stdin, printing `label` first as a prompt. `Ok(None)`
/// means stdin hit EOF — callers must stop asking, not loop forever.
fn prompt_line(stdin: &mut impl BufRead, label: &str) -> Result<Option<String>> {
    print!("{label}");
    io::stdout().flush().ok();
    let mut line = String::new();
    let read = stdin.read_line(&mut line).context("could not read stdin")?;
    if read == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim().to_string()))
}

/// Asks the four questions a new deck needs — title, template, author,
/// title banner — and returns sensible answers for whichever were
/// skipped. Only reached when `fireside new` is run without a name.
fn interactive_new() -> Result<(String, Template, Option<String>, bool)> {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();

    let name = loop {
        match prompt_line(&mut stdin, "Deck title: ")? {
            None => bail!("no input received — pass a name directly: fireside new <name>"),
            Some(s) if s.is_empty() => println!("  a title is required."),
            Some(s) => break s,
        }
    };

    println!("\nTemplates:");
    println!("  1) linear     a straight-through talk, no branching");
    println!("  2) branching  a talk with one choice that rejoins (default)");
    println!("  3) workshop   an agenda that jumps into a sequence of exercises");
    let template = loop {
        match prompt_line(&mut stdin, "Pick a template [1-3, default 2]: ")? {
            None => break Template::Branching,
            Some(s) if s.is_empty() => break Template::Branching,
            Some(s) => match s.as_str() {
                "1" | "linear" => break Template::Linear,
                "2" | "branching" => break Template::Branching,
                "3" | "workshop" => break Template::Workshop,
                _ => println!("  please enter 1, 2, or 3."),
            },
        }
    };

    let author = prompt_line(&mut stdin, "Author (optional): ")?.filter(|s| !s.is_empty());

    let banner = matches!(
        prompt_line(&mut stdin, "Add an ASCII title banner? [y/N]: ")?,
        Some(s) if matches!(s.to_lowercase().as_str(), "y" | "yes")
    );

    Ok((name, template, author, banner))
}

/// `pub(crate)`, not private: `edit.rs`'s create-if-missing flow (spec 013,
/// T024) reuses this exact template builder to seed a new deck at an
/// arbitrary target path, rather than `new_deck`'s own path-from-slug
/// scaffolding flow.
pub(crate) fn starter_deck(name: &str, template: Template, author: Option<&str>) -> Result<Graph> {
    let json = match template {
        Template::Linear => linear_template(name),
        Template::Branching => branching_template(name),
        Template::Workshop => workshop_template(name),
    };
    let mut graph: Graph =
        serde_json::from_value(json).context("the starter deck template is broken")?;
    graph.author = author.map(str::to_owned);
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireside_engine::{Severity, validate};

    #[test]
    fn every_starter_template_validates_clean() {
        for template in [Template::Linear, Template::Branching, Template::Workshop] {
            let graph = starter_deck("Test Deck", template, None)
                .unwrap_or_else(|e| panic!("{template:?} template builds: {e}"));
            let diags = validate(&graph);
            let serious: Vec<_> = diags
                .iter()
                .filter(|d| d.severity >= Severity::Warning)
                .collect();
            assert!(
                serious.is_empty(),
                "{template:?} template must be spotless: {serious:?}"
            );
        }
    }

    #[test]
    fn every_starter_template_carries_speaker_note_hints() {
        for template in [Template::Linear, Template::Branching, Template::Workshop] {
            let graph = starter_deck("Test Deck", template, None).expect("template builds");
            assert!(
                graph.nodes.iter().any(|n| n.speaker_notes.is_some()),
                "{template:?} template should hint the author via speaker notes"
            );
        }
    }

    #[test]
    fn starter_deck_embeds_the_given_author() {
        let graph = starter_deck("Test Deck", Template::Branching, None)
            .expect("branching template builds");
        assert_eq!(graph.author, None);

        let graph = starter_deck("Test Deck", Template::Branching, Some("Ada Lovelace"))
            .expect("branching template builds");
        assert_eq!(graph.author.as_deref(), Some("Ada Lovelace"));
    }

    #[test]
    fn add_title_banner_prepends_an_ascii_art_block_and_stays_clean() {
        let mut graph =
            starter_deck("Test Deck", Template::Branching, None).expect("template builds");
        let before = graph.nodes[0].content.len();
        assert!(add_title_banner(&mut graph, "Test Deck"));
        assert_eq!(graph.nodes[0].content.len(), before + 1);
        match &graph.nodes[0].content[0] {
            ContentBlock::AsciiArt { alt, .. } => assert_eq!(alt.as_deref(), Some("Test Deck")),
            other => panic!("expected an ascii-art block, got {other:?}"),
        }
        let diags = validate(&graph);
        let serious: Vec<_> = diags
            .iter()
            .filter(|d| d.severity >= Severity::Warning)
            .collect();
        assert!(
            serious.is_empty(),
            "banner must not break validation: {serious:?}"
        );
    }

    #[test]
    fn add_title_banner_skips_gracefully_when_the_title_is_too_wide() {
        let mut graph =
            starter_deck("Test Deck", Template::Branching, None).expect("template builds");
        let before = graph.nodes[0].content.len();
        let long_title = "A Title So Long It Cannot Possibly Fit The Card";
        assert!(!add_title_banner(&mut graph, long_title));
        assert_eq!(graph.nodes[0].content.len(), before, "no block inserted");
        let diags = validate(&graph);
        assert!(
            diags.iter().all(|d| d.severity < Severity::Warning),
            "skipped banner must leave the deck spotless: {diags:?}"
        );
    }
}
