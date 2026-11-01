# monochromatic-lint-closure

A Dylint library with one late lint,
`avoidable_closure`,
that reports a Rust closure expression unless a named function,
a loop,
or a `match` cannot replace it.
The decision and its evidence are in `doc/decision/rust-closure-lint-dylint.md` and `doc/planning/linter-selective-anonymous-functions.md`.
`monochromatic-lint` runs it for every Rust package as the rule `rust/avoidable-closure`
(`package/linter/monochromatic-lint/README.md`,
section "Closure check"),
so no package has a task of its own for it;
the IDE's former `lint:closures` task was removed on 2026-10-19.
The library path is the rule's `library` option in `monochromatic-lint.config.jsonc`.
It replaced the syntax-only `rust/no-anonymous-functions` rule of `monochromatic-lint`,
which was removed on 2026-10-18.

## Package name

The directory and crate are `package/linter/monochromatic-lint-closure`,
and the Dylint library is `monochromatic_lint_closure`.
The `monochromatic-lint-` prefix puts the package beside `monochromatic-lint` and its `.fuzz` sidecar and names the family it belongs to.
It is not a `.fuzz`,
`.bench` or `.conformance` sidecar,
so the sidecar suffix convention does not apply;
the directory segment is singular.
The crate name is also the string users write in `dylint_lib = "monochromatic_lint_closure"`,
and the lint name `avoidable_closure` is what follows `expect(` in a suppression.
The package is `publish = false`:
Dylint loads a library from a path or a Git URL,
not from crates.io.

## Criterion

The lint reports a closure unless both conditions hold:

- rustc's capture analysis says the closure captures something.
  The lint reads the minimal captures (`TypeckResults::closure_min_captures_flattened`),
  not `tcx.upvars_mentioned`,
  because edition 2021 precise capture makes `let _ = n` capture nothing while `upvars_mentioned` still lists `n`.
  Before edition 2021 the same source captures `n`;
  the fixture `edition_2018.rs` shows the difference.
- The closure is a direct call or method-call argument whose declared callee parameter requires `'static`.
  Parentheses are already absent from the compiler's IR.
  `Box::new(closure)` is looked through once,
  because that is how a closure reaches a `Box<dyn FnMut() + 'static>` parameter.

A callee parameter requires `'static` in two ways:

- Its type is a generic parameter,
  including an `impl Trait` argument,
  with a `TypeOutlives(parameter, 'static)` clause in the callee's `param_env`.
  `param_env` already includes the parent impl and trait clauses
  and the clauses that a supertrait such as `trait Task: 'static` implies
  (a measured fact:
  an explicit elaboration pass changed no fixture result,
  so the package does not have one).
- Its type is `Box<dyn Trait + 'static>`,
  written or implied by the default object bound.

Such a callee keeps or runs the callable after the call,
so a `for` loop or a `match` cannot replace the closure,
and when the closure owns state no named function can either.
Async closures are handled;
`async` blocks and `async fn` bodies are not closure expressions and are skipped.
Callees checked:
functions,
inherent and trait methods (concrete receiver,
generic receiver,
universal call syntax,
`dyn` receiver).

## Generated and foreign files

A closure is also skipped when its span lies in a file that the package's author did not write:
a file below `OUT_DIR` (build-script output such as Slint's `include_modules!` window code,
which is overwritten on every build),
and any file with an absolute path outside `CARGO_MANIFEST_DIR`.
Cargo exports both variables to the compiler process.
A package source reaches the compiler as a path relative to the package root,
so a relative path is always the package's own.
`..` components are resolved before the comparison,
and `OUT_DIR` wins over the manifest directory because a build script's output can sit inside the package's `target`
directory.
When the compiler was not started by cargo,
the manifest variable is absent and only `OUT_DIR` can exclude.
`src/own_source.rs` holds the decision with unit tests;
the fixture `generated_code.rs` includes one file from each case and one closure of the package's own as the
positive control.
The UI test exports `CARGO_MANIFEST_DIR` and `OUT_DIR` for the compiler processes it starts,
and `tests/outside/` holds the file that lies beside the package directory.
Measured effect:
before the change `package/music-player/desktop-app` reported 5252 closures and
`package/terminal/app` 118 inside the generated `app.rs`;
see the measurement in `doc/planning/closure-lint-everywhere.md` and the end-to-end run recorded in
`doc/decision/rust-closure-lint-dylint.md`.

## Closures from macros

The decision:
a closure is reported only where its span is in the author's own code.
A closure that comes from an external macro expansion (a dependency's `macro_rules!`,
a derive,
a procedural macro such as the one behind Slint's `slint!`) is skipped,
because the author cannot rewrite it.
A closure the author writes as an argument of an external macro is checked normally,
and so is a closure written in the crate's own `macro_rules!` body.
rustc already refuses to report a lint inside external macros unless the lint declares `report_in_external_macro` (`rustc_middle/src/lint.rs`,
`lint_level`,
line 473 in the pinned nightly),
so the explicit span test in `src/lib.rs` is a second guard of the same policy;
removing it changed no fixture result (see the guard controls below),
while inverting it fails six fixtures.
The fixture `external_macro.rs` uses an auxiliary crate that stands in for a dependency.
No fixture covers a procedural macro;
the IDE run below is the only evidence that closures generated by one are skipped.

## Message and help

The message names the failed condition in neutral terms and lists the captured variables when there are any.
The help gives the remedy for that condition:

- No captures:
  move the body into a named function and pass its path.
- Captures,
  but the callee does not keep the closure,
  or the callee is not a declared function,
  or the closure is not a direct argument:
  use a named function that takes the state as parameters,
  or a loop;
  if the callee keeps the state,
  attach the suppression attribute.
- A let-bound closure:
  pass the closure directly to the call,
  restructure,
  or attach the attribute to the `let` statement (see the next section).

## Suppression

Suppression is rustc's own:
put

```text
#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "the callee keeps the state"))]
```

on the closure argument or on an enclosing item.
`expect` makes rustc report an expectation that suppresses nothing (`unfulfilled_lint_expectations`),
so stale suppressions surface by themselves;
`allow` works too but does not.
The `cfg_attr(dylint_lib = ..)` wrapper keeps the attribute inert when the library is not loaded,
so plain `cargo check` and rust-analyzer do not see an unknown lint.

Verified placements (fixture `suppression.rs`):

- On a call argument:
  `std::thread::spawn(#[cfg_attr(..)] move || ..)` works on stable-compatible syntax.
- On an enclosing item,
  and on a `let` statement.
- Not on a `let` initializer:
  `let f = #[cfg_attr(..)] move || n;` is rejected by rustc with E0658 (fixture `suppression_let_initializer.rs`).
  The lint's help for a let-bound closure says so.
- A misspelled lint name inside `expect` draws rustc's `unknown_lints` warning and suppresses nothing.
- An attribute for another library name suppresses nothing.

### Required check-cfg entry

Every package that uses the attribute needs this in its `Cargo.toml`,
or rustc warns about an unexpected `cfg` name `dylint_lib`:

```toml
[lints.rust.unexpected_cfgs]
level = "warn"
check-cfg = ["cfg(dylint_lib, values(any()))"]
```

Package `Cargo.toml` files are managed by file-enforcer,
so the entry belongs in the file-enforcer configuration,
not in a hand edit.

## Usage

### Toolchain

The package uses the repository's shared floating nightly,
declared in the root `mise.toml` with the `rustc-dev` component
(a Dylint library links rustc's own crates)
and the tools `cargo-dylint` and `dylint-link` (the owner approved this on 2026-10-18).
`mise install` provisions all three;
nothing else is needed,
and the package has no `rust-toolchain.toml` and no toolchain override of its own.
The exact nightly is whatever `mise.lock` resolves for the `rust` entry.
Dylint builds a library and every package it lints with that nightly,
and `cargo dylint` caches its driver in `~/.dylint_drivers` (the optional `DYLINT_DRIVER_PATH` moves it).

A floating-nightly upgrade can break the lint source:
rustc's internal API moves between nightlies,
so after `mise.lock` moves the nightly,
build the library and run the tests before relying on the lint,
and fix the source if the compiler's API changed.
The `dylint_linting` and `dylint_testing` crates and the two tools must stay on the same Dylint release.

### Tasks

- `mise run //package/linter/monochromatic-lint-closure:build` builds the library.
- `mise run //package/linter/monochromatic-lint-closure:test` runs the unit tests and the UI fixtures through nextest.
- `mise run //package/linter/monochromatic-lint-closure:lint` runs `cargo check`,
  Clippy with `--deny warnings`,
  and `cargo fmt --check`;
  `lint:clippy` and `lint:rustfmt` run the last two alone.
- `mise run //package/linter/monochromatic-lint-closure:run <package directory>` lints another package with `cargo dylint --path . --all-targets`.
  `monochromatic-lint` itself checks the library and the binaries only;
  this task shows what that exemption of tests,
  examples and benchmarks leaves out.
- `mise run //package/linter/monochromatic-lint-closure:bless` rewrites `tests/ui/*.stderr` from the lint's current output.

The aggregate tasks carry the names `build` and `lint`,
as in the sibling packages,
so the root `build` and `lint` tasks fan out to them in every session.
A session whose tools are not installed yet runs `mise install` first
(`auto_install` does it on the first `mise run`).

### Building the target with the library's toolchain

Dylint builds the linted package with the toolchain that built the library,
so the target must compile on that nightly.
The IDE package (`package/ide/app`) has no `rust-toolchain` file and builds in a container image on the floating stable channel;
it compiled on `nightly-2026-10-17` in the stage-1 run,
so no incompatibility showed.
Only code that compiles under the chosen features,
`cfg` and targets is linted,
so pass `--all-targets` (the `run` task does) to reach test sidecars and examples.
The repository's shared `~/.cargo/config.toml` sets `build.build-dir` outside the package,
which moves the `lib<name>@<toolchain>.so` copy that `dylint-link` writes out of `target/`.
`cargo dylint --path <this package>` builds the library itself and finds it under the name Dylint expects,
so neither the `run` task nor the linter stages a copy (measured on 2026-10-19 under that setting;
the IDE task removed on the same day staged one because it set its own `CARGO_BUILD_BUILD_DIR`).

## Limits

- Wrappers between the closure and the bound are not recognised,
  so the closure stays reported:
  `Arc::new(closure)`,
  `Some(closure)`,
  tuple-struct constructors,
  struct fields,
  `Option<F>`,
  `Rc::new(closure)`,
  and `Box::leak(Box::new(closure))` into a `&'static dyn` parameter.
  `Box::new` is the only wrapper looked through.
  Fixture `known_gaps.rs` holds one case per gap,
  each marked `GAP`,
  so a future fix flips the marker.
- A callee that is a value of fn-trait type (a generic `G: Fn(..)`,
  a `Box<dyn Fn(..)>`,
  a function pointer) has no declared parameter list to read and stays reported.
- Only code that compiles is linted:
  features,
  `cfg` and the selected targets decide coverage.
- The library is coupled to one nightly.
  The compiler's internal API moves between nightlies;
  a new nightly in `mise.lock` means rebuilding and rerunning the tests,
  and fixing the source when the API moved.
- Linux x86_64 only was exercised.
- The Slint-generated window type was unverified before stage 1;
  the IDE run below shows its `on_*` setters resolve under rustc.

## Tests

- `tests/ui/*.rs` are UI fixtures run by `dylint_testing` (a `compiletest_rs` harness) against the golden `.stderr` files.
  Each closure line is preceded by a hand-written marker such as `// verdict: allowed`,
  `reported`,
  `reported 2`,
  `suppressed`,
  `unfulfilled`,
  `unknown-lint` or `error E0658`.
- `src/verdict_tests.rs` compares each golden file with the markers,
  independently of the lint,
  so a wrongly blessed verdict fails.
  Its own controls prove it flags an unmarked report,
  a missing report,
  and an unknown diagnostic.
- `src/message_tests.rs` unit-tests the wording branches and that no message uses a judging word.
- Fixtures:
  `kept_by_callee.rs` (each allowed case with a twin that drops `'static` and must be reported;
  the two std callees `thread::spawn` and `Builder::spawn` have no twin because their bound lives in std,
  and `thread::scope` in `not_kept_by_callee.rs` is their unbounded counterpart),
  `not_kept_by_callee.rs`,
  `known_gaps.rs`,
  `adversarial.rs`,
  `edition_2018.rs`,
  `external_macro.rs` with `auxiliary/external_macro.rs`,
  `suppression.rs`,
  `suppression_let_initializer.rs`.
- The staging step in `src/ui_tests.rs` pads golden files that end in a warning summary with the blank line rustc prints,
  because the repository's commit policy trims every text file to one final newline.
- `compiletest_rs` replaces the fixture directory name inside the output with `$DIR`,
  so the fixtures live in `tests/ui` and not `ui`,
  where the text `ui` inside words such as `required` would be corrupted.
- `bless` is needed after an intended change:
  review the `.stderr` diff against the markers,
  then commit.

### Guard controls seen failing

Each guard was disabled in turn,
the tests were run,
and the source was restored;
the first column is the change,
the rest is what failed.
The driver and results are not part of the package.

- Parameter-type branch always false:
  every fixture file with an allowed case failed.
- `dyn` object branch always false:
  `kept_by_callee.rs`;
  always true:
  `kept_by_callee.rs` and `not_kept_by_callee.rs`.
- `Box::new` look-through disabled:
  `known_gaps.rs` and `kept_by_callee.rs`;
  applied to any single-argument call:
  all fixtures.
- Box not unwrapped in the parameter type:
  `kept_by_callee.rs`.
- Captures from `upvars_mentioned` instead of minimal captures:
  six fixtures,
  including the `let _ = n` case in `adversarial.rs` that stops being reported.
- Captures forced empty:
  six fixtures.
- External-macro test inverted:
  six fixtures;
  removed:
  nothing failed (redundant with rustc,
  see above).
- Closure-kind filter removed:
  `kept_by_callee.rs` and `not_kept_by_callee.rs` (the inner coroutine of an async closure,
  and the `async` block).
- Argument index forced to zero:
  `kept_by_callee.rs`;
  method receiver not counted as parameter zero:
  `kept_by_callee.rs`.
- Constructors accepted as declared functions:
  `known_gaps.rs`.
- Capture names not deduplicated:
  `adversarial.rs`.
- Let initializer classified as another position:
  three fixtures.
- Captures condition dropped:
  `adversarial.rs` and `not_kept_by_callee.rs`;
  callee condition dropped:
  six fixtures.
- Explicit supertrait elaboration removed:
  nothing failed,
  so the elaboration was deleted.

## Clippy suppression

`src/declaration.rs` carries the only suppression,
`#![allow(clippy::disallowed_methods)]`.
The repository root `clippy.toml` bans `std::result::Result::unwrap` through `disallowed-methods`.
`dylint_linting::declare_late_lint!` expands to `dylint_library!`,
which calls `CString::new(..).unwrap()` (`dylint_linting` 6.2.0,
`src/lib.rs`,
line 262).
Clippy's configuration has no per-path exemption,
and a package-level `clippy.toml` would replace the fleet-wide ban for this whole package.
An allow attribute on a macro call is itself rejected by rustc (`unused_attributes`),
so the macro sits alone in its own module and the module carries the allow.

## Stage-1 measurements on the IDE

Lint run over a disposable copy of the 67-closure IDE snapshot with `cargo dylint --all-targets`,
offline,
on the dated nightly.
62 of the 67 listed closure sites pass.
The five reported:
`src/native/language/actions.rs:134` (let-bound),
`src/native/minimap_build.rs:399` (now `src/native/minimap_worker.rs`, renamed on 2026-10-21; the `catch_unwind` sits
near line 268 there) and `src/native/tile_build.rs:140` (`catch_unwind(AssertUnwindSafe(..))`,
whose callee is a tuple-struct constructor and which has no `'static` bound anyway),
and `src/syntax_loader.rs:188` and `:311` (`tree_house` callbacks taking `impl FnMut` without `'static`).
All 37 Slint `on_*` setter sites resolve and pass.
The lint also reports 78 closures outside the 67:
46 under `tests/`,
27 in `*_tests.rs` files,
and 5 in `src/bin` (all inside macro arguments such as `json_value!`,
which the syntax-only rule cannot see).
Wall time of `cargo check --all-targets` after touching the crate:
median 2.72 s (n = 10,
standard deviation 0.16 s);
with `cargo dylint` and the release library:
median 3.12 s (n = 10,
standard deviation 0.22 s);
load average stayed below 8.

## Stage-2a re-run on the shared nightly

On 2026-10-18 the tests,
Clippy,
`cargo fmt --check` and the IDE run were repeated on `nightly-2026-10-18`,
the nightly that `mise.lock` resolved after `rustc-dev` joined the shared toolchain,
with no source change.
All 13 tests passed.
The IDE run reported the same 84 warnings at the same locations as stage 1:
62 of the 67 listed sites pass and the same five are reported.
Dylint's driver cache in `~/.dylint_drivers` held 42 MB after the first run.
The notes on offline fetching and on the IDE build directory are in `doc/troubleshooting/dylint-library-tests-and-shared-build-dir.md`.
