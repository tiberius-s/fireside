# Quickstart: Validating `validate --watch`

Prerequisites: a built `fireside` binary (`cargo build`) and a deck file.
`docs/examples/hello.json` works for the happy path.

## 1. Confirm non-watch behavior is unchanged (FR-002)

```sh
cargo run -p fireside-cli -- validate docs/examples/hello.json
```

Expect: identical output to before this feature (`✓ … — no problems found`),
process exits immediately.

## 2. Start watch mode and see the immediate first check (FR-003)

```sh
cp docs/examples/hello.json /tmp/watch-demo.json
cargo run -p fireside-cli -- validate --watch /tmp/watch-demo.json
```

Expect: a success line prints right away, before touching the file again.

## 3. Introduce a semantic error and save (User Story 1, FR-004/FR-005)

In another terminal:

```sh
python3 -c "
import json
d = json.load(open('/tmp/watch-demo.json'))
d['nodes'][0]['traversal'] = 'no-such-node'
json.dump(d, open('/tmp/watch-demo.json', 'w'))
"
```

Expect: within ~250ms–1s, the watching terminal updates to show the
`valid-traversal-target` diagnostic, matching the format non-watch
`validate` would show for the same file.

## 4. Fix it and save again

Restore the file (`cp docs/examples/hello.json /tmp/watch-demo.json`).
Expect: the watching terminal updates back to the success line.

## 5. Introduce a JSON syntax error (User Story 2, FR-006)

```sh
echo '{ broken' > /tmp/watch-demo.json
```

Expect: the watching terminal shows the caret-block report — offending
line, caret under the error column — matching the format
`fireside validate /tmp/watch-demo.json` (non-watch, on a malformed file)
already produces today.

## 6. Delete the file (edge case, FR-009)

```sh
rm /tmp/watch-demo.json
```

Expect: a "file missing" message; the process keeps running (does not
exit). Recreate the file and confirm it picks the change back up.

## 7. Stop with Ctrl-C (FR-010)

Expect: clean exit, code `0`, no hang.

## Automated coverage

Steps 1–6 are equivalent to the unit tests added for the new pure
report-building function (`fireside-cli/src/main.rs` `#[cfg(test)]`
module). Step 2 (flag accepted, first check happens) is equivalent to the
new `cli_e2e.rs` test. Run:

```sh
cargo test --workspace
```
