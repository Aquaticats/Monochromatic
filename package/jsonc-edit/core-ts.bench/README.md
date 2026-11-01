# @monochromatic-dev/jsonc-edit-core-ts.bench

Parse benchmark sidecar for `@monochromatic-dev/jsonc-edit-core-ts`.

This package holds the benchmark so the runtime package's `src/` stays pure production code
(no benchmark,
 fuzz,
 or conformance tooling),
 which keeps a whole-package mutation run scoped
to real runtime files.
 See the fuzz and conformance sidecars for the other tooling.

## Run

```sh
mise run //package/jsonc-edit/core-ts.bench:bench
```

It compares `parseJsonc` against microsoft `jsonc-parser` and `jsonc-eslint-parser` on a clean
document (where jsonc-edit takes the native `JSON.parse` fast-path) and a commented document
(where it uses the structured parser).
 Numbers are machine-dependent;
 run the task to reproduce
them locally.
