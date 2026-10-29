# @monochromatic-dev/jsonc-edit-core-ts.conformance

Curated JSONC conformance corpus for `@monochromatic-dev/jsonc-edit-core-ts`,
 kept as a sidecar
so the runtime package's `src/` stays pure production code (and a whole-package mutation run
stays scoped to real runtime files).

The corpus targets VS Code JSONC semantics:
 JSON plus `//` and block comments and trailing
commas,
 but not JSON5 (no single quotes,
 unquoted keys,
 or hex).
 Valid cases parse to an
expected value;
 invalid or JSON5-only cases throw.

## Run

```sh
mise run //package/jsonc-edit/core-ts.conformance:test:conformance
```

The corpus file is also a `*.unit.test.ts`,
 so `test:unit` runs it as well.
