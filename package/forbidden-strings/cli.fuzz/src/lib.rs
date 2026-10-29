// What:     `forbidden_strings_fuzz` is a tiny library crate that
//           hosts only the shared structured generator. Every fuzz
//           target in `fuzz/fuzz_targets/*.rs` imports from here so
//           the bounded `Arbitrary` types live in exactly one place.
// Why:      Without a `[lib]` entry plus this `lib.rs`, each target
//           would have to inline the generator -- or worse, drift
//           from each other. One library makes the byte->AST mapping
//           identical for every target, which keeps libFuzzer's
//           cross-corpus learning useful.
//
// In TS you'd write (pseudocode):
// ```ts
// // fuzz/src/index.ts
// export * from "./generators";
// ```

// What:     `pub mod generators;` registers the sibling file
//           `generators.rs` and re-exposes it publicly. `pub mod`
//           differs from plain `mod`: with `mod`, the module is
//           private to this crate; with `pub mod`, downstream
//           consumers (here, the fuzz target binaries) can name
//           `forbidden_strings_fuzz::generators::...` directly.
// Why:      Targets need to write `use forbidden_strings_fuzz::generators::*;`.
//           Without `pub`, that import would fail with E0603.
//
// In TS you'd write (pseudocode):
// ```ts
// export * from "./generators";
// ```
pub mod generators;
