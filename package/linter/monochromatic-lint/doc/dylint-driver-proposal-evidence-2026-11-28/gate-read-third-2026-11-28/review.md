This is a same-vendor gate read: a second pass by Claude Sonnet, the vendor that wrote the proposal,
and not an independent review.
It is unapproved and non-final; only the owner declares work complete.

# Third gate read of the Dylint driver proposal for issue 6643

Written after the clock read of 2026-11-28T07:35:29-05:00 (`date --iso-8601=seconds`),
finished after the reads listed in the closing sections.

Subject: `doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md`
as `git show e63bf58c19:<path>` holds it (2521 lines; every `:NNN` line number in this read is a line of that file
unless a path says otherwise).
This read changed nothing in the repository and nothing on GitHub.

## Counts

- High: 3.
- Medium: 8.
- Low: 6.

High means a finding could change which option the owner picks, or states as fact something the sources do not bear out.
Medium means the owner would answer with a distorted cost, a missing mechanism or a reason that does not hold.
Low means a citation, label or consistency error a reader could trip on.

## Short answer

The proposal is not ready to put to the owner.
Its first-ranked answer (I, the components on the floating channel) is described with a mechanism the repository's
installer does not have (High 1),
is ranked first on a measurement of a floating channel that a fresh machine cannot have,
while the measurement of a real floating channel fails (High 2),
and leans on an owner preference the ranking text says it does not weigh,
read from words that name the lock's nightly and not the rustup channel (High 3).
The fewest changes that would make it ready are at the end, under "Verdict".

## What I read and what I ran

Order of work, as the brief asks: I formed my own findings before opening any of the three earlier files.
They are on disk, with the order of the writes, in `keep/own-findings-written-before-reading-the-earlier-reads.md`
(two times in that file were first written as estimates and were corrected after the clock was read; the file says so).

Read before forming findings:
- The brief, whole.
- The proposal, whole, at `e63bf58c19`.
- The evidence folder at that commit
  (`package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/`, 165 files),
  extracted with `git archive` into scratch.
  I read whole: `README.md`, the `run.txt` of M7, M8, M10 and M11, the in-container scripts of M7, M8, M10 and M11,
  the first 90 lines of the M2 script,
  and the M2a run.
  Of the logs I read the small ones that carry the claims (component lists, sysroot and active-toolchain answers,
  driver tree, version and dynamic section,
  `execve` counts, sizes, the `E0603` and `E0463` errors of M2 and M5, the six finding lines of M7 arm D and M8 arm C);
  I did not open every `.err` and `.out` file.
  `machine/machine-state.txt` I used through the proposal's quotations of it and one grep for the Android targets.
- `package/linter/monochromatic-lint-closure/{Cargo.toml,.cargo/config.toml,README.md}`,
  the `rust` row of `toolchain.plan.jsonc`,
  `package/mpm-shadow/cli/src/toolchain/{model.rs (lines 330 to 420),install_rust_android.rs}`,
  `package/cargo-shadow/cli/src/lean_rustup_toolchains.rs` (header),
  `package/linter/monochromatic-lint/src/{closure_step_package.rs,closure_driver.rs,closure_driver_probe.rs}` (the
  passages named),
  `doc/planning/toolchain-always-latest.md` (lines 1 to 140 and the passages at 205 to 222, 385 to 400, 438 to 455,
  530 to 545, 1000 to 1015),
  `doc/planning/rust-nightly-always-latest.md` (the passages at 240 to 310),
  `doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md` (the Verification section and the passage on
  the default toolchain).

Read after forming findings: the second read (`review.md`, whole, 849 lines),
the second revision's report (whole, 273 lines),
and the first read (headings, its sixteen finding titles, "What the owner would need to know" and "What I did not
check";
I did not read its findings whole, because the brief asks for the second read's findings and the first read's own are
answered in the proposal's last reads section).

Read after the advisor's prompting, once the High findings were first written:
`doc/decision/rust-closure-lint-dylint.md` (the sections "Root toolchain change approved on 2026-10-18",
"Owner requirement of 2026-10-19" with its answers, "Known limits", "Amendment of 2026-11-07";
lines 36 to 91 and 150 to 258)
and the kept capture of issue 6643 (`issue/issue-6643.md`, whole).
The decision record changed the wording of High 1 and High 3.

Fetched today with the search-fetch tool and `gh` (reads only; each probe file names its URL and the clock):
the rustup book page "Overrides", rustup's `CHANGELOG.md`, rustup issue 4836,
Dylint's `README.md` and `docs/how_dylint_works.md` at `v6.4.0`,
the crates.io record of `dylint_linting`,
Dylint's pull request 2203 and its releases,
the Rust channel manifests of 2026-11-13, 16, 17, 26 and 28 and the latest nightly,
issues 6316 and 6643.

Ran, each one in its own `systemd-run --user --scope --slice=agent-work.slice` with `MemoryMax`, `CPUQuota`,
`TasksMax`, `IOWeight=1` and `RuntimeMaxSec`
(except the first three commands, see "Rules of the brief I broke"):
file copies and comparisons in scratch, `jq` and `grep` over saved fetches,
`curl` of seven Rust manifest URLs (the one for 2026-11-27 does not exist, status 22), `gh` reads, `git show`,
`git grep` and `git archive` (reads),
and `stat` of three files in the main checkout.
I ran no container, no `cargo`, no `rustup`, no `rustc`, no linter run, no build, no test, no editor.

Kept in `keep/`, one file per probe:
- `own-findings-written-before-reading-the-earlier-reads.md`
- `probe-copy-check-second-read-and-first-read-against-the-repository.txt`
- `probe-m7-m8-m10-m11-citations-against-their-logs.txt`
- `probe-nightly-manifests-what-rustup-resolves.txt`
- `probe-crates-io-dylint-linting-newest-versions.txt`
- `probe-gh-dylint-upstream-errors-and-releases.txt`
- `probe-gh-dylint-pr-2203-and-main-ahead-of-6-4-0.txt`
- `probe-gh-issue-6316-and-6643-read-only.txt`
- `probe-rustup-and-dylint-doc-excerpts-fetched-2026-11-28.txt`
- `probe-what-in-the-repository-moves-the-floating-nightly.txt`
- `probe-stat-of-toolchain-links-and-lock-in-the-main-checkout.txt`
- `probe-proposal-greps-for-the-claims-this-review-tests.txt`
- `probe-proposal-labels-and-heading-lines.txt`
- `probe-record-and-source-passages-quoted-at-e63bf58c19.txt` (the record and source passages this review quotes,
  with line numbers)

Does the repository's copy of the earlier reads match?
Yes, byte for byte.
`diff --recursive --brief` of the second read's `keep/` folder (`review.md` and eleven probe files)
against `gate-read-second-2026-11-28/` at `e63bf58c19` is empty,
and the `sha256` of `review.md` is the same on both sides (`161bbbed5715...`).
The first read's `review.md` (`71b636ad5095...`) and its CI excerpt also match their copies (`gate-read/review.md`,
`ci/ci-run-41274002049-static-step-excerpt.txt`).
Output: `keep/probe-copy-check-second-read-and-first-read-against-the-repository.txt`.

# Findings

## High

### High 1: Answer I names a mechanism the repository's installer does not have, and says "nothing in the repository" changes

Passages: Answer I (`:1359-1404`), question 1 (`:1897-1907`), the reply to the second read's M4 (`:2196-2200`),
"What changed since the rule last ran", item 3 (`:734-745`) and its reading (`:762-768`).

The proposal says
"the plan's `rust` row already carries that list,
so `toolchain:install` puts them back after an update",
"What changes for good: nothing in the repository",
"for a channel that moves daily",
and, in the reply to the second read's M4,
"Answer I names who installs the components (the plan's row)".

Evidence, all read at `e63bf58c19`:
- `package/mpm-shadow/cli/src/toolchain/model.rs`, `expand_version`:
  "A copy of the declaration with `{version}` replaced by `version` in `packageId` and in every `env` value."
  `rustup_channel()` returns the `packageId`.
- `package/mpm-shadow/cli/src/toolchain/install_rust_android.rs` runs
  `rustup toolchain install --no-self-update <channel>`
  and then `rustup component add <name> --toolchain <channel>` for each declared component, with that `channel`.
- `toolchain.plan.jsonc`, the `rust` row, comment:
  "Floating nightly channel, resolved to the dated toolchain string;
  the pin rides in the MPM rustup package id because that adapter keeps only the id."
  Its `packageId` and its `env.RUSTUP_TOOLCHAIN` are both `{version}`.
- The lock names that dated string: `"version":"nightly-2026-11-16"`, `"packageId":"nightly-2026-11-16"`,
  `env.RUSTUP_TOOLCHAIN` the same
  (`keep/probe-stat-of-toolchain-links-and-lock-in-the-main-checkout.txt`).
- The host's state is what that code produces: the dated toolchain holds `rustc-dev`, `llvm-tools`,
  `rust-src` and the Android targets,
  the floating `nightly` holds none of them (the proposal's own machine-state section, `:619-637`).
- Nothing in the repository moves the floating channel.
  `git grep` for `rustup update` outside `doc/` and Markdown finds no hit;
  the only installs of a floating `nightly` are workflow lines `rustup toolchain install nightly --profile minimal`
  (17 in `.github/workflows/cargo-publish.yml`, one in `crate-release.yml`)
  (`keep/probe-what-in-the-repository-moves-the-floating-nightly.txt`).
  The lean cleaner's header says "The repository installs a dated nightly per `toolchain:update` and rustup never
  removes one" and "Stable releases and the channel names are never touched."
- The record the proposal reads as the earlier working arrangement was a frozen floating channel.
  `doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md` (Verification section,
  and the passage on the default toolchain) says the driver directory was
  `nightly-x86_64-unknown-linux-gnu`, "the rustup default, `rustc 1.101.0-nightly (19833af72 2026-10-20)`,
  not the locked `nightly-2026-10-30`".
  The floating default was ten days behind the lock, and the harness set `RUSTUP_AUTO_INSTALL` to `0`.
  So the 2026-10-30 arrangement that "worked" was a floating default ten days behind the lock, with the components present
  (how they got there is not recorded in what I read), and nothing in the repository moves such a default.
- The second read's Medium 4 had already said it:
  "the plan's `rust` row installs the dated toolchain, not the floating default (`toolchain.plan.jsonc:184-217`),
  so no existing part does it."
  The revision answered with the opposite claim.

Why it matters: the owner's yes covers only what the option's text stated.
A yes to I as written is a yes to "no change, the installer already does it", and the installer does not.
The owner would be choosing a mechanism that needs new work (a new row or task that adds components to the floating
channel and a decision on who moves that channel),
and the option's cost line ("nothing in the repository") is false.
The reading in "What changed" (the reinstall removed the floating channel's components,
so I restores the 2026-10-18 arrangement) does not match the decision record either.
In `doc/decision/rust-closure-lint-dylint.md` (section "Root toolchain change approved on 2026-10-18")
the "shared floating nightly" that received `rustc-dev` is the `rust` entry of the lock:
"Changing the component list changes the locked options of the `rust` entry,
so refreshing the lock also moved the floating nightly from `nightly-2026-10-17` to `nightly-2026-10-18`."
That is a lock-resolved dated nightly, not the rustup channel named `nightly`.
So the proposal's "floating channel" (a rustup channel name) is not the record's "floating nightly",
and a yes to I is not a return to the arrangement the owner approved.
(I read that record after High 1 and High 3 were first written; it changed the wording of both. Passages:
`keep/probe-record-and-source-passages-quoted-at-e63bf58c19.txt`.)

How sure: high (source and the host's state; the second read cited the same row).
What would settle it: reword I to say who adds the components to the floating channel and when,
and who updates that channel;
or, if the intent is that the plan's row install the floating channel, say so,
because that is a change to the installer and the lock.

### High 2: the first-ranked answer is ranked on a state a fresh machine cannot reach, and the criterion's own words fail for it

Passages: "Decision 1, the toolchain" criterion and order (`:1776-1810`), M8 (`:893-956`), "I over K1" (`:1793-1797`).

The criterion is
"findings the rule produces on a machine whose `cargo` and `rustc` are the rustup proxies,
with no exported variable and no hand step,
which is the state a fresh machine and a runner have."
The order is I > K1 > K2 because I "delivers 6 findings (M8 arm C)" and K1 only with the variable exported.

Evidence:
- M8's floating channel is not a rustup install of `nightly`.
  `scripts/m8-in-container.sh` runs `rm -rf` on the floating toolchain folder and `cp -a` of the `nightly-2026-11-17`
  install over it, then `rustup default nightly`
  (`log/default-set.txt`: "using existing install ... unchanged - rustc 1.102.0-nightly (5df8c02af 2026-11-16)").
  The proposal discloses the copy (`:901-905`).
- What a fresh machine resolves today is another compiler.
  The channel manifest `https://static.rust-lang.org/dist/channel-rust-nightly.toml`, fetched at 07:27 today,
  is dated 2026-11-28 and holds `rustc 1.102.0-nightly (c82ce1563 2026-11-27)`.
  The manifest of 2026-11-17 holds `5df8c02af` (so M8's compiler is that nightly, as the proposal says);
  the manifest of 2026-11-16 holds `5f4effd57` (the lock's)
  (`keep/probe-nightly-manifests-what-rustup-resolves.txt`).
- On `c82ce1563`, Answer I with components added to a real floating channel fails.
  M2b added the components (`m2-f2-proxy-cargo/log/component-add.txt`, `floating-components-after.txt`)
  and the library build ended
  `error[E0603]: enum ErrorOutputType is private` at `dylint_linting-6.4.0/src/lib.rs:794`
  (`log/lib-proxy.err.e0603.txt`); M5 shows the same in `dylint_driver-6.4.0`.
  M2b is therefore I's result in the state the criterion names, and M8 is I's result in a state only a stale floating
  channel can have.
- K1 delivers in the fresh-machine state today only with the variable:
  M7 arm D gave six findings while the floating channel in that container was untouched at `c82ce1563`
  (`m7/log/rustc-versions.txt`), because the library's file names the dated toolchain and the variable was exported.
  Without the variable K1 skips (M7 arms A and E).
- Dylint has no release for it.
  The newest `dylint_linting` is 6.4.0 (2026-11-10) (`keep/probe-crates-io-dylint-linting-newest-versions.txt`).
  Upstream pull request 2203, merged 2026-11-24, says it "keeps the driver and linting utility compatible with nightly
  toolchains that moved ErrorOutputType out of rustc_session::config",
  and `master` is 51 commits past `v6.4.0` (`keep/probe-gh-dylint-pr-2203-and-main-ahead-of-6-4-0.txt`).
  So a change for this very error is merged and unreleased; whether `master` builds on `c82ce1563` I did not test.
- What M2b and M5 are: both `E0603` logs come from direct `cargo build` emulations of the library and driver steps
  (`m2-in-container.sh`, `m5-in-container.sh` line 53),
  as the proposal says of M2b (`:1049-1050`), not from a `cargo dylint` run on `c82ce1563`.
  They show that the crates do not compile on that compiler, which is the step `cargo dylint` runs;
  the merged upstream change for the same error corroborates it.
  No kept log is a `cargo dylint` run on `c82ce1563`.

The result in both rows of the criterion, on the channel's current compiler:
- With no exported variable (the criterion's wording): I fails at the library build (M2b); K1 skips (M7 arms A and E).
- With the variable exported: I still fails (the variable does not change the compiler); K1 delivers (M7 arm D).
So on the criterion's own state neither delivers, and I's one delivery (M8)
needs a floating channel frozen at an older nightly.

The proposal knows part of this.
It says the failure "is a feature of that nightly and not of I" (`:948-952`)
and that a future nightly "breaks I until Dylint publishes" (`:953-956`).
It does not say that the nightly in question is the channel's current one,
and its criterion is worded for the fresh machine.
On that wording the order I > K1 is not shown: I's delivery is a stale-channel result,
and K1's is conditional on a variable the criterion excludes.
The order holds only for a host whose floating channel is frozen at an older nightly,
which is this host's state and was the 2026-10-30 host's state (High 1).

Why it matters: this is the first ranking and the first question.
How sure: high for the facts (logs, manifests, crates.io); the conclusion about the order depends on which state the
owner means by "a fresh machine", which the proposal should say.
What would settle it: state M2b beside M8 as I's two results, name the state each stands for,
and either change the criterion to the state M8 stands for or change the order.

### High 3: the ranking's real reason is the owner's "always latest", which the ranking text says it does not weigh, and the words are stretched to I

Passages: `:1805-1810`, `:1378-1381`, `:2016`, question 1 (`:1897-1907`);
the report's section "What I chose that the brief did not decide".

The proposal says
"What this ranking does not weigh: the owner's "always latest".
On that recorded preference I is also the answer the owner's words point at,
and the breakage policy for I is the recorded "Wait for Dylint"."
It says of I's breakage
"the answer to it is the owner's recorded one: wait, and do not pin".
The revision's report says what decided the order:
"Cost: the owner's "always latest" decides it, so a proposal ranked against a recorded preference would have needed a
pin the owner never refused."
So the reason that moved I from last to first is stated in the report and disowned in the proposal.

Evidence on the stretch:
- The record's own "floating nightly" is the lock's.
  `doc/decision/rust-closure-lint-dylint.md`: the 2026-10-18 answer added `rustc-dev` to "the shared floating
  nightly", and refreshing the lock "moved the floating nightly from `nightly-2026-10-17` to `nightly-2026-10-18`";
  the 2026-10-19 defaults say "the floating nightly stays (no pin)".
  So "no pin" in the record means the lock keeps tracking, and the lock holds a dated name.
  K1, a file generated from the lock, keeps that; I (the rustup channel `nightly`) is something the record never names.
  The proposal reads the record's "floating nightly" as the rustup channel (`:1359`, `:1805-1810`).
  The same record's "Known limits" (line 201) says "Dylint builds the target with the toolchain that built the
  library", the second surface of Medium 1.
- The plan implements "always latest" as a dated name in the lock (High 1):
  `rust@nightly` resolved to `nightly-YYYY-MM-DD` and moved by the daily update.
  A file generated from the lock (K1) moves with it.
  I, the floating `nightly` channel, is moved by nothing in the repository.
  The words "Rust nightly should always be latest" and "Make the plan file always track latest"
  (`doc/planning/toolchain-always-latest.md`, "Owner's words") therefore describe K1's arrangement at least as well as
  I's.
  The proposal's own sentence "If the owner prefers a compiler that moves only when the lock moves,
  K1 is the ranking's second" treats the lock as a stopped alternative, while the lock is what "always latest" moves.
- "Wait for Dylint" answered a different question.
  In `doc/planning/rust-nightly-always-latest.md` (lines 267 to 302) the options were:
  patch the closure package's source now, wait for a Dylint release, or retire the lint.
  The owner chose to wait, and the record says of waiting "the package stays red until then, and it was already red."
  That is acceptance that the closure package's own check stays red.
  It does not say "do not pin", and it was not given about the rule failing in every Rust package's lint run (Medium 1).

Why it matters: the owner would read "I is what my words point at" and "the answer to a break is the one I recorded"
and confirm both.
The brief to briefers is to carry as requirements only what the owner decided, citing the record.
How sure: high that the report and the proposal disagree on the reason,
and that the record's options were the three named; medium on which arrangement "always latest" better describes (the
plan's code is clear, the owner's intent is the owner's).
What would settle it: say in the proposal that the order is a preference to confirm,
with the words quoted and the plan's dated-name mechanism stated; ask it as a preference question,
not as a delivery ranking;
drop "do not pin" from the attribution.

## Medium

### Medium 1: the proposal does not say plainly what breaks on a nightly the library cannot be built on, or who notices

Passages: Answer I "What it gives up" (`:1374-1381`), "Not known" (`:2447-2451`),
M8 "What M8 does not settle" (`:953-956`), question 1.

What the proposal says is that "a nightly that moves rustc's internal API breaks the library until Dylint publishes a
release",
and "what to do on a nightly that breaks the library (wait for Dylint ...)" is left undecided.
It does not say what the linter returns, whom that reaches, or what else breaks.

Evidence:
- What the linter returns follows from the source: `check_package` (`src/closure_step_package.rs`)
  turns a failed `cargo dylint` run into a `core/processing-failure` finding on the package's `Cargo.toml` with the
  words
  "could not check this package: `cargo dylint` exited with {}. The result is incomplete, so the run cannot be trusted."
  and status `2`.
  The proposal says this in its description of the run (`:339-345`) and does not apply it to Answer I.
  Under I a breaking nightly turns every Rust package of a root lint run red with status 2,
  not only the closure package.
  No kept log shows that output for a library failure (M3's two status-2 packages keep no cause, `:1127-1134`),
  so this is source reading, not a measurement.
- A second surface: Dylint builds the linted package with the library's toolchain (Dylint's
  `docs/how_dylint_works.md`, Limitations:
  "Dylint tries to build the package with the same toolchain used to build the library";
  the closure package's README, "Building the target with the library's toolchain").
  Under I every linted Rust package must compile on the floating nightly,
  not on the lock's nightly that every other task uses.
  A package can pass its own build and fail the rule's run.
  Under K1 the packages build on the lock's nightly, which the daily update already checks.
  The proposal states only the library side.
- Who notices is partly answered by a record the proposal does not use for this.
  `doc/planning/toolchain-always-latest.md` (lines 385 to 452, 538 to 539, 1005 to 1009)
  describes the daily update's before and after check of every package,
  an issue for each package a nightly newly breaks, "failed then failed is already red and is never reported again",
  and names the closure library as the known "Wait for Dylint" red (E0797 and E0277, Dylint 6.3.2).
  That check runs a package's own tasks, so it covers the closure library's build and tests and not the rule's run
  over other packages.
  The rule's break in other packages is noticed by whoever runs the root lint task.
- The "wait" has a concrete state today: no release since 2026-11-10,
  a fix merged on `master` on 2026-11-24 and unreleased (High 2).
  The proposal lists "what a nightly that moves rustc's API does to Answer I beyond waiting for Dylint" under "Not
  known" (`:2447-2449`);
  the effect (status 2 per package) is knowable from the source.
- When the floating channel moves, the driver filed under its name stops loading (its RUNPATH names a
  `librustc_driver-<hash>.so` that the new build replaces, M4 and M8 `b-driver-readelf.txt`).
  From `closure_driver.rs` (`driver_state`, the `found` and `driver_unresolved` branches)
  the linter then answers `Ready` if the driver crate resolves offline and `cargo dylint` rebuilds in place,
  or a driver-gap skip if not.
  That is inference from source, not run; it is not in the proposal.

Why it matters: the owner is asked to accept "wait" as I's policy without being told its visible form (a red root lint
for every Rust package, with no commit to blame) or that the daily update would not report it.
How sure: high on the source; medium on the stale-driver path (not run).
What would settle it: three sentences in Answer I: the result (status 2 per package,
the first compiler error in the message), who meets it (anyone running the root lint task;
the daily update reports only the closure package, and only once),
and the state of Dylint's fix today.

### Medium 2: the reasons for I over K1 are not in delivered terms, and one is a timing with no spread

Passages: `:1793-1797`, `:942-947`, `:1385-1390`.

The rule quoted in the brief: "Rank options by what each delivers for the target workload; effort to build,
raw system calls or a new code path are costs to state, never reasons to rank an option lower."
Each reason given for I over K1:
- "both deliver the same six findings when K1's variable is set": true of M7 arm D and M8 arm C,
  and it is a tie on delivery.
  M8's state is the stale channel (High 2); on a fresh machine today K1 delivers and I does not.
- "I needs neither the file nor the variable": a count of parts, not delivery.
  Under Mechanism T (the proposal's own first-ranked mechanism, `:1424-1444`) the task sets the variable for K1,
  so K1's extra part is the one T already adds;
  whether T sets it for I the text leaves open (`:1443`: "whether `RUSTUP_TOOLCHAIN` is set (K1)
  or the components make it unnecessary (I)"), so I's advantage here depends on a choice not yet made.
  I needs something else the text does not name: a way to put components on the floating channel (High 1).
- "I's measured warm build is 91 s with no clone against K1's 123 s with 3 `git` executions": one run each,
  in different containers, on different compilers (`5df8c02af` and `5f4effd57`), at different times, on a busy host,
  with no spread measured and no control arm interleaved.
  The repository's rules say a timing is a finding only as a ratio to a control run in the same round with its range,
  and that differences below the run-to-run spread are noise.
  The clone difference (0 against 3 `git` executions) is solid, because the channel word decides it in source
  (`extra_symbols.rs`) and in the counts; the 32 seconds are not.
  It is a one-off cost per driver build, a cost to state.
- "I files the driver under the directory the issue's own message names": a coincidence of naming.
  The directory name is what Dylint computes for a library with no file; K1 changes the name on purpose.
- K1 over K2: "K1 follows the lock, so it cannot drift", a maintenance reason on a tie in delivery.
  It holds (source read of `pins_in`) and is also a cost-side reason.

In delivered terms the measured results are: I fails on the channel's current compiler (M2b)
and delivers on a stale one (M8); K1 delivers with the variable (M7); K2 equals K1.
That gives no strict order between I and K1 without the preference of High 3.

How sure: high.
What would settle it: restate the order as preference plus the two delivery results,
or run an interleaved pair if the 91 and 123 seconds are to be kept.

### Medium 3: the decisions as a set: a stated independence that the criterion breaks, combinations not named, and "cannot be built" reasons that do not all hold

The rule in the brief: "Never bundle separable decisions into one option set or offer complements as alternatives;
"both, in this order" must be reachable."

Independence:
- `:1225` says "Nothing in one ranking is a reason against an answer in another."
  Decision 1's criterion (`:1779`) is "with no exported variable", which is Decision 2's subject,
  and I's first reason is that it "needs neither the file nor the variable".
  The two decisions are not independent as ranked.
- Mechanism T says it sets "`RUSTUP_TOOLCHAIN` to the toolchain Answer I or K1 names" (`:1427-1429`).
  The lock records `RUSTUP_TOOLCHAIN` as the dated name in the `rust` entry's `env`
  (`"RUSTUP_TOOLCHAIN":"nightly-2026-11-16"`).
  Whether a lint task receives that value I did not check (`.cache/toolchain.env.json` is absent in the main checkout
  today, and the proposal says `task-run` does not see the toolchain environment, `:590-592`).
  If it does, then under I the name to export would be the floating channel against the lock's value,
  and the text does not say which wins; if it does not, the text should say where T's value comes from.

Combinations that are neither reachable as described nor named:
- Answer I with Answer H: H "installs the lock's nightly with `rustc-dev`" (`:1637-1643`), which is the K shape;
  I in CI would need the floating channel plus components.
- Answer A with Answer I: A files a driver under "the toolchain the library directory reports" (`:1543-1546`),
  which under I is a floating name that goes stale at the next nightly.
- Designs CL and SH with Answer K1: they restore the variable for Dylint's children (need one);
  the linter's own `rustc --print sysroot` (need two) is not addressed per option.
  M7 arms A and E show the skip when `rustc` is the proxy and the variable is unset.
- Options M and E appear only in "Combinations that cannot be built" (`:1968-1980`)
  with no description anywhere in the file.
  The owner cannot tell what M or E is; the proposal's own text for them is one clause each.

Reasons in the list that do not hold when checked:
- "Answer I with option M": the stated reason is that the library "would be built by the dated compiler" while the
  driver is filed under the floating name.
  The compiler is chosen by the proxy before `cargo` runs, and the proposal itself says Dylint removes the variable
  first (`:1974-1975`), so the stated reason is unmeasured.
  The conflict that follows from the proposal's own premises is different:
  `dylint-link` reads `RUSTUP_TOOLCHAIN` itself (`:255-257`) and names the library copy from it,
  so a dated value would file the library under a name Dylint does not look for under the floating directory.
  That is my inference from the proposal's premises, not run.
- "Answer K2 with the lean cleaner": a hand-pinned date "may" be deleted after the grace period.
  Under the proposal's own M11 the file then reinstalls it on first use, so the combination can be built,
  at a download per grace period (and only while rustup auto-installs, Medium 4).
- "A status keyed on any skip with a severity keyed by cause": both can be built (an `error` finding for one cause and
  a new status for any skip);
  what conflicts is intent, not construction.

"Both, in this order" is reachable for Decisions 3, 4 and 5 (B then A when the pause lifts is written down,
`:1854-1857`; the four skip parts are combinable).

Why it matters: the owner is told which pairs are impossible and which are not,
and some of the impossibilities are neither defined nor checked.
How sure: high for the text findings; medium for the dylint-link inference.
What would settle it: describe M and E in the file or remove them from the list;
restate the criterion without the variable; name the four missing pairs;
relabel the two non-impossibilities as conflicts.

### Medium 4: K1's "installs what it names" rests on a rustup behaviour measured for one command and announced as ending; M11 is thinner than the text says

Passages: K1 (`:1324-1329`), the reply to the second read's M4 (`:2196-2200`) and L5 (`:2372-2383`),
M11 in "Measured" (`:2425`).

Who is right about the second read's L5?
The second read said K's component list "is inert for a missing toolchain under rustup 1.28.0 and later,
which "will no longer automatically install the active toolchain"".
The report says that is wrong and cites M11.
- The second read is wrong for the rustup in use.
  rustup's `CHANGELOG.md`: 1.28.0 (2025-03-04) "will no longer automatically install the active toolchain if it is not
  installed";
  1.28.1 (2025-03-05) "This is a patch release to restore the automatic install behavior by default",
  opt-out with `RUSTUP_AUTO_INSTALL` set to `0`.
  The read cited the 1.28.0 announcement, which the next day's release reversed.
  M11's log shows the result on 1.29.3: "the missing active toolchain `nightly-2026-11-20-x86_64-unknown-linux-gnu`
  has been auto-installed", "info: downloading 8 components".
- The report is right for that case and says more than the log shows.
  The log never lists what was installed.
  Eight is consistent with the default profile's six components plus the two the file lists,
  so "installs that toolchain and the file's components" is an inference from a count.
  The rustup book's "Overrides" page says only that `components` is "a list of additional components to install" and
  "additive with the current profile"; it says nothing on when.
  The `rustc --version` arm of M11 ran after `rustup show` had already installed the toolchain,
  so it cannot show whether `rustc` alone installs.
  No arm covers a present toolchain that lacks a listed component, which is the case a K1 file meets when the dated
  toolchain exists without `rustc-dev`.
- The behaviour is deprecated and is going.
  The M11 log itself says "auto-installation is deprecated for most `rustup` commands" and points to rustup issue 4836
  (milestone 1.30.0, closed).
  That issue's comment lists the commands that stop auto-installing "starting from rustup v1.30",
  with `show` among them ("No, because it's query-only") and `run`, `doc` and `man` kept.
  The 1.28.0 changelog gives the supported way: "To ensure its installation,
  run `rustup toolchain install` with no arguments."
  The proposal reports the warning and still writes "So K1 installs what it names,
  at the price of a network download on first use."
  M11 measured `show`, the command the issue says will stop.
- The repository's own harness opted out: the 2026-10-30 harness sets `RUSTUP_AUTO_INSTALL` to `0`.
- The second read's underlying question was what the linter reports when the toolchain is absent.
  The proposal does not answer it: with no auto-install, `active_toolchain` fails and `driver_state` returns
  `Unknown`, which is shown only under `--debug` (`closure_driver.rs`, the `Err(reason)` arm),
  and the run then fails at `cargo dylint` (inference from source, not run).
- On machines that run `toolchain:install` the dated toolchain with the components is already installed by the plan
  row (High 1), so auto-install matters only on a runner or a bare machine.

So: the second read's L5 is wrong as written for rustup 1.28.1 to 1.29.3,
the report's reply is right for the absent-toolchain case and unshown for the components,
and neither covers rustup 1.30 or a present toolchain missing a component.
M11's folder name, "m11-rustup-does-not-install-an-absent-toolchain", says the opposite of its log (Low 3).

Why it matters: K1 is a ranked option whose first-use story is a behaviour its own log calls deprecated.
How sure: high on the changelog, the issue and the log; medium on the components (a count).
What would settle it: say in K1 that first use relies on a deprecated rustup behaviour,
name `rustup toolchain install` as the supported step, say what the linter reports without it,
and add an M11 arm that lists the components and one for a present toolchain missing one (a container run,
which this read may not do).

### Medium 5: Z over A rests on a pause the proposal calls undetermined, and question 7 is not the owner's to answer in the form asked

Passages: `:1848-1857`, Decision 7 (`:1753-1772`), question 7 (`:1956-1960`).

- The pair: "A delivers nothing today because the installer it needs is the paused one,
  and while paused every run still skips."
  The rule quoted in the brief ranks by delivery and treats a state that changes as a cost.
  The reason is a state, and Decision 7 says the pause is "a reading of it that the session made,
  not the owner's words".
  Z does not deliver today either: every provider assumes Decisions 1 and 2 first (`:1497`),
  so "Z delivers today" is not A's alone.
  And the host does not read as paused: `.cache/toolchain-bin/cargo` and `rustc` were rewritten at 2026-11-28 07:22:34
  -05:00, under a minute after the lock's last commit (07:21:46)
  (`keep/probe-stat-of-toolchain-links-and-lock-in-the-main-checkout.txt`);
  the issue 6316 comment says "Only `mise run toolchain:install` creates that folder".
  I have not shown that the installer wrote those links; it is the likelier writer by that comment.
- The quote's attribution.
  Decision 7 says "the coordinating session wrote:" and quotes "Because we need to make mpm-shadow prefer system
  package managers on non-atomic distros first."
  The comment says "Asked on 2026-11-23 whether to run it in the main checkout, the project answered: "...""
  (`keep/probe-gh-issue-6316-and-6643-read-only.txt`).
  The comment attributes the sentence to "the project", and every comment on the issue is under the one account
  `Aquaticat`, so the account does not say who answered.
  The proposal can neither call it the owner's nor call it the session's.
- Question 7 asks the owner what that text covers: "the installer run in the main checkout only,
  or a plan change too (Answer A's row, Answer I's component install)".
  Its second half is settled by source: the plan row does not install components on the floating channel (High 1),
  so I does not use the installer as described, and A needs a new route.
  Its first half is a record question: who answered, and what was asked.
  The coordinating session can answer that from the record; the owner is asked to interpret a text they may not have
  written.

How sure: high on the text and the attribution; medium on the links' writer.
What would settle it: give the pair a delivery reason or mark it as a state; quote the comment's own attribution;
fold the question into what the installer does (which a read answers)
and ask the owner only the part that is a choice.

### Medium 6: several things the proposal read and did not run are written as results

The brief asks whether anything "read and did not run" is stated elsewhere as a result.
- Mechanism T: "Measured: this is the state M7 and M8 ran in, and it delivers the rule in both answers" (`:1438-1441`).
  M7 and M8 ran the released linter binary inside a container with `PATH=/usr/bin:...`;
  the `lint:monochromatic-lint` task, `mise`, `task-run` and the shadow launchers were not run.
  The proposal's own CI section says `task-run` does not see the toolchain environment (`:590-592`).
  The state is measured; that a task can create it, and that the variable reaches the linter, is not.
  M8 also ran with the variable unset (`env -u RUSTUP_TOOLCHAIN`), so it does not measure T with the variable set to
  the floating name.
- Option F under I: "so a warm registry is all it needs and F makes the first run offline-sufficient" (`:1590-1593`).
  No kept run is an offline driver build with a pre-filled registry under I.
  M8 arm B ran online (`env --unset=CARGO_NET_OFFLINE`).
  The support is 0 `git` executions and the older record's arm E (24.6 s on the floating name, another host,
  Dylint 6.3.0).
- Answer C: "the measured traps a fresh image meets are the components (M2a)
  and a writable temporary `TMPDIR` for the clone" (`:1623-1625`).
  The reply to the second read's L5 says M1's refusal "was not established" (`:2364-2367`).
  No kept run shows a missing `TMPDIR` failing.
- K1's "the file's components" (Medium 4).
- "Measured, inferred, and not known" (`:2415-2451`) lists the shadow's hand-off, the 2026-11-07 state,
  option M and the ignored host test as inferred, and none of these four.

Why it matters: a reader who trusts "Measured" takes these four as settled.
How sure: high (each is a quoted line and a named log).
What would settle it: add the four to the "Inferred" list, and word T, F and C as designs with the nearest measurement
named.

### Medium 7: the owner cannot decide from the first screen

Passages: lines 1 to 66 (purpose, status, revision history), the questions at `:1892-1964`, "Measured, inferred,
and not known" at `:2415-2451`.

The file opens with a purpose paragraph, a status line and the history of two gate reads (lines 22 to 66).
The eight questions begin at line 1892 of 2521, the recommended order of each decision at line 1774,
and the split between measured and inferred at line 2415.
About 420 lines (`:1996-2413`) are the answers to the two earlier reads,
which belong to the evidence folder and not to the page the owner reads.
A reader who stops at the first screen knows what the file is and that it was reviewed,
and nothing about what is asked or recommended.

What the first screen must hold:
- The eight questions, one line each, in the owner's words (what you choose, not the option's letter),
  with the recommended answer and the one reason for it.
- For each recommendation, whether it rests on a measurement (name it), a line read (name the file and symbol)
  or an inference (say so).
- What a yes leaves undecided, in one line each.
- Which two or three facts would change the order (today's nightly breaks Dylint 6.4.0;
  the plan row installs dated names; the pause is not the owner's words).
- That the proposal is unapproved and non-final, with who reviewed it and that every review was same-vendor.

How sure: high.
What would settle it: the owner reading only the first screen and answering without opening the rest.

### Medium 8: options that are labels, options whose example shows no result, and decisions with no ranking

The rules quoted in the brief: "Each option says how it works, not only what it is called: the mechanism,
what it stores or changes for good, and what it gives up. A label ("encrypted", "atomic") is a name,
not a description." and "Option examples use concrete inputs, relevant constraints, and each option's visible result;
abstract labels aren't examples."

Options that are label-thin on mechanism:
- Answer I: the mechanism of installing the components is wrong (High 1).
- K1: "a generator the owner would name" is a label for the part that decides who writes the file;
  it is flagged as undecided.
- Mechanism N: "no change; the person runs the lint with the proxy first" names the status quo; no command is shown.
- Design CL and Design SH: "a rust change setting the child's `PATH` (or its program)" and "`cargo-shadow` ...
  exports `RUSTUP_TOOLCHAIN`" say what changes, with no program, file or example run.
- Answer A: "a new `toolchain.plan.jsonc` row" with "a route that compiles, which the plan has none of today":
  the route is a label.
- Answer C and Answer H: the steps are listed; what the pull request shows is not.
- Option L: "an exact request" with no example row.
- Option J: "a rule option beside `library`" with no configuration block and no printed record (the second read's
  Medium 4 asked for both).

Options whose example shows no result:
- K2 ("delivers and costs as K1"), N, T (a container state, not a task run), CL, SH, B (the command's times,
  not a lint task's run), A, C, H, J, D1, L.
- The options with a result are K1 and I (six findings, with positions in the logs), Z (the printed skip),
  D3 (its line of text) and the field of D2 (`"checked": false`, no full record).

Decisions with no ranking, though each puts options to the owner:
- Decision 5 (four parts), Decision 6 (L, yes or no), Decision 7 and the eighth question have no "Ranking:" line,
  and Decision 5's choices inside a part (keep or J, field or none, none or D1, none or D3) are unranked.
- Designs CL and SH are left out of Decision 2's order "because neither was measured" (`:1832`),
  while the criterion for that order is "run shapes the mechanism covers" (`:1814`)
  and the text says each "covers more run shapes than T".
  On its own criterion they would rank above T, with the cost stated.

Why it matters: a yes to a name decides only the name.
How sure: high (text).
What would settle it: for each option, the program that runs, the file it writes,
one printed result for `package/wg/cli-quicker-exempt`, and what stops working.

## Low

### Low 1: M7 cites a log for "the two Android targets" that lists none

`:826-828` says the dated toolchain "carries `rustc-dev`, `llvm-tools`,
`rust-src` and the two Android targets (`log/dated-components.txt`)".
`m7-k-with-minimal-floating-channel/log/dated-components.txt` has six lines: `cargo`, `llvm-tools`, `rust-src`,
`rust-std-x86_64-unknown-linux-gnu`, `rustc`, `rustc-dev`.
No M7, M8 or M10 log mentions Android (`keep/probe-m7-m8-m10-m11-citations-against-their-logs.txt`);
the Android targets are the host's (`machine-state.txt`).
The container's toolchain was installed without them (the README's Containerfile).
Why it matters: a reader who opens the cited log meets a line that is not there and starts to doubt the other citations.
How sure: high. Settle: delete "and the two Android targets".

### Low 2: M10 prints no linter digest, its arm B names the floating directory, and `--silent` is not measured

- `:810-812` says the digest "(printed in each `run.txt`)".
  M7's and M8's `run.txt` print `34ca8f26d6...`; M10's does not (and M11 does not run the linter).
  The binary M10 ran is untraced in its own log.
- M10 arm B is described as the driver-gap skip "with `RUSTUP_TOOLCHAIN=nightly-2026-11-16`" (`:973-974`);
  the skip it printed names `/root/.dylint_drivers/nightly-x86_64-unknown-linux-gnu/dylint-driver`,
  because that library has no toolchain file and Dylint ignores the variable.
  Correct, and worth a sentence: it shows the variable does not name the driver directory.
- `:390-395` and `:1729` say `--quiet` and `--silent` hide a skip; M10 ran `--quiet` only (the `--silent` part is the
  source and the README).
Why it matters: the statuses are the facts the owner's skip decision (Decision 5) rests on,
so each should trace to a run of a named binary and a named option.
How sure: high. Settle: print the digest in M10, add one sentence, or run `--silent`.

### Low 3: M11's folder name, its missing log folder, its missing section

- The folder is `m11-rustup-does-not-install-an-absent-toolchain/`; its log shows rustup did install it,
  and its script's last heading is "toolchains after (nothing may have been added)", the expectation it disproved.
- The README says "Each folder holds the container's whole output in `run.txt` and its logs in `log/`";
  M11 has `run.txt` only.
- The reply to L2 says the measurement sections run "(M7, M8, M10, M11)" (`:2326-2329`);
  "What this round measured" has M7, M8 and M10, and no M11 section and no M9.
  M11 is cited only in K1, in two replies and in the measured list.
Why it matters: M11 is the only evidence for K1's first-use story and for the dispute about L5,
yet it has no section where a reader can find its arms, and its name says the opposite of its result.
How sure: high. Settle: rename the folder, add a short M11 section with the arms and their confounds.

### Low 4: stale letters and colliding labels

- "Design S" (`:2175`) for Design SH; "Answers F and P" (`:2275`), letters of an earlier draft;
  "F skips (M7 arm A)" (`:2075`), where K1 is meant.
- The reads section names findings H1 to H4, M1 to M8 and L1 to L6, and the measurements are named M1 to M11.
  `M7`, `M8`, `M3` and `M5` each mean two different things in the same file ("M7 and M8 sections" at `:2049` means the
  measurements; "M7, records that bear and are not cited" at `:2259` is a finding).
Why it matters: the owner and the next reader cannot tell which "M7" a sentence cites,
and a stale letter points at an option that no longer exists.
How sure: high. Settle: prefix finding names with the read (for example "second read, Medium 7") or renumber.

### Low 5: figures and sentences the replies say are in the body that are not

- The reply to the second read's M8 says the disk cost "(`package/ide/app/target/dylint/debug` at 2.09 GiB ...)
  beside it" in the M3 description and implication 10 (`:2290-2294`).
  The only place the figure appears in the file is that reply (`:2293`); no body passage puts it next to B, Z or H,
  where the second read asked for it.
- The reply to L5 says Option C's temporary-directory sentence "is corrected" (`:2364`);
  the body still lists `TMPDIR` among "measured traps" (`:1623-1625`, Medium 6).
Why it matters: a reader of the replies believes the body changed, and the owner weighing B,
Z and H does not see their disk cost.
How sure: high. Settle: put the figure beside B, Z and H, and correct Answer C.

### Low 6: M7's setup is not kept

M7's state depends on a library directory holding option K's file and on `monochromatic-lint-k.config.jsonc`;
the folder keeps neither.
The README says the host wrapper "is not kept", and `m5-option-k-end-to-end/library-rust-toolchain.toml` holds the
same two lines the proposal gives (`:829-831`),
but M7's own folder shows only that a file exists (`log/k-active-without-env.txt`:
"overridden by '/work/repo/lint-closure-k/rust-toolchain.toml'").
Why it matters: M7 is the measurement of K alone, so its input should be in its folder for a rerun,
and the folder is what survives when the scratch expires.
How sure: high. Settle: copy the file and the config into the M7 folder.

# The brief's nine questions

## 1. Each finding of the second read

Closed, closed in part or not closed, with the passage that decides:
- High 1 (the pin question): closed in part.
  Closed: "declined" and "the owner adopted" are gone (`:1244-1274`);
  the owner's words and the record's "Still not approved" and "Adopted as defaults without asking" are quoted
  (`:1229-1281`); K is split into K1 and K2 (`:1292-1357`).
  Open: the words are stretched to I and "do not pin" (`:1378-1381`, `:1805-1810`), High 3.
- High 2 (K never measured alone; I's disqualification): closed in part.
  M7 (`:817-891`) measures K alone and confirms the read's mechanism; M8 (`:893-956`) is the run the read asked for.
  M8's state is a copy of a stale install, High 2 of this read.
- High 3 (sets not separable): closed in part. The split is done (`:1214-1225`, `:1892-1964`);
  the interaction and the combinations are Medium 3.
- High 4 (F's last place): closed. F is a complement and in no ranking (`:1574-1598`), and arm E is quoted;
  the offline claim under I is Medium 6.
- Medium 1 (what changed): closed in part. The section exists (`:705-770`);
  its item 3 and its reading rest on a floating channel the plan never put components on (High 1).
- Medium 2 (statements with no kept line): closed. M10 and the withdrawals (`:1127-1144`); new instances are Low 1,
  2 and 3.
- Medium 3 (which `cargo`, the shadow): closed. `:639-651`, Design SH (`:1476-1493`), "read and not run" (`:2173`);
  the stale label is Low 4.
- Medium 4 (options half-described): not closed for I, closed in part for the rest.
  I's "who installs the components" is answered "the plan's row", against the row's source and the line the read cited
  (High 1).
  The others state what a yes leaves undecided; their examples show no result (Medium 8).
- Medium 5 (pairs): closed in part. The old order is gone; I over K1 and Z over A fail the delivery test (Medium 2,
  5); T over N, B over Z and C over H hold.
- Medium 6 (the seven questions): closed in part. Each question names what a yes leaves undecided;
  question 7 is not understandable alone and half of it is settled by source (Medium 5);
  question 1 carries a false mechanism (High 1).
- Medium 7 (records not cited): closed in part. The four records are cited;
  `toolchain-always-latest.md` is used for the owner's words and not for who notices a break (Medium 1);
  the handover line is flagged for the coordinating session in the report.
- Medium 8 (M3's facts): closed in part. The counts, 1688 s and implication 10 are in the body (`:1095-1144`,
  `:1207-1212`); the disk cost is only in the reply (Low 5).
- Low 1 (line citations): closed. Headings and symbols replace line numbers.
- Low 2 (file names, order): closed.
- Low 3 (text defects): closed. The duplicate line is gone and D3's stream is stated (`:1722-1723`).
- Low 4 ("reproduced"): closed (`:1025-1029`, `:1093-1094`).
- Low 5 (tool statements): closed in part. The clone floor (`:273-280`) and M's `force` reason (`:1973-1975`)
  are corrected; the TMPDIR sentence is not (Low 5 of this read); the component-list point is the dispute in Medium 4.
- Low 6 (letters, cleaner): closed. Word names exist; `pins_in` is read (`:1347-1352`);
  the "cannot be built" overstatement is Medium 3.

Eight closed, nine closed in part, one not closed (the second read's Medium 4, for Answer I).

The report calls the second read's L5 wrong: who is right is in Medium 4.
The second read is wrong for rustup 1.28.1 to 1.29.3 (the announcement it cited was reversed by 1.28.1 and M11's log
shows the install);
the report is right for the absent-toolchain case and its log does not itemise the components;
neither speaks for rustup 1.30 or for a present toolchain missing a component.

## 2. What the revision added: M7, M8, M10, M11

M7, line by line against `m7-k-with-minimal-floating-channel/`:
- Matches: floating channel `c82ce1563 2026-11-27` with `cargo`, `rust-std`,
  `rustc` only (`floating-components-before.txt`, `rustc-versions.txt`);
  the two sysroot answers (`probe-sysroot-unset.txt`, `probe-sysroot-set.txt`)
  and the two `show active-toolchain` answers (`k-active-*.txt`);
  arm A, B, E statuses 0 and their skips; arm C status 0 after 123 s, library 47.13 s, driver 1 m 13 s (`run.txt`),
  3 `git` (`c-execve-counts.txt`: `/usr/bin/git` 3), the driver's `-V`, one RUNPATH entry, "generated 6 warnings";
  arm D six findings.
- Does not match: "the two Android targets" (Low 1).
- State: as the proposal says, no component was added by any step of the run;
  the library directory's file and config are not kept (Low 6).

M8, line by line against `m8-option-i-on-the-host-like-floating-build/`:
- Matches: `5df8c02af 2026-11-16`, which the 2026-11-17 manifest names (verified today); components `rustc-dev`,
  `llvm-tools`, `rust-src`, `cargo`, `rust-std`, `rustc`; the library has no toolchain file and the active name is the
  floating one;
  arm A the driver-gap skip for the floating directory, status 0; arm B status 0 after 91 s, library 46.88 s,
  driver 38.85 s, 0 `git`, 245 floating `rustc` and no dated one, `-V` `nightly-x86_64-unknown-linux-gnu 6.4.0`,
  one RUNPATH entry into the floating toolchain;
  arm C the same six offsets as M7 arm D (7146, 1649, 1962, 2526, 4978, 2292), status 0; arm D `--max-warnings 0`,
  status 1.
- Was the state the one described? Yes as described: the script deletes the floating folder and copies the dated
  2026-11-17 install over it, and rustup accepted it ("using existing install ... unchanged").
  What the description does not say is that rustup's `nightly` can no longer be that build (High 2):
  a real `nightly` install today is `c82ce1563`, and M2b is that state.
  The copy also leaves the components of a `--profile minimal` install, not the host's `clippy` and `rustfmt`;
  that does not affect the result.

M10, line by line against `m10-skip-statuses/`:
- Matches: arm A default status 0 and one record of 439 bytes, `--max-warnings 0` status 1 with the same 439 bytes,
  `--quiet` status 0 with 0 bytes; arm B the same three with 962 bytes (`a-sizes.txt`, `b-sizes.txt`);
  floating channel minimal; no driver.
- Does not match: no digest in `run.txt` (Low 2). Arm B's skip names the floating directory (Low 2).
- Not measured: `--silent`.

M11, line by line against `m11-rustup-does-not-install-an-absent-toolchain/run.txt`:
- Matches: rustup 1.29.3 (2026-11-11); `show active-toolchain` in a directory whose file names `nightly-2026-11-20`
  printed the name "overridden by '/work/probe/rust-toolchain.toml'", exit 0, then installed the toolchain,
  "downloading 8 components", the auto-install and deprecation warnings; the list after has the new toolchain.
- Does not show: which components; the case of a present toolchain missing one;
  anything for `rustc` alone (the arm ran after the install); rustup 1.30 (Medium 4).
- The folder is named for the opposite of its result (Low 3).

Anything the proposal says it "read and did not run" that is stated as a result elsewhere: yes, four items (Medium 6):
Mechanism T, option F under I, Answer C's `TMPDIR` trap, and K1's components.

## 3. The ranking that changed

Does every reason for I > K1 > K2 hold on the evidence? No (High 2, High 3, Medium 2).
- "Same six findings": a tie, and on the channel's current compiler I fails (M2b).
- "Needs neither file nor variable": not delivery, and false under T; I needs a component step the plan row does not
  provide (High 1).
- "91 s against 123 s": one run each, different compilers, no spread (Medium 2).
- "The directory the issue names": a naming coincidence.
- "The owner's words point at I": the plan implements them as dated names (High 3).

Does the proposal say plainly what breaks on a nightly the library cannot be built on, and who notices? No (Medium 1).
It says the library breaks until Dylint publishes and that "wait" is the policy.
It does not say that every Rust package's root lint run then returns a processing failure with status 2,
that every linted package must also compile on that nightly, that the daily update reports only the closure package
and only once, or that Dylint's fix is on `master` and unreleased.

## 4. Each option against the repository's rules

The list is in Medium 8: label-thin options (I's mechanism, K1's generator, N, CL, SH, A's route, C, H, L, J)
and options whose example shows no result (K2, N, T, CL, SH, B, A, C, H, J, D1, L).
The options with both a mechanism and a result are K1 (the file and the six findings) and Z (the printed skip),
and D3 for its line of text.

## 5. The decisions as a set

Separable as written: Decisions 1, 3, 4, 5 and 6 are separable; Decision 2 is not,
because Decision 1's criterion names the variable (`:1225` against `:1779`)
and T's variable value depends on Decision 1; Decision 7 asks the meaning of a text.
Every combination the owner could want is not reachable or named: four pairs are missing (I with H, A with I,
CL or SH with K1, T with I against the lock's exported variable); M and E are undefined;
two of the six "cannot be built" reasons are conflicts of intent or are contradicted by M11, and a third (M)
rests on an unmeasured reason (Medium 3).

## 6. The rankings, pair by pair

- Decision 1, I over K1: not in delivered terms (Medium 2). K1 over K2: a maintenance reason on a tie.
- Decision 2, T over N: holds (T removes the step a person must remember). CL and SH:
  unranked for "not measured" though the criterion would place them above T (Medium 8).
- Decision 3, B over Z: holds (B removes the memory step, costs stated, not used to rank). Z over A: a state,
  and the state is not shown (Medium 5).
- Decision 4, C over H: a delivery-robustness reason (how much of the job's state must be right); neither is measured;
  the effort of an image pipeline is stated and not used. Holds as written. "H over neither":
  holds (delivery on the surface the issue says never ran).
- Decision 5 and Decisions 6 to 8: no ranking (Medium 8).

## 7. The questions for the owner

- Question 1 (toolchain): the owner's, as a preference and a pin authorization; not understandable as asked,
  because I's text carries a mechanism that is false (High 1), and "what a yes leaves undecided" for I ("whether the
  plan's row is the only installer") misdescribes it. A measurement already answers part of it (I fails on today's
  nightly).
- Question 2 (how the variable reaches the tree): the owner's for the shadow bypass; the rest is design;
  the second need (the linter's own `rustc` probe) is not stated per option.
  A yes to T leaves the shadow's rule undecided, which is stated.
- Question 3 (local provider): the owner's (a lint task that writes to the home directory and needs the network).
  Stated undecided parts are the real ones.
- Question 4 (pull-request surface): the owner's (a workflow and a cost).
- Question 5 (skip parts): the owner's for the contract change; each part says what a yes leaves undecided.
- Question 6 (pin `cargo-dylint`): the owner's; it says it leaves "update EVERYTHING daily".
- Question 7 (the 6316 answer): not the owner's in the form asked, and not understandable alone (Medium 5).
- Question 8 (where the counts live): a placement a record settles: the repository's rules put package facts in the
  package's docs and repo-wide facts in `doc/`; a session can adopt a placement and say so.
  The "whether the host run is repeated" half is a choice.

Each question names what a yes leaves undecided: yes, all eight, with the corrections named in this section.
The rule on what an answer covers ("An owner's yes covers only what the option's text stated.
Whatever the option named by a label alone ... is undecided") is respected in form and broken for I,
where a yes would cover a mechanism the installer lacks.

## 8. Whether the owner can decide from it

No (Medium 7). The first screen must hold the eight questions in plain words with a recommendation and its basis each,
what is measured against read against inferred, what a yes leaves open,
and the two or three facts that would change the order.

## 9. What it still does not say and should

- The plan row installs a dated toolchain; the floating channel is not repository-managed; nothing moves it (High 1).
- Today's `nightly` is `c82ce1563`, on which Dylint 6.4.0 fails to build;
  Dylint's fix is merged on `master` (2026-11-24) and unreleased; no release since 2026-11-10 (High 2).
- What a library failure returns and who meets it, and that linted packages must build on the library's nightly
  (Medium 1).
- The daily update's own report of a package a nightly breaks, and that it names the closure library as the known red.
- That K1's first use relies on a deprecated rustup behaviour and what the linter says without it (Medium 4).
- How the lock's exported `RUSTUP_TOOLCHAIN` (a dated name) interacts with I and with T (Medium 3).
- A way to get the fix before a Dylint release: the library could take `dylint_linting` from a Git revision,
  but the driver is built by `cargo-dylint` from `dylint_driver = "=<its own version>"` on crates.io (`:236-241`),
  which the library cannot patch; only a `cargo-dylint` release or a locally built `cargo-dylint` fixes the driver.
  I did not test this; it is the proposal's own description of the driver build.
- The disk cost of a second target directory per linted package under the library's toolchain (the figure is in a
  reply only, Low 5).

# Verdict

The proposal is not ready to put to the owner.
Its structure is much improved (separate decisions, what a yes leaves undecided, the owner's words quoted,
measured against inferred), and eight of the second read's findings are closed.
It cannot go as written because the first question asks the owner to confirm an option whose mechanism is not the
repository's and whose first place rests on a state a fresh machine does not have.

The fewest changes that would make it ready:
1.  Correct Answer I: say who puts the components on the floating channel, who updates that channel,
    and that the plan row does not (High 1).
   Put M2b beside M8 as I's two results and say which state each stands for (High 2).
2.  Restate the Decision 1 ranking as what it is: delivery results (I fails on the current nightly and delivers on a
    stale one; K1 skips without the variable and delivers with it; K2 equals K1) plus the owner's preference,
    asked as a preference, with the record's "floating nightly" read as the lock's.
   Drop "do not pin" from the attribution of "Wait for Dylint" and say what that answer was given about (High 3,
   Medium 2).
3.  Add to Answer I what a breaking nightly returns (status 2 per Rust package),
    that linted packages also build on it, who is told (the daily update, only the closure package, once),
    and Dylint's state today (Medium 1).
4.  Put a first screen in the file (Medium 7).
5.  Say in K1 that first use relies on a deprecated rustup behaviour and name the supported step (Medium 4);
    rename M11 and add its section (Low 3).
6.  Give Z over A a delivery reason or mark it a state, quote the 6316 comment's own attribution,
    and reduce question 7 to the part that is a choice (Medium 5).
7.  Define M and E or drop them, name the four missing pairs, and relabel the two non-impossibilities (Medium 3).
8.  Add the four read-not-run items to the "Inferred" list (Medium 6).

The Lows and the label and example gaps of Medium 8 are worth doing in the same pass and do not block.
With the first three done the owner can answer question 1 on what is true.

# What I did not review, and why

- The measurements M1 to M6 and M3's recount: the second read checked them against their logs and the revision changed
  nothing in them; I read the M2 and M5 logs that bear on I and read M2a's run, and did not recount `results.tsv`.
- Dylint's source at the tag (the `extra_symbols.rs`, `driver_builder`, `dylint-link` lines):
  I relied on the proposal's quotations, the second read's probe file and the fetched Dylint documentation;
  I did not clone upstream.
- The quotation checker `scripts/check-quotations.ts`: not run (it is a script;
  this read ran no script from the repository).
- `cargo-shadow`'s source and README: read only the cleaner's header;
  the shadow's hand-off is the proposal's own "read and not run".
- The first read's findings, whole: I read its headings, titles and closing sections (see "What I read").
- A rendered check of the proposal's Markdown (the repository's rule on rendered Markdown): not done.
- Anything that needs a container, `cargo`, `rustup` or the linter on this host: the brief forbids it.
  That includes the three arms Medium 4 asks for (components listed, a present toolchain missing one, rustup 1.30)
  and the stale-driver path of Medium 1.
- `doc/planning/monochromatic-lint-skip-warning-and-fence-marker.md` and
  `doc/planning/github-hosted-runners-for-package-checks-and-a-guest.md`:
  the second read read them and the proposal quotes them; I did not reread them.
- `doc/decision/rust-closure-lint-dylint.md`: read for the sections named in "What I read";
  lines 1 to 35 and 92 to 149 (the accepted decision, suppression and the removal of the syntax-only rule)
  I did not read.
- Issue 6643's comments: the issue has none (`comments` is 0 in `keep/probe-gh-issue-6316-and-6643-read-only.txt`);
  its body I read from the kept capture.

# Rules of the brief I broke

- Two commands ran outside a `systemd-run --user --scope` unit before I switched to wrapping every write and fetch in
  one:
  the first (`mkdir --parents`, `chmod`, and the `git show` redirect that copied the proposal into scratch, in one call)
  and the second (`git archive | tar --extract` of the evidence folder).
  Both wrote only inside my scratch folder, and nothing was heavy.
- I used short options where long forms exist, in about a dozen read commands (`wc -l -c`, `grep -n -i -B2`, `sed -n`,
  `cut -c`, `head -150`, `ls -d`); I then switched to long forms.
- The search-fetch tool saved one oversized response, the crates.io record of `dylint_linting`,
  under
  `/home/user/Monochromatic/node_modules/.monochromatic/search-fetch/pi-search-fetch-response-658de32d-nx4Zwr/response.txt`
  (inside the repository's ignored `node_modules`).
  I did not write it; the tool did when its output was too large. I read it with `jq` and left it,
  because deleting a file in the repository is also a change.
  It is a 153125-byte file and the owner may delete it.
- The same tool saved two other fetches under
  `/home/user/.claude/projects/-home-user-Monochromatic/b990a53d-bf22-4522-98cf-71a43a562d9c/tool-results/` (outside
  the repository); I read one of them with `jq` to extract the rustup changelog into my scratch folder.
- Two times in my first notes were estimates and were corrected (the notes file says so).
- I ran no container, no `cargo`, no `rustup`, no `rustc`, no linter run, no `mise` task, no `sudo`,
  and printed no environment.
  I read `.cache/toolchain-bin` and `toolchain.lock.json` with `stat` and `jq` in the main checkout;
  I read nothing under `~/.cargo`.
  The `gh` calls were `issue view`, `search`, `release list`, `pr view`, `pr list` and `api .../compare` (reads).
