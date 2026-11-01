# module-rust-catalog

Parsing and validation of `rust-catalog.toml`,
the repository's pnpm-style catalog of Rust dependency requirements.
The design and its rationale are in `doc/decision/rust-dependency-catalog.md`.

## File format

```toml
# rust-catalog.toml
minimumReleaseAge = 1440
trustPolicy = "no-downgrade"

[catalog]
anyhow = ">=1.0.104"
tracing-log = { version = ">=0.2", package = "tracing-log" }
helix-core = { git = "https://github.com/helix-editor/helix", rev = "ba40e547426b0f9896c8bdc699a4ab11f2b37dbc" }

[catalogs.windows-legacy]
# COM drag-and-drop code targets the 0.58 API.
windows = "0.58"
```

- `[catalog]` holds `>=` floors and git pins (`git` plus `rev`).
- `[catalogs.<name>]` holds bounded requirements (`"0.58"`, `"^0.58"`, `"~0.58.1"`, `"=0.58.0"`);
  a `>=` floor there would be pulled to the newest release by the next update.
- `package` renames a dependency, as in Cargo.
- `features`,
  `optional`,
  and `default-features` belong to the crate that uses the dependency,
  because Cargo only adds use-site features on top of an inherited entry.
- Internal crates are never catalog entries;
  crates depend on them by `path`.

Supply-chain settings use pnpm's names and pnpm's defaults:
`minimumReleaseAge` (1440),
`minimumReleaseAgeExclude`,
`minimumReleaseAgeExcludePrune` (`false`),
`minimumReleaseAgeIgnoreMissingTime` (`true`),
`minimumReleaseAgeStrict` (`true` only when `minimumReleaseAge` is set),
`trustPolicy` (`off`),
`trustPolicyExclude`,
`trustPolicyExcludePrune` (`false`),
and `trustPolicyIgnoreAfter`.

## Errors

Parsing fails closed.
Any invalid entry or setting throws `RustCatalogError`,
whose `issues` list names every problem in the file with its location and fix,
so one run reports everything.
Invalid TOML throws `TomlEditError` from `@monochromatic-dev/toml-edit-core`.

## API

- `parseRustCatalog({ source, sourcePath })` validates catalog text.
- `readRustCatalog({ rootDir })` reads `rust-catalog.toml` from a directory and returns its source and document.
- `RUST_CATALOG_FILENAME` is `rust-catalog.toml`.
- `isCrateName`,
  `isVersion`,
  `isFloorRequirement`,
  `isBoundedRequirement`,
  and `floorVersion` are the string-scan validators the parser uses.

## Tests

`mise run //package/dependency-catalog/core-rust:buildAndTest` builds the Node bundle and runs the unit tests against it.
