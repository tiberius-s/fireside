import { createTypeSpecLibrary } from "@typespec/compiler";

export const $lib = createTypeSpecLibrary({
  name: "@fireside/typespec",
  diagnostics: {},
} as const);

export const { reportDiagnostic, createDiagnostic } = $lib;
