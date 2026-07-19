# Quickstart: validating Presenter Polish

Prerequisites: built binary (`cargo build -p fireside-cli`), a scratch
directory to create decks in. Each scenario is independently runnable —
see `contracts/` for the exact behavior each checks against.

## 1. Reserved-key validator warning (US1)

```sh
cat > /tmp/reserved-key.fireside.json <<'EOF'
{
  "protocol-version": "0.1.0",
  "title": "Reserved Key Test",
  "nodes": [
    {
      "id": "start",
      "content": [{"kind": "heading", "level": 1, "body": "Start"}],
      "traversal": {
        "branch-point": {
          "options": [
            {"label": "Edit this", "key": "e", "target": "start"},
            {"label": "Fine", "key": "1", "target": "start"}
          ]
        }
      }
    }
  ]
}
EOF
fireside validate /tmp/reserved-key.fireside.json
```

Expect: a `reserved-branch-key` warning naming `e`, `start`, and "Edit
this"; no warning about the `1` option; exit code reflects "valid with
warnings" (not the error exit code — this deck is still presentable).

## 2. Exit summary (US2)

```sh
fireside demo
```

Advance a few slides, press `q`. Expect: after the terminal UI closes, one
line on stdout — `Presented N/7 slides in M:SS.` — with `N` matching the
number of distinct slides actually viewed and `M:SS` a plausible elapsed
time for the session.

## 3. Resume toast (US3)

```sh
fireside new resume-demo   # or any deck with 2+ linked nodes
fireside resume-demo.fireside.json
```

Advance past the first slide, quit with `q` (not `Ctrl+C` from outside raw
mode — use the in-app quit). Relaunch:

```sh
fireside resume-demo.fireside.json
```

Expect: a flash reading `Resumed where you left off — --restart starts
over` on the first frame. Relaunch again with `--restart`:

```sh
fireside resume-demo.fireside.json --restart
```

Expect: no resume flash; the deck opens at its entry node.

## 4. Wizard momentum (US4)

```sh
fireside new
```

Answer the prompts (title, template, author, banner), then at `Present it
now? [Y/n]:` press Enter. Expect: the presenter launches immediately on the
just-created deck, no second command typed. Repeat and answer `n` at the
final prompt — expect the wizard exits to the shell without presenting, deck
file still written.

Also confirm the non-interactive path is unaffected:

```sh
fireside new another-demo
```

Expect: no present-now prompt at all.

## 5. `art text` width guard (US5)

```sh
fireside art text "A Very Long Phrase That Will Not Fit"
```

Expect: stdout still shows the complete FIGlet banner; stderr shows a note
naming the measured width (> 76). Then:

```sh
fireside art text "Hi"
```

Expect: stdout shows the banner; stderr is empty.

## Full verification

After implementing, run the project's standard gates before calling this
feature done:

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
scripts/verify.sh
graphify update .
```

Per constitution Principle VII, any of the above scenarios that are
TUI-visible (2 and 3) also need a real-terminal tmux smoke test, not just a
`TestBackend` scenario test — see memory
`feedback_tmux_smoke_catches_timing_bugs`.
