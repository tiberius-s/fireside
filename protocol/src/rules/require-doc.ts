import { createRule, getDoc, paramMessage } from "@typespec/compiler";

/**
 * All public models, enums, unions, interfaces, and scalars in the
 * Fireside namespace must have documentation.
 *
 * This ensures the generated JSON Schemas carry descriptions and
 * the protocol spec stays self-documenting.
 */
export const requireDocRule = createRule({
  name: "require-doc",
  severity: "warning",
  description: "Require documentation on all public protocol types.",
  messages: {
    default: paramMessage`${"kind"} "${"name"}" must have a documentation comment.`,
  },
  create(context) {
    return {
      model: (model) => {
        if (!model.name || model.name === "") return;
        if (model.namespace?.name !== "Fireside") return;

        if (!getDoc(context.program, model)) {
          context.reportDiagnostic({
            format: { kind: "Model", name: model.name },
            target: model,
          });
        }
      },

      enum: (type) => {
        if (!type.name || type.name === "") return;
        if (type.namespace?.name !== "Fireside") return;

        if (!getDoc(context.program, type)) {
          context.reportDiagnostic({
            format: { kind: "Enum", name: type.name },
            target: type,
          });
        }
      },

      union: (type) => {
        if (!type.name || type.name === "") return;
        if (type.namespace?.name !== "Fireside") return;

        if (!getDoc(context.program, type)) {
          context.reportDiagnostic({
            format: { kind: "Union", name: type.name },
            target: type,
          });
        }
      },

      interface: (type) => {
        if (!type.name || type.name === "") return;
        if (type.namespace?.name !== "Fireside") return;

        if (!getDoc(context.program, type)) {
          context.reportDiagnostic({
            format: { kind: "Interface", name: type.name },
            target: type,
          });
        }
      },

      scalar: (type) => {
        if (!type.name || type.name === "") return;
        if (type.namespace?.name !== "Fireside") return;

        if (!getDoc(context.program, type)) {
          context.reportDiagnostic({
            format: { kind: "Scalar", name: type.name },
            target: type,
          });
        }
      },
    };
  },
});
