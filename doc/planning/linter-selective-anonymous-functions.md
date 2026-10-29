# Selective `rust/no-anonymous-functions` through types and directive comments

Status:
proposal of 2026-10-18.
On the same day the owner chose a Dylint lint plus comment directives,
recorded in `doc/decision/rust-closure-lint-dylint.md`.
Later that day the owner removed the rule `rust/no-anonymous-functions` altogether instead of making it selective
(section "Removal of `rust/no-anonymous-functions`" of that record),
so Change 1 and Change 2 below were never built for `monochromatic-lint`.
The sections below are the evidence behind the Dylint choice,
and describe the rule as it was before its removal.

## Problem

`rust/no-anonymous-functions` reports every `CLOSURE_EXPR` node
(`package/linter/monochromatic-lint/src/rust_no_anonymous_functions.rs`).
It has no option besides severity and no inline suppression.
After the IDE closures branch, 1068 of 1135 findings in `package/ide/app` became named code,
and 67 remain.
Those 67 are closures that own state for a callee that keeps or runs the callable later:
17 thread bodies, 38 Slint `on_*` callbacks, 8 `Timer::start` bodies, 2 `catch_unwind` bodies,
and 2 `tree_house` callbacks.
A named `fn` item cannot own state, and stable Rust cannot implement `FnMut` or `FnOnce` for a named type,
so no named item can replace them.
The syntax-only rule cannot tell these from closures that a `for` loop or a `match` replaces.

## What the repository already has

- The linter already holds a semantic session on rust-analyzer
  (`ra_ap_hir`, `ra_ap_hir_ty`, `ra_ap_ide_db`, `ra_ap_load-cargo`, all `=0.0.336` in `Cargo.toml`;
  `src/rust_semantic_session.rs`; used by the explicit type and generic argument rules).
  The type resolution the owner asked about is not a new dependency.
- `ra_ap_hir` 0.0.336 exposes the exact capture set of a closure:
  `hir::Closure::captured_items` returns the minimal captures with their kind
  (`CaptureKind::Move`, `SharedRef`, `UniqueSharedRef`, `MutableRef`), and `Type::as_closure` reaches the closure
  from its expression (`ra_ap_hir-0.0.336/src/lib.rs`, lines 4931 and 5029 to 5060).
  A closure with no captures can always be a named function, so that case stays reported.
- `doc/planning/unified-linter.md` lists "Directive comments of any kind" under
  "Deferred until a consumer exists".
  The consumer now exists, so accepting this proposal lifts that deferral for one directive.

## Proposal

Two independent changes, each landing and releasing on its own.

### Change 1: directive comments

A line comment `// monochromatic-lint-disable-next-line <rule> -- <reason>` silences one rule for the next
code line.
The reason is mandatory; a directive without one is itself a finding.
A directive that silences nothing is a finding too, so stale ones cannot accumulate.
Scope is one line and one named rule; there is no file-wide or block form.
Findings keep their records in the JSONL output with a `suppressed` flag and the reason,
so a suppression stays auditable.

### Change 2: a semantic criterion for the closure rule

Report a closure unless both of these hold:

1. `captured_items` is not empty, so no named function can take its place.
2. The callee requires the callable to outlive the call: the closure is a direct argument whose parameter type
   carries a `'static` bound (`thread::spawn`, `Builder::spawn`, `Timer::start`, and the generated Slint `on_*`
   setters).
   Such a callee stores or runs the state later, so a `for` loop or a `match` cannot replace the closure.

A closure that captures nothing, a closure that only borrows for the duration of the call
(an iterator adaptor, `catch_unwind`), and a let-bound closure stay reported.

What this leaves in the IDE, as designed: the 63 closures with a `'static` callee pass, and the 4 without one
(`catch_unwind` twice, `tree_house` twice) take a directive comment with a reason, which is the use the directive
exists for.
The measured result under the shipped load mode is 14 of 67 (see "Spike results").

### Evidence and gaps

- Verified in source: `Timer::start` takes `impl FnMut() + 'static` (`i-slint-core-1.18.1/timers.rs`, line 93);
  `std::panic::catch_unwind` takes `F: FnOnce() -> R + UnwindSafe` with no `'static`
  (`library/std/src/panic.rs`, line 358).
- The spikes of the same day settled the points that were open when this was first written;
  see "Spike results" below, which supersedes the 63 of 67 estimate.

## Spike results

Full report: `~/temp/agent/linter-spike/report.md` (branch `spike/linter-closure-static`, commits `7c5cee6d30` and
`6c44d820e8`, spike code in `src/rust_closure_static_spike.rs`).

- The generated Slint setter is `pub fn on_ping (& self , mut f : impl FnMut (i32) -> () + 'static)`.
  The IDE's own generated source (`target/debug/build/ide-*/out/app.rs`) holds 28 setters, all `+ 'static`.
- `ra_ap_hir` 0.0.336 answers "captures state and is a direct argument whose declared callee parameter carries
  `'static`" for every callee that resolves to a declared function: std, user functions, trait methods (concrete,
  generic and `dyn` receivers), impl-level bounds, `impl Trait`, `where` clauses, `Box<dyn ..>` and supertraits.
  Ten twin cases (same source without `+ 'static`) all flipped to reported; a deliberately wrong expectation is
  flagged. No new direct dependency; the spike reads `inner().internee`, an internal, tolerable under `=0.0.336`.
- Run over the 67 remaining IDE closures with the shipped load mode: 14 pass (all `thread::spawn` and
  `Builder::spawn`), 41 stay reported because the callee does not resolve (30 generated Slint setters, 7
  `Timer::start`, 2 `catch_unwind(AssertUnwindSafe(..))`, 2 `tree_house`), 11 sit in `#[cfg(test)]` sidecars that the
  session does not load, 1 is not a direct argument.
  Causes: the executable loads with `WorkspacePreparation::SourceOnly` (`src/run_finish.rs`, lines 147 and 149), so
  the Slint-generated window type does not exist for it; and the IDE's dependency graph did not load here
  (`cargo metadata --offline --locked` fails on `encoding_rs`), so even `slint::Timer` did not resolve.
  Whether `BuildGenerated` would resolve the setters is not verified.
- Cost on `package/ide/app`: 0.067 s today against 5.9 s with the closure rule made semantic, about 80
  times, from loading the session and a serial per-file pass; the classification itself is inside the noise. On
  `package/linter/monochromatic-lint`: 0.043 s against 2.3 s.
- Gaps: a callee that is a value of fn-trait type is not answered; closures inside macro token trees need
  `expand_macro_call` and range mapping; a closure whose only use of a variable is `let _ = n` captures nothing.

Consequence: the type criterion alone does not clear the IDE.
Either a callee allow-list option covers the generated setters and `Timer::start`,
or the executable gains a build-generated load mode, which runs the project's build script.
Files outside the loaded target need a defined fallback (report, as today).

## Research results

Fetched on 2026-10-18 by a research agent; the owner's oxlint setup was read by the coordinating session.

- Change 1 is already decided.
  `doc/planning/rust-linter-oxlint-parity.md`, decision D2 (answered 2026-07-25), chose comment directives
  (`<tool>-disable|-enable|-disable-line|-disable-next-line`), a mandatory `--` justification (a directive
  without one suppresses nothing and is itself reported), `--report-unused-disable-directives`, and rules that
  declare themselves non-suppressible (`max-lines`, `require-rustdoc`).
  It also settled that directives are comments, not attributes: a tool attribute needs `register_tool`, which is
  nightly-only, and the other tool namespaces (`clippy::`, `rust_analyzer::`) belong to other tools.
  The mechanism is not built yet (`package/linter/monochromatic-lint/src` has no directive code).
  `doc/planning/unified-linter.md` line 290 ("No rule can be suppressed") predates D2.
- The owner's oxlint setup (`package/config/oxlint/src/config-base.ts`): `reportUnusedDisableDirectives: 'warn'` with
  `denyWarnings: true`; 698 of 734 next-line directives carry a reason; an audit removed 301 unused directives; the
  `no-restricted-syntax` plugin forbids disabling selected rules, including the TypeScript arrow ban, whose
  replacement is a named function expression; tests are exempted from that ban by an override
  (`package/config/oxlint/src/overrides.ts`, line 201).
- Precedent for the spelling and audit: Biome requires an explanation (`MissingReason`) and reports unused
  suppressions by default; ESLint and oxlint accept `--` descriptions and report unused directives when asked;
  Ruff reports unused `noqa` only with `RUF100`; ast-grep requires the keyword to start the comment and gates the
  unused report; golangci-lint `nolintlint` has `require-explanation` and `require-specific`.
  Scope is the literal next physical line in oxlint, ESLint and Biome.
  An unused-directive report must say "not verifiable" for a site that a semantic rule could not analyse.
- No linter found decides "unavoidable" by captures plus the callee's `'static` bound.
  Clippy uses captures and `'static` only as false-positive guards (`redundant_closure`, `redundant_async_block`);
  ESLint `prefer-arrow-callback` skips cases where the arrow would change semantics; unicorn
  `consistent-function-scoping` decides by captured values.
  So the criterion is a new design either way.
- A rustc-driver route (Dylint, 6.2.0 released 2026-10-16) was probed on nightly-2026-10-17: the criterion
  (captures from `upvars_mentioned`, `'static` from the callee's `TypeOutlives` clauses) passed all controls, native
  `#[cfg_attr(dylint_lib = "..", expect(.., reason = ".."))]` suppresses and `unfulfilled_lint_expectations`
  reports unused expectations, and the extra time was about 0.15 s on a 19.5 kloc crate.
  Costs: `rustc-dev` in the root toolchain components, lockstep with the floating nightly (the shipped Dylint
  template lagged two months), a second tool and result channel beside `monochromatic-lint`, per-package
  `check-cfg`, and only compiling code is linted. Whether Dylint resolves the Slint-generated setters is
  unverified (the IDE build script needs its runtime preparation first).
- A fourth shape: a syntactic callee allow-list option (last path segment or method name: `spawn`, `start`,
  `on_*`), no session, today's speed. Risk: a name collision lets a non-`'static` callee through, and it needs a
  positive-control test because the TypeScript `no-class` allow-list option was once silently ignored
  (`doc/troubleshooting/oxlint-class-and-callback-bans.md`).
- rust-analyzer offers no supported custom semantic check; its `ide` API is documented as unstable.
  The linter's `BuildGenerated` mode exists (`src/rust_workspace.rs`) but the executable only requests `SourceOnly`.

## Round two results

Measured on 2026-10-18 on the spike branch (local commits `fad1074bd2` and `37bcb6f5b3`), on the IDE snapshot of
the 67 closures, with the full dependency graph (a copy of the `ide-cargo` volume as `CARGO_HOME`).
These figures supersede "14 of 67" and "63 of 67" above.

- **Test sidecars:** the 11 unanalysable sites are 10 `*_tests.rs` files plus `src/native/inspect.rs`, which is
  `#[cfg(debug_assertions)]`, not a test file. The loader uses `CargoConfig::default()` (`set_test` false) and
  reads the release cfg. Enabling `set_test` plus `debug_assertions` through `cfg_overrides` recovers all 11 and
  about 105 files in total, with no change in load time (the extra analysis costs 1.6 s, near the spread).
  A package with `cfg(not(test))` items would lose coverage under that cfg; not measured.
- **Dependency graph:** without the full graph the loader falls back to a no-dependencies graph (88 crates
  against 612), and `Timer::start` and the two `tree_house` calls do not resolve. A no-dependencies graph also gives
  silent wrong verdicts: closures that do capture report zero captures, so allowed sites flip to reported (the
  conservative direction). On this machine the graph needs `CARGO_HOME` pointed at a copy of the volume.
- **`tree_house` 0.4.0:** `read_query` and `configure` take `impl FnMut`, with no `'static`, `Send` or `Sync`; their
  callbacks borrow caller locals, so they are correctly reported and need a directive or a restructure.
- **Generated Slint setters:** under `SourceOnly` the 37 `on_*` sites (30 in sources, 7 in tests) do not resolve.
  `WorkspacePreparation::BuildGenerated` resolves them on a probe crate (5 of 5 runs) but it runs
  `cargo check --workspace`, which executes the build script of every crate in the graph (59 for the IDE) and
  compiles proc macros, unsandboxed. The IDE as written does not resolve even then, because
  `slint::slint! { export { .. } from "../ui/app.slint"; }` takes a file-relative path that the proc-macro server
  resolves outside the package (reading `slint-macros-1.18.1/lib.rs:399-411`; mechanism not proven). With that
  line replaced by `slint::include_modules!()` in a disposable snapshot: 62 allow and 5 reports (the 2
  `catch_unwind`, the 2 `tree_house`, one let-bound closure at `src/native/language/actions.rs:134`).
- **Wrappers:** `Arc::new(closure)`, `Some(closure)`, struct fields, `Option<F>` and fn-trait values are not
  recognised (7 gaps); async closures and `Holder::new(Box::new(closure))` are. The final IDE snapshot has none of
  these shapes, so they do not affect the 67; the repository elsewhere has `Arc::new(closure)` 8 times
  (`package/music-player`) and `Box::new(closure)` 37 times.
- **Cost, closure rule alone on the IDE package:** syntax only 0.063 s; semantic `SourceOnly` on the full graph
  22 to 24 s (lazy hir initialisation 8.8 s, classification 12.4 s, load 0.8 s); `BuildGenerated` about 39 s warm
  and 46 s cold (n=1). Skipping files without a direct-call-argument closure saves about 2 s (noise).
  The existing explicit-types rule alone costs about 25 s on this package, and adding the closure rule to it costs
  about 4.4 s more, since inference is shared. Round one's 5.9 s came from the fallback graph and was not
  reproduced (3.9 s now).
- **Loader hazard:** a failed workspace load is not cached, so every file retries it (411 loads, 485 s, one
  error repeated) in both modes.
- **Environment trap:** inside `systemd-run` units `rustc` resolves to toolchain 1.98.1 without `rust-src`, so the
  semantic load fails for every file unless `RUSTUP_TOOLCHAIN` names the nightly.

Verdict: the type criterion needs no allow-list only with three conditions together: the full dependency graph,
`cfg(test)` plus `cfg(debug_assertions)`, and `BuildGenerated` (build scripts and proc macros run by the linter,
a prepared IDE runtime, and the `slint!` to `include_modules!()` change in the IDE).
Under `SourceOnly` it resolves 25 of the 67.

## Rejected for now

- Allowing every `move` closure that is a call argument: coarse, and adding `move` is an easy escape.
- Allowing any closure that captures a local: it would have allowed most of the 1068 closures the IDE branch
  converted, since a loop replaces an iterator adaptor that captures a variable.
- Per-file `off` entries in `monochromatic-lint.config.jsonc`: the exemption sits away from the code and covers
  every closure later added to the file.

## Order of work if accepted

1. Spike on the two unverified points, with a positive control (a `thread::spawn(move || ..)` that must pass
   and a capture-free closure that must be reported).
2. Change 1 with tests, a release, and the config of this repository left unchanged.
3. Change 2 with tests that include the adversarial cases (shadowed names, nested closures, closures in macro
   token trees, which the syntax frontend does not see), a release, then a lint run over every Rust package to
   record how the count moves.
4. In the IDE: directive comments on the remaining sites, then the `warn` to `error` switch the owner scheduled
   on 2026-10-05 ("warn everywhere until no finding is left, then error").
