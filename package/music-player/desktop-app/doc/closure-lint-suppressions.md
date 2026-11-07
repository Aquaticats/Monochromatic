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
`library/std/src/thread/scoped.rs` and `library/std/src/panic.rs`):

- `std::thread::scope`:
  `scope<'env, F, T>(f: F) -> T where F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T`.
  The callable is bounded by the scope,
  not by `'static`,
  because the threads borrow the caller's data for the scope only.
  Its `Scope::spawn` takes `F: FnOnce() -> T + Send + 'scope` and no argument,
  so the thread body must carry its state.
- `std::panic::catch_unwind`:
  `catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R>`.
  The guarded call takes no arguments,
  so the arguments it needs must be captured.

No named function can replace these closures,
because neither function passes an argument to its callable.
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

## Callables that the callee stores and a named type cannot be

The Fn traits cannot be implemented for a named type on stable Rust
(`impl Fn for T` needs the unstable `unboxed_closures` and `fn_traits` features),
so a value that must be passed where a `dyn Fn` or an `impl Fn` is expected and that owns state is a closure:

- `src/engine.rs`,
  `run` (the statement `let sweep_sink: SweepSink = Arc::new(move |report| ...)`):
  `SweepSink` is `Arc<dyn Fn(SweepReport) + Send + Sync>` and the closure owns the engine's command sender.
  The type is shared with `measure.rs`,
  `sweep_report.rs` and the controller tests,
  which build their sinks as closures too.
- `src/track_supporting_line.rs`,
  `supporting_line_from_template`:
  the function returns the callable that owns the parsed template and the field list,
  and its consumers (`view`,
  `build_player_content`,
  `track_list_entries`) take `&dyn Fn(&TrackFields) -> Option<String>`.
- `src/ui_search.rs`,
  `posting_delivery`:
  `SearchService::start` keeps the delivery callable for the life of the service and takes it as
  `impl Fn(SearchAnswer) + Send + 'static`,
  so the callable owns the window handle.
  The lint does not see the callee through the `return`,
  so the attribute sits on the function that builds the callable.

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

`derive_scheme` carries an `expect` that predates this note:
every lightness fit in it passes a closure over colors that the function derived,
and the fit helpers take the closure as a parameter.
This note does not widen or review it.
