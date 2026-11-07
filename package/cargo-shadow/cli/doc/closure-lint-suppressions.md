# Closure lint suppressions in cargo-shadow

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule `rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the item or statement that holds the closure:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
Configuration cannot do it instead:
the rule's settings give a severity and a library path per file pattern and no allow-list of callees (`package/linter/monochromatic-lint/src/rust_rule_settings.rs`),
and lowering the severity would also let a replaceable closure in (`AGENTS.md` LN7).

This crate carries three suppressions,
none of them in a callback API that this crate owns.
Every closure in a callback API of the crate was replaced by a value that implements a trait:
`RecordedWork` for the recorded runs (`run_recording`,
`run_recording_manifests`,
`timed`),
`Judge` for the recovery,
`RowHandler` and `TableRow` for the CSV and dump readers,
and `PlaceholderBody` for the placeholder lockfile.
Each trait has a blanket implementation for the closure shape,
so a test still passes a closure.
The lazy trace takes a `format_args!` value,
and the statements that compute their text exist in a test build only.

## Returned readers in `git_base.rs` and `shadow_step.rs`

`git_lock_reader_at` returns a `Box<dyn Fn(&str) -> String>` that keeps the repository root and the commit after the function returns,
and `trust_base_setup` returns a maker that keeps the resolved base and the repository root.
The closure is wrapped in `Box::new`,
which the lint does not recognize as a `'static` callee (its README lists wrappers among its known limits).
A named function cannot hold the captured values,
and a struct would need a trait object of its own for no gain.

## Clap's derive in `cli.rs`

`#[derive(Parser)]` and `#[derive(Subcommand)]` generate the missing-argument closures of clap's `FromArgMatches` implementation,
with the field type as their span.
No author-written closure exists there to refactor.
The crate-level attribute names the derive as the cause.
`package/forbidden-strings/cli/doc/closure-lint-suppressions.md` records the same cause for the same derive.
