//! Authoring transforms (spec `013-authoring-editor`, ADR-018).
//!
//! Pure `(Graph, Op) -> Result<Graph, AuthoringError>` construction, not
//! detection: every function here is written so its *result*, if `Ok`,
//! can never contain a dangling `next`/target reference, a duplicated
//! node id, a node with both `next` and `branch_point` set, or a gapped
//! reveal-step sequence (spec FR-023, `SC-007`). `fireside-tui::editor`
//! is the only intended caller; nothing here touches rendering, I/O, or
//! `App`/`EditorApp` state (Constitution Principle III).
//!
//! See `specs/013-authoring-editor/contracts/authoring-ops.md` for the
//! full per-operation contract this module implements.

use std::collections::HashSet;

use fireside_core::{
    BranchOption, BranchPoint, ContainerLayout, ContentBlock, Graph, Node, Traversal, TraversalSpec,
};
use thiserror::Error;

/// Addresses a block within a node's (possibly nested, via `Container`)
/// content tree by index path. For `Op::AddBlock` the path addresses the
/// *parent* container (empty = the node's top-level content) and `at` is
/// the insertion index within it; for every other block op the path
/// addresses the block itself (its last element is its index within its
/// immediate parent).
pub type BlockPath = Vec<usize>;

/// The eight authoring-facing block kinds (spec FR-006), used by
/// [`Op::AddBlock`] to pick a placeholder [`ContentBlock`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Heading,
    Text,
    Code,
    List,
    Image,
    Divider,
    Container,
    AsciiArt,
}

/// One authoring operation. See
/// `specs/013-authoring-editor/contracts/authoring-ops.md` for the full
/// precondition/postcondition table.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    AddSlide {
        after: String,
        title: String,
    },
    DeleteSlide {
        id: String,
    },
    DuplicateSlide {
        id: String,
    },
    RetitleSlide {
        id: String,
        title: String,
    },
    /// Move `id` so it immediately precedes `before` in the `next` chain
    /// (`before: None` moves it to the end of its run). Only supported
    /// within one unbranched linear run — see
    /// [`AuthoringError::CrossesBranchBoundary`].
    ReorderSlide {
        id: String,
        before: Option<String>,
    },
    SetNext {
        id: String,
        target: String,
    },
    ClearNext {
        id: String,
    },
    TurnIntoChoice {
        id: String,
        prompt: Option<String>,
        first_label: String,
        first_target: String,
    },
    /// Keeps the first answer's target, per spec.
    TurnBackIntoSlide {
        id: String,
    },
    AddAnswer {
        id: String,
        label: String,
        key: Option<String>,
        target: String,
    },
    RemoveAnswer {
        id: String,
        index: usize,
    },
    RetargetAnswer {
        id: String,
        index: usize,
        target: String,
    },
    AddBlock {
        node: String,
        path: BlockPath,
        kind: BlockKind,
        at: usize,
    },
    DeleteBlock {
        node: String,
        path: BlockPath,
    },
    /// Replaces the block at `path` with `content`, preserving the
    /// existing block's `reveal` value (reveal is only ever changed by
    /// [`Op::SetRevealStep`]).
    EditBlock {
        node: String,
        path: BlockPath,
        content: ContentBlock,
    },
    MoveBlock {
        node: String,
        path: BlockPath,
        to: usize,
    },
    SetRevealStep {
        node: String,
        path: BlockPath,
        step: Option<u32>,
    },
}

/// Every precondition failure an [`Op`] can hit. Each variant carries
/// enough context (ids, an index, a character) for a caller to build a
/// plain-language toast — none of this `Display` text is meant to reach
/// the audience-facing editor UI verbatim (spec FR-024 governs that
/// layer, not this one).
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AuthoringError {
    #[error("no slide with id \"{0}\"")]
    UnknownSlide(String),
    #[error("\"{0}\" is already used by another slide")]
    DuplicateId(String),
    #[error("the first slide can't be deleted")]
    CannotDeleteEntry,
    #[error("\"{0}\" is reached only through a branch answer — change the answer's target instead")]
    CrossesBranchBoundary(String),
    #[error("'{0}' is reserved for a presenter key and can't be used as a branch key")]
    ReservedBranchKey(char),
    #[error("\"{0}\" is not a branch point")]
    NotABranchPoint(String),
    #[error("\"{0}\" is already a branch point")]
    AlreadyABranchPoint(String),
    #[error("a branch point needs at least one answer")]
    LastAnswer,
    #[error("no answer at position {0} on \"{1}\"")]
    UnknownAnswer(usize, String),
    #[error("no block at that position on \"{0}\"")]
    UnknownBlock(String),
    #[error("that position doesn't exist on \"{0}\"")]
    InvalidPath(String),
    #[error("the graph has no slides")]
    EmptyGraph,
}

/// Applies `op` to `graph`, returning a new [`Graph`] on success. `graph`
/// itself is never mutated — on `Err`, nothing changed.
///
/// # Errors
///
/// Returns the specific [`AuthoringError`] variant for whichever
/// precondition `op` violated; see
/// `specs/013-authoring-editor/contracts/authoring-ops.md`.
pub fn apply(graph: &Graph, op: &Op) -> Result<Graph, AuthoringError> {
    let mut next = graph.clone();
    match op {
        Op::AddSlide { after, title } => add_slide(&mut next, after, title)?,
        Op::DeleteSlide { id } => delete_slide(&mut next, id)?,
        Op::DuplicateSlide { id } => duplicate_slide(&mut next, id)?,
        Op::RetitleSlide { id, title } => retitle_slide(&mut next, id, title)?,
        Op::ReorderSlide { id, before } => reorder_slide(&mut next, id, before.as_deref())?,
        Op::SetNext { id, target } => set_next(&mut next, id, target)?,
        Op::ClearNext { id } => clear_next(&mut next, id)?,
        Op::TurnIntoChoice {
            id,
            prompt,
            first_label,
            first_target,
        } => turn_into_choice(&mut next, id, prompt.clone(), first_label, first_target)?,
        Op::TurnBackIntoSlide { id } => turn_back_into_slide(&mut next, id)?,
        Op::AddAnswer {
            id,
            label,
            key,
            target,
        } => add_answer(&mut next, id, label, key.as_deref(), target)?,
        Op::RemoveAnswer { id, index } => remove_answer(&mut next, id, *index)?,
        Op::RetargetAnswer { id, index, target } => {
            retarget_answer(&mut next, id, *index, target)?;
        }
        Op::AddBlock {
            node,
            path,
            kind,
            at,
        } => add_block(&mut next, node, path, *kind, *at)?,
        Op::DeleteBlock { node, path } => delete_block(&mut next, node, path)?,
        Op::EditBlock {
            node,
            path,
            content,
        } => edit_block(&mut next, node, path, content.clone())?,
        Op::MoveBlock { node, path, to } => move_block(&mut next, node, path, *to)?,
        Op::SetRevealStep { node, path, step } => set_reveal_step(&mut next, node, path, *step)?,
    }
    Ok(next)
}

// ─── Id / slug algorithm ───────────────────────────────────────────────────

/// Derives a unique node id from `title`: lowercase, every run of
/// non-alphanumeric characters becomes a single `-`, leading/trailing `-`
/// trimmed, an empty result falls back to `"slide"`, then deduped against
/// `existing` with `-2`, `-3`, … suffixes.
#[must_use]
pub fn slug(title: &str, existing: &[String]) -> String {
    let mut out = String::new();
    let mut last_was_dash = true; // suppresses a leading dash
    for ch in title.chars() {
        if ch.is_alphanumeric() {
            out.extend(ch.to_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            out.push('-');
            last_was_dash = true;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    if out.is_empty() {
        out.push_str("slide");
    }
    dedupe(&out, existing)
}

fn dedupe(base: &str, existing: &[String]) -> String {
    let seen: HashSet<&str> = existing.iter().map(String::as_str).collect();
    if !seen.contains(base) {
        return base.to_owned();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{base}-{n}");
        if !seen.contains(candidate.as_str()) {
            return candidate;
        }
        n += 1;
    }
}

// ─── Slide ops ──────────────────────────────────────────────────────────────

fn node_index(nodes: &[Node], id: &str) -> Result<usize, AuthoringError> {
    nodes
        .iter()
        .position(|n| n.id == id)
        .ok_or_else(|| AuthoringError::UnknownSlide(id.to_owned()))
}

fn add_slide(graph: &mut Graph, after: &str, title: &str) -> Result<(), AuthoringError> {
    let after_idx = node_index(&graph.nodes, after)?;
    let existing: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let new_id = slug(title, &existing);
    let wire_as_next = graph.nodes[after_idx].traversal.is_none();
    let new_node = Node {
        id: new_id.clone(),
        title: Some(title.to_owned()),
        view_mode: None,
        transition: None,
        speaker_notes: None,
        traversal: None,
        content: Vec::new(),
    };
    graph.nodes.insert(after_idx + 1, new_node);
    if wire_as_next {
        graph.nodes[after_idx].traversal = Some(TraversalSpec::Rules(Traversal {
            next: Some(new_id),
            branch_point: None,
        }));
    }
    Ok(())
}

fn delete_slide(graph: &mut Graph, id: &str) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    if idx == 0 {
        return Err(AuthoringError::CannotDeleteEntry);
    }
    let replacement = graph.nodes[idx].next_target().map(str::to_owned);
    graph.nodes.remove(idx);
    for node in &mut graph.nodes {
        let mut clear = false;
        match &mut node.traversal {
            Some(TraversalSpec::Target(t)) if t == id => {
                if let Some(r) = &replacement {
                    *t = r.clone();
                } else {
                    clear = true;
                }
            }
            Some(TraversalSpec::Rules(rules)) => {
                if rules.next.as_deref() == Some(id) {
                    rules.next = replacement.clone();
                }
                if let Some(bp) = &mut rules.branch_point {
                    bp.options.retain(|o| o.target != id);
                    if bp.options.is_empty() {
                        rules.branch_point = None;
                    }
                }
                if rules.next.is_none() && rules.branch_point.is_none() {
                    clear = true;
                }
            }
            _ => {}
        }
        if clear {
            node.traversal = None;
        }
    }
    Ok(())
}

fn duplicate_slide(graph: &mut Graph, id: &str) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    let existing: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let title = graph.nodes[idx].title.clone().unwrap_or_default();
    let new_id = slug(&title, &existing);
    let mut clone = graph.nodes[idx].clone();
    clone.id = new_id;
    clone.traversal = None;
    graph.nodes.insert(idx + 1, clone);
    Ok(())
}

fn retitle_slide(graph: &mut Graph, id: &str, title: &str) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    let others: Vec<String> = graph
        .nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != idx)
        .map(|(_, n)| n.id.clone())
        .collect();
    let new_id = slug(title, &others);
    graph.nodes[idx].title = Some(title.to_owned());
    if new_id != id {
        graph.nodes[idx].id = new_id.clone();
        for node in &mut graph.nodes {
            match &mut node.traversal {
                Some(TraversalSpec::Target(t)) if t == id => *t = new_id.clone(),
                Some(TraversalSpec::Rules(rules)) => {
                    if rules.next.as_deref() == Some(id) {
                        rules.next = Some(new_id.clone());
                    }
                    if let Some(bp) = &mut rules.branch_point {
                        for opt in &mut bp.options {
                            if opt.target == id {
                                opt.target = new_id.clone();
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

/// The node whose plain `next` edge points at `id`, if any — `id`'s
/// linear-run predecessor. `None` covers both "no predecessor" (id is the
/// entry node) and "id is not reached via a plain `next` edge at all"
/// (reached via a branch option instead, or unreachable); callers that
/// need to distinguish those cases check `id == graph.nodes[0].id`
/// themselves.
fn next_predecessor<'a>(nodes: &'a [Node], id: &str) -> Option<&'a Node> {
    nodes.iter().find(|n| n.next_target() == Some(id))
}

fn is_branch_target(nodes: &[Node], id: &str) -> bool {
    nodes.iter().any(|n| {
        n.branch_point()
            .is_some_and(|bp| bp.options.iter().any(|o| o.target == id))
    })
}

fn reorder_slide(graph: &mut Graph, id: &str, before: Option<&str>) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    if is_branch_target(&graph.nodes, id) {
        return Err(AuthoringError::CrossesBranchBoundary(id.to_owned()));
    }
    let is_entry = idx == 0;
    if !is_entry && next_predecessor(&graph.nodes, id).is_none() {
        // Unreachable node: no linear run to reorder within.
        return Err(AuthoringError::CrossesBranchBoundary(id.to_owned()));
    }
    if let Some(before_id) = before {
        node_index(&graph.nodes, before_id)?;
        if is_branch_target(&graph.nodes, before_id) {
            return Err(AuthoringError::CrossesBranchBoundary(before_id.to_owned()));
        }
        if before_id == graph.nodes[0].id {
            return Err(AuthoringError::CrossesBranchBoundary(before_id.to_owned()));
        }
    }

    // Heal the gap id's removal from the chain leaves behind.
    let id_next = graph.nodes[idx].next_target().map(str::to_owned);
    if !is_entry {
        let pred_id = next_predecessor(&graph.nodes, id).map(|n| n.id.clone());
        if let Some(pred_id) = pred_id {
            let pred_idx = node_index(&graph.nodes, &pred_id)?;
            set_next_field(&mut graph.nodes[pred_idx], id_next.clone());
        }
    }

    match before {
        Some(before_id) => {
            let new_pred_id = next_predecessor(&graph.nodes, before_id).map(|n| n.id.clone());
            if let Some(new_pred_id) = new_pred_id {
                let new_pred_idx = node_index(&graph.nodes, &new_pred_id)?;
                set_next_field(&mut graph.nodes[new_pred_idx], Some(id.to_owned()));
            }
            let id_idx = node_index(&graph.nodes, id)?;
            set_next_field(&mut graph.nodes[id_idx], Some(before_id.to_owned()));
        }
        None => {
            // Move to the end of the run: id becomes the new terminal,
            // taking over whatever the run's current last node's
            // "onward" state was is out of scope for this contract —
            // callers pass an explicit `before` for every non-terminal
            // placement; `None` simply detaches id from the chain,
            // leaving it as its own ending, ready to be rewired.
            let id_idx = node_index(&graph.nodes, id)?;
            set_next_field(&mut graph.nodes[id_idx], None);
        }
    }

    // Physically move `id` in `nodes` right before `before` (or to the
    // end) so declaration order tracks the new linear order too.
    let id_pos = node_index(&graph.nodes, id)?;
    let node = graph.nodes.remove(id_pos);
    match before {
        Some(before_id) => {
            let insert_at = node_index(&graph.nodes, before_id)?;
            graph.nodes.insert(insert_at, node);
        }
        None => graph.nodes.push(node),
    }
    Ok(())
}

fn set_next_field(node: &mut Node, target: Option<String>) {
    match target {
        Some(t) => {
            node.traversal = Some(TraversalSpec::Rules(Traversal {
                next: Some(t),
                branch_point: None,
            }));
        }
        None => node.traversal = None,
    }
}

fn set_next(graph: &mut Graph, id: &str, target: &str) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    node_index(&graph.nodes, target)?;
    if graph.nodes[idx].branch_point().is_some() {
        return Err(AuthoringError::AlreadyABranchPoint(id.to_owned()));
    }
    set_next_field(&mut graph.nodes[idx], Some(target.to_owned()));
    Ok(())
}

fn clear_next(graph: &mut Graph, id: &str) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    graph.nodes[idx].traversal = None;
    Ok(())
}

fn turn_into_choice(
    graph: &mut Graph,
    id: &str,
    prompt: Option<String>,
    first_label: &str,
    first_target: &str,
) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    node_index(&graph.nodes, first_target)?;
    graph.nodes[idx].traversal = Some(TraversalSpec::Rules(Traversal {
        next: None,
        branch_point: Some(BranchPoint {
            prompt,
            options: vec![BranchOption {
                label: first_label.to_owned(),
                key: None,
                target: first_target.to_owned(),
                description: None,
            }],
        }),
    }));
    Ok(())
}

fn turn_back_into_slide(graph: &mut Graph, id: &str) -> Result<(), AuthoringError> {
    let idx = node_index(&graph.nodes, id)?;
    let bp = graph.nodes[idx]
        .branch_point()
        .ok_or_else(|| AuthoringError::NotABranchPoint(id.to_owned()))?;
    let first_target = bp.options[0].target.clone();
    set_next_field(&mut graph.nodes[idx], Some(first_target));
    Ok(())
}

fn branch_point_mut<'a>(
    nodes: &'a mut [Node],
    id: &str,
) -> Result<&'a mut BranchPoint, AuthoringError> {
    let idx = node_index(nodes, id)?;
    match &mut nodes[idx].traversal {
        Some(TraversalSpec::Rules(Traversal {
            branch_point: Some(bp),
            ..
        })) => Ok(bp),
        _ => Err(AuthoringError::NotABranchPoint(id.to_owned())),
    }
}

fn add_answer(
    graph: &mut Graph,
    id: &str,
    label: &str,
    key: Option<&str>,
    target: &str,
) -> Result<(), AuthoringError> {
    node_index(&graph.nodes, target)?;
    if let Some(k) = key
        && let Some(c) = k.chars().next()
        && crate::validation::RESERVED_PRESENTER_KEYS.contains(&c)
    {
        return Err(AuthoringError::ReservedBranchKey(c));
    }
    let bp = branch_point_mut(&mut graph.nodes, id)?;
    bp.options.push(BranchOption {
        label: label.to_owned(),
        key: key.map(str::to_owned),
        target: target.to_owned(),
        description: None,
    });
    Ok(())
}

fn remove_answer(graph: &mut Graph, id: &str, index: usize) -> Result<(), AuthoringError> {
    let bp = branch_point_mut(&mut graph.nodes, id)?;
    if index >= bp.options.len() {
        return Err(AuthoringError::UnknownAnswer(index, id.to_owned()));
    }
    if bp.options.len() == 1 {
        return Err(AuthoringError::LastAnswer);
    }
    bp.options.remove(index);
    Ok(())
}

fn retarget_answer(
    graph: &mut Graph,
    id: &str,
    index: usize,
    target: &str,
) -> Result<(), AuthoringError> {
    node_index(&graph.nodes, target)?;
    let bp = branch_point_mut(&mut graph.nodes, id)?;
    let opt = bp
        .options
        .get_mut(index)
        .ok_or_else(|| AuthoringError::UnknownAnswer(index, id.to_owned()))?;
    opt.target = target.to_owned();
    Ok(())
}

// ─── Block ops ──────────────────────────────────────────────────────────────

fn placeholder(kind: BlockKind) -> ContentBlock {
    match kind {
        BlockKind::Heading => ContentBlock::Heading {
            reveal: None,
            level: 2,
            text: "New heading".to_owned(),
        },
        BlockKind::Text => ContentBlock::Text {
            reveal: None,
            body: "New text".to_owned(),
        },
        BlockKind::Code => ContentBlock::Code {
            reveal: None,
            language: None,
            source: String::new(),
            highlight_lines: None,
            show_line_numbers: None,
        },
        BlockKind::List => ContentBlock::List {
            reveal: None,
            ordered: None,
            items: vec!["New item".to_owned()],
        },
        BlockKind::Image => ContentBlock::Image {
            reveal: None,
            src: String::new(),
            alt: None,
            caption: None,
            width: None,
            height: None,
        },
        BlockKind::Divider => ContentBlock::Divider { reveal: None },
        BlockKind::Container => ContentBlock::Container {
            reveal: None,
            children: Vec::new(),
            layout: Some(ContainerLayout::Stack),
        },
        BlockKind::AsciiArt => ContentBlock::AsciiArt {
            reveal: None,
            art: String::new(),
            alt: None,
        },
    }
}

fn children_mut<'a>(
    content: &'a mut Vec<ContentBlock>,
    path: &[usize],
) -> Option<&'a mut Vec<ContentBlock>> {
    match path.split_first() {
        None => Some(content),
        Some((&i, rest)) => match content.get_mut(i) {
            Some(ContentBlock::Container { children, .. }) => children_mut(children, rest),
            _ => None,
        },
    }
}

fn node_content_mut<'a>(
    nodes: &'a mut [Node],
    node_id: &str,
) -> Result<&'a mut Vec<ContentBlock>, AuthoringError> {
    let idx = node_index(nodes, node_id)?;
    Ok(&mut nodes[idx].content)
}

fn add_block(
    graph: &mut Graph,
    node: &str,
    parent_path: &[usize],
    kind: BlockKind,
    at: usize,
) -> Result<(), AuthoringError> {
    let content = node_content_mut(&mut graph.nodes, node)?;
    let parent = children_mut(content, parent_path)
        .ok_or_else(|| AuthoringError::InvalidPath(node.to_owned()))?;
    if at > parent.len() {
        return Err(AuthoringError::InvalidPath(node.to_owned()));
    }
    parent.insert(at, placeholder(kind));
    Ok(())
}

fn split_block_path(path: &[usize]) -> Result<(&[usize], usize), AuthoringError> {
    match path.split_last() {
        Some((&last, parent)) => Ok((parent, last)),
        None => Err(AuthoringError::InvalidPath(String::new())),
    }
}

fn delete_block(graph: &mut Graph, node: &str, path: &[usize]) -> Result<(), AuthoringError> {
    let (parent_path, index) =
        split_block_path(path).map_err(|_| AuthoringError::UnknownBlock(node.to_owned()))?;
    let content = node_content_mut(&mut graph.nodes, node)?;
    let parent = children_mut(content, parent_path)
        .ok_or_else(|| AuthoringError::InvalidPath(node.to_owned()))?;
    if index >= parent.len() {
        return Err(AuthoringError::UnknownBlock(node.to_owned()));
    }
    parent.remove(index);
    Ok(())
}

fn edit_block(
    graph: &mut Graph,
    node: &str,
    path: &[usize],
    content: ContentBlock,
) -> Result<(), AuthoringError> {
    let (parent_path, index) =
        split_block_path(path).map_err(|_| AuthoringError::UnknownBlock(node.to_owned()))?;
    let node_content = node_content_mut(&mut graph.nodes, node)?;
    let parent = children_mut(node_content, parent_path)
        .ok_or_else(|| AuthoringError::InvalidPath(node.to_owned()))?;
    let existing = parent
        .get_mut(index)
        .ok_or_else(|| AuthoringError::UnknownBlock(node.to_owned()))?;
    let preserved_reveal = existing.reveal();
    let mut replacement = content;
    set_reveal(&mut replacement, preserved_reveal);
    *existing = replacement;
    Ok(())
}

fn move_block(
    graph: &mut Graph,
    node: &str,
    path: &[usize],
    to: usize,
) -> Result<(), AuthoringError> {
    let (parent_path, index) =
        split_block_path(path).map_err(|_| AuthoringError::UnknownBlock(node.to_owned()))?;
    let content = node_content_mut(&mut graph.nodes, node)?;
    let parent = children_mut(content, parent_path)
        .ok_or_else(|| AuthoringError::InvalidPath(node.to_owned()))?;
    if index >= parent.len() || to >= parent.len() {
        return Err(AuthoringError::UnknownBlock(node.to_owned()));
    }
    let block = parent.remove(index);
    parent.insert(to, block);
    Ok(())
}

fn set_reveal(block: &mut ContentBlock, value: Option<u32>) {
    match block {
        ContentBlock::Heading { reveal, .. }
        | ContentBlock::Text { reveal, .. }
        | ContentBlock::Code { reveal, .. }
        | ContentBlock::List { reveal, .. }
        | ContentBlock::Image { reveal, .. }
        | ContentBlock::Divider { reveal }
        | ContentBlock::AsciiArt { reveal, .. }
        | ContentBlock::Container { reveal, .. } => *reveal = value,
    }
}

fn collect_positive_reveals(content: &[ContentBlock], out: &mut Vec<u32>) {
    for block in content {
        if let Some(v) = block.reveal()
            && v > 0
        {
            out.push(v);
        }
        if let ContentBlock::Container { children, .. } = block {
            collect_positive_reveals(children, out);
        }
    }
}

fn remap_reveals(content: &mut [ContentBlock], mapping: &std::collections::HashMap<u32, u32>) {
    for block in content {
        if let Some(v) = block.reveal()
            && v > 0
            && let Some(&mapped) = mapping.get(&v)
        {
            set_reveal(block, Some(mapped));
        }
        if let ContentBlock::Container { children, .. } = block {
            remap_reveals(children, mapping);
        }
    }
}

fn set_reveal_step(
    graph: &mut Graph,
    node: &str,
    path: &[usize],
    step: Option<u32>,
) -> Result<(), AuthoringError> {
    let (parent_path, index) =
        split_block_path(path).map_err(|_| AuthoringError::UnknownBlock(node.to_owned()))?;
    let content = node_content_mut(&mut graph.nodes, node)?;
    let parent = children_mut(content, parent_path)
        .ok_or_else(|| AuthoringError::InvalidPath(node.to_owned()))?;
    let block = parent
        .get_mut(index)
        .ok_or_else(|| AuthoringError::UnknownBlock(node.to_owned()))?;
    set_reveal(block, step);

    let idx = node_index(&graph.nodes, node)?;
    let mut distinct = Vec::new();
    collect_positive_reveals(&graph.nodes[idx].content, &mut distinct);
    distinct.sort_unstable();
    distinct.dedup();
    let mapping: std::collections::HashMap<u32, u32> = distinct
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, u32::try_from(i + 1).unwrap_or(u32::MAX)))
        .collect();
    remap_reveals(&mut graph.nodes[idx].content, &mapping);
    Ok(())
}

// ─── Outline ordering ───────────────────────────────────────────────────────

/// One row of the editor's outline: a slide's id, its 1-based display
/// position, and whether it's reachable from the graph's entry node.
/// `display_number` is a display coordinate, recomputed after every
/// structural op — never an identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineRow {
    pub node_id: String,
    pub display_number: usize,
    pub reachable: bool,
}

/// Deterministic outline order: depth-first from `graph.entry()`,
/// following `next` before branch options in declared order, each node
/// appearing once at its first visit (cycles terminate for free); nodes
/// never visited that way are appended after the reachable ones, in
/// declaration (`graph.nodes`) order. See `research.md` §8 for why this
/// is implemented fresh here rather than shared with the map screen.
#[must_use]
pub fn outline_order(graph: &Graph) -> Vec<OutlineRow> {
    let Some(entry) = graph.entry() else {
        return Vec::new();
    };
    let mut visited: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut stack = vec![entry.id.clone()];
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        visited.push(id.clone());
        let Some(node) = graph.node(&id) else {
            continue;
        };
        let mut to_push = Vec::new();
        if let Some(bp) = node.branch_point() {
            for opt in bp.options.iter().rev() {
                to_push.push(opt.target.clone());
            }
        } else if let Some(next) = node.next_target() {
            to_push.push(next.to_owned());
        }
        // Depth-first, `next`/first option before later ones: since this
        // is a stack, push in reverse so the first-declared target pops
        // first. Branch options are already reversed above; a plain
        // `next` is a single element so order is moot there.
        for id in to_push {
            stack.push(id);
        }
    }
    let mut rows: Vec<OutlineRow> = visited
        .iter()
        .enumerate()
        .map(|(i, id)| OutlineRow {
            node_id: id.clone(),
            display_number: i + 1,
            reachable: true,
        })
        .collect();
    let next_number = rows.len() + 1;
    let mut unreachable_count = 0;
    for node in &graph.nodes {
        if !seen.contains(&node.id) {
            rows.push(OutlineRow {
                node_id: node.id.clone(),
                display_number: next_number + unreachable_count,
                reachable: false,
            });
            unreachable_count += 1;
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;
    use fireside_core::{ContentBlock as CB, Graph, Node};

    fn node(id: &str) -> Node {
        Node {
            id: id.to_owned(),
            title: Some(id.to_owned()),
            view_mode: None,
            transition: None,
            speaker_notes: None,
            traversal: None,
            content: Vec::new(),
        }
    }

    fn linked(id: &str, next: &str) -> Node {
        let mut n = node(id);
        n.traversal = Some(TraversalSpec::Rules(Traversal {
            next: Some(next.to_owned()),
            branch_point: None,
        }));
        n
    }

    fn graph_of(nodes: Vec<Node>) -> Graph {
        Graph {
            fireside_version: None,
            title: None,
            author: None,
            date: None,
            description: None,
            version: None,
            defaults: None,
            nodes,
        }
    }

    // ── slug ──

    #[test]
    fn slug_lowercases_and_dashes() {
        assert_eq!(slug("Pick A Path!", &[]), "pick-a-path");
    }

    #[test]
    fn slug_falls_back_on_empty() {
        assert_eq!(slug("???", &[]), "slide");
    }

    #[test]
    fn slug_dedupes() {
        let existing = vec!["welcome".to_owned(), "welcome-2".to_owned()];
        assert_eq!(slug("Welcome", &existing), "welcome-3");
    }

    // ── AddSlide ──

    #[test]
    fn add_slide_wires_as_next_when_predecessor_was_terminal() {
        let g = graph_of(vec![node("a")]);
        let g2 = apply(
            &g,
            &Op::AddSlide {
                after: "a".into(),
                title: "New One".into(),
            },
        )
        .unwrap();
        assert_eq!(g2.nodes.len(), 2);
        assert_eq!(g2.nodes[0].next_target(), Some("new-one"));
    }

    #[test]
    fn add_slide_leaves_new_node_unwired_when_predecessor_already_has_next() {
        let g = graph_of(vec![linked("a", "b"), node("b")]);
        let g2 = apply(
            &g,
            &Op::AddSlide {
                after: "a".into(),
                title: "C".into(),
            },
        )
        .unwrap();
        assert_eq!(g2.nodes[0].next_target(), Some("b"));
        assert!(g2.node("c").unwrap().is_terminal());
    }

    #[test]
    fn add_slide_unknown_after_errors() {
        let g = graph_of(vec![node("a")]);
        assert_eq!(
            apply(
                &g,
                &Op::AddSlide {
                    after: "zzz".into(),
                    title: "X".into()
                }
            ),
            Err(AuthoringError::UnknownSlide("zzz".into()))
        );
    }

    // ── DeleteSlide ──

    #[test]
    fn delete_slide_heals_next_reference() {
        let g = graph_of(vec![linked("a", "b"), linked("b", "c"), node("c")]);
        let g2 = apply(&g, &Op::DeleteSlide { id: "b".into() }).unwrap();
        assert_eq!(g2.nodes.len(), 2);
        assert_eq!(g2.node("a").unwrap().next_target(), Some("c"));
    }

    #[test]
    fn delete_slide_of_terminal_clears_predecessor_next() {
        let g = graph_of(vec![linked("a", "b"), node("b")]);
        let g2 = apply(&g, &Op::DeleteSlide { id: "b".into() }).unwrap();
        assert!(g2.node("a").unwrap().is_terminal());
    }

    #[test]
    fn delete_slide_removes_dangling_branch_options() {
        let mut a = node("a");
        a.traversal = Some(TraversalSpec::Rules(Traversal {
            next: None,
            branch_point: Some(BranchPoint {
                prompt: None,
                options: vec![
                    BranchOption {
                        label: "B".into(),
                        key: None,
                        target: "b".into(),
                        description: None,
                    },
                    BranchOption {
                        label: "C".into(),
                        key: None,
                        target: "c".into(),
                        description: None,
                    },
                ],
            }),
        }));
        let g = graph_of(vec![a, node("b"), node("c")]);
        let g2 = apply(&g, &Op::DeleteSlide { id: "b".into() }).unwrap();
        let bp = g2.node("a").unwrap().branch_point().unwrap();
        assert_eq!(bp.options.len(), 1);
        assert_eq!(bp.options[0].target, "c");
    }

    #[test]
    fn delete_entry_slide_errors() {
        let g = graph_of(vec![node("a")]);
        assert_eq!(
            apply(&g, &Op::DeleteSlide { id: "a".into() }),
            Err(AuthoringError::CannotDeleteEntry)
        );
    }

    // ── RetitleSlide ──

    #[test]
    fn retitle_rewrites_every_reference_atomically() {
        let mut a = node("a");
        a.traversal = Some(TraversalSpec::Rules(Traversal {
            next: None,
            branch_point: Some(BranchPoint {
                prompt: None,
                options: vec![BranchOption {
                    label: "B".into(),
                    key: None,
                    target: "b".into(),
                    description: None,
                }],
            }),
        }));
        let g = graph_of(vec![a, linked("c", "b"), node("b")]);
        let g2 = apply(
            &g,
            &Op::RetitleSlide {
                id: "b".into(),
                title: "Brand New".into(),
            },
        )
        .unwrap();
        assert!(g2.node("b").is_none());
        let renamed = g2.node("brand-new").unwrap();
        assert_eq!(renamed.title.as_deref(), Some("Brand New"));
        assert_eq!(
            g2.node("a").unwrap().branch_point().unwrap().options[0].target,
            "brand-new"
        );
        assert_eq!(g2.node("c").unwrap().next_target(), Some("brand-new"));
    }

    #[test]
    fn retitle_same_slug_keeps_id() {
        let g = graph_of(vec![node("a")]);
        let g2 = apply(
            &g,
            &Op::RetitleSlide {
                id: "a".into(),
                title: "A".into(),
            },
        )
        .unwrap();
        assert!(g2.node("a").is_some());
    }

    // ── ReorderSlide ──

    #[test]
    fn reorder_slide_within_linear_run() {
        let g = graph_of(vec![linked("a", "b"), linked("b", "c"), node("c")]);
        let g2 = apply(
            &g,
            &Op::ReorderSlide {
                id: "c".into(),
                before: Some("b".into()),
            },
        )
        .unwrap();
        assert_eq!(g2.node("a").unwrap().next_target(), Some("c"));
        assert_eq!(g2.node("c").unwrap().next_target(), Some("b"));
        assert!(g2.node("b").unwrap().is_terminal());
    }

    #[test]
    fn reorder_across_branch_boundary_refuses() {
        let mut a = node("a");
        a.traversal = Some(TraversalSpec::Rules(Traversal {
            next: None,
            branch_point: Some(BranchPoint {
                prompt: None,
                options: vec![
                    BranchOption {
                        label: "B".into(),
                        key: None,
                        target: "b".into(),
                        description: None,
                    },
                    BranchOption {
                        label: "C".into(),
                        key: None,
                        target: "c".into(),
                        description: None,
                    },
                ],
            }),
        }));
        let g = graph_of(vec![a, node("b"), node("c")]);
        assert_eq!(
            apply(
                &g,
                &Op::ReorderSlide {
                    id: "b".into(),
                    before: Some("c".into())
                }
            ),
            Err(AuthoringError::CrossesBranchBoundary("b".into()))
        );
    }

    // ── Choice ops ──

    #[test]
    fn turn_into_choice_then_back_keeps_first_answer_target() {
        let g = graph_of(vec![node("a"), node("b"), node("c")]);
        let g2 = apply(
            &g,
            &Op::TurnIntoChoice {
                id: "a".into(),
                prompt: Some("Pick one".into()),
                first_label: "B".into(),
                first_target: "b".into(),
            },
        )
        .unwrap();
        assert!(g2.node("a").unwrap().branch_point().is_some());
        let g3 = apply(
            &g2,
            &Op::AddAnswer {
                id: "a".into(),
                label: "C".into(),
                key: None,
                target: "c".into(),
            },
        )
        .unwrap();
        assert_eq!(
            g3.node("a").unwrap().branch_point().unwrap().options.len(),
            2
        );
        let g4 = apply(&g3, &Op::TurnBackIntoSlide { id: "a".into() }).unwrap();
        assert_eq!(g4.node("a").unwrap().next_target(), Some("b"));
    }

    #[test]
    fn add_answer_rejects_reserved_key() {
        let g = graph_of(vec![node("a"), node("b")]);
        let g2 = apply(
            &g,
            &Op::TurnIntoChoice {
                id: "a".into(),
                prompt: None,
                first_label: "B".into(),
                first_target: "b".into(),
            },
        )
        .unwrap();
        assert_eq!(
            apply(
                &g2,
                &Op::AddAnswer {
                    id: "a".into(),
                    label: "Quit".into(),
                    key: Some("q".into()),
                    target: "b".into()
                }
            ),
            Err(AuthoringError::ReservedBranchKey('q'))
        );
    }

    #[test]
    fn remove_last_answer_refuses() {
        let g = graph_of(vec![node("a"), node("b")]);
        let g2 = apply(
            &g,
            &Op::TurnIntoChoice {
                id: "a".into(),
                prompt: None,
                first_label: "B".into(),
                first_target: "b".into(),
            },
        )
        .unwrap();
        assert_eq!(
            apply(
                &g2,
                &Op::RemoveAnswer {
                    id: "a".into(),
                    index: 0
                }
            ),
            Err(AuthoringError::LastAnswer)
        );
    }

    // ── Block ops ──

    #[test]
    fn add_block_inserts_placeholder() {
        let g = graph_of(vec![node("a")]);
        let g2 = apply(
            &g,
            &Op::AddBlock {
                node: "a".into(),
                path: vec![],
                kind: BlockKind::Text,
                at: 0,
            },
        )
        .unwrap();
        assert_eq!(g2.node("a").unwrap().content.len(), 1);
    }

    #[test]
    fn edit_block_preserves_reveal() {
        let mut a = node("a");
        a.content.push(CB::Text {
            reveal: Some(1),
            body: "old".into(),
        });
        let g = graph_of(vec![a]);
        let g2 = apply(
            &g,
            &Op::EditBlock {
                node: "a".into(),
                path: vec![0],
                content: CB::Text {
                    reveal: None,
                    body: "new".into(),
                },
            },
        )
        .unwrap();
        assert_eq!(g2.node("a").unwrap().content[0].reveal(), Some(1));
    }

    #[test]
    fn move_block_reorders_siblings() {
        let mut a = node("a");
        a.content.push(CB::Text {
            reveal: None,
            body: "1".into(),
        });
        a.content.push(CB::Text {
            reveal: None,
            body: "2".into(),
        });
        let g = graph_of(vec![a]);
        let g2 = apply(
            &g,
            &Op::MoveBlock {
                node: "a".into(),
                path: vec![0],
                to: 1,
            },
        )
        .unwrap();
        let CB::Text { body, .. } = &g2.node("a").unwrap().content[1] else {
            panic!()
        };
        assert_eq!(body, "1");
    }

    #[test]
    fn set_reveal_step_keeps_steps_consecutive() {
        let mut a = node("a");
        a.content.push(CB::Text {
            reveal: Some(1),
            body: "1".into(),
        });
        a.content.push(CB::Text {
            reveal: Some(3),
            body: "2".into(),
        });
        let g = graph_of(vec![a]);
        let g2 = apply(
            &g,
            &Op::SetRevealStep {
                node: "a".into(),
                path: vec![0],
                step: None,
            },
        )
        .unwrap();
        let node = g2.node("a").unwrap();
        assert_eq!(node.content[0].reveal(), None);
        assert_eq!(node.content[1].reveal(), Some(1));
        assert_eq!(node.reveal_levels(), vec![1]);
    }

    #[test]
    fn delete_block_removes_it() {
        let mut a = node("a");
        a.content.push(CB::Divider { reveal: None });
        let g = graph_of(vec![a]);
        let g2 = apply(
            &g,
            &Op::DeleteBlock {
                node: "a".into(),
                path: vec![0],
            },
        )
        .unwrap();
        assert!(g2.node("a").unwrap().content.is_empty());
    }

    #[test]
    fn block_ops_reach_into_containers() {
        let mut a = node("a");
        a.content.push(CB::Container {
            reveal: None,
            children: vec![],
            layout: None,
        });
        let g = graph_of(vec![a]);
        let g2 = apply(
            &g,
            &Op::AddBlock {
                node: "a".into(),
                path: vec![0],
                kind: BlockKind::Text,
                at: 0,
            },
        )
        .unwrap();
        let CB::Container { children, .. } = &g2.node("a").unwrap().content[0] else {
            panic!()
        };
        assert_eq!(children.len(), 1);
    }

    // ── outline_order ──

    #[test]
    fn outline_order_depth_first_next_before_branch() {
        let mut a = node("a");
        a.traversal = Some(TraversalSpec::Rules(Traversal {
            next: None,
            branch_point: Some(BranchPoint {
                prompt: None,
                options: vec![
                    BranchOption {
                        label: "B".into(),
                        key: None,
                        target: "b".into(),
                        description: None,
                    },
                    BranchOption {
                        label: "C".into(),
                        key: None,
                        target: "c".into(),
                        description: None,
                    },
                ],
            }),
        }));
        let g = graph_of(vec![a, node("b"), node("c")]);
        let rows = outline_order(&g);
        let ids: Vec<&str> = rows.iter().map(|r| r.node_id.as_str()).collect();
        assert_eq!(ids, vec!["a", "b", "c"]);
        assert!(rows.iter().all(|r| r.reachable));
    }

    #[test]
    fn outline_order_terminates_on_cycles() {
        let g = graph_of(vec![linked("a", "b"), linked("b", "a")]);
        let rows = outline_order(&g);
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn outline_order_lists_unreachable_after_reachable() {
        let g = graph_of(vec![node("a"), node("orphan")]);
        let rows = outline_order(&g);
        assert_eq!(rows[0].node_id, "a");
        assert!(rows[0].reachable);
        assert_eq!(rows[1].node_id, "orphan");
        assert!(!rows[1].reachable);
    }

    #[test]
    fn outline_order_empty_graph() {
        let g = graph_of(vec![]);
        assert!(outline_order(&g).is_empty());
    }

    // ── Proptests: the crown-jewel invariants (spec SC-007) ──

    mod proptest_support {
        use super::*;
        use proptest::collection::vec as pvec;
        use proptest::prelude::*;

        #[derive(Debug, Clone)]
        pub(super) enum SmallOp {
            Retitle {
                idx: usize,
                title: String,
            },
            Delete {
                idx: usize,
            },
            Reorder {
                idx: usize,
                before_idx: Option<usize>,
            },
        }

        pub(super) fn arbitrary_linear_graph(n: usize) -> Graph {
            let mut nodes: Vec<Node> = (0..n).map(|i| node(&format!("n{i}"))).collect();
            let count = nodes.len();
            for (i, nd) in nodes.iter_mut().enumerate().take(count.saturating_sub(1)) {
                nd.traversal = Some(TraversalSpec::Rules(Traversal {
                    next: Some(format!("n{}", i + 1)),
                    branch_point: None,
                }));
            }
            graph_of(nodes)
        }

        pub(super) fn arbitrary_small_op(n: usize) -> impl Strategy<Value = SmallOp> {
            prop_oneof![
                (0..n, "[a-zA-Z ]{1,8}").prop_map(|(idx, title)| SmallOp::Retitle { idx, title }),
                (1..n).prop_map(|idx| SmallOp::Delete { idx }),
                (0..n, proptest::option::of(0..n))
                    .prop_map(|(idx, before_idx)| SmallOp::Reorder { idx, before_idx }),
            ]
        }

        pub(super) fn arbitrary_ops(n: usize) -> impl Strategy<Value = Vec<SmallOp>> {
            pvec(arbitrary_small_op(n), 0..10)
        }

        fn no_dangling_reference(graph: &Graph) -> bool {
            let ids: HashSet<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
            graph.nodes.iter().all(|n| {
                let next_ok = n.next_target().is_none_or(|t| ids.contains(t));
                let branch_ok = n
                    .branch_point()
                    .is_none_or(|bp| bp.options.iter().all(|o| ids.contains(o.target.as_str())));
                next_ok && branch_ok
            })
        }

        fn no_duplicate_id(graph: &Graph) -> bool {
            let mut ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();
            let before = ids.len();
            ids.sort_unstable();
            ids.dedup();
            ids.len() == before
        }

        fn no_next_and_branch_conflict(graph: &Graph) -> bool {
            graph
                .nodes
                .iter()
                .all(|n| !(n.next_target().is_some() && n.branch_point().is_some()))
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(200))]

            #[test]
            fn retitle_never_dangles_a_reference(title in "[a-zA-Z ]{0,12}") {
                let g = arbitrary_linear_graph(4);
                if let Ok(g2) = apply(&g, &Op::RetitleSlide { id: "n2".into(), title }) {
                    prop_assert!(no_dangling_reference(&g2));
                    prop_assert!(no_duplicate_id(&g2));
                }
            }

            #[test]
            fn arbitrary_op_sequences_never_violate_invariants(ops in arbitrary_ops(5)) {
                let mut g = arbitrary_linear_graph(5);
                for op in ops {
                    let ids: Vec<String> = g.nodes.iter().map(|n| n.id.clone()).collect();
                    let translated = match op {
                        SmallOp::Retitle { idx, title } => {
                            ids.get(idx).map(|id| Op::RetitleSlide { id: id.clone(), title })
                        }
                        SmallOp::Delete { idx } => ids.get(idx).map(|id| Op::DeleteSlide { id: id.clone() }),
                        SmallOp::Reorder { idx, before_idx } => ids.get(idx).map(|id| Op::ReorderSlide {
                            id: id.clone(),
                            before: before_idx.and_then(|bi| ids.get(bi).cloned()),
                        }),
                    };
                    if let Some(op) = translated
                        && let Ok(next) = apply(&g, &op)
                    {
                        g = next;
                    }
                    prop_assert!(no_dangling_reference(&g));
                    prop_assert!(no_duplicate_id(&g));
                    prop_assert!(no_next_and_branch_conflict(&g));
                }
            }
        }
    }
}
