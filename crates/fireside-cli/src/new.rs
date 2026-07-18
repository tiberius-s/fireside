//! `fireside new`: scaffolds a starter deck, either immediately (given a
//! name) or after asking a few quick questions.

use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use fireside_core::Graph;

use crate::Template;
use crate::slugify;
use crate::templates::{branching_template, linear_template, workshop_template};

pub(crate) fn new_deck(
    name: Option<String>,
    template: Option<Template>,
    author: Option<String>,
) -> Result<()> {
    let (name, template, author) = match name {
        Some(name) => (name, template.unwrap_or(Template::Branching), author),
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

    let json = starter_deck(&name, template, author.as_deref())?
        .to_json_pretty()
        .context("could not serialize the starter deck")?;
    std::fs::write(&path, json + "\n")
        .with_context(|| format!("could not write {}", path.display()))?;

    println!("Created {}.", path.display());
    println!("\nPresent it:   fireside {}", path.display());
    println!("Check it:     fireside validate {}", path.display());
    Ok(())
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

/// Asks the three questions a new deck needs — title, template, author —
/// and returns sensible answers for whichever were skipped. Only reached
/// when `fireside new` is run without a name.
fn interactive_new() -> Result<(String, Template, Option<String>)> {
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

    Ok((name, template, author))
}

fn starter_deck(name: &str, template: Template, author: Option<&str>) -> Result<Graph> {
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
}
