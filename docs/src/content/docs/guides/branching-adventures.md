---
title: 'Branching Adventures'
description: 'Three patterns for non-linear storytelling in Fireside: Branch-and-Return, Hub-and-Spoke, and Open World.'
---

The best campfire stories aren't monologues — they invite the listener in.
_"What would you do?"_ someone asks, and suddenly the story belongs to
everyone. Fireside makes that moment a first-class concept.

This guide explores three branching patterns you can use to build
non-linear Sessions. Each pattern solves a different storytelling problem.
By the end, you'll know which one to reach for — and how to wire it up in
JSON.

## Why Branching Matters

Linear presentations work when the audience is passive. But the moment you
want **engagement** — a learner choosing their own path through a lesson, a
player deciding what happens next, an employee navigating a simulated
scenario — you need branches.

Branching does three things:

1. **Increases engagement.** The audience has agency. They're not watching —
   they're _choosing_.
2. **Adapts to context.** A security trainer can branch to remediation content
   only when the learner picks the wrong Answer. A teacher can skip the
   basics when the student signals they're ready.
3. **Enables reuse.** One Session can serve multiple audiences by offering
   different paths through the same material.

In Fireside, every branch starts with a **Question** (a `branch-point`) and
offers one or more **Answers** (options with targets). The engine displays the
prompt, waits for a selection, and Flows to the chosen Moment.

## Pattern 1 — Branch and Return

**Use when:** You want to offer a side trip — a deep dive, an example, an
aside — and then bring the audience back to the main thread.

**Real-world example:** A security training Session that shows a phishing
email. The learner chooses "Click the link" or "Report it." Both paths
explain the consequences, then return to the main lesson.

### How It Works

```text
intro → question → [path-a] → summary
                 → [path-b] → summary
```

The Question Moment has a `branch-point`. Each Answer targets a different
Moment. Those branch Moments have `"after": "summary"` in their Flow, which
tells the engine: _"When this Moment ends, go to `summary`."_

### JSON Example

```json
{
  "$schema": "https://fireside.dev/schemas/0.5/Graph.json",
  "title": "Phishing Awareness",
  "nodes": [
    {
      "id": "intro",
      "content": [
        { "kind": "heading", "level": 1, "text": "Spot the Phish" },
        {
          "kind": "text",
          "body": "You receive an email from 'IT Support' asking you to verify your credentials. What do you do?"
        }
      ]
    },
    {
      "id": "question",
      "traversal": {
        "branch-point": {
          "prompt": "What do you do?",
          "options": [
            { "label": "Click the link and log in", "key": "c", "target": "clicked" },
            { "label": "Report it to security", "key": "r", "target": "reported" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 2, "text": "Decision Time" },
        { "kind": "text", "body": "Trust your instincts." }
      ]
    },
    {
      "id": "clicked",
      "traversal": { "after": "summary" },
      "content": [
        { "kind": "heading", "level": 2, "text": "You Clicked the Link" },
        {
          "kind": "text",
          "body": "The page looked real, but it captured your credentials. Attackers now have access to your account."
        },
        {
          "kind": "list",
          "items": [
            "Change your password immediately",
            "Enable multi-factor authentication",
            "Report the incident to your security team"
          ]
        }
      ]
    },
    {
      "id": "reported",
      "traversal": { "after": "summary" },
      "content": [
        { "kind": "heading", "level": 2, "text": "Good Call" },
        {
          "kind": "text",
          "body": "You reported the email. The security team confirmed it was a phishing attempt and blocked the sender."
        },
        { "kind": "text", "body": "Here's what tipped you off:" },
        {
          "kind": "list",
          "items": [
            "Urgent language demanding immediate action",
            "A sender address that didn't match the real IT domain",
            "A link pointing to an external URL"
          ]
        }
      ]
    },
    {
      "id": "summary",
      "layout": "center",
      "content": [
        { "kind": "heading", "level": 2, "text": "Key Takeaways" },
        {
          "kind": "list",
          "ordered": true,
          "items": [
            "Always verify the sender's address",
            "Hover over links before clicking",
            "When in doubt, report it"
          ]
        }
      ]
    }
  ]
}
```

### The `after` Field

The magic of Branch-and-Return lives in the `after` field. When a branch
Moment reaches its end (no more content, no explicit `next` override), the
engine checks `after` and Flows there. Both `clicked` and `reported` set
`"after": "summary"`, so no matter which Answer the learner picks, the
Session converges back to the same place.

The traversal priority is:

```text
traversal.next  →  traversal.after  →  nodes[i+1]  →  Complete
```

`next` is checked first (explicit override), then `after` (branch return),
then the implicit linear sequence.

## Pattern 2 — Hub and Spoke

**Use when:** The audience should explore multiple topics freely, returning
to a central hub Moment after each one. Think of a museum kiosk, a product
tour, or a conference session menu.

**Real-world example:** A museum curator builds a Session for an exhibit.
Visitors can explore any of four topics in any order, always returning to the
exhibit map.

### How It Works

```text
hub → [topic-a] → hub
    → [topic-b] → hub
    → [topic-c] → hub
    → [exit]
```

The hub Moment has a `branch-point` with options for each topic, plus an
"exit" option. Each topic Moment uses `"after": "hub"` to Flow back.

### JSON Example

```json
{
  "$schema": "https://fireside.dev/schemas/0.5/Graph.json",
  "title": "Ancient Egypt Exhibit",
  "nodes": [
    {
      "id": "hub",
      "layout": "center",
      "traversal": {
        "branch-point": {
          "prompt": "What would you like to explore?",
          "options": [
            { "label": "The Pyramids", "key": "p", "target": "pyramids" },
            { "label": "The Nile", "key": "n", "target": "nile" },
            { "label": "Hieroglyphics", "key": "h", "target": "hieroglyphics" },
            { "label": "Exit the exhibit", "key": "x", "target": "exit" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 1, "text": "Ancient Egypt" },
        { "kind": "text", "body": "Choose a topic to explore. You can always come back." }
      ]
    },
    {
      "id": "pyramids",
      "traversal": { "after": "hub" },
      "content": [
        { "kind": "heading", "level": 2, "text": "The Pyramids of Giza" },
        {
          "kind": "text",
          "body": "The Great Pyramid was the tallest structure in the world for over **3,800 years**. It consists of roughly 2.3 million limestone blocks."
        },
        {
          "kind": "image",
          "src": "assets/pyramids.jpg",
          "alt": "The three pyramids of Giza at sunset"
        }
      ]
    },
    {
      "id": "nile",
      "traversal": { "after": "hub" },
      "content": [
        { "kind": "heading", "level": 2, "text": "The River Nile" },
        {
          "kind": "text",
          "body": "At 6,650 km, the Nile is one of the longest rivers in the world. Its annual floods deposited rich silt that made agriculture possible in an otherwise desert landscape."
        },
        {
          "kind": "list",
          "items": [
            "Source: Lake Victoria (White Nile) and Ethiopian Highlands (Blue Nile)",
            "Annual flood season: June through September",
            "The Aswan Dam (1970) ended natural flooding"
          ]
        }
      ]
    },
    {
      "id": "hieroglyphics",
      "traversal": { "after": "hub" },
      "content": [
        { "kind": "heading", "level": 2, "text": "Hieroglyphics" },
        {
          "kind": "text",
          "body": "The ancient Egyptian writing system used over **700 symbols**. The Rosetta Stone, discovered in 1799, was the key to decipherment."
        },
        {
          "kind": "code",
          "language": "text",
          "source": "Rosetta Stone Languages:\n  1. Egyptian hieroglyphics\n  2. Demotic script\n  3. Ancient Greek"
        }
      ]
    },
    {
      "id": "exit",
      "layout": "center",
      "content": [
        { "kind": "heading", "level": 2, "text": "Thank You for Visiting" },
        { "kind": "text", "body": "We hope you enjoyed the exhibit. Come back anytime." }
      ]
    }
  ]
}
```

### Tips for Hub and Spoke

- **Always offer an exit.** Without one, the audience is stuck in the hub
  forever.
- **Keep spokes short.** Each topic should be 1–3 Moments. If a spoke grows
  longer, consider making it a sub-hub.
- **Use speaker notes** (`speaker-notes`) on the hub Moment to remind
  yourself which topics have been covered.

## Pattern 3 — Open World

**Use when:** Choices have lasting consequences. The story doesn't converge —
it diverges. Think tabletop RPGs, choose-your-own-adventure books, or
training simulations where mistakes compound.

**Real-world example:** A tabletop GM builds a one-shot adventure. The party
arrives at a crossroads. Each direction leads to a different encounter, and
the encounters chain into further choices.

### How It Works

```text
crossroads → [cave]   → [dragon]  → [treasure]
                                   → [defeat]
           → [forest] → [hermit]  → [wisdom]
                                   → [lost]
           → [river]  → [bridge]  → [town]
```

No `after` fields. Each Moment's `next` or `branch-point` leads deeper into
the story. The audience commits to their choices.

### JSON Example

```json
{
  "$schema": "https://fireside.dev/schemas/0.5/Graph.json",
  "title": "The Crossroads",
  "author": "The GM",
  "nodes": [
    {
      "id": "crossroads",
      "layout": "center",
      "traversal": {
        "branch-point": {
          "prompt": "Three paths lie before you. Choose wisely.",
          "options": [
            { "label": "Enter the cave", "key": "c", "target": "cave" },
            { "label": "Walk into the forest", "key": "f", "target": "forest" },
            { "label": "Follow the river", "key": "r", "target": "river" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 1, "text": "The Crossroads" },
        {
          "kind": "text",
          "body": "The road splits into three. A sign reads: *'Choose wisely — there is no turning back.'*"
        }
      ]
    },
    {
      "id": "cave",
      "traversal": {
        "branch-point": {
          "prompt": "The dragon stirs. What do you do?",
          "options": [
            { "label": "Fight the dragon", "key": "f", "target": "dragon-fight" },
            { "label": "Sneak past to the treasure", "key": "s", "target": "treasure" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 2, "text": "The Cave" },
        {
          "kind": "text",
          "body": "You step inside. The air is warm and smells of sulfur. In the dim light, you see a pile of gold — and the dragon sleeping atop it."
        }
      ]
    },
    {
      "id": "dragon-fight",
      "content": [
        { "kind": "heading", "level": 2, "text": "Battle!" },
        {
          "kind": "text",
          "body": "The dragon roars and breathes fire. Your armor holds. With a well-placed strike, you fell the beast. **Victory!**"
        },
        {
          "kind": "text",
          "body": "The treasure is yours. Songs will be sung about this day."
        }
      ]
    },
    {
      "id": "treasure",
      "content": [
        { "kind": "heading", "level": 2, "text": "The Treasure" },
        {
          "kind": "text",
          "body": "You move like a shadow. The dragon doesn't stir. You fill your pockets with gold and slip out into the daylight."
        },
        { "kind": "text", "body": "Sometimes the bravest thing is knowing when *not* to fight." }
      ]
    },
    {
      "id": "forest",
      "traversal": { "next": "hermit" },
      "content": [
        { "kind": "heading", "level": 2, "text": "The Forest" },
        {
          "kind": "text",
          "body": "The trees grow dense. Sunlight barely reaches the forest floor. After an hour of walking, you find a small cottage."
        }
      ]
    },
    {
      "id": "hermit",
      "traversal": {
        "branch-point": {
          "prompt": "The hermit offers you a choice.",
          "options": [
            { "label": "Accept the hermit's wisdom", "key": "a", "target": "wisdom" },
            { "label": "Decline and press on alone", "key": "d", "target": "lost" }
          ]
        }
      },
      "content": [
        { "kind": "heading", "level": 2, "text": "The Hermit" },
        {
          "kind": "text",
          "body": "An old figure sits by the fire. *'I can show you the way,'* they say, *'but only if you're willing to listen.'*"
        }
      ]
    },
    {
      "id": "wisdom",
      "content": [
        { "kind": "heading", "level": 2, "text": "Wisdom Gained" },
        {
          "kind": "text",
          "body": "The hermit shares ancient knowledge. You leave the forest changed — wiser, calmer, and ready for whatever comes next."
        }
      ]
    },
    {
      "id": "lost",
      "content": [
        { "kind": "heading", "level": 2, "text": "Lost in the Woods" },
        {
          "kind": "text",
          "body": "You wander for days. Eventually you find your way out, but the journey has cost you. Sometimes pride is the most dangerous enemy."
        }
      ]
    },
    {
      "id": "river",
      "traversal": { "next": "bridge" },
      "content": [
        { "kind": "heading", "level": 2, "text": "The River" },
        {
          "kind": "text",
          "body": "The water is clear and fast. You follow it downstream until you reach a weathered stone bridge."
        }
      ]
    },
    {
      "id": "bridge",
      "traversal": { "next": "town" },
      "content": [
        { "kind": "heading", "level": 2, "text": "The Bridge" },
        {
          "kind": "text",
          "body": "You cross the bridge. On the other side, smoke rises from chimneys. A town."
        }
      ]
    },
    {
      "id": "town",
      "layout": "center",
      "content": [
        { "kind": "heading", "level": 2, "text": "The Town" },
        {
          "kind": "text",
          "body": "You've arrived. The innkeeper greets you warmly. *'Welcome, traveler. Pull up a chair by the fire.'*"
        }
      ]
    }
  ]
}
```

### Tips for Open World

- **Map it first.** Sketch your graph on paper before writing JSON. Open
  World sessions grow fast.
- **Dead ends are okay.** Not every path needs to converge. A dramatic dead
  end is satisfying when the audience made the choice that led there.
- **Use `speaker-notes`** to remind yourself of branching logic during
  presentation.
- **Test every path.** With exponential branching, it's easy to leave a
  dangling reference. Run validation to catch missing targets.

## Designing Good Branch Structures

Across all three patterns, a few principles hold:

1. **Every Question needs a purpose.** Don't branch for the sake of
   branching. Each choice should teach, reveal, or engage.
2. **Label Answers clearly.** The audience should understand the consequences
   (or the mystery) of each option before choosing.
3. **Keep branch depth manageable.** Two or three levels of branching is
   plenty for most Sessions. Deeper nesting makes the Session hard to
   maintain and test.
4. **Use `after` liberally.** The Branch-and-Return pattern keeps Sessions
   manageable. Save pure divergence for stories where consequences matter.
5. **Validate your graph.** Fireside engines check that every `target`
   references a real Moment and every `after` points somewhere valid. Run
   validation early and often.

## Further Reading

- **[§3 Traversal](../../spec/traversal/)** — The formal specification of
  Next, Choose, Goto, and Back, including the history stack contract and
  state machine.
- **[§2 Data Model](../../spec/data-model/)** — Complete type definitions
  for `branch-point`, `BranchOption`, and `Traversal`.
- **[Your First Fireside Session](../getting-started/)** — If you haven't
  built your first Session yet, start there.
