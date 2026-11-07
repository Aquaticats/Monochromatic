# Questions for the owner

Open questions from the coordinator session,
newest first.
This file is open in Zed,
which reloads it when it changes on disk and the tab has no unsaved edits.
Write your answer under a question and save;
the session re-reads this file before every update,
so a saved answer is never overwritten.
An answered question moves to the canonical doc it belongs to and leaves this file.

For this session the owner said:
"Please make any non-destructive options (when they are the ones I would likely have picked) yourself from now on for
this session."
So this file holds only choices that are destructive,
reach outside this machine,
or where the likely pick is unclear.
The non-destructive choices the session made itself are listed at the end,
so any of them can be undone.

## Open: may the decision records get the linter's whitespace-only line breaks? (not asked yet)

The repository-wide lint pass left `doc/decision/` alone (rule:
never edit `doc/decision/` without the owner's approval):
1,785 findings in 172 files.
A probe on a scratch copy:
32 files change,
31 of them whitespace only;
`music-player-third-party-licenses.md` would also gain `>` markers in a blockquote,
which is a linter defect being fixed now (branch `fix/monochromatic-lint-fix-changes-only-whitespace`).
The choice:
allow the whitespace-only reflow of `doc/decision/` after that fix,
or leave the decision records as they are and exclude them from the line-break rule
(an exclusion loosens the rule,
which also needs approval).

## Open: how the repository linter stays at zero (not asked yet)

Nothing ran the repository linter before today:
the commit path checks one Markdown rule at `warn`;
CI's step fails with `monochromatic-lint: command not found` and is skipped by its budget gate in 394 of 400 runs;
`mise run lint` is run by nothing.
Proposal with five options and the ranking A > C > B > D > E:
`doc/planning/repository-linter-stays-at-zero.md`
(recommendation:
a check-only commit policy for all Markdown rules,
then a required whole-tree CI workflow).
A commit-time check adds to the commit path,
which the owner's 5 s rule bounds.

## Open: where a blank at the end of a row goes in the IDE (not asked yet; the owner asked to focus on the Git wrapper)

The text-layout library the IDE uses (Parley) moved to 0.12 in the daily update,
and it now follows the CSS Text 4 rule for blanks at a line end:
an overflowing blank "hangs" past the edge instead of staying on the row and pushing what follows down.
Two IDE tests describe the old behavior and now fail
(`ide::wrap_keys a_hanging_blank_at_the_end_of_the_text_leaves_no_empty_row`,
`ide::wrap_keys the_fixtures_wrap_as_described`);
their fixture text says "an overflowing blank stays on the upper row,
so a tab right after it starts the next row".
Measured at width 240,
scale 1:
a continuation row now starts with a tab (rows `0..28`,
`28..38`),
and no fixture produced an empty tail.
Source:
`parley-0.12.0/src/layout/whitespace.rs` (header doc),
merge `d2598b2842`.
The choice:
keep the IDE's old rule (our code moves the blank back)
or adopt the CSS rule (the tests change).

## Answered 2026-11-07T13:52:06-05:00 to 13:57:11-05:00 (question tool, then plain messages)

- Enforcing "agents never run sudo" after an agent ran `sudo -n true`
  (options:
  guardrail hook plus AGENTS.md rule,
  AGENTS.md rule only,
  leave as is):
  "Leave as is".
- Rule SCR (no bash scripts) against the wizard skill's bash root scripts:
  "Wizard writes TypeScript,
  and you just do the move yourself,
  no need to delegate to me."
  The 860 move is run by the coordinator,
  not handed to the owner;
  its runner becomes TypeScript (sent to the preparation agent).
  The wizard skill itself is to emit TypeScript (queued).
- The plugin bundle drift check (options A,
  B,
  C):
  "Open a gh issue.
  Do not address it in this session as we need to focus on our git wrapper."
  Opened as issue #6265;
  nothing else this session.
- Sudo,
  in plain messages after the questions:
  "Please run sudo."
  Then:
  "Please run sudo -n true to test,
  and run sudo whenever needed for the move,
  and run sudo for anything.
  I authorize you to do and use all of it without my prior approval."
  Then:
  "I authorize you to send any messages about sudo to subagents."
  The coordinator ran `sudo -n true`:
  exit status 0 (passwordless).
  Coordinator's practice:
  subagents still run no privileged command;
  they send the coordinator the exact command,
  so every privileged command has one record in the main session.
- Also said:
  "I'll be away for a few hours."

## Answered 2026-11-07T11:52:20-05:00 (in the question tool)

- How much longer than the measured worst case the Git wrapper waits before giving up,
  for the wait before force-killing a Git it asked to stop
  and for the wait on a file name another thread is publishing
  (worst case 6.03 s,
  seen once at load about 70 on 28 CPUs,
  cause unmeasured):
  "3× worst case",
  so about 18 s for both.
  Over the tokio agent's ten times and the two times.
  Built on `feat/git-policy-cli-on-tokio`,
  recorded in `doc/planning/git-policy-cli-full-tokio.md`.
  With it the coordinator settled,
  from "Commits should never fail."
  and "nothing should ever wait forever or indefinitely":
  after the 5 s grace a stop keeps waiting only while a member is in uninterruptible disk sleep,
  and force-kills at the bound at the latest.

## Answered 2026-11-07T11:05:21-05:00 (in the question tool)

- Group folders:
  "Shelf + summaries (Recommended)".
- Libraries:
  "Four libraries (Recommended)".
- Library logs:
  "tracing (Recommended)".
- All three live in `doc/planning/cli-git-source-layout-target-tree.md`.

## Answered 2026-11-07T02:03:01-05:00 (in the question tool, with the owner present)

Each answer now lives in its canonical doc;
nothing here needs an answer.

- The rename note in old records:
  "Run the fix as described".
  Done in `31f3223d25`
  (the note left the 6 decision records and was reworded in the other 111).
- Which crates go to a registry:
  replaced by "deploy all releases on pnpr,
  but only publish rc and stable releases on
  crates.io",
  with "Label decides" for what counts as rc and stable (`doc/planning/crates-io-footprint.md`).
- pnpr login for crates,
  pnpr version,
  crate copies:
  "Patch pnpr in your fork",
  "Together with Cargo",
  "No copies,
  like npm" (`doc/planning/self-hosted-cargo-registry.md`).
- When the one release of each crate happens:
  "After the test gap is fixed" (`doc/planning/crates-io-footprint.md`).
- Adding QFL to AGENTS.md:
  "Do not add it".
- One shared build folder for the split packages:
  "One shared build folder,
  but first,
  I'd like you to confirm with me
  each pkg can stand on its own itself."
  (`doc/planning/cli-git-source-layout-target-tree.md`).
- The wrapper's target layout:
  "Not good enough,
  assume every crate must stand on its own and no one else is going to
  use our crates to build a git wrapper."
  The crate split is being redone against that rule;
  no file moves until the
  owner confirms the redone tree.
  The open points on how far to split and the program folder's name come back with it.

## Choices the session made itself (non-destructive; say "undo" next to any of them)

- 2026-11-05T03:58:47Z,
  program rename:
  this repository's settings file keeps its old name `cli-git.config.jsonc`
  until the release step,
  when what this repository installs switches to the new program.
  Reason:
  the installed wrapper is the old program and reads only the old name,
  so renaming the file earlier would run every commit on `main` with no policies until then;
  the new program reads the old name too,
  with a one-line notice.
  Your "rename them."
  still holds;
  only the timing of this one file moves.
  Done 2026-11-07T08:54:45-05:00 (`c0b4812639`,
  merged in `71498b658f`) once 0.6.0 was installed.
- 2026-11-05T03:58:47Z,
  program rename:
  the link folder `.cache/cli-git-bin` becomes
  `.cache/monochromatic-git-policy-cli-bin` at the release step
  (you said to rename it and gave no name;
  this follows the program's name).
  Done 2026-11-07T09:00:24-05:00 (lock `f44c638269`,
  merged in `71498b658f`).
- 2026-11-05T03:58:47Z,
  program rename:
  before the renamed program is released,
  it must finish any work an old
  background helper left half done (an agent is checking whether that can happen and fixing it if so),
  following "never discard user data".

- 2026-11-05,
  target layout:
  the proposal's technical points are taken as drafted:
  the failure codes stay with the commit service's messages;
  the crash points for tests and the durable index-lock code get their own crates;
  the source-text guard tests move to the program crate;
  sidecar tests move with their code and no new tests are written for the move;
  the two extra call changes (the copy error type,
  the hint functions moving with their users);
  the `include` list goes at the split;
  pins are kept in step by the catalog sync;
  names that fail the "monkey" test are renamed while grouping;
  `commit-command`,
  `doctor-report`,
  `merge-in-isolation`,
  `typed-push-planning` fold into the program,
  and `service-after-commit-work` into the commit service program,
  because no outside program would use them alone;
  `new-worktrees-quickly` is split first after the lower half.
- 2026-11-05:
  the layout rule reads a folder's file-list marker even when it wraps over two lines,
  because the repository's line-break rule demands a break after its comma (merged to `main`;
  854 tests,
  122 in the container;
  the "no file list" help now shows the wrapped form).
  A blank `//!` line or a plain `//` comment inside the marker still means no marker.

- 2026-11-05:
  index files in the new layout lint rules (`mod.rs`,
  and `lib.rs` used as an index) may hold only
  the top doc,
  `mod` lines and re-exports;
  anything else is a finding,
  and a name reached through a re-export counts as a use of the file that defines it.
  Reason:
  your principle "Who depends on who must be immediately visible from looking at the file structure";
  code inside `mod.rs` hides who uses whom.
  Built on branch `feat/monochromatic-lint-index-files-hold-only-declarations` and merged to `main`
  (840 tests pass,
  108 layout tests in the container);
  rules stay off by default.
  On the wrapper it finds no index holding code,
  and no loop was hidden behind a re-export.
  A `main.rs` that is its folder's index may hold `fn main` and its `use` lines as well;
  `extern crate` and an inline `mod` with a body count as code.
  The same check now also covers a `main.rs` beside a `lib.rs` and single-file `src/bin/*.rs` programs
  (merged to `main`;
  842 tests,
  110 in the container).
  A repo-wide survey with the file rule on finds 74 entry-point findings in 15 program files outside the wrapper,
  and 107 index-file findings in 11 files;
  none in the wrapper.
- 2026-11-05:
  after the four lint branches land,
  one more pass removes most closure-lint suppressions
  by giving the called functions a guard or trait form,
  keeping only those that `std::thread::scope` and `catch_unwind` cannot do without,
  and merges the three suppression notes into one note beside the wrapper's code.
  Reason:
  the repository's lint rules say to restructure rather than suppress.
- 2026-11-05:
  the one test function inside `testing/no_threads_child_test_support.rs`
  moved into a `*_tests.rs` sidecar,
  which clears the last `rust/tests-in-sidecars` finding in the wrapper
  (`5983d1603d` on the lint integration branch;
  the probe and a test that relies on it pass).
- 2026-11-05:
  our seven tools will install from their GitHub releases (the `archive` route `mpm-shadow` uses)
  and stop publishing to crates.io after their one release each.
  Reason:
  it is your first pick ("a workspace:
  protocol or something"),
  and the research found it the only route that works both for crates using each other and for installing tools.
  Not started:
  it waits until the layout and lint work land,
  as you asked.
- 2026-11-05:
  the toolchain lock keeps following the newest GitHub release of each tool,
  as `mpm-shadow`'s row does,
  rather than the version written in the checkout;
  that needs no new code.
- 2026-11-05:
  test files stay inside the crates we still publish.
  Reason (`doc/research/tests-inside-published-rust-crates.md`):
  Debian,
  Fedora,
  Gentoo,
  NixOS and the Rust
  project's crater runs all run `cargo test` from a published crate,
  and leaving our sidecar test files out makes that fail with "file not found for module" (E0583)
  unless the sources change too.
  Once the tools stop publishing,
  what is left to publish is small
  (about 153 KiB of test code in `forbidden-strings` and 165 KiB in `forbidden-regex`,
  before compression).
