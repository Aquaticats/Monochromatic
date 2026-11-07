# Closure lint suppressions in the desktop music player

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a
direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the statement,
function or module that holds the closure:

```text
# package/music-player/desktop-app/src/measure.rs
#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]
```

`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
The manifest declares the cfg name that the attribute uses
(`[lints.rust.unexpected_cfgs]` in `Cargo.toml`,
written by file-enforcer from `file-enforcer.config.jsonc`,
entry `cargo-dylint`).

## Why configuration cannot do it

`monochromatic-lint.config.jsonc` gives the rule a severity and a library path per file pattern and nothing else
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`,
the settings of `rust/avoidable-closure`).
There is no allow-list of callees,
so a callee that takes a callable without a `'static` bound cannot be named there.
Lowering the severity for a file would also let a replaceable closure into it,
which `AGENTS.md` LN7 forbids.
The lint decides from rustc's own answer:
`parameter_requires_static` in `package/linter/monochromatic-lint-closure/src/static_bound.rs` reads the `TypeOutlives`
clauses of the callee,
and `capture.rs` reads `tcx.upvars_mentioned`.

## Standard-library functions that accept only a callable

The proof for each site is the signature in the sysroot source of the toolchain that builds the lint
(`nightly-2026-11-07`,
`library/std/src/thread/scoped.rs`,
lines 141 to 143 and 201 to 204,
and `library/std/src/panic.rs`,
line 358):

- `std::thread::scope`:
  `scope<'env, F, T>(f: F) -> T where F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T`.
  The callable is bounded by the scope,
  not by `'static`,
  because the threads borrow the caller's data for the scope only.
- `std::thread::Scope::spawn`:
  `spawn<F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope, T> where F: FnOnce() -> T + Send + 'scope, T: Send + 'scope`.
  The thread body takes no argument and is bounded by `'scope`,
  so it must carry its state.
- `std::panic::catch_unwind`:
  `catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R>`.
  The guarded call takes no arguments,
  so the arguments it needs must be captured.

No named function can replace these closures,
because none of the three functions passes an argument to its callable.
No loop can,
because a loop cannot run on another thread,
and `catch_unwind` is the only way to turn a panic into a value.
The sites:

- `src/search_feed.rs`,
  `prepare_in_order`:
  the statement `thread::scope(|scope| ...)` holds the scope closure and the closure of each `scope.spawn`,
  which carry the counters and the work list that the preparer threads share.
- `src/search_index_query.rs`,
  `tier_in_parallel`:
  the `return thread::scope(...)` statement holds the scope closure and the closure of each `scope.spawn`,
  which carry `self`,
  the bucket slice and the terms for one worker.
- `src/measure.rs`,
  `spawn_queue_measurement`'s statement `let outcome = std::panic::catch_unwind(...)`:
  the closure carries the sweep job.
  The fail-closed worker depends on turning a panic of one sweep into a value instead of ending the worker.

## What replaced the other suppressions

Three callables need no suppression.
`sweep_report::sink_from` takes the engine's sink callable as an `impl Fn + Send + Sync + 'static` parameter,
so the closure in `engine.rs` is a direct argument of a callee that keeps it.
The Search delivery closure is written as the direct argument of `ui_search::install`,
whose `deliver` parameter requires `'static`.
`supporting_line_from_template` returns a `TemplateSupportingLine` that implements the `SupportingLine` trait,
and the row builders take `&dyn SupportingLine`.
The walk's batch callback (`scan_stream::BatchListener`) and the search feed's sinks (`search_feed::BatchSink`) are traits
with named implementations,
and a blanket implementation lets a test pass a plain callable.

## Clap's derive in `cli.rs`

`#[derive(Parser)]` generates the missing-argument closures of clap's `FromArgMatches` implementation,
with the field type as their span.
The lint reports them at the fields (`start_playing: bool` and `licenses: bool`),
although no author-written closure exists there to refactor.
The module-level attribute names the derive as the cause.
`package/file-enforcer/cli/src/args.rs` and `package/forbidden-strings/cli/doc/closure-lint-suppressions.md` record the
same cause for the same derive.
Replacing the derive with clap's builder API would trade the declaration for hand-written parsing code to satisfy a
finding about code the author did not write,
so the suppression stays until the lint skips the closures that a derive generates.

## The color derivation in `theme_roles.rs`

`derive_scheme` carries no suppression.
It used to pass a closure to `fit_lightness` for each of seven roles,
because each fit measures contrast against colors that the function derived earlier.
`fit_lightness` now takes a `FitCheck`,
and the enum `FitRule` holds one variant per role with the colors that role is measured against.
A plain function of the color is a `FitCheck` too (a blanket implementation),
which the unit tests use for a positive control and an impossible target.
`theme_roles_fixture_tests.rs` compares the derived palette with the Android `theme-roles-vector.txt` fixture,
so the move cannot change a color unnoticed.
