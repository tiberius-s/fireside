#!/usr/bin/env node

/**
 * Fireside Semantic Validator — Tier 2
 *
 * Validates Fireside JSON documents beyond what JSON Schema (Tier 1) can check.
 * Catches graph-integrity issues: broken references, contradictory traversal,
 * unreachable nodes, cycles, and dead ends.
 *
 * Usage:
 *   node validate.mjs <file.json> [options]
 *
 * Options:
 *   --errors-only   Show only errors, suppress warnings
 *   --json          Output diagnostics as JSON
 *   --quiet         Suppress output on success
 *   --help          Show help
 *
 * See: docs/src/content/docs/spec/validation.md §Layer 2
 */

import { readFile } from "node:fs/promises";
import { resolve, basename } from "node:path";

// ─── Diagnostic Helpers ──────────────────────────────────────────────────────

/**
 * @typedef {"error" | "warning"} Severity
 * @typedef {{ severity: Severity, rule: string, message: string, [key: string]: unknown }} Diagnostic
 */

/** @returns {Diagnostic} */
function diagnostic(severity, rule, message, context = {}) {
  return { severity, rule, message, ...context };
}

// ─── Graph Helpers ───────────────────────────────────────────────────────────

/**
 * Collect all outgoing edge targets from a node.
 * Returns array of { target, kind, label? } where kind is "next" or "branch-option".
 */
function getEdges(node) {
  const edges = [];
  const t = node.traversal;
  if (!t) return edges;

  if (typeof t === "string") {
    edges.push({ target: t, kind: "next" });
  } else {
    if (t.next) {
      edges.push({ target: t.next, kind: "next" });
    }
    if (t["branch-point"]?.options) {
      for (const opt of t["branch-point"].options) {
        edges.push({ target: opt.target, kind: "branch-option", label: opt.label });
      }
    }
  }

  return edges;
}

// ─── Rule Implementations ────────────────────────────────────────────────────

/**
 * ERROR: All node IDs must be unique within the graph.
 *
 * Spec: §4 Validation — Required Check 1
 */
function checkUniqueNodeIds(graph) {
  const diagnostics = [];
  const seen = new Map();

  for (let i = 0; i < graph.nodes.length; i++) {
    const id = graph.nodes[i].id;
    if (seen.has(id)) {
      diagnostics.push(
        diagnostic("error", "unique-node-ids", `Duplicate node ID "${id}" at index ${i} (first seen at index ${seen.get(id)})`, {
          nodeId: id,
          nodeIndex: i,
          firstIndex: seen.get(id),
        }),
      );
    } else {
      seen.set(id, i);
    }
  }

  return diagnostics;
}

/**
 * ERROR: All traversal targets (next and branch-option) must reference
 * existing node IDs.
 *
 * Spec: §4 Validation — Required Checks 2, 3
 */
function checkValidTargets(graph, nodeIds) {
  const diagnostics = [];

  for (const node of graph.nodes) {
    for (const { target, kind, label } of getEdges(node)) {
      if (!nodeIds.has(target)) {
        const detail = kind === "branch-option" ? ` (branch option "${label}")` : "";
        diagnostics.push(
          diagnostic("error", "valid-traversal-target", `Node "${node.id}" has ${kind} targeting non-existent node "${target}"${detail}`, {
            nodeId: node.id,
            target,
            kind,
          }),
        );
      }
    }
  }

  return diagnostics;
}

/**
 * ERROR: A node MUST NOT have both `next` and `branch-point` in its traversal.
 *
 * Spec: §4 Validation — Required Check 5
 */
function checkNextBranchPointConflict(graph) {
  const diagnostics = [];

  for (const node of graph.nodes) {
    const t = node.traversal;
    if (t && typeof t === "object" && t.next && t["branch-point"]) {
      diagnostics.push(
        diagnostic("error", "next-branch-point-conflict", `Node "${node.id}" has both "next" and "branch-point" — these are mutually exclusive`, {
          nodeId: node.id,
        }),
      );
    }
  }

  return diagnostics;
}

/**
 * ERROR: Branch option key values must be unique within a single branch-point.
 *
 * Spec: §4 Validation — Required Check 4
 */
function checkUniqueBranchKeys(graph) {
  const diagnostics = [];

  for (const node of graph.nodes) {
    const t = node.traversal;
    if (!t || typeof t === "string") continue;

    const bp = t["branch-point"];
    if (!bp?.options) continue;

    const seen = new Map();
    for (const opt of bp.options) {
      if (opt.key == null) continue;
      if (seen.has(opt.key)) {
        diagnostics.push(
          diagnostic(
            "error",
            "unique-branch-keys",
            `Node "${node.id}" branch-point has duplicate key "${opt.key}" (options: "${seen.get(opt.key)}" and "${opt.label}")`,
            { nodeId: node.id, key: opt.key },
          ),
        );
      } else {
        seen.set(opt.key, opt.label);
      }
    }
  }

  return diagnostics;
}

/**
 * WARNING: All nodes should be reachable from the entry point (index 0).
 *
 * Spec: §4 Validation — Recommended Check 1
 */
function checkReachability(graph, nodeIds) {
  const diagnostics = [];
  if (graph.nodes.length === 0) return diagnostics;

  const entryId = graph.nodes[0].id;
  const nodeMap = new Map(graph.nodes.map((n) => [n.id, n]));
  const reachable = new Set();
  const queue = [entryId];

  while (queue.length > 0) {
    const id = queue.shift();
    if (reachable.has(id)) continue;
    reachable.add(id);

    const node = nodeMap.get(id);
    if (!node) continue;

    for (const { target } of getEdges(node)) {
      if (nodeIds.has(target) && !reachable.has(target)) {
        queue.push(target);
      }
    }
  }

  for (const node of graph.nodes) {
    if (!reachable.has(node.id)) {
      diagnostics.push(
        diagnostic("warning", "unreachable-node", `Node "${node.id}" is not reachable from entry point "${entryId}"`, {
          nodeId: node.id,
          entryId,
        }),
      );
    }
  }

  return diagnostics;
}

/**
 * WARNING: A node's traversal should not point to itself.
 *
 * Spec: §4 Validation — Recommended Check 2
 */
function checkSelfLoops(graph) {
  const diagnostics = [];

  for (const node of graph.nodes) {
    for (const { target, kind } of getEdges(node)) {
      if (target === node.id) {
        diagnostics.push(
          diagnostic("warning", "self-loop", `Node "${node.id}" has a ${kind} edge pointing to itself`, {
            nodeId: node.id,
            kind,
          }),
        );
      }
    }
  }

  return diagnostics;
}

/**
 * WARNING: Trivial two-node cycles (A → B → A).
 *
 * Spec: §4 Validation — Recommended Check 4
 */
function checkTrivialCycles(graph) {
  const diagnostics = [];
  const nodeMap = new Map(graph.nodes.map((n) => [n.id, n]));
  const reported = new Set();

  for (const node of graph.nodes) {
    for (const { target } of getEdges(node)) {
      if (target === node.id) continue; // self-loops handled by checkSelfLoops
      const targetNode = nodeMap.get(target);
      if (!targetNode) continue;

      for (const { target: backTarget } of getEdges(targetNode)) {
        if (backTarget === node.id) {
          const key = [node.id, target].sort().join("\u2194");
          if (!reported.has(key)) {
            reported.add(key);
            diagnostics.push(
              diagnostic("warning", "trivial-cycle", `Trivial cycle: "${node.id}" \u2192 "${target}" \u2192 "${node.id}"`, {
                nodeA: node.id,
                nodeB: target,
              }),
            );
          }
        }
      }
    }
  }

  return diagnostics;
}

/**
 * WARNING: Branch option targets that have no outgoing traversal.
 * These are dead ends reachable only via back().
 *
 * Spec: §4 Validation — Recommended Check 5
 */
function checkDeadEndBranches(graph) {
  const diagnostics = [];
  const nodeMap = new Map(graph.nodes.map((n) => [n.id, n]));

  for (const node of graph.nodes) {
    const t = node.traversal;
    if (!t || typeof t === "string") continue;

    const bp = t["branch-point"];
    if (!bp?.options) continue;

    for (const opt of bp.options) {
      const targetNode = nodeMap.get(opt.target);
      if (!targetNode) continue;

      if (!targetNode.traversal) {
        diagnostics.push(
          diagnostic(
            "warning",
            "dead-end-branch",
            `Branch option "${opt.label}" in node "${node.id}" leads to "${opt.target}" which has no traversal (dead end \u2014 only back() can exit)`,
            { nodeId: node.id, branchTarget: opt.target, label: opt.label },
          ),
        );
      }
    }
  }

  return diagnostics;
}

// ─── Main Validator ──────────────────────────────────────────────────────────

/** Run all Tier 2 semantic checks against a parsed Fireside document. */
function validate(graph) {
  const nodeIds = new Set(graph.nodes.map((n) => n.id));

  return [
    ...checkUniqueNodeIds(graph),
    ...checkValidTargets(graph, nodeIds),
    ...checkNextBranchPointConflict(graph),
    ...checkUniqueBranchKeys(graph),
    ...checkReachability(graph, nodeIds),
    ...checkSelfLoops(graph),
    ...checkTrivialCycles(graph),
    ...checkDeadEndBranches(graph),
  ];
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

const HELP = `Fireside Semantic Validator — Tier 2

Usage: node validate.mjs <file.json> [options]

Options:
  --errors-only   Show only errors, suppress warnings
  --json          Output diagnostics as JSON
  --quiet         Suppress output on success
  --help          Show this help

Rules (errors):
  unique-node-ids            All node IDs must be unique
  valid-traversal-target     All traversal/branch targets must reference existing nodes
  next-branch-point-conflict A node must not have both next and branch-point
  unique-branch-keys         Branch option keys must be unique per branch-point

Rules (warnings):
  unreachable-node           Nodes should be reachable from entry point
  self-loop                  Traversal should not point to the same node
  trivial-cycle              Two-node cycles (A→B→A) are likely accidental
  dead-end-branch            Branch targets with no traversal are dead ends

Exit codes:
  0  No errors (warnings may still be present)
  1  Semantic errors found
  2  File read or parse failure`;

async function main() {
  const args = process.argv.slice(2);

  if (args.length === 0 || args.includes("--help")) {
    console.log(HELP);
    process.exit(0);
  }

  const filePath = resolve(args.find((a) => !a.startsWith("--")));
  const errorsOnly = args.includes("--errors-only");
  const jsonOutput = args.includes("--json");
  const quiet = args.includes("--quiet");

  let raw;
  try {
    raw = await readFile(filePath, "utf-8");
  } catch (err) {
    console.error(`Failed to read file: ${filePath}`);
    console.error(err.message);
    process.exit(2);
  }

  let doc;
  try {
    doc = JSON.parse(raw);
  } catch (err) {
    console.error(`Failed to parse JSON: ${err.message}`);
    process.exit(2);
  }

  if (!doc.nodes || !Array.isArray(doc.nodes)) {
    console.error(`Not a valid Fireside document: missing "nodes" array`);
    process.exit(2);
  }

  let results = validate(doc);

  if (errorsOnly) {
    results = results.filter((d) => d.severity === "error");
  }

  if (jsonOutput) {
    console.log(JSON.stringify(results, null, 2));
    process.exit(results.some((d) => d.severity === "error") ? 1 : 0);
  }

  if (results.length === 0) {
    if (!quiet) {
      console.log(`\u2713 ${basename(filePath)}: no semantic issues found`);
    }
    process.exit(0);
  }

  const errors = results.filter((d) => d.severity === "error");
  const warnings = results.filter((d) => d.severity === "warning");

  for (const d of results) {
    const icon = d.severity === "error" ? "\u2717" : "\u26A0";
    console.log(`  ${icon} [${d.rule}] ${d.message}`);
  }

  console.log(`\n${basename(filePath)}: ${errors.length} error(s), ${warnings.length} warning(s)`);
  process.exit(errors.length > 0 ? 1 : 0);
}

main();
