#!/usr/bin/env node

/**
 * Runs the shared conformance fixture corpus at `fixtures/` and checks it
 * against `fixtures.expected.json` — the same corpus and expectations file
 * the Rust `fireside-engine` test suite runs, proving Rust/Node rule-id
 * parity is a tested fact, not just an assertion resting on matching
 * rule-name strings. See
 * ../specs/004-spec-patch-0-1-1/contracts/fixture-corpus.md.
 *
 * Usage: node run-fixtures.mjs
 * Exit codes: 0 all fixtures match, 1 a mismatch was found.
 */

import { readFile, readdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { validate } from "./validate.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const fixturesDir = join(here, "fixtures");

async function fixturePaths(subdir) {
  const names = (await readdir(join(fixturesDir, subdir))).filter((n) => n.endsWith(".json"));
  names.sort();
  return names.map((name) => `${subdir}/${name}`);
}

async function main() {
  const expected = JSON.parse(await readFile(join(here, "fixtures.expected.json"), "utf-8"));

  const relKeys = [...(await fixturePaths("valid")), ...(await fixturePaths("invalid"))];

  let failures = 0;
  let checked = 0;

  for (const relKey of relKeys) {
    const expectRules = expected[relKey];
    if (expectRules === undefined) {
      console.error(`✗ ${relKey}: no expectation entry in fixtures.expected.json`);
      failures++;
      continue;
    }

    const raw = await readFile(join(fixturesDir, relKey), "utf-8");
    const graph = JSON.parse(raw);
    const diagnostics = validate(graph);

    const actualRules = [...new Set(diagnostics.map((d) => d.rule))].sort();
    const expectedSorted = [...expectRules].sort();

    const rulesMatch = JSON.stringify(actualRules) === JSON.stringify(expectedSorted);

    const isInvalidDir = relKey.startsWith("invalid/");
    const hasErrors = diagnostics.some((d) => d.severity === "error");
    const errorsMatchDir = hasErrors === isInvalidDir;

    if (!rulesMatch || !errorsMatchDir) {
      console.error(`✗ ${relKey}`);
      if (!rulesMatch) {
        console.error(`    expected rules: ${JSON.stringify(expectedSorted)}`);
        console.error(`    actual rules:   ${JSON.stringify(actualRules)}`);
      }
      if (!errorsMatchDir) {
        console.error(`    expected hasErrors=${isInvalidDir}, got ${hasErrors}`);
      }
      failures++;
    } else {
      checked++;
    }
  }

  const documentedKeys = Object.keys(expected).sort();
  const seenKeys = [...relKeys].sort();
  if (JSON.stringify(documentedKeys) !== JSON.stringify(seenKeys)) {
    console.error("✗ fixtures on disk and fixtures.expected.json entries don't match exactly");
    console.error(`    on disk:    ${JSON.stringify(seenKeys)}`);
    console.error(`    documented: ${JSON.stringify(documentedKeys)}`);
    failures++;
  }

  if (failures > 0) {
    console.error(`\n${failures} fixture(s) failed, ${checked} passed`);
    process.exit(1);
  }

  console.log(`✓ all ${checked} fixtures match protocol/fixtures.expected.json`);
  process.exit(0);
}

main();
