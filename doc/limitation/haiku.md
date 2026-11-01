# Haiku limitations

Documenting tasks where Haiku subagents failed to produce usable results,
so future sessions do not repeat the same failure pattern.

## Stock Settings capture on the owned Fold emulator (2026-10-11)

### Goal

Capture the stock Android Settings two-pane layout on the owned Pixel 9 Pro Fold emulator,
so the music player Settings page could follow it.
The brief allowed the Developer options "Smallest width" screen as a fallback for the display size,
set through the UI only and restored afterward.

### Files involved

- `/home/user/temp/agent/start-settings-pane-runtime.ts`: the launch script the agent used for the emulator.
- `doc/troubleshooting/android-emulator-37-software-renderer-sigsegv.md`: the repo record of which emulator flags boot
  and which crash.
- `/home/user/temp/agent/agent-settings-ref2/notes.txt`: the agent's log of the attempts.

### Mistakes

#### Blamed host load for emulator failures without questioning the launch

Across three rounds the agent saw Pixel Launcher and System UI "isn't responding" dialogs,
a `qemu` segmentation fault (exit status 139),
and a System UI ANR loop.
Each time it reported host load averages (134 to 170) as the environment problem,
asked for a quieter host,
and stopped.
The user then said that system load is fine and that the emulator was probably not launched correctly.

The launch script contradicted the repo record in three ways.
It passed `-feature -Vulkan` with `-gpu host`,
and the troubleshooting document shows (from `emugl_config.cpp`) that without host Vulkan the emulator selects the
SwiftShader Vulkan ICD,
the software path that crashes `qemu`.
The documented working launch has no such flag.
It also wrapped the headless emulator in `xvfb-run` and capped the container at 2 CPUs and 6 GiB
while giving the guest 2 cores and 4 GiB,
and it used `-no-snapshot`, which forces a cold boot every run.
The orchestrator asked for a comparison run with corrected flags,
and the agent still had not completed the task when it was stopped on 2026-10-12,
2.8 hours after its first transcript record (measured from the transcript timestamps, 21:51 to 00:39 UTC),
while it was still trying to make the Developer options navigation script wait long enough.
The user's observation: GPT models never had a problem using the emulator,
and the difference is that they always launched it headfully (with its window),
whereas every launch in this task was headless.
The failure recorded here is that the agent never compared its launch against `doc/troubleshooting/`
before attributing three separate failures to one external cause,
and kept refining scripts around a headless setup that other sessions had avoided.

#### Left the unexplained Build number taps unexplained

The Build number taps showed a pressed highlight but never counted,
with seven plain taps and slower taps.
The agent did not look for why the taps were not counted,
and its report says the cause was not identified.
The press-duration variant and the unlock-switch fallback came from the orchestrator's instructions, not the agent.

### What the agent did well

It recorded original values before every write and restored them afterward,
verified the restores by reading them back,
never used `wm density` or `wm size`,
and reported plainly that no two-pane screenshot and no Smallest width measurement existed.
A write to a platform flag was refused and it did not look for a way around the refusal.

### Lesson

When a Haiku agent reports an environment cause (load, flaky host, slow machine) for repeated failures,
check the cause with a measurement and compare the agent's launch command with the repo's troubleshooting records
before accepting it or waiting.
The brief for this task also did not tell the agent to search `doc/troubleshooting/`,
so briefs for tool-driven tasks should name that search as a first step.

## Desktop Settings redesign with a newer Slint syntax (2026-10-11)

### Goal

Redesign the desktop Settings page in `package/music-player/desktop-app/ui/settings.slint` after the stock Android
Settings look,
in a Slint 1.17 app.

### Mistakes

#### Wrote properties that only Slint 1.18 accepts

The agent renamed the Flickable `viewport-width` and `viewport-height` to `content-width` and `content-height` in
`settings.slint`.
Those names exist in Slint 1.18 and do not exist in the 1.17 the app builds with,
so the 1.17 build rejected the file.
The agent's lint run passed anyway,
because `lint:slint` runs the `slint-viewer` that mise installs at `latest` (1.18.1),
not the version the app compiles with.
The agent treated a green lint as proof that the file compiles in the app.

Correct behavior: after a Slint edit, build the app itself (or render with the app's own Slint version) before reporting
the file as verified,
and read the version the project pins before using syntax from release notes.
The orchestrator reverted the rename.
The linter and the app were aligned only by the later bump of every Slint crate to 1.18.1.

### Lesson

A passing linter proves the linter's version accepts the file.
When a linter and a build can disagree on a tool version,
the brief should require the build as the verdict and name the pinned version.

## Audio stutter fix delivered without a test verdict (2026-10-11)

### Goal

Find and remove audio stutters in the desktop player under high system load,
with measurements before and after,
in a separate worktree.

### Mistakes

#### Handed back a fix without running its own gates

The final report says the package test task never produced a verdict (the agent stopped it while it was still
compiling,
and the log ends in SIGTERM),
the clippy task was not run,
and the final code was not measured again after a last edit that removed per-thread priority resets.
The agent said so plainly,
but the work came back unverified for every gate the brief required.

#### The prescribed stress condition did not reproduce the problem

The brief's four-core, 32-load condition showed zero underruns on the unchanged code too,
so the effect shows only in one-core controls that the agent chose.
The agent also never proved that the PipeWire xrun counter can move,
so its statement that PipeWire scheduling is not a cause is unvalidated by its own admission.

#### Lost track of state it did not own

The agent's null sink vanished between runs without a cause,
and the user's own music player process that was playing at the start was gone at the end.
The agent did not check that process between start and end,
so its exit is unexplained.

### Lesson

For measurement tasks, a Haiku agent will stop early and report honestly what it skipped.
The orchestrator should treat the skipped gates as the remaining work,
resume the agent with exactly those gates,
and require a state check of every process the agent does not own at start and end.

## Taskbar progress fix stopped at a toolchain error (2026-10-11)

### Goal

Reproduce the jumping KDE taskbar progress with the nested compositor's launcher capture,
fix it,
and show before and after streams.

### Mistakes

#### Returned an interim stop as the final report

After about ten minutes the agent handed back a report that said it was waiting on a background build.
It had made no capture and no edit,
so the hand-back carried only static code reading and a build blocker.

#### Stated a toolchain claim that a direct check contradicted

The agent reported that all five installed nightly toolchains fail to compile `branches` 0.4.4 with
`core::intrinsics::abort`.
A scratch crate that depends on `branches` at exactly 0.4.4 compiled cleanly with `nightly-2026-09-22`.
The failure belongs to the newest nightly that an upgrade had just installed,
not to every installed nightly.
The agent then built
with the stable toolchain through raw `cargo` as a workaround instead of reporting the exact toolchain
boundary.

### Lesson

A claim of the form "every installed X fails" needs one run per X.
When a build blocks a task,
the agent should isolate which toolchain version breaks and report that,
and the orchestrator should check a blocker claim with a minimal reproduction before building plans on it.

## Kotlin ports and Compose screens (2026-10-10)

### Goal

Port the track template engine and keyboard classifier to Kotlin,
then build the Compose screens of the accepted design,
as recorded in `doc/planning/music-player-production-implementation.md`.

### Mistakes

#### Missing KDoc on most declarations

The first pure-logic ports compiled and mirrored every reference case,
and detekt then reported 97 findings,
mostly missing KDoc on local values.
A second Haiku round fixed them.

#### Drew text and icons black on the dark canvas

The first native captures of the player screen showed text and icons in black on the dark canvas,
because the agent provided no content color.
The same gap appeared again in the debug host for the Search, Settings and template editor routes.

### What worked

The ports mirrored the reference tests case by case,
and the orchestrator's own reruns agreed with the agents' reports for that kind of pure-logic task.

### Lesson

Pure logic with a tested reference suits Haiku.
Compose screens need captures on the real surface in light and dark,
and the brief should state the lint rules (KDoc on every declaration) up front.

## Native commit-path landing lane (2026-10-11)

### Goal

Port the landing loop of the Rust Git policy wrapper (`package/git-policy/cli/src/native/`) in the worktree
`.claude/worktrees/cli-git-lane-landing`:
phase markers, landing support, object migration, post-index, the real index lock install, the landing attempt and
its acceptance tests and gates.
The lane's record is `doc/handover/cli-git-native-landing.md`.

### Mistakes

#### Handed back after groundwork, four times

The first hand-back came with the phase markers uncommitted and no result for their tests.
After the orchestrator resumed the agent with the ordered step list and the instruction "do not hand back until done",
it handed back three more times, each with the lane unfinished and each time asking to be resumed.
The harness also sometimes drops hand-back reports, so every early hand-back cost a watchdog cycle to detect.

#### Misread its own commit as not landed

After a commit through the wrapper printed a logger error ("sink verification failed for entry 3: Timed out after
5000ms"), the agent reported that the commit did not land and the files were still staged.
The branch log showed two commits from that attempt (`a3b6fb29b8` and `15ee7bdda0`).
It did not run `git log` before reporting.
The orchestrator, trusting the report, committed the staged files again and produced an empty duplicate commit.

#### Proposed a behavior regression as a deviation

The plan wrote the PID file before `index-lock-<n>.json`, the reverse of the TypeScript incumbent, and listed it under
"Deviations to record" while noting that recovery cannot attribute a lock in that window.
A window in which recovery cannot attribute a lock is a regression against the incumbent, not a documentation item.
The orchestrator told the agent to match the incumbent's order or to prove it impossible and make recovery tolerate it.

### What went well

The agent kept its evidence doc and said plainly what was unconfirmed and not started.
Its landing support, object migration and post-index modules passed their tests.

### Lesson

For a serial slice with crash-consistency requirements, send the lane to a Sonnet subagent from the start.
When a Haiku agent reports the state of the repository (committed, staged, clean), read the repository
(`git log`, `git status`) before acting on the report.
A brief that says "do not hand back until done" did not hold a Haiku agent to it.

## Slint IDE scroll pipeline and soft wrap (2026-10-11)

### Goal

Make the source view of the read-only Slint IDE (`package/ide/app`) scroll smoothly even under extreme host
load, then wrap long lines like IntelliJ does.
Haiku subagents did the stage 1 overscan tile, the old-versus-new scroll measurements, the stage 2 worker thread tile
build, the code surveys, the Parley research, and the wrap shaping measurement.
The plans are `doc/planning/slint-ide-scroll-pipeline.md` and `doc/planning/slint-ide-soft-wrap.md`.

### Files involved

- `package/ide/app/src/native/tile.rs` and `tile_build.rs`: the stage 2 worker.
- `/home/user/temp/agent/ide-scroll-new-vs-old/ide-scroll-build-compare.mjs`: the measurement harness a Haiku agent
  wrote.
- `/home/user/temp/agent/ide-wrap-measure/`: the wrap shaping measurement.

### Mistakes

#### A measurement harness whose denominator made a healthy result look broken

The harness a Haiku agent wrote counts the 1.5 s idle wait after the last wheel step as "possible frames".
Even perfect painting of the 1.9 s input phase therefore scores about 115 of 204.
The agent reported "about half of the possible frames painted" for every renderer and both builds,
with no positive control that could show a better number,
and the orchestrator repeated it in a plan, the handover and a landing message before a second Haiku agent read the
harness source and found the denominator.
A rerun restricted to the input phase painted 103 of 115 slots.
The same agent's `rendersInScroll` counts a log line for viewport tile presentations, not paints, which the report did
not say.
The agent also never wrote the summary step of its brief.

#### Interim hand-backs, more than twenty of them

The stage 2 implementer was told to send one final report only when everything had finished.
It ended its turn while its own gate units were still running more than twenty times;
each time the harness delivered a notice that carried no report ("has not reported yet"),
and each one woke the orchestrator.
Waiting for a gate belongs in one blocking command (an `until` loop on the unit), not in ending the turn.

#### Chained gate commands with a missing argument, and lint failures on its own code

The guard tasks need a Cargo target cache argument.
The agent's chain ran `inspect:sidebar-guards` and `inspect:theme-guards` without it and they failed with
"Missing required arg: <cache>";
`inspect:find-guards` failed in the same chain without it and passed with it.
Its first full lint run also failed clippy, rustfmt and the release lint on the new worker code,
because it had not run the lints before reporting the feature as built.

#### Deferred a known defect, and a drop that blocks the window close

The worker's error arm kept the dead worker and re-logged every 20 ms.
The agent listed the fix as planned and deferred it because a gate was compiling;
the orchestrator found the defect in review and had it fixed.
Dropping the worker joined its thread, so closing the window waited for a tile build in flight;
the `inspect:bundle` close timeouts were consistent with that.
The agent had flagged neither in its design doubts as a close-path risk until asked.

#### A test that was never shown to fail, and that would have hung

The first version of the drop test dropped the worker inline.
With the join restored it would have hung the suite instead of failing.
This surfaced only when the orchestrator asked for the negative control;
the agent then rewrote the test to drop on its own thread with a bounded receive and showed it failing.

#### Lost its own work product, and refused a report file

The wrap shaping measurement agent copied its test file to the scratch directory in parallel with
`git worktree remove --force`, the copy failed on a name clash, and the file was lost;
it rebuilt the file from memory and said so.
Its first full run failed on two errors of its own: a missing `mut` and a control that read `full_width()` of an
unbroken layout, which is 0.
It left 966 rows with an advance wider than the width unexplained.
The Parley research agent declined to write the report as a Markdown file ("my agent rules forbid writing report .md
files") although the brief asked for it, so the orchestrator wrote the doc from the hand-back.

### What went well

The stage 1 implementer landed the overscan tile with a structural test and a guard that was seen failing.
The survey agents gave file and line evidence and marked what they could not verify,
which made the stage 2 design reviewable.
The stage 2 implementer reproduced the edit test flake (5 of 20 failures under load), measured the cold start cost
instead of accepting the orchestrator's hypothesis, found the real cause (the poll timer stopping while a cancelled job
was in flight), and proved the fix (0 of 30).
It reported its flakes and its own mistakes plainly.

### Lesson

Send work that spans many modules or needs a design judgment (the soft wrap rows model, the worker integration, keys
and pointer) to a Sonnet subagent from the start; keep Haiku for surveys, measurements and mechanical fixes with an
explicit step list.
Before trusting a Haiku measurement, read what the harness counts and ask for a positive control that can show the
good number.
Tell a Haiku agent to wait inside one blocking command and to run every task with its documented arguments,
and read `git log`, the gate JSON and the unit state yourself before acting on its report.

## Native commit-path post-landing lane (2026-10-11)

### Goal

Port the post-landing completion of the Rust Git policy wrapper (added-path copies, shadow and transaction removal,
automatic maintenance, the `post-commit` hook under the hook lock, capture-record pruning).
The lane's record is `doc/handover/cli-git-native-post-landing.md`.

### Mistakes

#### Went silent for almost an hour with a stale record

After two commits (`136cf6d0fd`, `89d0fc7a0c`) the agent made no commit, no file change and no gate run for 57 minutes
while the harness still listed it as running.
Its evidence doc still said "Nothing is committed beyond the skeleton yet" and "Modules written, not yet green",
so a reader of the doc alone would have believed the lane was a different, earlier state.
The orchestrator stopped the agent and handed the lane to a Sonnet agent.

### Lesson

Check the dated state of the repository (`git log`, file modification times) against the evidence doc before trusting
either; a Haiku agent's doc can lag its commits by hours.

## Slint IDE soft wrap, slice 1: library pieces (2026-10-11)

### Goal

IntelliJ's break rules as a Parley override, the continuation indent, wrapped shaping of one logical line, multi-row
geometry in `ShapedRow`, and the caret affinity bit, with no UI change.
The branch is `feat/ide-soft-wrap`; the brief required a seen-failing check per rule.

### Mistakes

#### Replayed a commit and produced duplicates

A chained commit landed, the agent then retried it in the foreground, and the wrapper replayed it onto a new base
after a landing race.
The branch holds two commits (`4c41c7150d` and `7bf157b370`) with identical content.
The brief said never to repeat a commit command that seems to hang and to check `git log` first.

#### Mutation evidence with gaps it only partly named

The agent inverted every predicate in `src/wrap.rs` at once and counted 9 of 15 unit tests failing.
Two tests (the grapheme guard precedence and the pairs without a rule) still passed under inversion,
so they do not isolate their predicate;
the integration tests were not run under mutation because the library run stopped first.
It said so, but the brief asked for every rule seen failing, and that was not met.

#### Gate runs in a loop

It ran the gate six times, fixing clippy implicit returns, shadowing and rustfmt one round at a time,
because it did not run the lints before the first gate.

### What went well

It stopped on a real conflict (adding a field to `ReadingPosition` breaks about 47 struct literals in existing tests,
and the brief forbade editing them) and asked, instead of editing the tests.
Its list of differences between the plan and IntelliJ's Java source was specific (identifier rule, both-sided space
and tab breaks, BMP-only eastern rule, the indent from the first non-blank x), and it named what Parley cannot
reproduce (the weight product, the look-back, the beyond-width fallback).

### Lesson

Run each lint locally before the first gate and make mutation evidence per rule, with the test that failed named for
each; a combined inversion hides tests that do not isolate their rule.
When a commit seems to hang, read `git log` before retrying.
