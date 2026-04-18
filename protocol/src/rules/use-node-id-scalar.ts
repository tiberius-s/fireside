import { createRule, paramMessage, type Model } from "@typespec/compiler";

/**
 * Properties that serve as node references (named "target" or "next")
 * should use the NodeId scalar type, not plain string.
 *
 * Catches drift where someone adds a new traversal property as a
 * bare string instead of the protocol's NodeId scalar.
 */
export const useNodeIdScalarRule = createRule({
  name: "use-node-id-scalar",
  severity: "warning",
  description: "Traversal target properties should use the NodeId scalar type.",
  messages: {
    default: paramMessage`Property "${"propName"}" on model "${"modelName"}" looks like a node reference but uses "${"actualType"}" instead of NodeId.`,
  },
  create(context) {
    const TARGET_PROP_NAMES = new Set(["target", "next"]);

    return {
      modelProperty: (prop) => {
        if (!TARGET_PROP_NAMES.has(prop.name)) return;

        const model = prop.model as Model | undefined;
        if (!model?.name) return;
        if (model.namespace?.name !== "Fireside") return;

        const propType = prop.type;

        // Accept NodeId scalar directly
        if (propType.kind === "Scalar" && propType.name === "NodeId") return;

        // Accept unions that include NodeId (e.g., NodeId | Traversal)
        if (propType.kind === "Union") {
          for (const variant of propType.variants.values()) {
            const vType = variant.type;
            if (vType.kind === "Scalar" && vType.name === "NodeId") return;
          }
        }

        const actualType =
          propType.kind === "Scalar"
            ? propType.name
            : propType.kind === "Model"
              ? propType.name || "anonymous model"
              : propType.kind;

        context.reportDiagnostic({
          format: {
            propName: prop.name,
            modelName: model.name,
            actualType: String(actualType),
          },
          target: prop,
        });
      },
    };
  },
});
