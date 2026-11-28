This is a same-vendor second pass: a narrow read by Claude Sonnet, the vendor that wrote the proposal, and not an independent review.
It is unapproved and non-final; only the owner declares work complete.

# Fourth narrow read of the Dylint driver proposal for issue 6643

Written on 2026-11-28 (the first probe file records the clock, `2026-11-28T08:46:43-05:00`).

Subject: `doc/planning/monochromatic-lint-who-provides-the-dylint-driver-and-what-a-skipped-rule-returns.md` as
`git show 88980bc4bb:<path>` holds it (2473 lines).
Every `:NNN` below is a line of that file unless a path says otherwise.
`HEAD` moved to `6ce456d382` while I read (`keep/probe-01-pinned-commit-and-head.txt`); every read here is of `88980bc4bb`.
This read changed nothing in the repository and nothing on GitHub.

## Counts

- High: 1.
- Medium: 6.
- Low: 6.

High means a finding could change which option the owner picks, or states as fact something the sources do not bear out.
Medium means the owner would answer with a distorted cost, a missing mechanism or a reason that does not hold.
Low means a citation, label or consistency error a reader could trip on.

## Short answers to the five points

1. **High 1 of the third read (installer mechanism): closed in substance, one residual.**
   The proposal now says, true to the sources, that the plan row installs `nightly-2026-11-16` and puts the components on
   that toolchain and on nothing else (`:494-543`; checked in `keep/probe-02-installer-sources-plan-row-lock-entry.txt`).
   One sentence still credits the floating channel with moving "on its own" (`:1395`).
   "A new installer step nobody has built" is stronger than the four sources bear (Medium 3).
2. **High 2 of the third read (ranked on a state a fresh machine cannot have): the three logs show what the proposal says;
   the order does not follow from them.**
   M12, M15a and M16 match their logs line by line, in the state described (one slip each, Low 1 to 4).
   What they support is only K1 over I, and only while the lock's compiler is older than the channel's (High 1).
   K2 has no delivered result, and K1 over K2 rests on source reads (Medium 1).
   The reasons that are not delivered results are listed in Medium 1.
3. **High 3 of the third read (preference read from words that do not hold it): closed as an attribution, open as a question.**
   Every sentence that attributes a wish to the owner matches its record; "do not pin" is gone.
   The preference between a dated name the daily update moves and the rustup channel is not produced as a result, but it is
   also not asked as its own question (Medium 2).
4. **The first screen: it gives a recommendation and a basis label for each of the seven questions, and it does not let the
   owner answer.**
   It omits the two facts that decide question 1 (High 1), states none of the alternatives, and four of the five labels I
   checked carry an error (Medium 4).
   M14 arm A's kept log shows a run with no record and status 0; what is offered for it is a defect with one fix and one
   small real choice, not a choice between "report" and "leave" (Medium 5).
5. **Seven questions: questions 1 and 2 must be answered together and first; question 3 follows from them under the
   file's recommendation; questions 4, 5 and 7 can wait; question 6's recommendation is the status quo.**
   The file does not say so, and one of its sentences says the opposite (Medium 6).

# What I read and what I ran

Read whole: the brief, the proposal, `gate-read-third-2026-11-28/review.md`, `third-round-2026-11-28/README.md`,
the `run.txt` of M12, M14, M15a and M16 with the logs they cite, the in-container scripts of M12, M14, M15a and M16 and the
host script of M16, the `Containerfile`.
Read in part (ranges, not whole files):
`findings-answered.md` (lines 1 to 155 and 561 to 870: the answers to the three Highs, and the revision's report);
the `run.txt` of M10, M13 and M17 (state and status lines only); `machine/machine-state.txt` (lines 1 to 40);
the second read's timer probe (lines 1 to 10); `probes/m3-counts-recounted.txt`.

Read at `88980bc4bb` (extracted with `git archive` into scratch, and `git show`), in the ranges named:
`package/mpm-shadow/cli/src/toolchain/model.rs` (lines 50 to 80 and 325 to 430), `install_rust_android.rs` (lines 1 to 150),
`install_rust_android/the_parts_a_rust_toolchain_lacks.rs` (lines 1 to 200), `install_tests.rs` (lines 1040 to 1100),
the `rust` row of `toolchain.plan.jsonc` (lines 154 to 230), the `rust` entry of `toolchain.lock.json`,
`package/cargo-shadow/cli/src/lean_rustup_toolchains.rs` (lines 1 to 80 and 127 to 175),
`package/linter/monochromatic-lint/src/closure_step_package.rs` (lines 195 to 292), `closure_driver.rs` (lines 178 to 210 and
278 to 312, plus a grep of the file), `closure_driver_probe.rs` (lines 150 to 193), `closure_cargo.rs` (a grep only),
`.github/workflows/advisory-core.yml` (a grep, then lines 90 to 101, 150 to 160 and 205 to 215),
`doc/planning/toolchain-always-latest.md` (lines 1 to 160 and 440 to 450), `doc/planning/rust-nightly-always-latest.md`
(lines 225 to 330), `doc/decision/rust-closure-lint-dylint.md` (lines 30 to 100 and 150 to 258),
`package/linter/monochromatic-lint-closure/README.md` (lines 180 to 262), and the 2026-10-30 record
`doc/troubleshooting/cargo-dylint-driver-pin-offline-after-upgrade.md` (lines 290 to 360).

Ran, all reads except the probe writes: `git show`, `git archive | tar --extract` into scratch, `git grep`, `git diff --stat`,
`jq`, `grep`, `awk`, `sed --quiet`, `wc`, `ls`.
The probe files were written by commands inside a `systemd-run --user --scope --slice=agent-work.slice` scope with
`MemoryMax=512M`, `CPUQuota=100%`, `TasksMax=64`, `IOWeight=1`, `RuntimeMaxSec=120`.
No container, no `cargo`, no `rustup`, no linter run, no build, no test, no `mise` task, no `gh`, no network, no editor,
no `sudo`; the environment was never printed.

Kept in `keep/`, one file per probe, each quoting the lines this review relies on:

- `probe-01-pinned-commit-and-head.txt`
- `probe-02-installer-sources-plan-row-lock-entry.txt`
- `probe-03-what-moves-the-floating-channel.txt`
- `probe-04-m12-m15a-m16-logs-against-the-proposal.txt`
- `probe-05-m14-arm-a-silent-run.txt`
- `probe-06-variable-set-or-unset-in-each-arm.txt`
- `probe-07-owner-records-whole-passages.txt`
- `probe-08-cleaner-keeps-and-the-stale-k1-file.txt`
- `probe-09-host-floating-channel-and-timer.txt`
- `probe-10-proposal-lines-the-findings-cite.txt`
- `probe-11-first-screen-labels-against-evidence.txt`
- `probe-12-stale-channel-setup-and-the-probe-that-strips-the-variable.txt`

# Findings

## High

### High 1: question 1's recommendation holds in two states the first screen does not name

Passages: first screen, question 1 (`:53-67`); "What would change the recommended order" (`:171-192`);
"What is true today" (`:788-821`); question 1 in "The questions for the owner" (`:2228-2249`).

The first screen says K1 is recommended, that the basis is "measured", that with K1's file and the variable "the rule
returned six findings" (M15a arm C), and that "with the components on the rustup channel named `nightly` and today's
compiler" the linter returned status 2 for every package (M12 arm A).
Two facts decide how much that is worth, and neither is on the first screen.

**First, K1's result expires when the lock moves.**
M16 shows K1 with a file naming `nightly-2026-11-28` (`c82ce1563`) returns the same `E0603` and status 2 as I
(`m16/run.txt`; `keep/probe-04-...txt`).
K1's lead over I in M12 against M15a exists only because the lock's compiler (`5f4effd57`, 2026-11-13) is older than the
channel's.
The owner's recorded policy is that the lock should not lag: "Rust nightly should always be latest", "Move anyway, fix
forward", and the plan "always track latest" with a daily update (`doc/planning/toolchain-always-latest.md` lines 19 to 22,
`keep/probe-07-...txt`).
The proposal says this in the body (`:800-813`, `:2234-2238`), but the section on the first screen that is meant to say what
changes the order does it backwards.
Its second bullet (`:179-185`) says: "If the lock does not move, K1's compiler does not move either, and the choice in
question 1 loses most of its meaning."
The change that matters is the lock moving: on the first daily update past the break, K1 and I return the same status 2
for every Rust package (M16), and nothing in either answer fixes that before a `cargo-dylint` release.
The timer is recorded `disabled; inactive (dead)` (`keep/probe-09-...txt`), which suggests the lock lags because the update
is off; why the timer is off is recorded as not known (`:2390`), so I do not know that nothing else holds the lock.

**Second, which "today's compiler", and what getting Answer I there does (conditional, unmeasured).**
M12 is a fresh `rustup toolchain install nightly` in a container, which resolves `c82ce1563 2026-11-27`
(`keep/probe-04-...txt`; `m12/run.txt` lines "floating rustc").
This host's floating channel is `nightly-x86_64-unknown-linux-gnu` at `5df8c02af 2026-11-16`, 274M, with no `rustc-dev`
(`machine/machine-state.txt` lines 6 and 31 to 38; `keep/probe-09-...txt`).
The proposal says Dylint 6.4.0 compiles on `5df8c02af` (`:1085-1090`, M8), that Answer I "on a channel frozen at an older
compiler ... returns the six findings" (`:1335-1339`), and that nothing in the repository moves the channel
(`:544-560`, checked in `keep/probe-03-...txt`).
It does not say what adding the components to this host's stale channel does to the channel.
M8 does not show it: M8's script copies an install that already holds the components over `nightly` (`m8-in-container.sh`
lines 20 to 21, `keep/probe-12-...txt`).
Answer I's own text includes an owner who moves the channel (`:1341-1346`), and its code path begins with
`rustup toolchain install --no-self-update nightly` (Medium 3); no kept run shows what that command, or
`rustup component add ... --toolchain nightly`, does to an installed older floating channel (M15b arm A synced a dated
channel, which cannot move).
So there are two cases the proposal does not separate.
If adding the components leaves the channel at `5df8c02af`, M8's state is this host's and I returns findings here today,
and "Today only the first two return findings" (`:2234-2235`) is true of a fresh install or a hosted runner and false of
this host.
If it moves the channel to `c82ce1563`, I on this host is M12's status 2 as well.
I could not tell which, and the first screen's "today's compiler" does not say which it means.

Why it matters to the owner's decision: an owner who answers from the first screen reads K1 as the answer that makes the
rule run and I as the one that fails.
K1 runs only until the owner's own daily update next moves the lock, and whether I fails on this host turns on a step no
run measures.
The question the owner actually faces is what the rule should do on the day the lock reaches a compiler Dylint cannot build
on, and the first screen does not put it.

How sure: high on the first half (M16, the record's words, the proposal's own `:800-813`); medium on the second, which turns
on rustup behaviour that no kept run shows; medium that the owner would pick differently, which is the owner's call.

What would settle it: two sentences in question 1 on the first screen (K1 returns status 2 once the lock reaches a compiler
past the break, M16; the compiler a fresh `nightly` resolves today is `c82ce1563`, this host's channel is `5df8c02af`,
and what adding the components does to that channel is not measured), and the second bullet of "What would change the
recommended order" rewritten to name the lock moving.
One container arm would close the second half: a floating channel frozen at `5df8c02af` with the components added by the
installer's own commands.
No new run is needed for the first half.

## Medium

### Medium 1: the order K1 over K2 over I does not follow from M12, M15a and M16, and one of its reasons is contradicted by the cleaner

Passages: "The ranking of Decision 1" (`:1353-1407`), "What the measurements imply" 5 (`:1097-1105`).

What M12, M15a and M16 deliver (`keep/probe-04-...txt`): K1 with the variable and a lagging lock, six findings (M15a arm C);
K1 without the variable, a rustc-dev skip (M15a arms A and D); I on a fresh `nightly`, status 2 (M12); K1 with the lock at
today's compiler, status 2 (M16).
From these alone: K1 over I while the lock lags, and K1 equal to I once it does not.
Nothing in the three says anything about K2.

Each reason in the ranking that is not a delivered result:

- "Both are the same file and the same mechanism, and on the day its date is installed, K2 delivers what K1 delivers"
  (`:1367-1370`): inference from identity of mechanism; the proposal says "K2 was not measured on its own date" (`:1246`).
  This is also the whole of K2 over I (`:1384-1388`).
- "K1's date is the lock's on the day the generator runs, so it names a toolchain the plan installs" (`:1371-1373`): a line
  read of the plan row, and it holds only on a machine where `toolchain:install` has run.
- "The lean cleaner keeps a toolchain the lock names" and "K2's date is installed by nothing the repository runs, and the
  cleaner keeps only a toolchain the lock or a checkout-root file names" (`:1372-1376`): a source read of `pins_in`.
  "Only" is incomplete: the header also keeps a toolchain that rustup names as default or override, one a process uses or
  has in its environment, and one younger than the grace period (`keep/probe-08-...txt`, header lines 9 to 15).
- "That is a difference in whether the run happens at all, not a difference in effort" (`:1376-1378`): a prediction.
  M14 arm B shows rustup installs a named absent toolchain when auto-install is on, so K2's date is installed on first use
  just as K1's would be.
  M14 arm A shows a silent pass when it is off, but only in the state that arm ran in (see Medium 5): a floating channel that
  holds `rustc-dev` and no exported variable; it is not shown for either name in the state K1 and K2 are recommended in.
- "A file the generator has not rewritten since the lock moved names an older toolchain, which rustup keeps, so that is a
  stale file and not a missing toolchain" (`:1379-1383`): rustup does not remove toolchains, but this repository's lean
  cleaner does, after the grace period, and `pins_in` reads only the lock, the environment fragment and a checkout-root
  `rust-toolchain` file, not a package-level file.
  So a stale K1 file names exactly what the proposal says makes K2 worse: a toolchain nothing pins.
  After the cleaner runs it is a missing toolchain, and with auto-install off it may reach M14 arm A's silent pass
  (inferred, not run; the conditions are in Medium 5).
  K1 is not shielded from the state K2 is ranked down for.
- The conditional order "K1 > I > K2" (`:1389-1398`) rests on "a compiler that moves on its own": the only sentence still
  crediting the repository with moving the floating channel, against `:544-560`, which says it moves only when a person runs
  rustup.
  It also counts parts ("without the file K2 needs"), which the repository's own rule treats as a cost to state and not a
  reason to rank.
- The criterion assumes Decision 2's answer T exporting the variable (`:1357-1359`, `:2142-2149`).
  M15a arm C set the variable by hand in a container; T as a `mise` task is read and not run (`:1453-1455`).
  The proposal says so; the ranking reads as measured regardless.

How sure: high (each reason is a quoted line against a quoted source).
What would settle it: say in the ranking that K1 over I holds while the lock lags (High 1); state K2's placement as
inference; replace "which rustup keeps" with what the cleaner keeps; delete "moves on its own".

### Medium 2: the preference is not asked as a preference, and the record's whole passage cuts against K1's form

Passages: `:1400-1407`, question 1 (`:2239-2241`), K1 (`:1156-1164`), K2 (`:1208-1256`).

Every attribution to the owner checks out against its record (`keep/probe-07-...txt`):
the five quoted words match `doc/planning/toolchain-always-latest.md` lines 19 to 25 with the parentheses outside the quotes;
"Wait for Dylint" is the closure package's red in that record too (lines 447 to 448, which the proposal does not cite);
the three options and their ranking match `doc/planning/rust-nightly-always-latest.md`;
"no pin" is a default a session adopted and left open to veto, and a pin is "still not approved" (`:737-756` against the
decision record lines 66 to 76 and 182 to 185);
"Fetch then warn." and "I run sync + warm" are verbatim.
"Do not pin" appears nowhere.
That part of the third read's High 3 is closed.

What is left:

- **The preference is not a question.**
  The only place that calls it "the preference to confirm" is inside Decision 1's ranking (`:1405-1407`).
  The list of questions has no question that asks the owner whether the library's compiler should follow the lock's dated
  name (which the daily update moves, when it runs) or the rustup channel (which only a person moves).
  Question 1 mixes that preference with delivery: the owner who prefers the channel is told it fails on a fresh install.
  Question 6 shows what a separable preference question looks like (`:2103-2105`: "a preference this proposal asks for,
  not a result"); question 1 has no such sentence, and the first screen has no word of it.
- **"Implemented" is stated as a fact.**
  Question 1 says the wish "is implemented by the lock's daily move, which K1 follows" (`:2239-2241`), while the same file
  records the timer as disabled and says why is not known (`:179-185`, `:2390`).
  The record's own status line says the timer was enabled on 2026-11-07 (`toolchain-always-latest.md` lines 3 to 9);
  on this machine now nothing runs it.
- **The whole 2026-10-18 passage is quoted for one half.**
  `:1162-1164` cites it for "the toolchain the owner's 2026-10-18 answer put `rustc-dev` on".
  The same section says: "The package therefore has no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override; it uses
  the root toolchain" (decision record, line 63), and the closure README repeats it
  ("the package has no `rust-toolchain.toml` and no toolchain override of its own", lines 190 to 191).
  The section came after a stage 1 that kept a package-local dated nightly because the root toolchain was not approved to
  change.
  K1 and K2 put a package-local `rust-toolchain.toml` back (generated from the lock for K1), and T adds an exported
  override; the proposal does not say that this reverses the arrangement the owner's answer produced, or that the README
  sentence would need to change.
  K2 is "a pin of the nightly", which the same section lists as "Still not approved"; question 1's K2 line does not say
  so.

Why it matters: the owner is asked to confirm the recommended answer as if the record favoured it, where the record's own
words about this package point to the root toolchain and no package file.
How sure: high on the quotes; the owner may well accept a generated file, which is the owner's call.
What would settle it: one separate question on the first screen ("when both work, should the library follow the lock's dated
name, or the channel?") with the record's words beside it; one sentence in K1 and K2 saying a package-level toolchain file
reverses the 2026-10-18 arrangement and that K2 is the pin the record lists as not approved; "implemented" changed to what
the record says and the timer's state.

### Medium 3: "needs an installer step nobody has built" is stronger than the four sources bear

Passages: `:541-542`, `:1259-1268`, "What it gives up" and "Cost" of Answer I.

What the four sources show (`keep/probe-02-...txt`): `rustup_channel()` returns the row's `packageId`; the plan writes
`"{version}"`; the lock holds `nightly-2026-11-16`; `install_rust_toolchain` and `install_rust_components` both pass
`tool.rustup_channel()`.
So the row as written installs a dated toolchain and puts the components on it and on nothing else.
That the row does not put components on the channel named `nightly` is true.
That the repository needs new installer code for it is not shown by those sources.
The two functions act on whatever channel `packageId` names, and a test in the same package asserts exactly that for
`nightly`: `rust_components_and_targets_use_the_package_id_channel_that_was_installed` (`install_tests.rs` lines 1058 to
1080) expects `rustup toolchain install --no-self-update nightly` followed by `rustup component add ... --toolchain nightly`.
The proposal never mentions this test.
What is missing for I is a plan row (or a changed `packageId`) naming the channel, plus a decision on what the lock and
`env.RUSTUP_TOOLCHAIN` hold for it (the plan's `env` is `{version}`, which expands to a dated name that would then not be
installed, the M17 case), and an owner for moving the channel.

How sure: high that the test exists and asserts that; medium on the last point, which is my reading of the plan model and
which I did not run.
I did not read whether the plan format allows a second `rust` row.
Why it matters: I's cost line reads "a new installer step and its owner", which overstates one half of the cost the owner
weighs; I is ranked last on delivery, so the order is not affected.
What would settle it: reword to "a plan row that names the channel, and a decision on what the lock records for it"; name
the test; keep the owner-for-the-channel's-move cost, which is real.

### Medium 4: the first screen does not let the owner answer, and four of five labels checked carry an error

Passages: `:42-152`.

What it does well: each of the seven has a recommendation, a basis label, and what a yes leaves undecided; the revision
history and the statement that no answer file is needed are right (`:19-40`).
What it lacks: **no question lists its alternatives**.
Question 1 names K1 and the channel but not K2; questions 2 to 4 name only the recommended one.
An owner who stops at `:152` can say "yes to all seven" and cannot say "no, the other one", because the options are
2000 lines down, at `:2222`.

The five labels I checked, each against its evidence (`keep/probe-11-...txt`, `keep/probe-04-...txt`, `keep/probe-06-...txt`):

- **Question 1, "measured".**
  The logs match: M12 arm A status 2, one record at `pkg/Cargo.toml`, `core/processing-failure`; M15a arm C six findings at
  the six offsets, status 0.
  Three slips: "six findings in 87 s of first-run build (M15a arm C)" joins arm C's findings to arm B's 87 s warm build
  (arm C prints no time); "for every package" is one package, `pkg` (M12 arm A holds one record); and the label covers
  the delivered results, while K1's place over K2 is a source read (Medium 1).
  It omits the two facts of High 1.
- **Question 2, "measured state, and it is the state every measurement here ran in".**
  M12 arms A and B, M14 arm A, M15a arms A, B and D all ran with `RUSTUP_TOOLCHAIN` unset (`probe-06`).
  M15a arm C, M16 and M17 ran with it set (and, from the README, M13 arm C and the earlier rounds' arms).
  The proxies-first half is true of every third-round arm; the variable half is not.
  The label for T is otherwise honest: "that a `mise` task can create that state is read and not run".
- **Question 3, "measured ... the same command cannot deliver today under question 1's other answers (M12 arm B)".**
  M12 arm B is Answer I only.
  Question 1's other answers are K2 and I; K2 delivers by the proposal's own text (`:1233-1246`), so "other answers" is
  wrong for K2.
  The step it recommends (a task step that runs the warm command) was never run: M15a ran the command by hand in a
  container, as it did for T, which is labelled "read and not run".
  The two labels should say the same thing.
- **Question 5, "four parts ... with the fifth part this round measured", "measured".**
  The body has five parts (`:1868`, `:2291`); the first screen counts four and then five.
  "A record and an exit status only for the causes the owner names" does not match Decision 5's recommendations
  (J, D2, by cause, D3, report), and no recommendation per part is on the first screen.
  "Measured" is true of today's statuses (M10) and of M14 arm A, and every recommended part reads "Result: none measured".
- **Question 7, "measured counts ... a placement, not a measurement".**
  Checked against `probes/m3-counts-recounted.txt`: seven packages, 1674 findings, 1611 in the four named packages, thirty
  rows, 1688 s, six skips and two failures; all as stated.
  The label is honest.
- Questions 4 and 6 read honestly: question 4 "not measured", with `advisory-core.yml` lines 95 and 152 to 159 and the CI
  excerpt (`command not found`, status 127) as stated; question 6 "line read ... nothing here is measured" (but see
  "One thing outside the five points", below the Lows).

Why it matters: the first screen is the page the owner is meant to answer from.
How sure: high (each is a quoted line against a log or a line).
What would settle it: list each question's options in one line each; say K1's expiry (High 1); correct the four labels;
give question 5 its five recommendations in one line each, or say that it is five questions.

### Medium 5: M14 arm A is a defect with one fix and one small choice, and the proposal offers a choice between "report" and "leave"

Passages: `:394-406`, Decision 5 part 5 (`:2009-2042`), question 5.

What the kept log shows (`keep/probe-05-...txt`): the arm ran a fresh container from the image with `RUSTUP_TOOLCHAIN` unset and
`RUSTUP_AUTO_INSTALL=0`, the library file naming `nightly-2026-11-21`, which is not installed.
The image's floating channel carries `rustc-dev`, `llvm-tools` and `rust-src` (`Containerfile`); the log never prints that,
and it is why the linter's machine-wide `rustc-dev` check passed and the run reached the driver check.
M15a arms A and D show what the same linter does when the floating channel lacks `rustc-dev` and the variable is unset:
a visible skip.
`rustup show active-toolchain` exits 1.
The linter's status line is `a linter status 0`; `a-linter.jsonl`, `a-linter.err` and `a-record.txt` are zero bytes.
The `--debug` notes say "the Dylint driver check for the library lint-closure did not run: `rustup show active-toolchain`
failed (exit status: 1)" and "rust/avoidable-closure checked pkg: 0 report(s) from the compiler".
So the log does show a run with no record and status 0, as the proposal says.

What the log does not show: `cargo dylint`'s own output or exit status in that arm.
From `check_package` (`closure_step_package.rs` lines 263 to 272) a failed `cargo dylint` adds a processing failure, so it
must have exited 0 with nothing on standard output.
"Checked nothing" then rests on a positive control, not on that arm: the same package returns six findings when the driver
and compiler are right (M15a arm C).
That is a sound inference; it should be labelled one, or the arm should keep `cargo dylint`'s output.

Choice or defect:

- The revision's own report calls it a defect: "that is a defect of the linter's `Unknown` branch, not of this proposal"
  (`findings-answered.md`, "Not creating an issue for the silent pass").
  The decision record the proposal cites says a skip "is not a silent message with exit status 0"
  (`doc/decision/rust-closure-lint-dylint.md` line 246).
  A run that returns clean having checked nothing is the harm the issue records, in a form no skip part catches.
  Offering "Leave it" as an equal option asks the owner to choose that harm; the proposal's own ranking says "Leaving it
  produces exactly the record the issue is about" (`:2041-2042`).
- The one real choice inside it is the form of the report, and the proposal makes it without saying so.
  The recorded contract says a missing prerequisite is skipped with a warning-level finding naming the command that fixes it,
  and "Failures that are not a missing prerequisite keep exiting with status 2" (decision record lines 246 to 254).
  A library file naming a toolchain that is not installed reads as a missing prerequisite, and rustup's own message names
  the fix; the proposal recommends a processing failure with status 2 and does not cite the record line that would make that
  a departure.
  The `Unknown` branch also covers an unreadable `rustup` answer and an unreadable `cargo dylint --version`
  (`closure_driver.rs` lines 186 to 207 and 288), which are not missing prerequisites.
- **The state it ran in is not the recommended one, and the proposal does not say so.**
  M14 arm A ran with a floating channel that holds `rustc-dev` (Answer I's component state) and no exported variable.
  In K1's and K2's recommended state the floating channel lacks `rustc-dev`: with the variable unset the linter skips
  visibly at the machine-wide check (M15a arms A and D), and with T's variable naming an absent toolchain it is loud
  (M17: status 2, one record at the configuration file).
  The K2 example (`:1241-1245`: "With a date nothing has installed and auto-install off, the linter prints no record and
  exits `0`") and Decision 5 part 5 present the silent pass without that condition.
  A route to the silent pass under K1 with T exists by analogy and is inferred: the variable names a present toolchain (so
  the machine-wide check passes), the library's file names one the cleaner removed (Medium 1), and the linter's own driver
  probe removes the variable before asking rustup (`closure_driver_probe.rs` lines 166 to 171,
  `keep/probe-12-...txt`), so it reads the file's name, fails, and returns `Unknown`.
  No kept run has that combination.

How sure: high that the arm shows no record and status 0; high on the source for why `cargo dylint` must have exited 0;
medium that "defect with one fix" is the right frame, which is the owner's and the coordinating session's to judge.
What would settle it: state part 5 as a defect the work will fix without asking, ask the owner only whether a named-but-absent
toolchain reports as a warning skip (the recorded contract for a missing prerequisite) or as a processing failure, and keep
`cargo dylint`'s output in M14 arm A or label "checked nothing" inferred.

### Medium 6: the file does not say which questions come first, and says the opposite

Passages: `:2222-2227`, `:2140-2168`, `:1118-1120`.

Seven questions with five parts in the fifth, three options in the first, four in the second and three in the third are many
to put at once; the owner is in effect asked about eleven or twelve things.
What the file's own text supports (`keep/probe-10-...txt`):

- **Answer first, together: questions 1 and 2.**
  Everything else assumes question 1: "Decision 3 and Decision 4 deliver only under an answer to Decision 1 whose compiler
  builds Dylint" (`:2157-2158`).
  Question 1 and question 2 cannot be answered apart: Decision 1's ranking "assumes Decision 2's answer T" (`:2142-2149`),
  and T's value is question 1's name (`:2150-2156`).
  Under the recommendation question 2 is two lines in a task, but it carries an owner-level cost (the shadow bypass) that the
  file marks as left undecided.
- **Follows from questions 1 and 2 under the file's recommendation: question 3.**
  B is the warm command measured under K1 with the variable exported, and it cannot deliver under a compiler Dylint cannot
  build.
  It carries an owner-level cost the owner may still veto (a lint run that writes `~/.dylint_drivers` and needs the network),
  which the file marks as left undecided.
- **Question 6: the recommendation is the status quo.**
  Keeping `@latest` is today's plan; nothing starts unless the owner chooses a pin.
- **Can wait without holding anything: questions 4, 5 and 7.**
  Question 4: "CI cannot start the linter at all today, so every CI answer waits on a workflow change that no provider answer
  includes" (`:1118-1120`), neither answer was measured.
  Question 5: the file says "Decision 5 is independent of all of them" (`:2163-2165`); part 5 is a defect (Medium 5) and
  parts 1 to 4 change the owner's recorded contract without blocking the work on questions 1 to 3.
  Question 7: "Decision 7 is independent" (`:2166-2167`); its first half is a placement for a repository-wide count, which
  the file itself says belongs in `doc/planning/` "either way" (`:2131-2133`); only whether the host run repeats is a choice.

Whether the file says so: it says Decisions 5 and 7 are independent and that 3 and 4 depend on 1, but never says which to
answer first or which can wait.
Worse, `:2224-2226` says "Each question is one the owner alone can answer, is answerable without the others", which
`:2142-2149` (question 2) and `:2157-2158` (questions 3 and 4) contradict.
How sure: high on the text; the grouping is mine and follows the file's own dependency section.
What would settle it: three lines on the first screen ("answer 1 and 2 together first; 3 follows; 4, 5 and 7 can wait;
6 changes nothing unless you choose a pin") and `:2224-2226` corrected.

## Low

### Low 1: M15a is cited for a RUNPATH entry its logs do not hold

`:928-931` and `:1106-1109` cite M15a arm B for "one RUNPATH entry into the dated toolchain".
No log of M15a holds a `readelf` or RUNPATH line; M13's `log/b-driver-readelf.txt` does (the dated toolchain's `lib`
directory, one entry) (`keep/probe-04-...txt`).
Why it matters: a reader who opens M15a meets nothing.
How sure: high.
Settle: cite M13 for the RUNPATH, or add the `readelf` line to M15a.

### Low 2: the `Containerfile` path is wrong

`:831-832` gives `third-round-2026-11-28/scripts/Containerfile`; the file is `third-round-2026-11-28/Containerfile`
(`keep/probe-04-...txt`; `ls` of the `scripts/` path fails).
How sure: high.
Settle: drop `scripts/`.

### Low 3: "printed in every run log" is false for M16 and M15b, and M10's binary is asserted

`:839-841` says the linter's digest is "printed in every run log".
`m15b/run.txt` runs no linter, which is fine; `m16/run.txt` runs the linter in arm B and prints neither its digest nor its version
(`keep/probe-04-...txt`, digest counts: M12, M13, M14, M15a, M17 print it once; M15b and M16 not).
M16 is one of the three measurements Decision 1 rests on (`:1361-1363`).
`:1023-1027` also states that "M10's binary is the same 0.5.2-alpha.25", and M10's own log prints no identity for it
(`keep/probe-11-...txt`).
Both are very likely true, since the same host script copies the same file; neither is shown by the log it is attached to.
Why it matters: Decision 1 and Decision 5's statuses rest on those runs.
How sure: high that the logs do not print it.
Settle: print the digest in M16, or say it is from the host script.

### Low 4: M12 arm A's record does not carry the compiler's error line

`:881-883`, `:897-898` and `:1292-1293` say the record "ends with the compiler's own `E0603` text" and "quotes the compiler's
first error", and `:1332-1333` that it "carries `error[E0603]: enum ErrorOutputType is private`".
The kept record begins "The end of cargo's messages was:" in the middle of the diagnostic (the `use` line and a `help` block)
and ends with `Error: \`cargo\` command failed; the full command can be found in ...`.
It holds `rustc --explain E0603` and the `794` help lines; the `error[E0603]` header line and the file path
`dylint_linting-6.4.0/src/lib.rs:794` are not in it (`keep/probe-04-...txt`: zero records hold `error[E0603]`).
The header line is in `b-warm.err`, which is the warm command's output, not the linter's record.
Why it matters: what a person sees from a failing run is the tail of the message; the proposal describes more.
How sure: high.
Settle: say "the end of cargo's messages, including the tail of the first error".

### Low 5: M16's script comment describes a floating channel the container did not have

`m16-in-container.sh` line 5 says "the floating channel is minimal (M15a left it so)".
M16 ran in a fresh container from the image (`run-m16.sh`: `podman run ... "$img"`), where the image's floating channel
carries `rustc-dev`, `llvm-tools` and `rust-src` (`Containerfile`).
The result (`E0603` at the library build) does not depend on it, and the proposal does not repeat the comment.
Why it matters: the kept script is what a rerun reads.
How sure: high.
Settle: correct the comment, or print the floating components in M16.

### Low 6: M12 has no arm with the variable set, though the criterion says T exports it

Decision 1's criterion is "with Decision 2's answer T exporting the variable" (`:1357-1359`); M12 ran with it unset
(`keep/probe-06-...txt`) and M16 with it set on a different library file.
The outcome for I is variable-independent by the proposal's own account of Dylint (it removes `RUSTUP_TOOLCHAIN` for its
children, `:305-316`, so the library builds with the proxy's default whatever the caller exported), but no arm shows it.
Why it matters: small; the criterion and the measured state differ by one input.
How sure: high that no arm sets it; medium that it would change nothing.
Settle: one M12 arm with `RUSTUP_TOOLCHAIN=nightly`, or one sentence saying the variable does not reach the library build.

## One thing outside the five points that would mislead an answer

Question 6's recommendation says keeping `@latest` "carries upstream's fix to the machine without a person"
(`:2314-2315`; `:2098-2099`: "on the next daily update with no person involved").
The proposal's own state says the daily update timer is `disabled; inactive (dead)` and that why is not known (`:640`,
`:2390`), and no release carrying the fix exists (`:173-178`, `:778-786`).
On this machine `@latest` moves only when a person runs `toolchain:update`, so the reason given for ranking `@latest` over a
pin does not hold today; the recommendation (keep, which is the status quo) is unaffected.

# Verdict

**Not ready to put to the owner as written; it needs a handful of text edits and no new run.**
The three Highs of the third read are closed as findings: the installer mechanism is stated correctly (with Medium 3's
softening), I's measurement is now a real fresh install beside K1's (M12 beside M15a), and "do not pin" is out of every
attribution.
What is not closed is what the revision's own change exposed: the ranking now rests on a fresh-install compiler and a
lagging lock, and the first screen states neither (High 1).

The fewest changes:

1. First screen, question 1: add that K1 returns status 2 once the lock reaches a compiler past the break (M16), and which
   compiler "today's" means (a fresh install's `c82ce1563`; this host's channel is `5df8c02af`, and what adding the
   components does to it is unmeasured); rewrite the second bullet of "What would change the recommended order" to name
   the lock moving (High 1).
2. Ask the lock-or-channel preference as its own question with the record's words beside it; say a package-level toolchain
   file reverses the 2026-10-18 arrangement and K2 is the pin the record lists as not approved; replace "implemented" with
   the timer's state (Medium 2).
3. Restate the ranking as K1 over I while the lock lags, K2 as inference, "which rustup keeps" as what the cleaner keeps,
   and delete "moves on its own" (Medium 1).
4. On the first screen: list each question's options; correct the question 2, 3 and 5 labels and the question 1 slips; give
   question 5 its five recommendations (Medium 4).
5. State part 5 as a defect the work will fix, and ask only warning skip against processing failure (Medium 5).
6. Add the order of asking (Medium 6) and soften I's "new installer step" (Medium 3).
7. The Lows are worth doing in the same pass and do not block; Low 3 and Low 6 are the two that touch the evidence Decision 1
   rests on.

# What I did not review, and why

- The measurements M1 to M11 and M13: the earlier reads checked M1 to M11 against their logs; I read M13 and M17 only for the
  state and status lines and M10 only for the status lines and binary identity.
  M15b I read in the run summary only.
- Dylint's source at `v6.4.0`: I relied on the proposal's quotations of it; no network fetch was made (the brief names none),
  so the claims about `rustup_toolchain` stripping, the driver build and `extra_symbols.rs` are unchecked here.
- The `cargo-shadow` source and README beyond the lean cleaner (`:1514-1540`); the shadow's hand-off is the proposal's own
  "read and not run".
- Decisions 3 to 7 beyond the passages the five points name; the ranking pairs of Decisions 2 to 4 and the options' examples
  (the third read's Medium 8) were not re-read.
- Whether the plan format allows a second `rust` row, and whether the sync step re-runs `rustup toolchain install nightly`
  on an installed channel (the second half of Medium 3's "who moves the channel").
- What `rustup toolchain install nightly` and `rustup component add ... --toolchain nightly` do to an installed older floating
  channel: no kept run shows it and I may not run rustup (the second half of High 1).
- A rendered check of the proposal's Markdown, and every container, `cargo`, `rustup`, linter or `mise` run: forbidden by the
  brief.
  That includes the arm Low 6 asks for and the arm that would close the second half of High 1.
- Issue 6643's text and the 6316 comments, and the numbers of M3 beyond the recount probe.

# Rules of the brief I broke

- Two commands ran outside a `systemd-run --user --scope` unit although they wrote files: the first call
  (`mkdir --parents`, `chmod 700`, and the `git show` redirect that copied the proposal into scratch) and the second
  (`git archive | tar --extract` of the evidence folder and sources into scratch).
  Both wrote only inside `/home/user/temp/agent/fourth-narrow-read-of-the-dylint-driver-proposal-2026-11-28/`, and neither
  was heavy.
  Every probe file was written inside a scope afterwards.
- I used short options where a long form exists in a number of read commands (`sed -n`, `head -200`, `git ls-tree -r`,
  `git log -1`, `git cat-file -t`, `git log --oneline`, and `awk -F` in one probe line, `--field-separator` being the long
  form); the other probe commands use the long forms.
- A first attempt at the probe scope failed in `zsh` (an unquoted variable holding the command did not word-split); nothing
  ran.
- `HEAD` moved during the read; all reads are of `88980bc4bb` as the brief says.
- I ran no container, no `cargo`, no linter, no `mise` task, no `sudo`, no `gh`; the environment was not printed; nothing
  under `~/.cargo` was read; nothing in the repository or on GitHub was changed.
