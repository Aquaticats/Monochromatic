# Rust nightly always latest

Status: the lock moved to the newest nightly and was verified (2026-11-07T05:22:22-05:00);
the proposal for keeping it latest is open and needs the owner's choice.

## Owner's rule

Recorded 2026-11-07 (owner, verbatim):
"Rust nightly should always be latest."

This note takes the rule at its plain meaning:
the `rust` row of `toolchain.lock.json` follows the newest published nightly,
and a person or a job moves it without being asked.
Whether "latest" tolerates a short lag when the newest nightly breaks a package is the one open question below.

## What was found before the move

- The lock row was `"request": "rust@nightly"` resolved to `nightly-2026-11-01`,
  with `RUSTUP_TOOLCHAIN` set to the same name.
  The lock had last changed on 2026-11-04T17:04:26-05:00, and nothing moves the Rust row automatically.
- No workflow schedules `toolchain:update`.
  The release workflows (`.github/workflows/crate-release.yml`, `.github/workflows/cargo-publish.yml`)
  run `rustup toolchain install nightly` and so already float to the newest nightly on every run,
  unverified.
  Only the developer-machine lock lags.
- `nightly-2026-11-01` is the one broken nightly between 2026-10-28 and 2026-11-07 for zbus
  (rust-lang/rust issue 165263, fixed by revert PR 165266, present from the 2026-11-02 nightly;
  the study is `doc/research/wrapper-git-free-jobs-and-published-crates.md`, commit `7956f6c4c9`).
  Keeping a stale pin therefore cost this repository real builds:
  `zbus` failed with E0391 ("cycle detected when computing when ... needs async drop") on the pin.

## What was done

### The newest nightly

- Newest published nightly at 2026-11-07: `nightly-2026-11-07`,
  `rustc 1.101.0-nightly (c980177d8 2026-11-06)`, `cargo 1.101.0-nightly (111443f3d 2026-11-05)`.
  Read from `https://static.rust-lang.org/dist/channel-rust-nightly.toml` (`date = "2026-11-07"`);
  the dist directory for 2026-11-08 answers 404.
- The manifest lists as available for `x86_64-unknown-linux-gnu`:
  `clippy-preview`, `rust-src`, `llvm-tools-preview` and `rustc-dev`;
  and `rust-std` for `aarch64-linux-android` and `x86_64-linux-android`.
  The manifest marks `llvm-tools-preview` unavailable for the two Android targets;
  that component belongs to the host, so the lock row (host components plus two target standard libraries) is whole.
- After `toolchain:install`, `rustup component list --installed` for the new toolchain shows
  clippy, llvm-tools, rust-src, rustc-dev, rustfmt, rust-docs, and the three standard libraries.

### The lock move

- Path: `mise run toolchain:update rust` in a throwaway worktree
  (`.claude/worktrees/rust-latest-nightly`, branch `chore/toolchain-rust-latest-nightly`, from `d1a9c90cc2`).
  `mpm-shadow update` with a tool id moves only that tool (`package/mpm-shadow/cli/README.md`, "Updating named tools");
  it reported one tool moved.
- Lock diff: the `rust` row's `version`, `packageId` and `env.RUSTUP_TOOLCHAIN`,
  and the file's `digest`, which any lock change moves.
  No other row and no plan file changed, so there was no unrelated drift to report.
- Commit `df38060025` (2026-11-07T04:31:02-05:00) holds the lock change alone.
  Its message says "not yet verified" because it was written before the checks;
  the checks below are the verification, and commits are not amended.
- `update` writes the lock and installs nothing, so `mise run toolchain:install` followed
  (one installed, the rest unchanged) and rewrote the worktree's `RUSTUP_TOOLCHAIN` fragment.
  `mise exec -- rustc --version` printed `c980177d8 2026-11-06` before any task ran.

### Verification

Each Rust package's own mise tasks ran one at a time in a separate unit with a memory, CPU, task and disk-weight cap.
The new nightly ran every task.
The old nightly (`nightly-2026-11-01`, in a second worktree at `d1a9c90cc2`) ran every task that failed on the new one,
because a package that passes on the new nightly cannot be harmed by the move.
Logs are in `/var/home/user/temp/agent/rust-nightly/logs/`.

Two facts about the gates change what a pass means:

- The container gates do not use the lock's nightly.
  The git-policy gate (`package/git-policy/cli:native:test:container`) compiles with pacman's `rust`
  (`package/git-policy/cli/native-test-base.Containerfile`); it ran anyway: 5051 tests passed on the new worktree.
  The `nested-wayland-session` cargo tasks fall back to a container because this host lacks the wayland-server,
  xkbcommon and libdrm development files, and `package/ide/app` builds on the `terminal` image.
  Those images install rustup with `ARG RUST_TOOLCHAIN=nightly-2026-10-29`, a hand-kept copy of the lock's version
  that is already stale (the lock said `nightly-2026-11-01`), so their results are the same on either lock.
  The same default sits in `package/terminal/app/Containerfile` and `Containerfile.release`,
  `package/nested-wayland-session/cli/Containerfile`, `package/music-player/desktop-app/Containerfile` and
  `Containerfile.windows-gnu`, `package/forbidden-strings/core/mutants.Containerfile`,
  and as `PINNED` in `package/cargo-shadow/cli/src/toolchain_tests.rs`.
- The host tasks use the lock's nightly through the mise environment fragment; those are the evidence.

Passed on the new nightly (host tasks, `lint:clippy` or `lint` plus `test` unless noted):

- `package/git-policy/cli` (`native:check`, `native:clippy`, `native:test:host`)
- `package/linter/monochromatic-lint` (`lint:types`, `test`), `package/linter/monochromatic-lint.fuzz` (`test`)
- `package/cargo-shadow/cli`, `package/mpm-shadow/cli`, `package/pnpm-shadow/cli`, `package/search-shadow/cli`
- `package/forbidden-strings/core`, `cli`, `cli.fuzz`, `core.bench`, `core.fuzz`
- `package/file-enforcer/cli`, `package/file-enforcer/core-rust`
- `package/browserslist/core-rust`, `package/catalog-specifier/core-rust`, `package/dependency-catalog/core-native`
- `package/jsonc-edit/core-rust` and its fuzz crate, `package/xml-edit/core-rust` and its fuzz crate
- `package/codex-plugin/bash-limit`
- `package/music-player/truepeak-core` and its bench crate, `package/music-player/desktop-app` (`test`)
- `package/music-player/android-app` (`lint:clippy`, through `cargo ndk`, so both Android targets of the lock row)
- `package/file-manager/app`, `variant-gtk-sticky`, `variant-qt`
- `package/wg/cli-quicker-exempt` (`lint:clippy`)

Failed on both nightlies, for a cause unrelated to the move:

- `package/linter/monochromatic-lint:lint:rustfmt`: the same rustfmt differences in the same count on both.
- `package/wg/cli-quicker-exempt:test:unit`: the same test,
  `update_entry_points_tests::source_tests::listed_mpm_shadow_verbs_match_the_current_grammar`
  (a source-text check against the mpm-shadow grammar); the run stops there with the rest not run on either.
- `package/git-policy/cli.fuzz:test`: the same E0432, unresolved import `git_policy_cli::owner_lock_record`.
- `package/ide/app:test` (container on the 2026-10-29 nightly image): the same single failing test,
  `ide::change_watch recovery::queue_overflow_requests_a_full_reread`.
- `package/music-player/desktop-app:lint:clippy`: five code lints
  (`shadow_unrelated` at `src/theme_roles_declaration.rs:320`, two `assertions_on_constants`,
  two `needless_borrows_for_generic_args`) that `nightly-2026-10-28` reports identically in a scratch run.
  On the old pin the task never reached them because zbus failed first.
- `package/terminal/app` (`lint:clippy`, `test`): the new nightly gets past zbus and stops in the build script of
  `libghostty-vt-sys`, which runs `git checkout` that the Git wrapper's `linked-worktree-only` guard refuses;
  `nightly-2026-10-28` stops at the same place.
  Whether the rest of the package compiles on the new nightly is not shown.
- `package/linter/monochromatic-lint-closure` (`lint`, `test`): see "What breaks on latest".

Fixed by the move (failed on `nightly-2026-11-01`, passes on `nightly-2026-11-07`):

- zbus 5.19.0 in `package/nested-wayland-session/cli`: `cargo check --package zbus` on the host fails with
  E0391 on `nightly-2026-11-01` and finishes on `nightly-2026-11-07`
  (the mise tasks of that package run in a container, so this was run as a plain cargo command in the package,
  with a private build directory).
- zbus 5.19.0 in `package/music-player/desktop-app` and 5.16.0 in `package/terminal/app`:
  E0391 in the `clippy` task on the old pin, compiled on the new.
- `package/music-player/android-app` (`lint:clippy`): E0391 on `turso_parser::ast::Select` on the old pin,
  passes on the new.
  This package was not on the list of suspected zbus packages; the same compiler cycle reaches `turso`.
- `package/nested-wayland-session/cli` in a scratch image built with `RUST_TOOLCHAIN=nightly-2026-11-07`
  (the package's own container flags): `rustc` reports `c980177d8 2026-11-06`, `cargo clippy --release --all-targets`
  with warnings denied finishes, and `cargo nextest run` passes all of its tests.
- zbus 5.19.0 in `package/ide/app`: the same plain `cargo check --package zbus` fails with E0391 on
  `nightly-2026-11-01` and finishes on `nightly-2026-11-07`.

Not run: Windows and macOS tasks (owner: ignore them until told), `test:host`/mutation/e2e/inspect tasks that need a
desktop session or hours, and the `monochromatic-lint-closure` host run against another package.

### What breaks on latest

Only `package/linter/monochromatic-lint-closure` is affected, and it already failed on the old pin.

- On `nightly-2026-11-01`: one error, E0797 "base expression required after `..`",
  from `dylint_linting::declare_late_lint!` in `src/declaration.rs`.
- On `nightly-2026-11-07`: that error plus E0277,
  "`CallerBoundsAccessor<TyCtxt<'_>, _, _>` is not an iterator" at `src/static_bound.rs:105`
  (`environment.caller_bounds()` no longer returns an iterator).
- A scratch fix on latest (not committed; saved as `/var/home/user/temp/agent/rust-nightly/closure-fix-on-latest.diff`):
  `#![feature(default_field_values)]` in `src/lib.rs` (the compiler's own suggestion, for the macro expansion),
  and `environment.caller_bounds().all_clauses(cx.tcx).iter()` with `use rustc_middle::ty::CallerBoundsIterator;`
  in `src/static_bound.rs`.
  With both, `cargo check --all-targets` and `cargo clippy --all-targets -- --deny warnings` finish clean on
  `nightly-2026-11-07`; the following `cargo fmt --check` step shows differences in
  `src/test_scratch.rs`, which the change does not touch.
  The test run was not repeated with the fix.

Options for the fix on latest (all keep the lock on the newest nightly):

- Patch the package source as in the scratch diff, and update `dylint_linting` and `dylint_testing` when a release
  already carries the macro change (the package README requires both crates and the two tools on one Dylint release).
  Pros: smallest change, no lock lag.
  Cons: the `default_field_values` feature is an unstable feature gate that can move again.
- Wait for a Dylint release that fixes the macro, and patch only the `caller_bounds` call.
  `dylint_linting` 6.3.2 is published (the package asks for at least 6.2); the E0797 comes from rustc's own
  `declare_lint!` expansion, and whether any Dylint release removes the need for the gate was not tested.
  Pros: no feature gate.
  Cons: the package stays red until then, and it was already red.
- Retire the lint, or build it only when a person asks.
  Pros: removes the coupling to compiler internals.
  Cons: loses a lint, an owner decision.

Ranking: patch now > wait for Dylint > retire.
Patching now beats waiting because the package is red on the current lock anyway and the fix is two small edits that
were seen to compile; waiting beats retiring because the lint has an owner-approved purpose
(the package README, "Toolchain") and a Dylint release is the normal repair path for the macro half.

## Merge decision

Every package that passed on `nightly-2026-11-01` passes on `nightly-2026-11-07`.
Nothing passed on the old pin and fails on the new one, so the lock change merges.
The one package whose error set grows (`monochromatic-lint-closure`) was already failing.

Merged to `main` as `c047a8594e` (2026-11-07). After the merge, `toolchain:install` in the main checkout
moved its environment fragment, `rustc --version` there prints `c980177d8 2026-11-06`, and `toolchain:lock:check`
reports the lock current.

## Proposal: keeping the nightly latest

Nothing here is built.

### What the existing owners already give

- `mpm-shadow` owns the lock.
  `mpm-shadow outdated rust` reports whether the Rust row would move and exits 1 when it would;
  `mpm-shadow update rust` moves that row alone (`package/mpm-shadow/cli/README.md`).
  The root tasks `toolchain:outdated` and `toolchain:update` are thin wrappers, and
  `.github/actions/repo-tools-setup/action.yml` already installs `mpm-shadow` on a runner from the lock row, checksum
  verified.
- `package/crate-release/cli` already lands a bot commit on `main` through GraphQL `createCommitOnBranch`, which
  GitHub signs, because `main` requires signed commits.
  Its push command is tied to the release crate registry (watch paths, commit subject), so reuse means generalising
  it, not writing a second signer.
- Scheduled workflows with the same shape exist (`cache-prune.yml`, `codeql.yml`, `scorecard.yml`).
- The runner has no mise ("mise-free" CI, `doc/decision/mise-free-ci.md`), so the verification a runner can run is
  the crates that build without the desktop development files, with plain cargo.

### The verification a new nightly needs before the lock moves

- For each crate that builds on a runner, `cargo clippy --all-targets` and the tests with the candidate nightly
  (the host tasks above are the list; a file next to the lock would name them so that adding a crate is one line).
- The container gates do not count: they use pacman's Rust or the hand-kept 2026-10-29 default.
  The job would also move that default, or the images would read it from the lock, so the images follow the lock.
- The four zbus-bearing packages need the development files a runner lacks, so a runner check of
  `cargo check --package zbus` against their lock files stands in for them.
- Packages that already fail on the current lock are recorded as the baseline, so a candidate is judged by
  new failures only, the comparison done by hand in this note.

### What happens when a new nightly breaks a package

The case seen on 2026-11-01. The owner's rule says latest; there are two honest readings.

- Hold: the lock stays on the newest nightly that passed, the job opens an issue naming the nightly, the package and
  the error, and tries again on the next schedule; the 2026-11-01 break lasted one nightly.
- Move anyway: the lock goes to the newest nightly regardless, and the failing package gets a fix-forward issue.

Which reading is the owner's is the open question for them.
Hold is what would have saved this repository from the 2026-11-01 pin;
move-anyway is the literal rule.

### Options

Option A: a scheduled workflow that verifies, then lands the lock through the signed-commit path.

- Daily schedule plus manual dispatch, as `cache-prune.yml` does.
  Steps: install `mpm-shadow` through `repo-tools-setup`; `mpm-shadow outdated rust`, ending the run when nothing
  moves; `mpm-shadow update rust`; run the runner verification with the candidate; on success land
  `toolchain.lock.json` through a generalised push command; on failure open or update one issue and land nothing.
- Pros: nobody has to remember; runs when nobody is at the machine; reuses the lock owner, the signer and the
  schedule pattern; a failure leaves the repository where it was.
- Cons: the runner covers fewer crates than a developer machine (desktop and Wayland packages are outside it);
  the push command needs a generalisation; a workflow commit does not start other workflows
  (`doc/decision/releases-run-in-ci.md`, GITHUB_TOKEN rule), which is fine here because the verification is inside
  the same run.
- Verification before landing: the runner verification above.
  When a nightly breaks a package: the hold reading by default (nothing lands, an issue is filed, the next day
  tries the next nightly); switching to move-anyway changes only the failure branch.

Option B: the same update and verification as one mise task, run by a person or an agent on a routine.

- A root task (for instance `toolchain:rust:latest`) that runs `outdated rust`, `update rust`, `install`, and the
  full local task list, and stops before committing on a failure.
  The routine is the start of a work session or the existing release routine.
- Pros: verifies on the developer machine, including every host-built package and the dev libraries; no CI change.
- Cons: depends on someone running it; the lag is as long as the routine's gap, which is how the pin sat on a broken
  nightly; local runs are long (cold builds of the desktop crates took minutes each, see the logs).
- Verification before landing: the full local list.
  When a nightly breaks a package: the person sees it at once and chooses hold or fix-forward.

Option C: a scheduled workflow that lands the lock unverified, with the verification as a follow-up monitor.

- The literal reading of the rule: the lock always equals the newest nightly; a later job reports breakage.
- Pros: the rule holds at every moment; simplest to build.
- Cons: the repository can sit on a broken nightly with every local build failing, which is the 2026-11-01 case;
  a red `main` blocks other work.
- Verification before landing: none.
  When a nightly breaks a package: the lock stays on the broken nightly until a later one repairs it.

Option D: make the plan row float without a lock row for Rust.

- Install resolves the newest nightly on every machine and the lock keeps no Rust version.
- Pros: always newest by construction.
- Cons: breaks the lock's role as the one record of what was built (`doc/decision/mpm-toolchain-adoption.md`
  makes `mpm-shadow update` the only writer); two machines on different days build with different compilers;
  an owner decision, so it is listed and not recommended.

Ranking: A > B > C > D.

- A over B: both verify before the lock moves, and A also removes the dependence on a person remembering;
  B covers more crates, which is the reason B follows, and its coverage can be added to A by running the
  desktop-library packages in a job image later.
- B over C: B costs a routine and some wall time, but a verified lock is worth that; C lands unverified and would
  have repeated the 2026-11-01 pin.
- C over D: C keeps the lock as the record and only skips the check, while D removes the record and so the
  review surface (the lock diff) the repository rules rely on.

Recommended with the open question: A with the hold reading, and B kept as the way a person re-runs the full list.
Before building, the owner picks hold or move-anyway.

## Limits of this verification

- Tasks that need a desktop session, a device, hours or Windows and macOS were not run.
- `package/terminal/app` stopped in a build script on both nightlies, so its own source was not compiled.
- `package/wg/cli-quicker-exempt:test:unit` stops at its first failure on both, so the later tests were not run.
- The container gates compile with pacman's Rust or the hand-kept 2026-10-29 default and so do not exercise the
  lock's nightly; this is a gap in what those gates prove, recorded here, not changed.
  Moving those defaults to the new nightly was not done, and is part of the proposal.
- `package/linter/monochromatic-lint-closure` tests were not run with the scratch fix.
