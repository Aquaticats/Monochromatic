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

## Open: decisions a read-only audit of all recorded work found, recorded 2026-11-08T16:05:49-05:00

None of these has been asked yet,
and none is holding up running work.
Each record is cited so the context is one read away.

- Pi agents started through the driver no longer inherit a credential-named variable (`9ae27d221e`).
  Two choices follow from it.
  Should mise get its own GitHub token home (`github.credential_command` or `mise token github`),
  since a subagent's `mise` now runs anonymously at 60 requests an hour instead of 4,875?
  Should `SSH_AUTH_SOCK` be allowed through for `ssh m1` probes?
  Nothing has proved it necessary.
  (Interactive Pi sessions keep their environment, answered 2026-11-10T08:23:02-05:00.)
- Do the `container-`,
  `windows-` and `test-` record prefixes belong in `cargo-shadow lean`'s list?
  And what should happen to the eighteen logs redirected by hand,
  and the one renamed 17.9 MB record,
  in the linter's verification folder?
- Fuzzing the wrapper: may the artifact reference stand without a landed evidence file?
  (Whether the checked-in seeds get a cheap native control in normal tests was answered
  "Leave them to campaigns", 2026-11-10T08:24:33-05:00.)
- The unmerged copy-in-service branch:
  the audit in `doc/planning/cli-git-copy-route-findings-on-main.md` ranks B over C over A.
  The coordinator picked option B,
  with the veto open:
  port three narrow fixes into `main` and leave the branch as it is,
  neither merged nor deleted.
  The reason is that a kill during a worktree add on the copy route silently loses that worktree's ignored files,
  and recovery can fill a registration the command did not create.
  Two agents are building it.
  A new `git worktree add` from a worktree still being filled now gets no ignored-state copy and one line saying so.
  Two gaps stay open:
  a copy still staging records no destination,
  so it is not detected until the record written before Git is on `main`;
  and the snapshot and clone routes can still carry a half-filled set from a busy source.
- `package/hall-monitor/daemon` fails its own `lint:oxlint` on `test-import(require-eventual-artifact)`,
  two findings before and four now that it has the tests issue 1297 asked for.
  The rule needs an import from a built artifact,
  and that package has none.
  Should it gain a `dist` build,
  or an exemption?
- `mpm-shadow` launcher records named a folder carrying the version,
  so any newer install left them dangling (299 of 1051 records at 2026-11-08T18:17:00-05:00).
  The fix on `fix/mpm-shadow-launcher-records-and-lock-warning` records a path that names no version.
  The records already dangling heal on the launcher's next run,
  which is the coordinator's pick with the veto open;
  the alternative is an `mpm install` sweep over every worktree after the release.
  The newest-installed launcher pick has no `doc/decision/` record yet.
- The umbrella list (`225a3b6318`) has three open points left:
  which of its disagreeing maintenance mutation records is right,
  whether to relink the 16,177 dangling links it counted in 172 of 239 readable worktrees,
  and whether its release-artifact gate also meant a check of the archive's own contents.
  Its Windows and release-gate entries are split in
  `doc/handover/cli-git-rust-implementation.md`,
  where the gate's latency half is answered:
  `native:perf`,
  run against the installed binary,
  measures what mpm installs from the release tarball,
  its limits are its own task description,
  and the release records of `doc/planning/one-release-each-batch-2026-11.md` hold its figures.
  A search of `doc/` and `.github/` on 2026-11-08 for `release-artifact` and `release performance`
  found only items forwarding to that list.
- The questions that end `doc/planning/cli-git-shipped-limits-audit.md`,
  which ask what each shipped limit should cite.
- Does "never discard user data" cover the audit history?
  The timing logs and the push trigger log keep one older generation,
  so a rotation drops the one before it.
- Should `package-paused/dev-script/inference-canary`,
  `package-paused/webapp-edu/paper2vn` and the wrapper's own test runners get the container limits?
- Should the mount unit's stop block guard its `luksClose`?
  A stop with no mapper now fails where the old stop hid the failure
  (`doc/handover/repository-move-dangling-link-items.md`).

Checked and not asked,
because you have already answered them:
deleting the merged branches with nondescript names (done with your approval),
and bumping and publishing ("I'm authorizing you to bump and publish any updates this session.").
The two library layout questions stay at your "Not yet.".

## Coordinator picks made 2026-11-07 with your veto open (not asked yet)

Each is built and in main;
saying nothing keeps it.
- How the 860 move copies each subvolume:
  every first-pass copy is sent against one empty read-only snapshot,
  with every earlier copy as a clone source,
  and only the final pass sends a subvolume as an increment on its own first-pass snapshot.
  The plan it replaced sent a worktree as an increment on a snapshot of `main`,
  which the kernel's send code can pair with the wrong file
  (it treats two inodes as one when their number and creating transaction match,
  `fs/btrfs/send.c` `changed_inode`);
  on the throwaway fixture that plan failed,
  and the new one copied every file exactly and kept sharing.
  Record:
  `doc/troubleshooting/btrfs-send-clone-sources-need-a-parent.md`
  and `doc/planning/repository-on-the-860-drive.md`.
- `monochromatic-lint --fix` now applies only whitespace fixes,
  and content-changing fixes need `--fix-content`
  (0.5.2-alpha.5);
  the repository's `format:monochromatic-lint` task therefore changes no word.
- When the fixer breaks a line inside a blockquote,
  the new line carries no `>`
  (a lazy continuation that renders the same under CommonMark 0.31.2,
  section 5.1,
  rule 2),
  so the fixer never adds a character to your text.
- After the repository move,
  the wrapper checks on every Git command that the repository link still leads to the 860
  (W2,
  merged `b842b34fff`),
  so a dangling link gives a plain message instead of creating things in the wrong place.
  Measured cost:
  about 1.0 ms per forwarded command (median,
  `status`;
  0.8 ms for `rev-parse`),
  beyond the same-binary spread;
  nothing before the move.
  Cheaper forms (skip the check for read-only commands,
  or trust a cached answer across commands) would let a dangling link go unnoticed,
  so the full check stays.
- Tokio step T10
  (the commit service's connections,
  claim renewal and syncs become tasks;
  gate 5313 of 5314,
  not merged yet)
  raised one idle service from 4 threads to 27,
  because the runtime starts one worker per CPU.
  Many services run at once,
  so the coordinator picked a cap on workers,
  measured at 2,
  4 and 8,
  taking the smallest within the measurement spread and never below 2
  (your ruling "Full on tokio.
  Multi-thread.").
- T10 also made the service-memory fault test find no limit at which the service starts but a commit cannot land
  (the service now reserves room for a commit before it starts).
  The agent recommended accepting that,
  which would leave the test testing nothing;
  the coordinator picked instead lowering the running service's address-space limit after it starts,
  so the test still reaches exhaustion in the middle of a landing and checks that the repository recovers.
- The claim-renewal stop still waits without a limit
  (as before T10),
  against your rule that nothing waits forever;
  the coordinator picked a measured ceiling,
  with the claim expiring by its lease if that ceiling passes.
- An equivalent mutant that no test can kill
  (`workspace_count -> 0` in `package/linter/monochromatic-lint/src/rust_file_engine_syntax.rs`,
  a function that already returns 0)
  stays recorded as equivalent,
  so the linter's `--syntax-only` mutation campaign reports one miss every run
  until `mutation:coverage` learns to skip recorded equivalent mutants
  (record:
  `doc/handover/monochromatic-lint-layout-and-workflow-mutants.md`).
- The wrapper's own `markdown/autofix` commit policy keeps applying its one content fix
  (the LFS image link rewrite),
  since that rewrite is the policy's whole purpose.

## Open: lean build folders, after the first run (not asked yet)

Built and run 2026-11-07 to 2026-11-08 (cargo-shadow 0.1.5-alpha.7;
details and per-removal proof in `doc/handover/cargo-shadow-lean-build-folders.md`).
The first run freed 128.1 GiB on the 860,
retired 12 of 17 dated nightlies (52.95 GB to 22.47 GB)
and removed 251 Podman images and 27 volumes;
the daily timer runs at 04:30 and never at login.
- Question:
  should images held only by exited containers be removed too?
  Today a container in any state keeps its image.
- Coordinator picks,
  veto open:
  - an old build tree of a live worktree
    (the folder Cargo used before the toolchain-keyed folders existed)
    is retired after the 7-day idle period once a keyed folder exists for the same workspace;
    without this nothing ever removes those trees,
    and most of the 6,884 trees the first run kept are these;
  - the size caps (E) read the same fingerprint fields as the sweep (B);
    your amendment said "for that sweep only",
    and the agent extended the reader to the caps because they evict by the same units;
  - grace 7 days,
    unidentified trees 21 days,
    run records 14 days and 2 GiB per folder,
    caps 12 GiB in a checkout and 24 GiB in the shared build folder,
    each measured (`doc/planning/lean-build-dirs.md`).
- The timer's pass order and Podman's three refusals are fixed in the sources (merge `a5b31ab7de`);
  the facts are in `doc/handover/cargo-shadow-lean-build-folders.md`,
  and they reach the timer with the next `cargo-shadow` release.

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

## Choices the session made itself (non-destructive; say "undo" next to any of them)

- 2026-11-10T04:28:34-05:00,
  issue 613,
  the fuzz coverage harness copied across the sidecars:
  one shared package that keeps both coverage generations,
  the issue's own first-ranked option,
  so no sidecar's baseline changes meaning.
  Say "undo" to take line coverage only and refreeze the baselines.
- 2026-11-10T04:28:34-05:00,
  issue 1355,
  the IDE inspection runs that need `rust-analyzer`:
  each run gets its toolchain for that one command,
  so the host's global `mise` configuration stays as it is.
  Say "undo" to set a global Rust version instead.
- 2026-11-10T04:28:34-05:00,
  issue 1372 of `package/kwin/key-helper`:
  discovery tries the remaining sockets when the newest is stale.
  Say "undo" to keep giving up at the first stale socket.
- 2026-11-10T04:28:34-05:00,
  issue 1369 of `package/kwin/key-helper`:
  region markers go on the whole package and not only on changed files.
  Say "undo" to limit them to changed files.
- 2026-11-10T04:28:34-05:00,
  issue 1673:
  the shared text-scan package also gains the run-scan helper the issue asks for,
  replacing the remaining hand-written scan loops.
  Say "undo" to keep those loops.
- 2026-11-09T19:25:35-05:00,
  issue 1555 of `package/ssg/aquati.cat`:
  a post's dates compare calendar days in the git author's own offset,
  carried through `ResolvedDates` and the build manifest,
  so the answer does not depend on the machine that builds.
  Say "undo" to compare in the build machine's zone instead.
- 2026-11-09T19:25:35-05:00,
  issue 1557 of `package/ssg/aquati.cat`:
  `@lezer/json` joins the catalog and that package so JSON fences keep their highlighting.
  Say "undo" to drop the alias and leave JSON fences unstyled.
- 2026-11-09T19:25:35-05:00,
  issue 1541 of `package/ssg/aquati.cat`:
  an agent writes the alert labels and the brand-image alt in Catalan and Chinese,
  and `lang` goes into the MDX component scope,
  which makes the cached post HTML depend on language.
  Say "undo" to keep English labels on every language.
- 2026-11-09T19:25:35-05:00,
  issue 1549 of `package/ssg/aquati.cat`:
  the dark-theme dropdown shadow is `rgb(0 0 0 / 45%)`,
  and the light theme keeps its earlier alpha.
  Say "undo" to pick another value.
- 2026-11-09T19:25:35-05:00,
  issue 1400 of `package/logger/core`:
  a record buffer holds its page lifecycle listeners only while records wait,
  and no public teardown method was added,
  because nothing calls one today.
  `DECISIONS.md` in that package records the alternative.
  Say "undo" to add `Sink.dispose` and a `Logger` method.
- 2026-11-09T19:25:35-05:00,
  issues 1500 and 1510 of `package/pi-shared/model-review`:
  `assertModelUsesApi` is gone,
  because a table keyed on `model.api` cannot mismatch,
  and the Bedrock and Google dispatch branches get tests through a fake table,
  so `@aws-sdk/client-bedrock-runtime` and `@google/genai` do not become dependencies for tests alone.
  Say "undo" to restore the assertion or add the SDKs.
- 2026-11-09T16:22:51-05:00,
  the merge's second pass becomes awaitable as one body:
  `GitRunner::run` hands back a boxed future,
  and the forwarded `git merge` path keeps one synchronous face that enters the runtime once.
  The owner's "Full on tokio.
  Multi-thread."
  settles that it goes awaited,
  and the alternative is two bodies that must not diverge,
  in the code a person's conflict resolution depends on.
  Both ways are recorded in `doc/planning/git-policy-cli-full-tokio.md`.
  Say "undo" to have awaited twins instead.
  Carried out on the branch `refactor/git-policy-cli-merge-second-pass-awaited`
  at `403254deeb`;
  the awaited push chain stops at the branch move,
  because the recovery of an interrupted merge still reads Git synchronously,
  as that document's step record measures.
- 2026-11-09T16:22:51-05:00,
  all 22 crates that declare a build profile are now covered by `file-enforcer` presets,
  with a new rule for the shape the shadow crates share (`cargo-profile-shadow`),
  so the "Enforced by file-enforcer" comments are true (issue 1641,
  landed as `f4c972b66f`).
  This was the option this file recommended.
  Say "undo" to delete those claims and leave the crates free to drift.
- 2026-11-09T05:12:19-05:00,
  `monochromatic-git-policy-cli fix` refuses a worktree whose Git directory is on another file system,
  before it writes anything,
  because its crash-safe swap needs both names on one file system (`package/git-policy/cli/SPEC.md`,
  on the branch `fix/git-policy-cli-commit-checks-keep-newer-writes-and-modes`).
  This session reads that branch's report as saying the refusal has not been exercised by a test.
  Say "undo" to have `fix` fall back to a copy in that case,
  which loses the guarantee that a kill cannot leave a half-written file.
- 2026-11-09T05:12:19-05:00,
  the publish-list reader in `dependent_version` accepts only the line forms
  `package/config/pnpr/src/generated-config-render.ts` writes and refuses every other line by name and number.
  No YAML crate was added.
  The agent found one viable crate,
  `yaml_serde` 0.10.7,
  and rejected it because a general parser accepts documents the generator never writes,
  which is the class of silent miss three reviews kept finding.
  Say "undo" to have the crate vetted instead.
- 2026-11-09T04:26:58-05:00,
  a shell alias (`!`) stays unguarded.
  Git runs it with its own folder first on `PATH`,
  so the `git` inside it is real Git and not the wrapper
  (the agent's control prints `/usr/libexec/git-core/git` both ways).
  Refusing every `!` alias would break ordinary ones such as a pretty `log`.
- 2026-11-09T04:26:58-05:00,
  the audit line for `git worktree remove` keeps its count of changed tracked files.
  It costs 0.09 s against 0.73 s for the removal on a full checkout of this repository.
  Say "undo" to drop the count and keep the line to two file reads.
- 2026-11-09T04:26:58-05:00,
  the in-process ref listing lands although it is slower here than starting Git,
  by a median of about 9 ms on this repository,
  because its callers are not on the commit path.
  A follow-up makes it cheaper or leaves large repositories to Git.
- 2026-11-09T03:14:38-05:00,
  `mpm-shadow` now refuses a `cargo-git` plan entry that has no `packageId`,
  where it used to take the plan key as the crate name (issue 2626).
  The cross-vendor review points out that an entry whose key equals its crate name worked before and is refused now.
  No entry in `toolchain.plan.jsonc` or `toolchain.lock.json` uses that route,
  so nothing changes today.
- 2026-11-09T02:08:59-05:00,
  `search-shadow link` when something else already sits at a tool's name:
  it leaves the entry in place,
  names it on stderr and exits 0,
  because the rest of the run succeeded.
  It takes over an entry only when the link resolves to this program,
  resolves to a shadow of any version,
  or is spelled `.../search-shadow`,
  the last so an upgrade can replace a link a version prune left dangling.
  A dangling link named `find` that points elsewhere is left alone.
  Say "undo" to make a conflict exit non-zero or refuse the whole run.
- 2026-11-09T02:08:59-05:00,
  the audit `doc/planning/cli-git-tests-that-pass-without-their-race.md` asked whether its weak controls should get
  fixed now.
  They are queued for a fix agent,
  in test code only,
  under your "Fix flaky tests properly."
  Say "undo" to leave it as an audit.
- 2026-11-08T23:47:13-05:00,
  conclusion record on repositories using the files backend:
  a commit no longer writes an empty `conclusion/store-held.json` (main `c13851b2fa`),
  and the new reader accepts its absence.
  A cross-vendor review found one consequence:
  if a commit is killed before its cleanup finishes and a wrapper from before that change recovers it,
  that older wrapper's cleanup fails instead of completing.
  The review established no data loss,
  and the case needs an older wrapper recovering a newer one's transaction.
  Say "undo" to keep writing the empty record.
- 2026-11-08T22:14:46-05:00,
  secret scanner:
  when `forbidden-strings --all` cannot read a repository's index,
  it prints that tracked files hidden by `.gitignore` were not covered,
  and it keeps its exit status,
  so a scan that finds nothing in the visible files still exits 0.
  The reason is your rule "Commits should never fail."
  The cost is that the commit gate and CI pass with less coverage while saying so.
  Say "undo" to make that case a failure.
- 2026-11-08T22:14:46-05:00,
  TOML library:
  a float wrapper holding a numeric string (`{ tomlKind: 'float', value: '1.5' }`) no longer turns into a number;
  it is refused.
  Only one unit test relied on the old behaviour.
  A plain number or bigint with no wrapper is still written unchecked,
  so a very large one can come out as `1e+21` or outside TOML's range;
  that is unchanged.
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

### A landing waiting for the flush of a marker made ahead (coordinator's pick, veto open)

Agent `commit-waiting-path-with-dozens-of-agents` measured landings waiting 9 to 13 seconds
for the index lock's marker made ahead to finish its flush (`lock_stage_kind=pre`),
and asked whether that marker may be published without its flush.
Under the owner's "Please make any non-destructive options (when they are the ones I would likely have picked)
yourself from now on for this session.",
"never discard user data" and "commits vs native git should not take 5s longer",
the coordinator picked neither side of that trade:
the flush stays, it starts as early as the code allows,
and a landing that finds the marker still flushing takes the lock the ordinary way and never waits for that flush.
The same agent built it (`c72e9f3174`):
the marker of the next batch is started as soon as a landing has its lock,
and a landing that finds the flush still running stages its own,
waiting for nothing,
with the marker left to finish and removed once it is done.
Measured:
`lock_wait_ms` 9,369 and 12,867 with `lock_stage_kind=pre` before,
0 across 46 landings after,
and 8,086 ms of a wrapped commit at the median before against 804 with the host quieter.
Say "undo" to publish without the flush instead.

### Two tokio choices made by an agent (coordinator's pick, veto open)

Agent `tokio-next-step-on-the-runtime`,
building the owner's "Full on tokio. Multi-thread.",
made two choices on branch `refactor/git-policy-cli-tokio-next-step-on-the-runtime`
and recorded both in `doc/planning/git-policy-cli-full-tokio.md`:

- At the service loop's three state steps,
  a job the system refuses a thread for no longer runs in turn on the awaiting task;
  the loop keeps its state and lets the pool or the next wake write it,
  so no storage call ever runs on a runtime worker.
- The stronger fix for a job that starts after its caller gave up
  (the job hands its value back through a store the caller awaits)
  was not built,
  because it changes the helper every command's blocking work goes through;
  the plan records it as the alternative.

Under the owner's "Please make any non-destructive options (when they are the ones I would likely have picked)
yourself from now on for this session.",
the coordinator keeps both as built,
and the next tokio agent builds on them.
Say "undo" for either one.



### A commit whose command died before its checks ran now stays local, changed 2026-11-09T04:26:58-05:00

A landing moves the branch first and runs the post-commit checks after,
holding the push until those checks approve it.
If the command is killed in that gap,
no verdict exists,
and the commit is on the branch with its push held.

This entry used to say the session would push such a commit,
and asked you to overrule that.
The session has reversed it,
for three reasons.
Cross-vendor reviews keep finding defects in the code that decides a command is gone and releases its push,
each in something added to fix the last
(`${HOME}/temp/agent/reviews/landing-route-round-6-sol.txt`).
The worry behind pushing was a commit held for ever,
and reading the code says that does not happen:
a queued push is `git push <remote> <branch>:<target>`
(`package/git-policy/cli/src/native/push/auto_push.rs:230-245`),
so the next commit's push on that branch carries the held commit.
The branch's agent reproduced that with real Git.
And keeping a commit local can be undone,
while a push of a commit whose secret check never ran cannot.

So the branch `fix/git-policy-cli-landing-recovery-keeps-the-owed-index-update` is being changed:
nothing releases a held push by judging a process,
and the next command in that worktree says in one line that the commit is held and why.
Write "push it" here to overrule this.

The same fact raises a question that does need you,
`What should the next push do with a commit its checks blocked?`.


### Should `add-explicit` judge a pathspec by its spelling or by what it selects?

The session picked the audit's recommendation,
extending the fixed list,
because it starts no process and is non-destructive.
A fix agent is queued for it.
Write "ask Git" here to change that.

The policy exists so `git add` names the paths it stages
(`package/git-policy/cli/src/native/command/rule_add_explicit.rs:1-20`).
It reads the arguments alone and compares each positional pathspec with four
byte strings,
`.`,
`./`,
`*` and `:/`
(`package/git-policy/cli/src/native/command/command_add.rs:113`).
Five spellings that Git resolves to the whole tree pass that list and staged
every file in a reproduction:
`:/*`,
`:/**`,
`:(top)`,
`**` and
`:(glob)**`
(`doc/planning/cli-git-policy-and-candidate-route-findings-on-main.md`,
finding 2).
`:(top)` is the one a person writes expecting a scope rather than a sweep.
A sixth,
`:(top)/*`,
fails inside Git 2.56.0 (`fatal: oops in prep_exclude`) and is reported as
`content-unavailable`,
which is Git's own failure and not this choice.

Recommended:
extend the list with those five and name them in the rejection text,
because the decision stays argument-only and starts no process before Git
does,
and the accidents this policy is about are the common spellings.
The alternative asks Git what the pathspec selects,
which catches every form including ones added to Git later,
at the cost of one Git start per `git add`
and a decision that no longer follows from the arguments.


### Picks made for the owner, veto open

These fall under the owner's "Please make any non-destructive options (when they are the ones I would likely have
picked) yourself from now on for this session."

- Issue 1678 is closed:
  the logger half is fixed on main,
  and the remaining matches of its acceptance regex are `console.*` terminal output,
  which rule `TLG` permits.
- Issue 1396:
  a bare `-0` in `@monochromatic-dev/toml-edit-core` emits `-0.0`,
  so the sign survives a reparse.
- `catalog-tighten`'s `test:matrix` adopts the passing arm of
  `doc/troubleshooting/catalog-tighten-matrix-cannot-start.md`: mount the resolved store read-only and take pnpm's
  first version line.
- The first trash hand-over in a repository whose service is not running keeps waiting for the service to start,
  bounded by the commit path's start wait.
  It measured 12.1 s,
  once per repository.
- Whether `node_modules/.monochromatic` can be a subvolume is measured on a throwaway checkout before anything changes:
  it would remove roughly 165,000 of the 269,000 deletions a snapshot's trash costs,
  and nothing yet shows that the tools leave a subvolume in place.
- `MD2` in `AGENTS.md` was reworded by an agent to say the marker width in words,
  with the same meaning.
- `package/test-fixture/data-sequence` keeps a `.` entry serving TypeScript source,
  following issue 1579's acceptance and the `hook-type` precedent.

## Open: two `AGENTS.md` rules proposed after agent failures, recorded 2026-11-19T22:53:13-05:00

Both wait for the owner's word, because `AGENTS.md` changes are the owner's to accept;
until then every brief the coordinator writes carries the same instruction.
Each code was checked against `AGENTS.md`, the skills and both forbidden-strings appendixes.

### Stop only what you started

What happened:
at 22:34:24 a ZFS step 4 fix round's agent listed the running containers,
took two that ran `cargo nextest` for its own,
and removed both by ID (`podman rm --force --time 0 7a5c4dce9245 5982b11d2ac9`).
One was another session's whole container gate,
stopped after 8446 of 9199 tests;
the other belonged to a third session.
Rule `NLD` tells an agent to stop every process and unit it left,
and says nothing about how it knows one is its own.
The proposed rule,
under the 50-word and 200-character limits:

```text
SYO:
 Stop or remove only processes, units and containers you started,
 known by a name or ID you recorded at start;
 never one because `ps` or `podman ps` lists it.
```

### Commit dates are the clock's

What happened:
two design writers set `GIT_COMMITTER_DATE` and `GIT_AUTHOR_DATE` on fifteen commits,
one of them to a time not yet reached,
so a revision label could equal the last commit's `%cI` as their briefs asked;
a label written into a commit cannot name that commit's own time.
Rule `WHN` asks for a second-precise timestamp and names `git log --format=%cI` for a commit,
which reads as permission to make the commit match.
The proposed rule:

```text
KDT:
 Never set `GIT_AUTHOR_DATE`, `GIT_COMMITTER_DATE` or `--date`;
 a label naming a commit's time names one already made,
 never the commit that writes the label.
```

## Open: link the already installed tools into the main checkout? (recorded 2026-11-19, about 23:20 -05:00)

The plan's step 1 records that nothing is installed on this host until the mpm-shadow change you put first
("A: mpm-shadow change first (Recommended)").
Tonight a ZFS step 4 agent ran `mise run toolchain:install` in its worktree,
and it reported `0 installed, 66 unchanged, 0 failed`:
the locked tools are already in `~/.local/share/monochromatic-dev/toolchain`,
`cargo-shadow` and `cargo-nextest` among them,
and the run only made that worktree's `.cache/toolchain-bin` link folder.
The main checkout has no such folder,
so tasks that start with `cargo-shadow` (the wrapper's `native:test:container`, `test:git-image`) exit 127 there
and every gate runs their second step by hand.
Running the same task in the main checkout would install nothing and make the link folder,
putting those tools on the checkout's `PATH` for every session.

- Link them now (recommended): the documented task routes work again and no tool is installed;
  the cost is that every session in the main checkout starts finding these tools at once.
- Wait for the mpm-shadow change: nothing on `PATH` changes; gates keep running the second step by hand.

Ranking: link now > wait, because the record's reason for waiting was an install that converges the whole toolchain,
and the measurement shows nothing left to install.

## Open: approve ZFS step 4's slot identity store format? (recorded 2026-11-20, about 02:00 -05:00)

What it is:
step 4 records, for each worktree slot, the identities the attach checks (device, inode, owner, permissions, access
lists of the folders), in one root-controlled file the wrapper appends to, `git-worktree-clone-slot-identities`, as
`package/git-policy/cli/SPEC.md` describes under "The recorded identities store".
Step 8 will build on this format, and no decision record approves it yet; step 4's second integration pass asked that
you do.

- (a) One file, one line per slot, appended under a lock only root can open, waited for with a bounded deadline
  (as built; recommended). One read sees the parent and every slot in one state, which step 8's checks rely on.
- (b) A folder with one file per slot, each made with `O_EXCL`, no lock. Lock-free, which the repository's concurrency
  rule prefers; it loses the single-read state and rewrites the format, every refusal that names a line and about a
  thousand lines of controls.

Ranking a > b: a keeps the one-read consistency step 8 needs and its lock covers one appended line, held only by root;
b buys lock-freedom at the cost of that consistency and a rewrite.

## Answered: accept the ZFS staging-under-cache design? (ready 2026-11-20, about 01:00 -05:00)

The owner, through the question tool on 2026-11-20: "Accept, boundary (a) (Recommended)".
So the design at `c685cabebe` is accepted with none of its seven veto-open choices vetoed; the owner's Q3 limit ("move
it back at once ... keep and report it only if the move back fails") covers only the scene where the clone landed on a
mount the account stacked, and refusing when a mount sits on the clone alone is the design's own rule; the nine lows
are fixed in the decision record's copy.
Next: the decision record and the amendments it names, then this section moves out.

This goes to the question tool when you are back, with the other ZFS designs.

What it is:
when the wrapper removes a worktree whose ZFS clone is still mounted and cannot unmount it yet,
it moves the clone's mount aside into a private bind under `~/.cache`,
records the clone's file handle there so a later run can find it again,
and moves it back at once if anything goes wrong,
answering your round of 2026-11-19 (Q1 "Private bind per removal", Q2 "Record the file handle", Q3 "Move it back at
once", "Veto none" for the four named proposals, the handle record "In ~/.cache" with its threat model) and the
proposal's "Keep, one exception (Recommended)".
It is `doc/planning/cli-git-zfs-staging-under-cache-design.md` on branch
`docs/planning-cli-git-zfs-staging-under-cache-design` at `c685cabebe`.

What checked it:
six revisions by DeepSeek writers against a gate review, a re-check, a narrow check and two diff-only checks on Claude
Sonnet; the last (`~/temp/agent/zfs-staging-under-cache-design-sixth-revision-diff-only-gate-check-2026-11-20/keep/review.md`)
found no high and no medium finding and called it ready. Its commit dates are real throughout.

What you are asked, beside accepting:

- One question with no default, because the answer decides whether a refusal is recorded in your name:
  where your Q3 answer's own limit ("move it back at once ... retaining and reporting it only if rollback fails")
  reaches when a mount sits on the clone.
  (a) Only the combined scene, where the clone landed on a mount the account stacked and a second mount sits on the
  clone, is your answer's limit; refusing when a mount sits on the clone alone is the design's own rule (recommended).
  (b) Your answer's limit reaches both scenes.
  (c) Neither: "Move it back at once" governs literally, and the wrapper tries the move back in both scenes, carrying
  the stacked mount along.
  Ranking a > b > c: (a) keeps the stacked mount where the account put it and names the design as the author of the
  clone-alone refusal; (b) records that refusal as yours; (c) carries a mount the account made to a place it did not
  choose.
- One choice that would change an accepted sentence, so it is yours, not a veto: entry 1's option (c), proving the
  rollback's destination by the fresh walk alone without the parent-row check, which loses the accepted "prove both"
  in the scene where the account stacks a mount at the slot. The design ranks it below its own route.
- Seven choices the design makes itself, each open to your veto, default "veto none": the rollback's destination and
  its proof; refusing when a mount is stacked on the clone; the end state and receipts after a rollback, a reattach and
  a cancel; clearing a record whose folder is gone only on a stale handle with no mount row at its path; clearing a
  record from another boot by its boot id with the wrapper's own settlement; reading the rollback as no exception to
  "No implicit remount", so your "Keep, one exception" and its sentences stand; and the record's compared fields (the
  dataset's source and the mount root's inode, the device recorded and never compared because ZFS numbers it anew each
  mount). The last check notes that every ZFS mount root here reads inode 34, so the inode tells no dataset from
  another and serves only to catch a subdirectory bound at `~/.cache`.

Nine lows the last check found, carried here (full text in that review): the field set marked for veto in one place
but not listed with options; the inode point above; a reboot test step with no starting state; the boundary
question naming one conflicting answer pair where there are several; entry 6's answer to the race reason not
separating the rollback from the reattach; stale counts and leads; five blurred wordings; an M5 assertion that cannot
fail; five ragged line breaks.

Options:

- Accept, with the question answered and no vetoes (recommended): the lows are fixed in the decision record's copy.
- Accept with vetoes or option (c): name which.
- Send it back for the lows first.

Ranking: accept > accept with vetoes > send back, because no remaining finding changes behaviour, and each further
round has so far found wording slips of the same size.

## Answered: ZFS step 3 makes a worktree add from a dirty checkout slower; what then? (recorded 2026-11-20, about 00:30 -05:00)

Answered 2026-11-20 on the third asking, "A: merge with cloning on", recorded as
`doc/decision/cli-git-zfs-clone-rung-dirty-source-waiver.md`, which also holds the partial-clone proposal that ends the
waiver and the miss behind it.
The same round chose "Tighten NQH" for `AGENTS.md`, committed the same day.
This section moves out once step 3 merges.

First asking: through the question tool on 2026-11-20, the owner replied "What? Why are we running checks?".
The coordinator answered that in the session and asked again, adding a question about cloning only the unchanged files.
The owner's replies, 2026-11-20:
to what step 3 merges with now, "Then this was an even worse decision made and implies multiple agents missed the rule
about the machine never being quiet." (read as: a fast path that needs a clean source was designed and measured as the
main case on a machine whose main checkout other sessions keep dirty; no option was picked, so the merge choice is
asked again);
to "Should a writer design and measure a different clone path?" (clone every tracked file the proof shows unchanged,
let Git write only the changed ones), "Yes, design and measure it (Recommended)".
That proposal goes to `doc/planning/` with measurements and comes back to the owner before any code changes.

### Why the clone path checks the source

Git's `worktree add` writes each file from the commit.
The clone path instead clones the files sitting in the source checkout (`FICLONE` of every tracked file) and installs
the source's own index with the clones' stat data (`worktree_clone.rs`, `install_index_in`), because a reflink needs a
file on disk with the same bytes and the commit's bytes live compressed in Git's object store.
So the clone path gives the right worktree only when the source's files are exactly the commit's.
Before it clones it proves that (`SPEC.md`, "Rung 2 needs a clean source"): no merge, rebase or other operation in
progress, the index equal to `HEAD`'s tree, every tracked file matching its index entry, and no untracked file
(the refusal kinds in `src/native/worktree/worktree_clean_stat_verdicts.rs`).
Without the proof an uncommitted edit would be cloned into the new worktree, and the copied index would make its
`git status` call the file clean.
On `main` the wrapper does not know ZFS can clone, so on this host it goes straight to Git's checkout and never runs
the proof; step 3 teaches it ZFS, so the proof now runs, and on a dirty source it finds the change and falls back to
Git's checkout after spending the time.
The steps measured on that path: `git ls-files` 79 to 133 ms against the control's 45 to 50, `git check-ignore` 36 to
47 ms, `git status` 36 to 57 ms, and an in-process stat walk of about 34 ms
(`~/temp/agent/zfs-step-3-integration-pass-findings-fixed-2026-11-19/keep/report-from-summary.md`).

What it is:
step 3 teaches the wrapper to see a ZFS dataset as able to share data blocks,
so `git worktree add` from a clean source clones the files instead of checking them out.
Your merge bar says a branch may not make anything `main` does well worse.

What was measured on this machine (24,313 tracked files, the branch against `main` built the same way, alternating):

- Clean source: the add now takes 0.51 to 1.57 times `main`'s time, median 0.66,
  since the round cloned the files in parallel slices
  (before that round it was about 1.9 times, which the integration pass rightly refused).
  A source written in the last few seconds can add at most about 0.7 s once,
  because OpenZFS waits for its write-out before cloning freshly written blocks.
- Dirty source (uncommitted changes, so no clone happens): about 1.2 times `main`'s time, about +0.2 s.
  `main` skipped the clone rung's checks on ZFS at once; the branch now runs them before it falls back to Git's
  checkout. The round tried two ways to make those checks cheaper and neither reached parity.

The main checkout here usually carries uncommitted files (it holds four now), so adds from it take the dirty path.

What the clone saves on disk, measured by step 3's guest arm on 2026-11-20 (an encrypted test pool, the crate's own
`src` as the tree, 2,456 files, 34.7 MB, Git's add and the wrapper's add alternated over two rounds, the pool's
allocation read after a sync): a clean add saved 22,979,072 bytes, about two thirds of the tree.

Options:

- C. Do not take the clone rung on ZFS until the dirty-source checks cost no more than `main`'s, then turn it on
  (recommended). Both paths behave exactly as `main` does today; the recognition, its tests and the guest arm stay. It
  postpones a part of the accepted design (the clone rung on ZFS), which is why it is yours to choose.
- A. Keep the clone rung on ZFS and waive the merge bar for this +0.2 s on a dirty-source add, recorded in
  `doc/decision/`. Clean adds get faster now and save about two thirds of the tree's disk; most adds here, from a dirty
  checkout, get slower by about 1 per cent of a real add (which also copies the ignored files) and save nothing.
- B. Run the tracked-file check first and the untracked scan only if it passes: the dirty path about 50 ms cheaper,
  the clean path about 34 ms dearer on every host whose snapshot rung is refused; still short of parity, so the bar is
  still missed.
- D. Choose the clone rung by a measured size threshold: departs from the ladder's design and rests on a guess.

Ranking: C > A > B > D. C over A because the dirty path is the common one here, so A's cost lands on most adds and
its saving on few, and C needs no waiver; A over B because B still misses the bar and moves cost onto other hosts; B over
D because D leaves the design for a guessed threshold. The round's report, with every measurement:
`~/temp/agent/zfs-step-3-integration-pass-findings-fixed-2026-11-19/keep/report-from-summary.md`.
