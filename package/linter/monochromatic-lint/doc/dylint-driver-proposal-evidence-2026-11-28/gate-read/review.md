This is a same-vendor gate review (a second pass by another Claude instance, not an independent review), unapproved and non-final.

# Gate review of the Dylint driver proposal for issue 6643

Drafted from 2026-11-27T21:52:54-05:00; this revision written after the clock read of 2026-11-27T22:35:09-05:00 (`date --iso-8601=seconds`).

Subject: `doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md` as `main` holds it.
The file is identical at merge commit `e63f6d9226` and at current `main`, `68bac882d77e9ad51730c2c5a372c2484d6e0bfb` (read at 22:30:28-05:00; `main` moved twice while I worked),
and that merge added only this file (`git diff --stat e63f6d9226^1 e63f6d9226`: one file, 1080 insertions).
Between `e63f6d9226` and that `main`, the cited paths changed only in the tail of the linter `README.md` (after every cited passage),
some Markdown-fix sources, and `toolchain.lock.json` (linter and `mpm-shadow` versions and the digest; the cited rows at lines 493, 568 and 579 did not move).
Line numbers below are the proposal's own lines unless a path is given.

## Counts by severity

- High: 4 (F1, F2, F3, F4).
- Medium: 6 (F5 to F10).
- Low: 6 (F11 to F16).

High means the finding could change which option the owner picks, or states a measured fact that is false.
Medium means the owner would choose with a distorted cost, a missing option, or a reason that does not hold.
Low means a citation, attribution or internal-consistency error that a reader could trip on.
The verdicts on the ranking's adjacent pairs are in their own section and are not counted again as findings.

## What I read

The whole proposal (1080 lines), then, whole where the claim depends on context:
`closure_step.rs`, `closure_cargo.rs`, `closure_driver.rs`, `closure_driver_probe.rs`, `closure_step_package.rs`,
`run_output.rs`, `run_command.rs` under `package/linter/monochromatic-lint/src/`, `closure_packages.rs` (head),
the linter `README.md` lines 370 to 630, `src/run_modes_rule_table.rs` 100 to 130, `src/closure_binary_tests.rs` 715 to 786,
the linter and closure-package `mise.toml`, the closure package's `README.md` 395 to 429, `Cargo.toml` and `.cargo/config.toml`,
`test.Containerfile`, `monochromatic-lint.config.jsonc` 1 to 120, `toolchain.plan.jsonc` 140 to 310, `toolchain.lock.json` 485 to 600,
root `mise.toml` 50 to 62, 615 to 712, 1020 to 1070 and 1222 to 1240,
`.github/workflows/advisory-core.yml` whole, `.github/actions/repo-tools-setup/action.yml`, `.github/actions/task-cli-setup/action.yml`,
`doc/decision/rust-closure-lint-dylint.md` whole,
`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md` whole,
`doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md` whole,
`doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md` 1 to 60,
`doc/planning/repository-linter-stays-at-zero.md` 1 to 30 and 95 to 110,
`doc/planning/linter-selective-anonymous-functions.md` 150 to 200 and 230 to 360,
issues 6643 and 6316 (`gh issue view`).
Upstream, at tag `v6.4.0` (commit `703f588db3e50ca3519bc9abea2eac2a9308a70d`, confirmed by `gh api .../git/ref/tags/v6.4.0`):
`dylint/src/driver_builder/mod.rs`, `internal/src/env.rs`, `README.md`, `docs/how_dylint_works.md`
(each fetched with `gh api repos/trailofbits/dylint/contents/<path>?ref=v6.4.0`),
plus, from the tag's tarball in my scratch folder, `internal/src/rustup.rs`, `internal/src/cargo.rs`, `internal/src/git.rs`,
`internal/src/clippy_utils/repository.rs`, `driver/build/extra_symbols.rs`, `dylint/build.rs`, `dylint/src/lib.rs`,
`dylint/src/library_packages/mod.rs`, `CHANGELOG.md`, `.github/workflows/release.yml`;
and the default-branch `README.md` for the `DYLINT_HOME` correction.
CI logs fetched with `gh run view <id> --log` (two runs read, see F4).
Read-only probes on this machine:
`rustup show active-toolchain`, `rustup toolchain list` and `rustup component list --installed` for both toolchains (all three at 21:58:23-05:00),
`type -a cargo` and `ls -la /usr/bin/cargo` (first read between 22:03 and 22:11, which printed no clock; read again at 22:30:37-05:00 with the same answer),
`ls -la .cache/toolchain-bin`, `du`, `ls ~/.dylint_drivers`, `git ls-files` counts, and `gh run` listings.

## Findings

### F1 (High): the options provide a driver for "the toolchain" without naming the directory segment, and three toolchain names are in play on this machine

Proposal lines 403 to 411, 619 to 626 (option A), 669 to 672 (option B), 797 to 803 (option E), 1062 to 1063, 1071.

Claim at 403 to 405: "The active one is the undated floating channel, which is the toolchain name in the skip message; the dated one is what the lock records."
Claim at 408 to 411: the size difference "is consistent with the dated one carrying `rustc-dev` and `llvm-tools` and the default carrying neither. That was not verified component by component, and it matters: the driver is built for the toolchain that is active in the library directory."
Claim at 619 to 623 (A): "produces the driver for the toolchain it was just installed beside ... run Dylint's own build once with the full toolchain triple in `RUSTUP_TOOLCHAIN` ... place the binary at `<driver root>/<toolchain>/dylint-driver`".
Claim at 1071: "what it proves is that the path works".

Established by source and by probe:

- `rustup component list --installed --toolchain nightly-x86_64-unknown-linux-gnu` (21:58:23-05:00) printed only
  `cargo`, `clippy`, `rust-std-x86_64-unknown-linux-gnu`, `rustc`, `rustfmt`.
  The dated `nightly-2026-11-16-x86_64-unknown-linux-gnu` also lists `llvm-tools`, `rust-src`, `rustc-dev`, and both Android `rust-std` targets.
  So the proposal's size-based inference is right (274M against 799M, both reproduced), and the floating default has neither `rustc-dev` nor `llvm-tools`.
- The linter's driver check asks `rustup show active-toolchain` in the library directory with `RUSTUP_TOOLCHAIN` removed
  (`package/linter/monochromatic-lint/src/closure_driver_probe.rs:162-193`, `.env_remove("RUSTUP_TOOLCHAIN")` at line 169),
  and mirrors upstream, which does the same:
  `internal/src/rustup.rs:16-21` (`self.env_remove(env::RUSTUP_TOOLCHAIN)`) and `:35-45` (`active_toolchain`),
  called for a `--path` library at `dylint/src/library_packages/mod.rs:409`, with the result put back into `RUSTUP_TOOLCHAIN` at `dylint/src/lib.rs:434`.
- The plan gives the shell the dated toolchain through the `rust` row's environment
  (`toolchain.plan.jsonc:214-216`, `"RUSTUP_TOOLCHAIN": "{version}"`; lock `"RUSTUP_TOOLCHAIN": "nightly-2026-11-16"`), which Dylint strips.
  That is why the skip names the floating toolchain.
- `rustc_dev_missing` asks `rustc --print sysroot` on `PATH` (`closure_cargo.rs:291-305`),
  and `.cache/toolchain-bin/cargo` is a symlink straight into the dated toolchain
  (`/home/user/.rustup/toolchains/nightly-2026-11-16-x86_64-unknown-linux-gnu/bin/cargo`).
  So the machine check in `closure_step.rs:202-205` looks at the dated toolchain and passes
  (the driver-gap skip the issue quotes is reachable only after that check returns `None`, `closure_step.rs:202-221`, unless `MONOCHROMATIC_LINT_CARGO` is set),
  while the toolchain Dylint names, the one the driver is filed under, is the floating one without the component.
- The recorded workaround files its binary under the short name `nightly-2026-11-16`
  (`doc/troubleshooting/dylint-driver-build-needs-a-rustup-proxy-cargo.md:160-167, 185`), the segment `dylint_testing` was compiled with,
  not the one `cargo dylint` computes; the proposal calls it "the recorded workaround" (lines 621 to 623, 670 to 672) for a different consumer.
- The container evidence had one toolchain, `nightly-2026-11-16`, so `rustup show active-toolchain` there named the dated one
  (proposal lines 551 to 553 and the driver path at line 566).
  This machine's state (a minimal floating default, the dated toolchain reached only through an environment variable Dylint strips,
  a non-proxy `cargo` first on `PATH`) was not reproduced.
- Upstream states the constraint: `docs/how_dylint_works.md:75` "Dylint tries to build the package with the same toolchain used to build the library".
  The proposal cites that sentence's lines (303 to 306: `:34-48`, `:73-78`) and does not apply it to the toolchain question.

Inference, not established (labelled so):
Dylint spawns `Command::new("cargo")` by name from `PATH` for the driver build, the library build and the check (`internal/src/cargo.rs:142-146`).
Here the first `cargo` is the shadow, which forwards to the direct dated link, and a non-proxy `cargo` ignores the `RUSTUP_TOOLCHAIN` Dylint sets.
So the compiler that builds the library and runs the check may be the dated one whichever name the driver directory carries,
while the driver is filed under, and rpath-linked to (`mod.rs:126`, `:132-136`), the toolchain `rustup which rustc` reports in the throwaway directory.
Whether those pieces fit on this machine is not known from the proposal or from anything I ran.

Consequence for the options.
No option says which directory segment a provider fills, who makes rustup report it, or which compiler builds the driver.
An option that files a driver under the dated name fills a directory the linter's check never reads here;
one that files it under the floating name depends on a toolchain without the components.
The records bear on the choice but the proposal does not put it to the owner:
`doc/decision/rust-closure-lint-dylint.md:51` approved "`rustc-dev` to the shared floating nightly",
`:63` "The package therefore has no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override",
`:66-68` "Still not approved: any other change to the root toolchain declaration, a pin of the nightly",
`:180-182` "the floating nightly stays (no pin)".
Putting the components on the floating channel matches that approval; setting rustup's default to the dated toolchain, or adding a `rust-toolchain.toml`, is a pin and does not.
Today's plan installs the dated name and leaves the floating default minimal.

What would settle it:
one measured run in a container that reproduces the machine's state
(floating default with no components, the dated toolchain with components, `RUSTUP_TOOLCHAIN` set to the short dated name,
a non-proxy `cargo` first on `PATH`; the repository's `cargo-shadow` if it can be had),
reporting which toolchain name `cargo dylint` computes, which driver directory it reads, which compiler built the library and ran the check, and whether the driver loads.
Then each option's design names the segment and who makes rustup report it, and the owner is asked the pin question.

### F2 (High): "this account has no rustup-proxy `cargo` on `PATH`" is false, and three conclusions rest on it

Proposal lines 412 to 420, 669 to 670, 842 to 851, 1001 to 1005.

Claim (lines 416 to 420): "`~/.cargo/bin` holds `cargo-binstall`, `lsd`, `maki` and `tokei`, and no `cargo`, so this account has no rustup-proxy `cargo` on `PATH`."
Used at line 848 ("this account has no rustup-proxy `cargo` on `PATH` (measured above)"),
line 669 ("On this host the plain warm command does not work"),
and line 1005 ("because its `cargo` is not the rustup proxy").

Source, read on this machine:
`type -a cargo` lists three candidates in `PATH` order:
`/home/user/Monochromatic/.cache/cargo-shadow-bin/cargo`, `/home/user/Monochromatic/.cache/toolchain-bin/cargo`, and `/usr/bin/cargo`.
`ls -la /usr/bin/cargo` prints `/usr/bin/cargo -> /usr/bin/rustup`, and `pacman --query --owns /usr/bin/rustup` says it belongs to `rustup 1.29.3-1`.
So a rustup proxy `cargo` is installed system-wide and is on `PATH`, behind the repository's shadow and the direct toolchain link.
The proposal looked only in `~/.cargo/bin`.
The 2026-11-25 record the proposal leans on makes the same false statement:
`dylint-driver-build-needs-a-rustup-proxy-cargo.md:113-115` ("this host has no rustup proxy in `PATH` at all") and `:204-206` ("this host installs no rustup proxies at all"),
so the arm it "could not be exercised" (`:204-206`) was never tried.

What this changes.
The disqualification of option F ("its disqualifying problem is measured on this machine") is about which `cargo` comes first, not about whether a proxy exists.
Using the proxy is not a one-variable change.
`MONOCHROMATIC_LINT_CARGO` replaces only the outer program (`closure_cargo.rs:36-49`, `:98-103`) and turns off the `rustc-dev` check (README lines 548 to 549);
Dylint then spawns `cargo` by name from `PATH` (`internal/src/cargo.rs:142-146`), so the proxy has to precede the shadow in the whole process tree, for the driver build, the library build and the check.
That is a design the proposal did not weigh, and it needs the components on whatever toolchain the throwaway crate's `rust-toolchain.toml` names
(`dylint/src/driver_builder/mod.rs:237-245`: `llvm-tools-preview`, `rustc-dev`), which F1 shows the floating default lacks.
The evidence in the 2026-11-25 record is also a `cargo test` run of `dylint_testing` (`:129-139`), not a `cargo dylint` driver build.
The same family records a successful `cargo dylint` warm on this repository's host on 2026-10-30
(`cargo-dylint-driver-pin-offline-after-upgrade.md:350-352`, "with the toolchain-exposed cargo (`.cache/toolchain-bin/cargo`)"),
on a host whose records use `/var/home/user`; that record does not say what `.cache/toolchain-bin/cargo` linked to then
(`:173` shows `.cache/toolchain-bin/cargo-dylint` linking into `~/.cargo/bin`, where rustup proxies live on such a host).
The proposal does not reconcile the two.

What would settle it:
rerun `type -a cargo`; then one measured `cargo dylint` driver build with `/usr/bin` ahead of the shadow on `PATH`, in the F1 container state.

### F3 (High): option D1 treats every skip alike, and six of the thirty selectable Rust packages skip for a reason no driver removes

Proposal lines 743 to 792, 784 to 787, 1015 to 1016.

Claim (lines 749 to 753): D1 is "a run that emitted any skip exits with a status of its own".
Claim (lines 784 to 787): "D1 also breaks every host run of the lint until a driver exists: that would be every Rust package on this machine, and every Rust pull request in CI".

Sources.
A skip has five causes in the code, and the proposal's list at lines 117 to 152 names them:
no `cargo`/`cargo-dylint`, no `rustc-dev`, a driver gap, a cold package cache, and a missing prerequisite
(`closure_step_package.rs:227-246`: `dependencies_missing` and `Readiness::Skipped` both push `skip_finding`).
`git ls-files '*Cargo.lock'` counts 39 tracked lock files;
nine of those directories end in `.fuzz` or `.bench` and are skipped by name (`closure_packages.rs:95`, README line 462), leaving at most thirty packages the rule can select.
`git ls-files '*monochromatic-lint.prerequisites.jsonc'` lists six manifests, all in lock directories of the thirty:
`package/file-manager/app`, `package/file-manager/variant-gtk-sticky`, `package/file-manager/variant-qt`,
`package/ide/app`, `package/music-player/android-app/rust`, `package/terminal/app`
(README lines 575 to 607: `pkg-config`, `qmake`, `ide-runtime-artifacts`, `libghostty-offline-inputs`, `android-ndk`).
Each of those skips on any machine that lacks gtk4, Qt, the NDK, the IDE runtime artifacts or the Ghostty offline inputs, whether or not a driver exists,
and the workflow as it stands installs only `pkg-config` as a system prerequisite (`advisory-core.yml:86-89`).
The configuration says the IDE and terminal trees have no finding left (`monochromatic-lint.config.jsonc:81-97`), so those packages were checked on some machine that had their prerequisites;
the skip is conditional on the machine, not permanent in the repository.
The owner decided that case on 2026-10-19: "A package whose prerequisites are missing on the machine is skipped, and the linter says so" (`rust-closure-lint-dylint.md:177-178`).

So D1 as designed is red, with a driver, for any run that selects a file in one of those six packages on a machine without their prerequisites, a hosted runner included.
It is not "until a driver exists".
The proposal's "What every option pays" and its owner questions never separate the machine-wide driver gap (the provider's responsibility)
from the per-package prerequisite skip (expected, decided).
A usable D1 is keyed on the skip cause, or opt-in per configuration block (F8, option J), and the owner needs the count (six of at most thirty) to weigh it.

What would settle it:
a design of D that names which of the five causes change the status, and a measured list, on this machine and on a hosted runner image, of the packages that still skip with a perfect driver.

### F4 (High): "the rule has never run in CI" is true for an earlier reason; the CI step cannot start the linter

Proposal lines 343 to 364, 649, 690 to 692, 785 to 788.

Claim (lines 359 to 364): "Because no `cargo-dylint` exists there, a CI run that reaches a Rust package skips it at `dylint_probe` ... So the rule has never run in CI either, and a green workflow never included it."

What I found.
- The last CI log I could retrieve in which `Static changed paths` ran the linter is run `41274002049` (event `pull_request`, created 2026-11-01T04:40:30Z).
  Its job `advisory core` log, step "Static changed paths" (log lines 795 to 812 of `gh run view 41274002049 --log`), printed
  `.github/CODE_OF_CONDUCT.md`, then `task-run: line 1: monochromatic-lint: command not found`, then `Error: task-cli: step exited 127`.
  The step reports conclusion `success` only because of `continue-on-error: true` (`advisory-core.yml:190-192`);
  the Enforce step fails the run on its outcome (`:262-266`), and the job's conclusion was `failure`.
  The linter never started, so `dylint_probe` was never reached.
  An 18-line excerpt of that step is kept beside this review as `ci-run-41274002049-static-step-excerpt.txt`;
  the rest of my scratch (the upstream tarball, the other logs, the issue copies) was deleted when I finished.
- At current `main` nothing in that job installs the linter.
  The setup action's `tools` input names `file-enforcer`, `mpm-shadow`, `cargo-shadow`, `git-policy-cli` (`.github/actions/repo-tools-setup/action.yml`),
  `package/monorepo-task/core-exec/src/repo-tools-setup.ts:138-140` rejects any other name, and the workflow passes `tools: file-enforcer mpm-shadow cargo-shadow`.
  `git log --since=2026-11-01` over the workflow and the action shows no commit that adds it (the newest, `3574042c16`, 2026-11-24, is about the container start sites ledger).
  I read no log of a run after the later commits in which the step ran the linter, so the log evidence is from 2026-11-01 and the rest is the configuration as it stands.
- The recent history is silent.
  `gh run list --workflow advisory-core.yml --limit 200` returns 200 runs, all `push`, spanning 2026-11-27T15:46:19Z to 2026-11-28T02:55:41Z (about eleven hours).
  For 199 the `advisory core` job was `skipped` by the gate (`advisory-core.yml:44-69`, "budgets: 2 per minute, 4 per hour, 8 per day" at line 61; the gate counts every run of the workflow, skipped ones included);
  one returned no conclusion (an empty `gh` answer, not read as skipped).
  The newest manual dispatch that ran the job (2026-11-25, run `45531170934`) printed "No changed static files" in that step.
  The other pull-request runs I tried (`41478019128`, `41419344331`, `41291093161`, `41278398681`, `41274345678`) returned no job or no log.
- `task-run` does not see the toolchain environment (`doc/troubleshooting/ci-steps-do-not-see-the-toolchain-environment.md:25-35`:
  "`task-run` is not mise ... neither output reaches a child process. The workflow therefore has to write both into the runner's own files itself").
  Options A and E export `DYLINT_DRIVER_PATH` through the lock's environment; nothing in `advisory-core.yml` would carry it to the lint step.
- The proposal does not size what a driver in CI would run:
  `lint:monochromatic-lint` depends on `fetch:rust-dependencies`, which runs `cargo fetch --locked` for all 39 locks (`mise.toml:1031-1051`, `:1057-1059`),
  and its own figure for one file's package is 89.65 s (line 927), against a 45-minute job (`advisory-core.yml:76`).

Consequence: every provider option, the first-ranked A included, delivers nothing to a pull request until CI can run the linter at all.
That prerequisite (extend the closed tool list, or install the `cargo:monochromatic-lint` archive row the lock already has, plus the toolchain components, plus the environment export) is shared by A, B, C, E and F.
The proposal tells the owner "A ... does nothing for CI" (line 649) and not that this holds for all of them, nor that the first thing missing in CI is the linter.
The scoring "for a pull request in CI" therefore depends on a CI change the proposal does not list.

What would settle it:
a current log of the step; the answer to how the job gets `monochromatic-lint` on `PATH`; for each option the list of workflow edits it needs.

### F5 (Medium): "the clone is not optional" is wrong for the channel named `nightly`, so the shared cost list and one trap are overstated

Proposal lines 914 to 924, 510 to 547, 960 to 967, 889 to 893.

Claim (lines 914 to 916): "The build also clones `rust-lang/rust-clippy` with `git`, and that clone is not optional: `dylint_driver`'s build script reads `clippy_utils/src/sym.rs` at a revision it picks from the channel (... `dylint_driver-6.4.0/build/extra_symbols.rs:51-99`)."
Claim (lines 889 to 893): these costs "attach to A, B, C, E and F, and no option avoids them."

Source, the same lines the proposal cites (`driver/build/extra_symbols.rs` at `v6.4.0`):

```rust
51  fn clippy_utils_has_extra_symbols(rustup_toolchain: &str) -> Result<Option<String>> {
...
55      if channel == "nightly" {
56          return Ok(None);
57      }
...
63      if date < EXTRA_SYMBOLS_SINCE {
64          return Ok(None);
65      }
```

and `:26-30`: on `None` the script writes an empty symbol list and returns.
The clone, the 88 s and 111 MB, the writable-temporary-directory trap and the `DYLINT_CLIPPY_REPO` detour (attempt 4) happen only when `RUSTUP_TOOLCHAIN` for the driver build names a dated nightly on or after 2025-05-06.
The container used `nightly-2026-11-16`, a dated channel.
A driver built for the channel named exactly `nightly`, the name this machine's rustup reports (F1), needs no clone.
Whether a floating-channel driver is acceptable is a separate question (its extra-symbol list is empty), but the cost list should say the clone depends on which name is chosen,
because that choice is the open question in F1.
The proposal's citation paths are also crate-relative (`dylint_driver-6.4.0/build/...`, `dylint_internal/src/...`) where its other upstream citations are repository paths at the tag;
at the tag these are `driver/build/extra_symbols.rs`, `internal/src/clippy_utils/repository.rs` and `internal/src/git.rs`.

What would settle it:
one container run of the driver build with `RUSTUP_TOOLCHAIN=nightly-x86_64-unknown-linux-gnu` showing no clone, and the same with the dated name showing the clone.

### F6 (Medium): option A's "environment the installer already writes" is mis-located, per worktree, and invisible to CI steps

Proposal lines 624 to 626, 636 to 637, 802 to 803.

Claim: export `DYLINT_DRIVER_PATH` "through the environment the installer already writes (the launcher's `config/mise/conf.d/30-toolchain-env.local.toml`, seen in the installed `mpm-shadow`'s own strings)".

Sources:
`package/mpm-shadow/cli/src/toolchain/scope.rs:42` `REPO_MISE_ENV_FILE_RELATIVE = ".config/mise/conf.d/30-toolchain-env.local.toml"`,
and `package/mpm-shadow/cli/README.md:785` "The repository scope writes `.config/mise/conf.d/30-toolchain-env.local.toml` in the worktree."
So the file is in each worktree (with a leading dot), written by `toolchain:install` per worktree, and not read by `task-run` (F4).
The repository documents this; the proposal took the path from binary strings and missed the documentation.
It matters for the E over B reason (verdicts below): "a second machine and a fresh worktree get findings without anyone remembering anything" is false for an environment file written per worktree after an install.

What would settle it: read `package/mpm-shadow/cli/README.md` around line 785 and restate A and E with the env file's real scope.

### F7 (Medium): the proposal reads the 2026-11-07 amendment as less than it says, and D2 conflicts with it too

Proposal lines 776 to 788, 1015 to 1016, 1030 to 1035.

Claim (lines 1015 to 1016): "D1 is the only part of D that conflicts with the 2026-11-07 answer".

The owner's words, as the record gives them (`doc/decision/rust-closure-lint-dylint.md:236-237`): "in their own words and not as one of the offered options: 'Fetch then warn.'"
The record's reading of them (`:239`, "This reads as two rules"), at lines 241 to 253:

- Line 233: "On 2026-11-07 a cold cargo cache skipped five packages and hid their findings behind a green run."
- Lines 243 to 249: "A skip that remains is reported as a warning-level finding of `rust/avoidable-closure`, at the package's `Cargo.toml`, naming the package and the command that fixes it. It is not a silent message with exit status 0, and it is not an error: it counts against `--max-warnings` like any warning, and `--quiet` and `--silent` hide it with the other warnings."
- Lines 251 to 253: "A machine-wide gap (no `cargo-dylint`, no `rustc-dev`) and a missing driver produce one such finding per package, so the report shows how many packages went unchecked."

Points:
- D2 (a code of its own, `rust/avoidable-closure-skipped`) conflicts with the record's reading ("a warning-level finding of `rust/avoidable-closure`"),
  with the code comment at `closure_step_package.rs:90-94` ("one wording keeps the findings searchable by the rule name"),
  and with README lines 487 to 489 ("A skip is reported as a finding of the rule `rust/avoidable-closure`").
  The proposal admits D2 "changes a public code string in a JSONL contract" (line 788) and then says only D1 conflicts (line 1015).
  Only the variant "the rule's code plus a field" avoids it.
- The record names a harm (a green run) and an exclusion (not an error) that pull in opposite directions.
  Owner question 2 (lines 1030 to 1031) labels "does a skip keep exit status `0` (the 2026-11-07 answer)".
  The record's sentence is "not a silent message with exit status 0", which describes the first implementation as too quiet;
  the lone-warning status `0` is a consequence of `exit_status` (`run_output.rs:59-87`), not something the owner chose.
  The owner should be shown both halves, not "exit status 0 is the answer", and be told that "Fetch then warn." is the owner's, the two rules the record's.
- D3, "one line at the end" (lines 760 to 764), is not placed on a stream.
  The linter's stdout is JSONL (`run_output.rs:111-115`); a non-JSON closing line there would break consumers,
  and the record already requires one finding per package "so the report shows how many packages went unchecked".

What would settle it: a restatement of D with D2 and D3 each marked for what they change in the recorded contract, and the amendment's four sentences quoted whole.

### F8 (Medium): options the proposal should have weighed

The seven options mix providers (A, B, C, E, F), a disclosure change (D) and a rejected design (G).
Six plausible options are absent.
Each is designed here only far enough to show its merit or its disqualifier; none was built or run.

Option H, a CI job that follows Dylint's own caching guidance.
Design: a job on the hosted runner that installs the lock's nightly with `rustc-dev`, `llvm-tools-preview` and `rust-src`,
installs `cargo-dylint` and `dylint-link` at the lock's versions, exports `RUSTUP_TOOLCHAIN` as the full dated triple so rustup, the throwaway crate and the linter agree,
restores and saves `~/.cargo/git/checkouts/`, `~/.dylint_drivers/` and `~/.rustup/toolchains/` keyed on the two versions,
and runs the linter over changed Rust files.
Source for the guidance: upstream `README.md:201-223` ("we recommend caching certain directories to avoid rebuilding libraries, rebuilding drivers, or redownloading toolchains"),
which the proposal quotes at lines 295 to 302 and does not turn into an option.
On a hosted runner the image's rustup provides the proxy `cargo` (not verified for this workflow, which also installs `cargo-shadow` and would have to keep it from taking the name `cargo` first),
so the F2 failure need not arise, and `git` and the network are present.
Merit: the only option that produces pull-request findings without the paused installer, and it matches what Dylint documents.
Disqualifiers to measure: nothing for the edit loop; the six prerequisite packages still skip (F3); the job's cost and the advisory gate; and F4, since the job needs the linter on `PATH` first.
What would settle it: one run of such a job on a branch, reporting the cold and warm times and the skips.

Option I, make rustup's active toolchain in the library directory the toolchain that has the components.
Design: the plan's `rust` row installs the floating channel `nightly` with `rustc-dev` and `llvm-tools` (what 2026-10-18 approved, `rust-closure-lint-dylint.md:51`),
and `RUSTUP_TOOLCHAIN` names it, so `rustup show active-toolchain`, `rustc` on `PATH` and Dylint agree and the driver directory the linter probes is the one a provider fills;
for the channel `nightly` the driver build skips the clippy clone (F5).
Disqualifier: it changes what `rust@nightly` means in the plan (it currently resolves to a dated string, lock `:495`), and the dated lock pin the plan gives the repository is lost.
What would settle it: the F1 container run, once with the floating channel carrying the components.

Option K, a `rust-toolchain.toml` in the library directory naming the dated toolchain.
Design: rustup's active toolchain in `package/linter/monochromatic-lint-closure` becomes the dated one with the components, so the driver directory carries the dated name the lock records and `rustc_dev_missing` and Dylint agree.
Precedent: stage 1 kept "a dated nightly in the package" (`rust-closure-lint-dylint.md:46-47`) and stage 2a removed it (`:63`).
Disqualifier: it is the pin the owner declined ("the floating nightly stays (no pin)", `:182`; "a pin of the nightly" not approved, `:66-68`), it names the toolchain a second time beside the lock (unless file-enforcer generates it from the lock), and the dated name triggers the clippy clone (F5).
What would settle it: the owner's answer on the pin, then the F1 container run.

Option J, a configurable severity for a skip.
Design: a rule option such as `skipSeverity` beside `library`, so a configuration block (or a CI-only configuration; `.github/actions/monochromatic-lint.config.jsonc` shows one exists for workflow files) can make a driver-gap skip an `error`,
leaving prerequisite skips at `warn`.
`exit_status` already returns `1` for an error finding (`run_output.rs:75-77`) and `--quiet` hides only warnings (`run_output.rs:97`),
so no new exit status is needed and a quiet run no longer looks clean.
Merit: smaller than D1, honours "a skip is a warning" by default, and answers F3 by scoping.
Disqualifier: today the severity is fixed in code (`closure_step_package.rs:90-94`), so it is a code and README change, and it needs the driver-gap skip to be told from the other skips (it can be, by `DriverGap`, `closure_driver.rs:140-150`).
What would settle it: a test over a stub `cargo` as the existing `closure_step` tests do, and the owner's view of an option beside `severity`.

Option L, pin `cargo-dylint` to an exact version.
This is option D of the 2026-10-30 record (`cargo-dylint-driver-pin-offline-after-upgrade.md:409-417`): replace `@latest` for the two Dylint tools (`toolchain.plan.jsonc:236`, `:243`) with an exact request.
Merit: a bump becomes deliberate, so whichever provider is picked is rebuilt only when the owner moves the pin, which removes the recurrence cost the proposal charges to A, B and E ("three releases in three days").
Disqualifier: it provides no driver, and it ends `@latest` tracking the repository chose.
What would settle it: it is a complement to any provider; the owner's choice is whether to take it with one.

Option Z, the baseline: keep the provider manual, document it, take D2 or J and D3.
The proposal never states the status quo as an option, yet the documented provider today is the owner by hand:
README lines 513 to 517 ("which the owner runs once with network access, and again after each `cargo-dylint` upgrade"),
and it was done on 2026-10-30 with the owner's go-ahead (`cargo-dylint-driver-pin-offline-after-upgrade.md:335-358`).
Every other option's cost should be read against it.
Merit: no repository change in the provider path.
Disqualifier: it repeats on every `cargo-dylint` move.
What would settle it: a count of the moves in a month against the minutes each warm takes.

Also not weighed: the existing task `//package/linter/monochromatic-lint-closure:run` already runs `cargo dylint` with no `CARGO_NET_OFFLINE`
(`package/linter/monochromatic-lint-closure/mise.toml`, `[tasks.run]`), so it already warms the driver and the registry on a machine where the build works (F13 on the proposal's contrary statement).

### F9 (Medium): precedent in the repository was not engaged, and "never" overreaches

Proposal lines 213 to 215, 397 to 399, 1076 to 1080.

- `doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md:360-429` already weighed, for the same failure, the design the proposal now ranks first.
  Its option C is "give `toolchain:sync` ownership of the warm step ... Cons: sync would need network and write `~/.cargo` and `~/.dylint_drivers`, which it does not touch today" (lines 401 to 408),
  and it ranked the linter-side probe above it: "B > C > A > D" (line 363).
  The probe landed (lines 431 to 453).
  The proposal's A, B and D are the same family, and it does not cite that ranking or its reasons.
- The proposal says the driver directory "has never been created on this account" (lines 398 to 399).
  The repository records driver runs on its hosts: 516 findings of the rule at `136a40fe69` (`git log` date 2026-11-07T11:36:20-05:00; `doc/planning/repository-linter-stays-at-zero.md:19`),
  "on this machine a run restricted to one Rust file still took 89.65 seconds" (`:102-104`, which the proposal cites at line 927),
  and a driver rebuilt on 2026-10-30.
  The records use `/var/home/user`, this checkout is `/home/user`, so the machine changed; the proposal does not say what changed between a 2026-11-07 run that produced 516 findings and today.
  "Does not exist now" is supported (`ls /home/user/.dylint_drivers`: No such file or directory); "never" is not.
- The issue says "The rule exists and its findings are unknown for every Rust package here."
  The record has counts for the rule from 2026-11-07, and the proposal's follow-up (owner question 5, lines 1042 to 1049) does not use them as a baseline.

What would settle it: cite the earlier ranking and say why it no longer applies, and say what the 2026-11-07 run used as a driver.

### F10 (Medium): option E's artifact may not load on another machine, because the driver carries an absolute rpath into the builder's toolchain

Proposal lines 794 to 823, 805 to 810.

Claim (lines 805 to 810): E gives "an artifact whose digest the lock can record; the linter's check passes with no code change."
Con given (lines 818 to 819): "the asset is only valid with the exact `rustc-dev` of the toolchain it was built against".

Source (`dylint/src/driver_builder/mod.rs:126-136`, `v6.4.0`):

```rust
let toolchain_path = toolchain_path(package)?;
// smoelius: The commented-out code was the old behavior. It would cause the driver to have
// rpaths like `$ORIGIN/../../`... The new behavior causes the driver to have absolute rpaths.
let rustflags = format!("{} -C link-args=-Wl,-rpath,{}/lib ", env::var("RUSTFLAGS").unwrap_or_default(), toolchain_path.to_string_lossy());
```

with `toolchain_path` the ancestor of `rustup which rustc` (`internal/src/rustup.rs:59-72`), that is `<builder's rustup home>/toolchains/<name>`.
A driver built on a release runner carries `<runner home>/.rustup/toolchains/<name>/lib`, which does not exist on this machine (`/home/user/.rustup/...`).
Whether the loader then finds `librustc_driver` by another route (for example a search path that `cargo` sets for the workspace wrapper) is not shown by anything I read.
So E's "no code change" and "one build per pair" are unexamined: an asset may be usable only where the toolchain sits at the builder's absolute path.
Option A builds in place and has no such gap.

What would settle it: `readelf -d` on a driver built in a container, then running it from a container whose home differs.

### F11 (Low): upstream statements at `v6.4.0` that the file does not bear out

- Lines 317 to 319: "`dylint_drivers` returns `DYLINT_DRIVER_PATH` when it is set and a directory, and otherwise `<home>/.dylint_drivers`".
  Source `dylint/src/driver_builder/mod.rs:70-73`:
  `if let Ok(dylint_driver_path) = env::var(env::DYLINT_DRIVER_PATH) { let dylint_drivers = Path::new(&dylint_driver_path); ensure!(dylint_drivers.is_dir()); Ok(...) } else { ...home... }`.
  A set variable that is not a directory is an error (`ensure!`), not a fallback to the home directory, and no `README.txt` is written in that branch.
  This matters for A and E, which export the variable: a not-yet-created directory makes `cargo dylint` fail while the linter's check reports only "driver absent"
  (`closure_driver_probe.rs:112-115` does not test `is_dir`), and the printed warm command then fails the same way.
- Lines 328 to 330: "`build` compiles that crate quietly". Source `mod.rs:143-145`: `.quiet(opts.quiet)`; it is quiet only when asked.
- Lines 322 to 323 cite "lines 18-24, 69-91"; the `README_TXT` constant is at `mod.rs:19-25`.
- Lines 436 to 437: "Dylint's own estimate of the cache this needs is 42 MB". Source `package/linter/monochromatic-lint-closure/README.md:427-428`:
  "Dylint's driver cache in `~/.dylint_drivers` held 42 MB after the first run" is this repository's measurement, not a Dylint estimate.
  The proposal's own driver measure (43,970,800 bytes, 16 MB "as `du` counts it", lines 567 to 570) is left unexplained;
  a directory holding a 44 MB file reports less than the file only on a compressing or sparse filesystem.

What would settle it: edit the four statements to the quoted source.

### F12 (Low): repository citations that are off

- Lines 237 to 241: "`dylint_drivers_directory` reads `DYLINT_DRIVER_PATH` or `<home>/.dylint_drivers`".
  Source `closure_driver_probe.rs:69-71`: `dylint_drivers_directory(home)` only joins `.dylint_drivers`;
  `driver_site_from_environment` at `:110-125` reads the variable.
  The ranges `:69-104`, `:110-141`, `:162-204`, `:205-230`, `:232-334` run past their functions (`active_toolchain` ends at 193, `driver_version` at 218, `driver_unresolved` at 266, `resolve_probe` at 340).
- Lines 273 to 275: "`bootstrap` and `toolchain:install:global` as aliases of it (`mise.toml:56`, `:1227-1235`)".
  `mise.toml:56` is a comment inside the `[hooks]` text; `toolchain:install:global` (`mise.toml:684-694`) is a different task that runs `mpm-shadow install --global`.
- Lines 525 to 527 and 975 to 977: the anstyle-wincon behaviour is attributed to `README.md:496-500`;
  that passage is about package lockfiles and driver caching in general.
  The every-platform resolve of the throwaway crate is shown by `closure_driver_probe.rs:306-319` (no platform filter).
- Lines 686 to 689 cite sections "What the issue records" and "The intermittency" of the pin-offline record for "three releases in three days".
  There is no section "What the issue records";
  the release dates are at `cargo-dylint-driver-pin-offline-after-upgrade.md:118-121` and `:380-381`, and "The intermittency" is about why one agent saw a pass.
- Lines 372 to 377 cite `src/closure_binary_tests.rs:727-759` for the host test.
  The test is `726-781`, and its positive control (`:760-780`, an `--all-targets` run) is the part that shows the absence of test findings can be told from a probe that could not find them; the proposal does not describe it.
- Line 879: "`doc/planning/linter-selective-anonymous-functions.md:249-265` records why the alternatives lost".
  Those lines (`:248-266`) describe the costs of the Dylint route that was chosen
  ("Costs: `rustc-dev` in the root toolchain components, lockstep with the floating nightly ...").
  What the in-linter alternative cost is at `:150-168` ("14 pass ... 41 stay reported because the callee does not resolve"), `:304-360` (62 allow and 5 reports after changes; `BuildGenerated` "executes the build script of every crate in the graph ... unsandboxed"; 22 to 24 s against 0.063 s) and `:279-281`.

What would settle it: correct the ranges and names to the lines above.

### F13 (Low): statements that contradict the cited source or the proposal's own text

- Lines 381 to 384: `//package/linter/monochromatic-lint-closure:run` "requires a driver to exist already".
  The task runs `cargo dylint --path ... --manifest-path ...` with no offline setting, and upstream `get` builds a driver when none exists (`mod.rs:61-64`, which the proposal cites at lines 313 to 316).
  It is a warming command, as the closure README's "held 42 MB after the first run" (`README.md:428`) says.
- Lines 168 to 175: "nothing downstream can tell them apart by code, file or position".
  The proposal's own lines 162 to 163 say a skip's file is the package's `Cargo.toml` at its first byte (`closure_step_package.rs:111-112`), and a real finding of this rule is in a `.rs` file.
  A consumer can tell them apart by file today, which bears on how much D2 buys.
- Lines 368 to 371: "Two tasks in `package/linter/monochromatic-lint/mise.toml` and one in `package/linter/monochromatic-lint-closure/mise.toml` ... none of them is on any automatic path",
  then three bullets, of which one is the root `lint:monochromatic-lint`.
  That task is run by `advisory-core.yml:213`, so it is on an automatic path, and only one of the three is in the linter package's `mise.toml`.
- Step 5 of the walk (lines 85 to 91), "a `cargo` that exists and cannot be started fails the run with status `2`":
  a `cargo` that cannot be spawned at all is `ProbeFailure::Absent` and skips (`closure_cargo.rs:198-203`);
  status `2` is for a `cargo` that starts and exits 126 or 127, or fails without the evidence (`:204-232`).
  The later bullet (lines 124 to 129) states it correctly.
- Lines 643 to 645: the quoted reason is in issue 6316, but the comment (2026-11-23) is the coordinating session's relay ("the project answered"),
  and the question asked was whether to run `toolchain:install` in the main checkout to create the missing tool links.
  The proposal widens it to "the installer, which the owner has paused" and line 1029 to "the mpm-shadow work the owner has already named as its precondition" for adding a driver row.
  Whether the pause is in force on 2026-11-27, and whether it covers a plan change rather than a run, is not shown.

What would settle it: correct the five statements to the sources quoted; for the last, ask the owner whether the 2026-11-23 answer covers a plan change.

### F14 (Low): timestamps and counts inside the proposal

- Line 15 says the file was "written 2026-11-27T21:03:56-05:00", but it reports `--rules` output at 21:12 (lines 205 to 210), container runs to 2026-11-28T02:26Z which is 21:26-05:00 (lines 532 to 534),
  and its first commit `d1bc52752d` is dated 21:31:49-05:00.
- Lines 291 and 773 give "02:0x UTC" and "21:0x -05:00", and lines 186 and 395 give minutes only, where the repository's own rule asks for second-precise stamps.
- Lines 538 to 539: the clone returned `0` at 02:19:41Z, "the same command that had failed 40 minutes earlier".
  By the proposal's own times the failures were at 02:03Z to 02:08Z and 02:10Z, 10 to 17 minutes earlier.
- Lines 463 to 469: "Six containers in all ... five attempts at the rule and one probe", then five numbered sections listing four attempts and one probe.
- Lines 434 to 435: the proposal says the issue's quoted message has a "second half" naming `jiff-static`;
  the issue quotes the message only up to "which this linter does not do" and does not quote cargo's error.

What would settle it: stamp the document at the time of its last change and correct the counts to the sections.

### F15 (Low): option G is mis-described, and option C's "same shape" is not the same shape

- G (lines 877 to 878): "the linter has no rustc driver of its own, which is the machinery the driver provides".
  The linter carries a rust-analyzer-based semantic engine (`RustFileEngine`, `run_command.rs:180`), and the 2026-10-18 evidence for the in-linter route is measured (F12, last bullet).
  The conclusion (G last, not a candidate) stands; the stated reason and the cited lines do not.
- C (lines 717 to 718): "in the same shape as `package/linter/monochromatic-lint/test.Containerfile`".
  That file is a pinned stable image with `COPY vendor`, `CARGO_NET_OFFLINE=true` and no nightly, `rustc-dev` or Dylint (`test.Containerfile:1-14`);
  a driver image needs network at build and a nightly with components, so only the folder and task naming are shared.

What would settle it: restate G's reason from `linter-selective-anonymous-functions.md:150-168, 304-360`, and C's shape as a new image.

### F16 (Low): unlabelled inference

- Line 380: the host test "fails today for the same reason a host run skips" is inferred from the assertion at `closure_binary_tests.rs:743-749`; it was not run.
- Lines 521 to 523 (attempt 4): "because the local repository libgit2 clones from a repository directory has no working tree of its own" is stated as the cause with no probe,
  while attempt 5 honestly says "the cause of the refusal was not established".

What would settle it: label both as inference, or run them.

## Where the proposal holds

- What a skipped rule returns (lines 154 to 211) is borne out by the source:
  same rule code and fixed `warn` (`closure_step_package.rs:100-114`),
  merged into the findings list (`run_command.rs:203-204`),
  `exit_status` counting a warning toward `--max-warnings` and giving `2` only for a processing failure (`run_output.rs:59-87`),
  `--quiet` and `--silent` hiding it (`:96-101`),
  `--rules` showing `"options":["library"]` and `suppressible:false` (`run_modes_rule_table.rs:116-122`).
  The measured behaviours it reports (one JSONL record and status `0`, status `1` under `--max-warnings 0`, silence under `--quiet`) follow from that source; I did not rerun them.
- The driver check's mirror of upstream (lines 236 to 248) is accurate in substance:
  toolchain from `rustup show active-toolchain`, `<drivers>/<toolchain>/dylint-driver`, the `-V` comparison, the throwaway crate resolved offline.
- No plan row, lock field or task provides a driver (lines 250 to 286):
  I ran the same search over the same three files; the only extra hits are `clippy-driver` bins.
  `plan:184-217`, `:235-248` and `lock:493`, `:568`, `:579` match.
- No workflow names Dylint: `rg --ignore-case dylint .github` returned nothing.
- `advisory-core.yml:152-159` and `:190-214` are as described.
- The upstream `get`, `initialize`, `is_outdated` and the `DYLINT_HOME` correction (default-branch `README.md:217`; no `DYLINT_HOME` anywhere in the `v6.4.0` tree; `internal/src/env.rs:27-38`) are accurate,
  and so is the Caching in CI quotation (README `:203`, `:218-220`).
  The release binaries are built with `--features=dylint/__driver_from_crates_io` (`release.yml:88`), so a binstalled `cargo-dylint` pins a plain version, as the linter's probe assumes.
- The 516 and 508 figures (`repository-linter-stays-at-zero.md:19`) are quoted correctly, and the 89.65 s is at `:102-104`.
- The 2026-10-18 record is quoted correctly for option G (see the decisions section).
- The proposal labels its own unknowns (the AppArmor mechanism, the component inference) and states that nothing was built; the toolchain sizes (274M, 799M) and the absence of `~/.dylint_drivers` reproduce.

## What the earlier decisions say

2026-10-18 (`doc/decision/rust-closure-lint-dylint.md:5-9`):
"On 2026-10-18 the user chose, from three options, a rustc-driver lint through Dylint plus comment directives. The alternatives were a callee allow-list in `monochromatic-lint` and a type criterion inside `monochromatic-lint` with a build-generated load."
The proposal's G ("the option the owner rejected on 2026-10-18, from three, in favour of a rustc-driver lint", lines 872 to 874) says what this says.
The comment-directive half was later dropped as an adopted default (`:30-35`, `:183-184`).
The decision that governs every option is the later one, which the proposal does not cite:
"Mechanism A: `monochromatic-lint` runs `cargo dylint` for each Rust package itself" (`:169-171`), with "the floating nightly stays (no pin)" (`:182`),
and the same day's owner answers that put `rustc-dev` on the floating nightly (`:51`) and left "a pin of the nightly" unapproved (`:66-68`).

2026-11-07 (`:227-256`) is quoted in F7.
The proposal quotes the second bullet accurately at lines 778 to 782.
What it omits is the first bullet's remedy ("The repository's own lint tasks fetch the Rust dependencies first"),
the harm sentence (line 233), and the phrase "and it is not an error" (line 247), all of which bear on D.
The owner's own words were "Fetch then warn." (line 237); option F is a reading of "fetch" for a second cache, which the record does not state (it speaks of "the Rust dependencies"),
though README line 529 applies "Fetch then warn" to the skip paragraph that includes the driver.

## Verdict on each adjacent pair of the ranking

The ranking is A > E > B > C > D > F > G (proposal lines 981 to 1008).
I scored each pair by what the option delivers once it exists, for a person who edits Rust here (the edit loop) and for a pull request in CI.
Effort to build and whether a component is paused today I set aside as costs to state, not reasons to rank.

A over E (lines 984 to 988): the order holds on delivery, for a reason the proposal did not give, and the reason it gave is partly effort.
Once both exist, A builds the driver on the machine that uses it, so the toolchain path, the rpath and the `rustc-dev` match by construction.
E ships one binary, which carries an absolute rpath into the builder's toolchain directory (F10) and is valid only with the builder's exact `rustc-dev` (the proposal's own con),
so E may not load on a machine whose toolchain sits at another path, and no pair of versions can be used until the asset for it has been published.
That second point is a delivery property and is the proposal's real argument ("a bump of either version cannot leave ... a machine without a driver for a day").
"A has no pipeline to build first" is build effort and should be a stated cost.
Both options are the same for CI today: they need the linter on `PATH` (F4) and the environment export that `task-run` does not see (F4, F6); A also compiles at every install, E downloads.
F1 applies to both identically.
So A over E stands, on the rpath and the publish dependency, with the clause about the pipeline struck.

E over B (lines 989 to 992): the reason does not hold.
"No per-machine manual step": E is installed per machine; B's warm would be a `depends` of `lint:monochromatic-lint` (the shape of `fetch:rust-dependencies`, `mise.toml:1057-1059`), so nobody has to run it by hand either.
"No cache outside the repository": E installs under the user toolchain directory (`~/.local/share/monochromatic-dev/toolchain/...`, as every link in `.cache/toolchain-bin` shows) and exports through a per-worktree file (F6); B writes `~/.dylint_drivers`; both are outside the checkout.
"A second machine and a fresh worktree get findings": not for a per-worktree env file written after an install.
"B warms only the machine that ran the task and only until the next `@latest` move": E is also per machine, and stale at the same event (it needs a new asset first).
On delivery, B builds in place, so it has no rpath question (F10), and it reaches CI wherever the workflow supplies the components and the network, as E does.
E's one edge is that a machine downloads instead of compiling, a time saving the proposal does not measure (its own warm run took 178 s on a dated toolchain, and F5 says part of that is the clone).
The order should be B at least level with E.

B over C (lines 993 to 996): the direction holds for the edit loop; the stated reason is the weaker half.
C gives nothing in the edit loop (its own con, lines 729 to 733), B does.
"C gives them only for the packages one gate names" is a choice of the sketch (lines 714 to 717), not a property of C.
For CI, C (or option H) is the better fit than B's "skipped there by condition".
So B over C holds for the person who edits Rust and reverses for a pull request.

C over D (lines 997 to 1000): the reason does not hold.
"C produces findings somewhere, and D produces none anywhere" scores the provider on findings, but the issue's recorded harm is that a skip was read as clean ("a run 'without findings' was read as clean more than once", issue 6643).
C leaves that harm in place for every local run (C's own con at lines 730 to 731: "a local `mise run lint` still reports the rule as skipped unless option D is taken as well").
The proposal then says D2 "is worth taking whatever else is picked" (lines 1012 to 1014), which makes D a complement to any provider, not an alternative ranked below one.
On the metric of the recorded harm, D2 or J is at or above C.
For a pull request a distinguishable skip is what lets CI fail on a driver gap without a warning backlog blocking `--max-warnings 0`
(lines 790 to 792 leave open "whether CI may pass `--max-warnings 0` instead of a code change";
the backlog answers it unless it is gone: 508 warnings of this rule at `136a40fe69`, `repository-linter-stays-at-zero.md:19`,
and the configuration still says "Warnings until no finding is left", `monochromatic-lint.config.jsonc:33-34`; I did not recount today).

D over F (lines 1001 to 1005): the conclusion is plausible, the reason is partly false.
"F spends a fetch that still leaves this machine unable to build the driver, because its `cargo` is not the rustup proxy" rests on F2, which is false as stated.
F alone would still not deliver findings (it prefills a registry; the first `cargo dylint` under the shadow `cargo` that is first on `PATH` still meets the recorded `RUSTUP_TOOLCHAIN` failure; F1 applies),
so D above F stands, but for those reasons.
"D costs an owner amendment and no build" omits that D1 needs the cause-keyed design of F3.

F over G (lines 1006 to 1008): holds.
G contradicts the recorded choice and the 2026-10-19 Mechanism A answer, and the in-linter route has measured coverage and sandboxing problems (F12).
The stated reason ("F ... could be dropped without losing anything") is accurate.
The cited lines for why G lost are wrong (F12), which does not change the order.

## What the owner would need to know to choose, and is not told

- The toolchain state (F1): three names are in play, the floating default has no `rustc-dev` or `llvm-tools` (verified), no option says which directory a provider fills, and the pin question the 2026-10-18 and 2026-10-19 records leave open is on the path.
- A rustup proxy `cargo` exists at `/usr/bin/cargo`, and using it needs `PATH` order for the whole process tree (F2).
- The count that bounds what any driver can unlock: 39 lock directories, 9 exempt by name, 30 selectable, 6 of those with prerequisite manifests (F3).
- CI cannot start the linter at all: the last retrievable log of the step ends `monochromatic-lint: command not found` (F4); `advisory-core`'s `core` job was skipped by its own budget gate in 199 of the 200 listed push runs; CI steps do not see the toolchain environment.
- The clone cost depends on which toolchain name is chosen (F5), and a published driver asset may not load on another path (F10).
- The same installer-owned design was ranked below the linter-side probe in the 2026-10-30 record, with reasons (F9).
- The rule produced 516 findings on 2026-11-07 and a driver was warmed by hand with the owner's go-ahead on 2026-10-30, so "manual by the owner" is today's provider and the baseline (F8 option Z, F9).
- The cost of a whole-repository run (at most 30 packages) is unmeasured; the only per-package figure is 89.65 s for one file's package.

## What I did not check, and why

- The container evidence (attempts 1 to 5, timings of 88 s, 111 MB, 178 s, 398 s, the `/tmp` refusal and the AppArmor question):
  the logs are in the writing agent's scratch folder, which I was not given, and I may not run containers.
  I checked that the numbers are internally consistent and that the failing step names match upstream source
  (`extra_symbols.rs:94` is `assert!(status.success())`, which the panic message quotes), not that they happened.
- Claims about the cargo registry (`jiff-static` 1.0.3 absent, the five `dylint*` crates cached, `env_logger-0.11.12`'s manifest): they sit under `~/.cargo`, which I am not to read.
- A rerun of the linter on `package/wg/cli-quicker-exempt` (status `0`, `1` under `--max-warnings 0`, silence under `--quiet`):
  a run starts `cargo dylint --version` and writes a scratch crate under `~/.cache`, and I may not run `cargo dylint`.
  I verified the behaviour from `run_output.rs`, `run_command.rs` and `closure_step_package.rs` instead.
- Any run of `cargo dylint`, a driver build, or `readelf` on a driver (F1, F2, F5 and F10 each name the measurement that would settle them).
- CI behaviour at current `main`: my log evidence is the 2026-11-01 run; I read no log of the step linting Rust after it, and five other pull-request runs returned no job or log.
- Who answered in issue 6316: the comment is the coordinating session's relay, and I have no other source.
- The upstream tree was read only where the proposal or the toolchain question points; I did not read all of Dylint.
- I did not open `doc/planning/closure-lint-everywhere.md` or the other documents the proposal does not cite.
- The probes in F1 and F2 are one shell's view on one machine at one time (21:58 to 22:31-05:00 on 2026-11-27).
  `.cache/toolchain-bin` was relinked at 22:03 by another session during this review; the `cargo` link target I recorded was read after that.

Same-vendor second pass; nothing in `/home/user/Monochromatic` was changed, no build, test, container, installer or `cargo dylint` was run.
