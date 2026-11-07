# Closure lint suppressions in the forbidden-strings scanner

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a
direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the statement (or module) that holds the closure:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
Configuration cannot do it instead:
the rule's settings give a severity and a library path per file pattern and no allow-list of callees
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`),
and lowering the severity would also let a replaceable closure in (`AGENTS.md` LN7).

This crate carries three suppressions.
Every other closure that the rule found was replaced by a named function,
a loop,
a `match`,
a value passed as data to a named function (`map_with`),
or the visitor API of the `ignore` crate.

## `std::panic::catch_unwind` in `frx_scan.rs` and `path_scan.rs`

Two statements wrap one matcher call in `AssertUnwindSafe(|| ...)` and pass it to `catch_unwind`.
`catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F)` passes no argument to its callable,
so the buffer,
the line starts or the pathname component the call needs must be captured.
No named function can take their place and no loop can,
because `catch_unwind` is the only way to turn an engine panic into a value;
the fail-closed scan depends on that.
The proof is the same signature the Git wrapper's record cites in
`package/git-policy/cli/doc/closure-lint-suppressions.md`,
section "Standard-library functions that accept only a callable".

## Clap's derive in `cli.rs`

`#[derive(Parser)]` and `#[derive(Subcommand)]` generate the missing-argument closures of clap's `FromArgMatches`
implementation,
with the field type as their span.
The lint reports them at the fields (`rules_path: String` and its siblings),
although no author-written closure exists there to refactor.
The module-level attribute names the derive as the cause.
`package/file-enforcer/cli/src/args.rs` records the same cause for the same derive.
Replacing the derive with clap's builder API would trade the declaration for hand-written parsing code to satisfy a
finding about code the author did not write,
so the suppression stays until the lint skips the closures a derive generates.
