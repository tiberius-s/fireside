import { defineLinter } from "@typespec/compiler";
import { requireDocRule } from "./rules/require-doc.js";
import { useNodeIdScalarRule } from "./rules/use-node-id-scalar.js";

const libName = "@fireside/typespec";

export const $linter = defineLinter({
  rules: [requireDocRule, useNodeIdScalarRule],
  ruleSets: {
    recommended: {
      enable: {
        [`${libName}/${requireDocRule.name}`]: true,
        [`${libName}/${useNodeIdScalarRule.name}`]: true,
      },
    },
  },
});
