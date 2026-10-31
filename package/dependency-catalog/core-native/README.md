# monochromatic-dependency-catalog

The native Rust implementation of the `rust-catalog.toml` renderer:
it turns the repository's single Rust dependency catalog into each Cargo workspace root's generated `[workspace.dependencies]` table.
The design and its rationale are in `doc/decision/rust-dependency-catalog.md`;
the contributor workflow is in `doc/runbook/rust-dependencies.md`;
the decision to build this library is in `doc/decision/file-enforcer-rust-rewrite.md`.

## Package name and path

The directory is `package/dependency-catalog/core-native` and the crate is `monochromatic-dependency-catalog` (library `monochromatic_dependency_catalog`).

- Crate names are exempt from the singular-segment rule (`SGD`),
  and the `monochromatic-` prefix follows `monochromatic-jsonc-edit` and `monochromatic-mpm-shadow`.
- `package/dependency-catalog/core-rust` already exists and is a TypeScript package:
  `core-rust` there means "core for the Rust catalog",
  not "written in Rust".
  The TypeScript update,
  verify and tighten tools import it,
  so it is not renamed.
- `core-native` names the Rust implementation,
  the way `package/jsonc-edit/core-rust` is the native sibling of `core-ts`.
  When the TypeScript package is retired at the file-enforcer cutover,
  the directory can be renamed.

## What it does

Pure functions from file contents to expected contents or drift.
The library reads and writes no file:
the caller passes the catalog text and every manifest text,
then writes (write mode) or diffs (check mode) what comes back.

- `parse_catalog(source, path)` validates catalog text and returns a `CatalogDocument`;
  every problem is reported at once.
- `sync(&SyncInput)` returns a `SyncOutput`:
  the expected text of `rust-catalog.toml` and of every manifest,
  the crates whose generated table changes,
  warnings,
  and a list of `Change` values per file explaining each edit.
- `check(&SyncInput)` runs the same sync and returns a `DriftReport` holding only the files that differ.

`sync` does everything the TypeScript file-enforcer step did:

- Adopts literal versions and git pins left by `cargo add` into `[catalog]` as `>=` floors,
  grouping literals across crates so the result does not depend on crate order,
  raising existing floors,
  and refusing exact pins and incompatible requirements with a named-catalog fix.
- Rewrites literal use sites to `workspace = true`,
  keeping `features`,
  `optional` and `default-features`.
- Writes `version = "=<target package.version>"` on internal path dependencies.
- Generates each crate's `[workspace.dependencies]` table,
  with `default-features = false` when every site of a dependency disables default features.
- Prunes entries no crate uses when `catalogPrune = true`,
  and warns about them otherwise.

```rust
// Write mode and check mode in the file-enforcer engine.
let input = SyncInput { catalog_path: "rust-catalog.toml", catalog_source: &catalog, manifests: &manifests };
let output = sync(&input)?;
for file in output.drift() {
    // write file.expected to file.path, or report file.changes
}
```

## Format preservation

All edits go through `toml_edit`:
a document that needs no change prints back byte for byte,
and an edited document keeps every comment,
blank line and key order outside the edited entries.
Values are rendered by hand (always double-quoted strings,
fixed field order) and then parsed,
so the generated text does not depend on `toml_edit`'s quoting choices.

Differences from the TypeScript text splicing,
none of which changes any committed file:

- Removing an entry removes the blank lines directly above its comment block as well as the comment block.
- A generated table whose entries are already correct is left alone even when its header line carries a comment.
- A new `[workspace.dependencies]` table is separated by exactly one blank line even when the file lacks a final newline.
- Invalid TOML in a manifest is an `Error::Toml` naming the file,
  where the TypeScript tool threw the parser's error.

## Tests

`mise run //package/dependency-catalog/core-native:test` runs nextest.

- `tests/golden.rs` runs the library over this repository's real `rust-catalog.toml` and every real Cargo workspace root and requires exactly what is committed:
  no changed byte,
  no affected crate,
  no warning.
- `tests/drift.rs` syncs byte copies of four real manifests as they were committed before the catalog floors were tightened,
  and requires exactly what the TypeScript file-enforcer wrote.
- The unit tests port the TypeScript parser,
  semver,
  syntax and sync tests,
  and cover each edit shape.

`mise run //package/dependency-catalog/core-native:lint:types` runs `cargo check` and clippy with warnings denied.

## Dependencies

`toml_edit` only (approved in `doc/decision/file-enforcer-rust-rewrite.md`).
Version parsing and requirement matching are written by hand,
because no semver crate is approved.
