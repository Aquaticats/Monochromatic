# Cli git wrapper owner answers 2026 11 15 to 2026 11 16

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45816 to 45829 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### A `worktree add` relinks by itself when no service took its request, and a Pi run is confined with its containers started by the account's manager (2026-11-16, answered through the question tool between 17:33:37-05:00 and 17:36:00-05:00)

Two questions,
each with its author's measurements beside it.

- **The add whose relink request no commit service took**:
  "Relink in the add (Recommended)".
  The add runs the relink itself before it returns,
  as the wake branch builds it at `a2f64e8f36`,
  so the worktree can build on return and the printed promise is true;
  the cost,
  about 5 s and up to about a minute of pnpm,
  falls only when no service took the request.
  The option not taken:
  return at once and say the relink is queued
  (measured:
  taken 4.9 s later by a running service that knows the repository,
  0.3 s later with a service and one more command,
  never within 30 s with a service and no later command).
  The owner was told this is separate from "No wait on the add",
  which is about other programs' writes.

From lines 45841 to 45852 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### A record that cannot be judged stops a command only where going on is unsafe (2026-11-16, answered through the question tool between 17:05:58-05:00 and 17:07:00-05:00)

The decision "A:
stays,
command says how" leaves "stops or prints" open for each command.
Asked which,
with the copy-owner review's finding beside it (`git status`,
`git add`,
`git commit` and `git worktree add` all exit 2 at `a303cb638f` where the released wrapper exits 0),
the owner answered:
**"Per command (Recommended)"**.

A command stops only where going on beside the record could lose or mix a person's work;
every other command prints the words (the record,
its owner,
its age,
the exact `rm -- <path>`) and continues with the exit status real Git would give.
Which command is which,
for each kind of record,
is the author's to establish and state,
and a case it cannot settle keeps the stop and is named.
The two options not taken:
every command prints and none stops;
every command stops.
The copy-owner author's fourth-run brief already carries this reading;
the held-push holder author's brief asks for the words and is to be held to the same rule at its next review.

From lines 45853 to 45864 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### A `worktree add` does not wait on other programs' writes, and the snapshot proof's two rare losses are a stated limit (2026-11-16, answered through the question tool between 15:09:57-05:00 and 15:11:23-05:00)

Asked,
with the measurements beside it (16.5 ms an add on an idle drive,
86.6 ms more under another writer's 64 MiB pending,
growing with others' pending data),
whether the wrapper waits for a commit of the file system before every `worktree add` so that no snapshot of a person can be taken for its own,
the owner answered:
**"No wait on the add"**,
against the session's recommendation of the wait.

So the snapshot branch's mechanism stands as built (`fix/git-policy-cli-snapshot-of-a-person-is-never-proved-to-be-the-wrappers-own`,
tip `8b7c98aa6e`),
and the `SPEC.md` sentence that rules such a wait out on an add's path stands unchanged.
A snapshot a person made in the same instant as the wrapper's record,
or with the clock set back,
is deleted as the wrapper's own:
that is written as a stated limit on the package's page,
which the snapshot author is given after its review.
The sibling branch that waits (`69154a083f`) is not merged;
it is kept,
unreviewed,
because its code is the only record of the mechanism the measurements came from.

From lines 45908 to 45929 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The wrapper:

- Hand-written lines to standard error and the non-blocking door:
  **"Only where another command can be held (Recommended)"**:
  writes by the resident service,
  or by a command while it holds a lock,
  go through the door;
  a one-shot command's own last words keep a blocking write.
- The discarding guard's window between its last look and Git's start:
  **"State the limit (Recommended)"**.
- A commit run a second time when its private files vanish after Git made the commit:
  **"Run it again (Recommended)"**.
- The rule that clears the record of a moved branch:
  **"No path holds the old tip (Recommended)"**.
- A killable process for storage that never answers:
  **"Bounds on stuck threads stand (Recommended)"**.
- A branch never pushed:
  **"Push and set the upstream (Recommended)"**.
- A retried push whose report cannot be written:
  **"Journal is enough (Recommended)"**.
- The wrapper's line for a live lock in the command's own worktree:
  **"Keep the line (Recommended)"**.
- Drawn trees:
  **"A lifetime"** (against the session's recommendation),
  and for its span **"Author measures and proposes"**.
- Git's messages quoting the absolute path the removal guard judged:
  **"Keep the absolute path (Recommended)"**.
- A log fragment left when a line is given up:
  **"Close the fragment (Recommended)"**.
- The sentence "Nothing releases a held record by judging a process":
  **"Correct the sentence (Recommended)"**.
- Notes naming a checkout that no longer exists:
  **"Sweep notes whose checkout is gone (Recommended)"**.
- Three adoptions confirmed,
  each ticked:
  **"Inherited ignore stays ignored"**,
  **"Test failures may quote the service"**,
  **"FETCH_HEAD protects a worktree"**.
- Compilers:
  **"Everything nightly.
  This doesn't break "pacman for everything it has" if you're implying pacman doesn't have nightly.
  And the lock's nightly is always the latest nightly anyway."**
  Host tasks,
  the container gate and the release workflow all build with the lock's nightly.
- A binary test's fixture root:
  **"Make the fixture root private (Recommended)"**.
- The three files of the old name `cli-git-native` in the main checkout's `.cache`:
  **"Move them to scratch (Recommended)"**.

From lines 45993 to 45993 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Wrapper,
  `UNASKABLE_HOLD_AGE`:
  **"Approve 150 seconds"**,
  against the session's recommendation of ten minutes.
  The sentence "awaits the owner's approval" in `SPEC.md` (line 8381) is now out of date.

From lines 45999 to 45999 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The "release-artifact gate" of the umbrella list:
  **"What exists is the gate (Recommended)"**:
  the lock's sha256,
  `native:perf` on the published archive's binary,
  and the sample commit after an install.
  No listing check is built.

From lines 46007 to 46008 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The fix for the empty-tree commit (`3882e11ddf`),
  if its author's measurement confirms that an abbreviated `git commit --amen` is refused with its index intact:
  **"Merge anyway;
  the fix follows (Recommended)"**.
- The push of a branch another worktree shows,
  when a path still holds the old content after the whole wait:
  **"Stop the command,
  as built (Recommended)"**.
  The question held for that author's report is hereby asked and closed.

From lines 46243 to 46243 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Its six findings are all `main`'s too:
  a reflog line whose time is 0 read as a surviving name;
  a detached `HEAD` in a reftable repository naming nothing to the guard;
  a reflog line older than `git gc` keeps counted as a surviving name;
  the state of a stopped `git notes merge` unguarded;
  a commit made while real Git's own cleanliness check runs lost;
  and,
  a stated limit,
  an ignored folder going with the worktree by the decision "Delete as ignored".
  These are follow-up work on `main`.

From lines 46291 to 46291 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Adopted by a rule,
  open to the owner's veto:
  on a stalled look at the runtime directory the answer is "not proven".**
  The review measured the pair issue 6293's author had put as a question:
  "not proven" writes four small files into the account's state directory and loses nothing (the retry plan survived a logout and was retried);
  "another session" loses the retry plan at logout and the next command still says the service will retry.
  The wrapper never discards a person's data,
  and a commit left unpushed under a sentence that says it will be pushed is that;
  so the first is built and the SPEC sentence it goes against is corrected.

From lines 46355 to 46355 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  **Asked through the question tool,
  the owner answered "No wait on the add"** (under "Owner decisions in force"):
  the mechanism stands,
  the two rare losses become a stated limit on the package's page,
  and the sibling branch that waits is not merged.

From lines 46393 to 46393 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  It brings two,
  both low:
  the tip keeps a dead holder `main` released when the number was reused and a part cannot be read (three situations),
  and a variant never constructed off Linux that would fail Clippy there.
  The first is the shape the owner decided on 2026-11-16 ("A:
  stays,
  command says how"),
  so the brief asks only for the second half of that decision to be shown:
  that past the patience a person is told which record,
  whose,
  how old,
  and the exact `rm -- <path>`.

From lines 46424 to 46424 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  How the brief reads the owner's "A:
  stays,
  command says how" ("past it the command stops or prints words naming the record,
  the owner,
  its age and the exact `rm -- <path>`"):
  the words reach the person where no switch hides them,
  on every surface;
  and stop or print is settled command by command,
  a stop only where going on beside the record could lose or mix a person's work,
  since commits should never fail.
  A `git status` that exits 2 over a record it only reads is the case the author is told to look at first;
  a case it cannot settle keeps the stop and is named as open.
  **That reading is this session's and is open to the owner's veto.**

From lines 46430 to 46430 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The owner confirmed the reading of "stops or prints":
  "Per command (Recommended)"** (under "Owner decisions in force",
  "A record that cannot be judged stops a command only where going on is unsafe").
  The copy-owner author's brief already carries it.

From lines 46444 to 46444 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  A fresh run is queued first (17:27:10;
  name `landed-removal-passes-every-control-main-passes-shell-aliases-forwarded-unread-and-three-removal-names-settled`,
  a new branch of that name from `915ef30e09`).
  The alias reading goes,
  by the decision "Forward them unguarded".
  For the three names it establishes what real Git does when the name is left to it;
  `main`'s controls stand unless leaving the name to Git is shown to lose work.
  Then a short review of its delta and pass 59.

From lines 46492 to 46492 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  **The copy-owner author's fourth run,
  whole at `eaacf58e50` (branch `fix/git-policy-cli-copy-owner-and-transaction-owner-spelled-another-way-are-never-taken-for-gone`;
  changed modules 1,609 of 1,609,
  binary controls 48 of 48):**
  by the owner's "Per command",
  a young record no longer stops `worktree add` (both adds exit 0,
  where the younger exited 2);
  the telling moved off the warning channel to the notices step,
  so it survives `MONOCHROMATIC_WARN=false`;
  the platform-refusal arms have seam controls that fail with the arm taken out;
  a copy whose owner record is no token is kept and named;
  the printed command is run in a control for a path with a space,
  both quotes and a newline;
  the claim's words name the stage the `rm` leaves and its own `rm --recursive`.

From lines 46509 to 46509 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A record whose holder can be shown neither alive nor gone:
  "A:
  stays,
  command says how (Recommended)".**
  The record stays;
  within the patience nothing is said;
  past it the command stops or prints words naming the record,
  the owner,
  its age and the exact `rm -- <path>`.
  Nothing deletes such a record on a timer,
  and no repair command is built.

From lines 46510 to 46510 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A finished worktree the automatic removal kept:
  "Record it,
  retry later (Recommended)".**
  A kept worktree is written to a record,
  each later sweep tries it again with the same checks,
  and it is named again each time it is kept.
  The cost of reading the process table for old candidates is measured by whoever builds it.

From lines 46511 to 46511 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The commit a run made into `main` by mistake (`293de6dae5`):
  "Leave it,
  review it (Recommended)".**
  `main` is not rewound and the commit is not reverted;
  its two files get a Sonnet review as they stand.

From lines 46512 to 46513 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A merge on a branch with no upstream:
  "Push it to origin".**
  Chosen against this session's recommendation (decline and say so).
  The recorded condition of 2026-11-02 is amended:
  such a merge,
  in a repository that has `origin`,
  is pushed to origin under the branch's own name and the upstream is set.
  It needs the amendments at `package/git-policy/cli/SPEC.md` line 9068 and `README.md` line 556,
  a new reason word,
  and the ancestry walk's cost measured;
  a fresh run builds it.

From lines 46534 to 46551 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


Adopted from the same prepared batch without asking,
each because a recorded rule or an earlier answer decides it,
each open to veto:

- Pi driver,
  numbered unit names (`pi-<name>@<n>.service`):
  kept,
  since going back brings the refused second start back;
  the author tests whether such a unit still gets its tunnel exemption (a probe put an instanced unit under `app.slice/app-<name>.slice`,
  where `wg-quicker-exempt` does not look) and names its slice if not.
- Pi driver,
  removal of a worktree at a run's end:
  the proof of who made it is checked where one exists,
  and today's rules hold where none does (in line with "Remove after a bound").
- mpm-shadow:
  a pass from the global scope looks for a repository above its working folder and keeps what it cannot judge ("what the pass cannot see is protected",
  and "Newest two plus pinned" needs the pins seen);
  the two-second clock of a removal starts with the pass,
  as built ("You should (almost) never be waiting."),
  and the doc's sentence is corrected;
  claim names it does not recognise get one line per pass with their count and the first path;
  the two one-hour constants (`DOWNLOAD_IN_USE_FLOOR`,
  `DEFAULT_PROGRAM_BOUND`) get a measurement beside them,
  since a shipped limit cites one;
  the archive prune gets its seam and a control.
- Queue of runs:
  a `reviewOfBranch` entry is converted to `reviewOfRange` at the changeover;
  `--queue` stays required;
  breaking a dead holder's claim is logged.
- Wrapper:
  a watch note naming another repository's directory stays refused (each service touches its own repository only),
  and `SPEC.md` says so;
  `SPEC.md` line 14568 (a wait that was removed) and line 14557 (a claim file that "stays") are corrected on the no-manager branch.
- cargo-shadow:
  the eighteen hand-made logs and the renamed record under `package/linter/monochromatic-lint/target/verification` stay where the docs cite them,
  never swept (the README says so already).
- The fuzzing artifact reference stays as history with one added line saying its bytes are not kept.
- `doc/handover/cli-git-native-post-landing.md` line 214 ("nothing remains") is corrected:
  the maintenance files were not mutated.
- The wrapper's mutation pause stays:
  the condition the owner set for lifting it (the commit protocol comparison) is not met.

From lines 46611 to 46621 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The folder of copies that earlier releases saved aside from a forced clean**
  (`<common git dir>/monochromatic-git-policy-cli/discarded/`):
  "Sweep on another trigger".
  The sweep that removes copies past their retention keeps running,
  on a trigger other than a completed save
  (the question named a commit or the `doctor` command as examples;
  which trigger is not decided and is the author's to propose with its reason).
  Not chosen:
  keep the copies until a person removes them
  (the session's recommendation),
  and delete the folder at upgrade.

From lines 46660 to 46673 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### Answers on waking, second round (2026-11-15, written here at 11:00:01-05:00)

Three more questions through the question tool,
answered between 10:57:32 and 10:59.
Two of them reverse what the coordinating session had adopted meanwhile and what the landed removal's branch builds at `5ca2149aea`.

- **A shell alias and the guard on a worktree's removal**:
  "Forward them unguarded".
  Every `!` alias is passed to real Git as it is;
  all aliases keep working with no opt-out;
  a `git <alias>` that ends in a worktree removal is not guarded,
  and the SPEC states that hole.
  The option not taken was the branch's refusal of every `!` alias the wrapper cannot read.

From lines 46674 to 46683 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **An ignored folder that is itself a repository with commits of its own**:
  "Delete as ignored".
  The decision of 2026-11-12
  ("Delete ignored,
  spare busy")
  stays literal:
  the nested repository goes with the worktree and the deletion is logged.
  The owner was told that a review had deleted the only copy of a commit this way.
  The reading adopted at 08:06:19
  (refuse with the opt-out)
  is withdrawn.

From lines 46689 to 46713 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### Answers on waking, third round (2026-11-15, written here at 11:07:00-05:00)

Two questions through the question tool,
about the guard on discarding commands
(`checkout`,
`reset --hard`,
`clean`,
`restore`),
whose author had ended at 11:00:13 with `2f3e2785ed`.

- **A shell alias in the discarding guard**:
  "Same answer:
  forward (Recommended)".
  This guard also passes every `!` alias to real Git as it is,
  and the SPEC states the hole once for both guards.
- **A forced `git clean` that would delete a nested repository**:
  "Delete and log (Recommended)".
  The clean deletes it as real Git does and the wrapper logs the command and what went;
  no saved copy.
  The owner was shown the three records that disagreed
  (the answer of 2026-11-10 on a clean of ignored files,
  the rule of 2026-11-12 that the wrapper moves nothing aside,
  and the morning's answer on the worktree removal)
  and the branch's save.

From lines 6563 to 6568 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The owner's word and four answers** are under "Owner decisions in force",
  "Answers on waking":
  the temperature limits are lifted and stay lifted;
  the cap steps to 20 under the probe;
  the rotation keeps waiting for the masking;
  a wrapper branch goes into a pass while its only open findings are in doc comments or test code.
