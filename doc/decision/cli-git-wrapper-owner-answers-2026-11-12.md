# Cli git wrapper owner answers 2026 11 12

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 25236 to 25260 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Wound down for a restart of the machine: the home drive is full (2026-11-12T16:41:25-05:00)

#### What happened

- `/var/home` is full:
  1906 GiB in size,
  1819 GiB used,
  1 GiB free,
  100% (`df` at 16:04:12).
  It had 280.68 GiB free at 11:35:19.
  The owner moved data onto it in the afternoon.
  The build volume `/var/mnt/encrypted` had 560 GiB free at the same reading.
- Twenty Pi runs ended between 15:56:45 and 15:59:48,
  every one with `stop=error`
  (`~/temp/agent/gate/runs-ended-since-1453-read-at-1606.log`).
  The feed of one holds "ENOSPC:
  no space left on device,
  write" beside connect time-outs to the model provider.
  Run folders,
  Pi sessions and the journal all live on that drive.
- The "disk pressure" of the afternoon,
  the starts that failed,
  the commits of minutes and the one that was not made
  all happened while that drive filled.
  How much of each was the full drive and how much is the wrapper's own defect is not separated yet;
  the findings recorded since 13:30 (in-process file work that stalls,
  a housekeeping round of forty minutes,
  a service that stops answering)
  stand as observed and need measuring again on a healthy machine before they are fixed or dropped.
- The owner could not reach the machine and wrote through GitHub:
  two commits on `main` made in GitHub's web editor (`6b88bafc75` at 15:52:40,
  `869cd1ff95` at 16:00:20,
  committer "GitHub")
  add a root file whose name is the message:
  "I am Aquaticat and if you are an agent and see this wind down asap gracefully.
  The disk pressure means nothing can be done anyway
  and I will try to fix them after restart.
  I can only communicate this way because the machine is not responding at all."
  Its content:
  "If you see this edit this or leave a comment or something!"
  This session saw it at about 16:10 through a subagent's report,
  read the commits,
  and was told in the terminal by the owner:
  "Act on it.
  Every input takes 10min."
- The owner's choice through the question tool at about 16:08,
  for the full drive:
  "Prune all unused images (Recommended)".

From lines 25316 to 25346 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### How far the protection of a worktree being removed reaches; free space; the second and third audits of 0.6.1-alpha.14 (2026-11-12T13:04:14-05:00)

#### How far the protection of a worktree being removed reaches

- The third Sonnet audit reported that the automatic removal of a landed worktree destroys a person's files with it.
  The owner,
  typed at about 12:58:
  "If someone puts personal data in worktrees,
  they deserve the data loss."
- Asked through the question tool how far that reaches,
  with the audit's four cases named.
  The picks:
  "Keep Git work only (Recommended)";
  for a typed unforced `git worktree remove`,
  "Same rule for both (Recommended)";
  and,
  asked separately because the first option's text had listed untracked files among what goes,
  "Still spare untracked (Recommended)".
- So,
  for the automatic removal and for a typed unforced removal alike:
  - Git work is never destroyed by a removal:
    a commit that no reference holds,
    and an edit to a tracked file,
    however it is hidden (`assume-unchanged`,
    `skip-worktree`).
  - An untracked file that is not ignored and stands directly in the worktree keeps the worktree,
    as today:
    the automatic removal leaves it and removes it later by itself;
    a typed unforced removal refuses as real Git does.
  - Everything else a person put into the worktree goes with it and is logged:
    ignored files and whole ignored directories,
    a nested repository even when its commits exist nowhere else,
    the untracked and ignored files of a nested worktree.
  - Unchanged:
    a worktree a live process works in is refused with an opt-out;
    a subvolume or mount the wrapper did not make is removed around.
- This session's additions,
  open to veto:
  a typed removal never does more than real Git does without `--force`;
  a nested repository that goes is logged with its path and the tips of its branches.
- It withdraws this session's adoption of 11:58:45 that a nested worktree with an untracked file is refused.
- Sent at 13:06 to the three runs on removal
  (`~/temp/agent/notes-to-send/2026-11-12T1306-how-far-the-protection-of-a-worktree-being-removed-reaches.md`);
  the brief of round 311 was corrected before its run read it or is corrected by that note.

From lines 25347 to 25355 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### Free space: the owner freed it

- 101 GiB at 12:31:17,
  543 GiB at 12:51:21,
  572 GiB at 13:04:14.
- The owner,
  at about 12:57:
  "I deleted all my Steam games and moved a bit more data to the main drive."
- Before that message this session checked that nothing of the work had gone:
  the volume's top-level folders,
  the 259 entries of the build base and the 143 worktrees were all there.
- The launcher's 100 GiB gate no longer holds anything back.
  The size of a build tree stays the limit on how many agents can work;
  the measurement run goes on.

From lines 25619 to 25634 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The five-second rule is changed; the hourly pass after 14:00 (2026-11-12T14:03:46-05:00)

##### The five-second rule is changed

- The owner,
  typed at about 14:04:
  "Change the 5s rule to max(5s,
  double what native git takes)".
- Asked which bound that is,
  with an example of real Git taking 30 s.
  The owner:
  "Extra over real Git".
- So the rule "commits vs native git should not take 5s longer."
  now reads:
  what the wrapper adds on top of real Git's own time for the same commit is at most the larger of five seconds and twice real Git's time.
  With `n` real Git's time:
  wrapper time minus `n` is at most max(5 s,
  2 × `n`).
- Every measurement therefore carries real Git's time for the same commit under the same conditions beside the wrapper's.
  Today's commits of 56 s to 199 s are over the rule unless real Git would itself have taken 17 s to 65 s;
  that has not been measured.
- Sent at 14:06:35 to `commit-does-not-list-every-reference-on-the-path-it-waits-on-1`,
  `commit-fails-within-a-bound-and-says-why-when-its-service-does-not-answer-1` and `integration-pass-18-for-alpha-16-1`
  (`~/temp/agent/notes-to-send/2026-11-12T1406-the-five-second-rule-is-changed.md`);
  written into the brief of the many-changed-files run before it started.

From lines 25720 to 25728 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The owner on the disk pressure, and round 312 (2026-11-12T13:31:45-05:00)

- The owner,
  typed at about 13:31,
  after the slow commits were reported:
  "Might be my move of a dir that's saturating it."
- It fits the figures:
  the stalls are on I/O (`/proc/pressure/io` full avg10=47.49 at 13:25:55,
  CPU pressure at zero),
  and they began in the window of the deletion and the move.
  It does not answer what the wrapper adds over real Git under that pressure;
  the diagnosis started at 13:30 measures that.
- Corrected:
  the section "The hourly pass after 13:00" names 13:35 three times;
  the clock read 13:29:55 to 13:31:46 during that pass.
  Round 311's `git worktree add` had then run for about twenty-eight minutes,
  not more than thirty.

From lines 25775 to 25833 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner's answers of 2026-11-12T12:26:24-05:00, through the question tool: who reviews, a file mode through a merge, what Git refuses, how far publishing reaches; and the first audit of 0.6.1-alpha.14

The answers were read before 12:26:24.
Each pick is quoted as the owner gave it.

#### Who reviews a change (item 3.5)

- Asked,
  with what was seen since 2026-11-11
  (qwen4 once wrote no defect where Kimi reported a blocker;
  a qwen4 review that read two fifths of a diff and ran nothing;
  reviews that ended without a review file).
- The owner:
  "qwen4 alone,
  read-checked".
- So:
  reviews stay with qwen4 alone.
  Every review brief demands that the review states how much of the diff it read
  and what it ran through a built wrapper,
  and a review that ran nothing is started again.
  The demand belongs in the package command that writes review briefs
  (`package/pi-shared/subagent-driver`,
  `write-a-brief-for-a-review`),
  not in a scratch script.

#### A file mode a person set, through a merge (item 4.11)

- The owner:
  "As real Git,
  both cases (Recommended)".
- So:
  the wrapper writes the mode real Git would write in both cases;
  the one case in the installed release that keeps the person's mode (from `aff52c9e64`) is brought back to Git's behaviour.
  The stopped run `mode-a-person-set-survives-a-merge-that-changes-the-file` stays stopped.

#### Commands real Git refuses that make a commit through the wrapper (items 3.8 and 3.9)

- The owner:
  "Refuse as Git does (Recommended)".
- So:
  every `git commit` form real Git refuses is refused by the wrapper with Git's own message,
  `-i` with `-o` among them.
  The one recorded exception stays:
  a commit that names an untracked path
  (`doc/decision/cli-git-policies-platform.md`,
  "Transactional commit gate").

#### How far the permission to publish reaches (item 4.9)

- The owner:
  "Standing,
  alphas,
  these tools (Recommended)".
- So,
  standing in every session:
  alpha releases of the wrapper and of the tools on its path
  (the build-folder cleaner `cargo-shadow`,
  the installer `mpm-shadow`,
  the Pi driver packages)
  may be bumped and published without asking;
  anything that is not an alpha is asked first.

#### Adopted from recorded decisions, not asked, open to veto

- Item 3.11,
  an operator switch that forces the native route when the commit service misbehaves:
  none is built.
  It follows from "A way back is intentionally not given,
  to avoid agents from becoming lazy."
  and the refusal of install switches.
- Item 4.5,
  the Cargo build directory:
  the exemption is from the linked-worktree rule only;
  a discarding command there is refused with an opt-out like anywhere else,
  by the rule restated at 10:33.
- Item 3.2,
  how long an add may take:
  "60s is fine for adding worktrees" and "nothing should ever wait forever or indefinitely" already decide that a 600-second window does not fit;
  the branch that takes the repair off the add's path is in pass 18.
- Item 3.4,
  the 6,000 ms bound of the agent-settled notification:
  a notification that did not arrive is a defect,
  tracked as its own task;
  no question.

From lines 25927 to 25973 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner's answers of 2026-11-12T12:11:18-05:00, through the question tool: a slow push, finished records, a way back, a foreign subvolume

The answers were read before 12:11:18.
Each pick is quoted as the owner gave it.

#### A slow server and a typed push (item 3.7)

- Asked:
  how "Stop only on no progress" and "nothing should ever wait forever or indefinitely" meet,
  since a server that sends one byte every few seconds holds a typed push without end.
- The owner:
  "Minimum rate (Recommended)".
- So:
  progress means at least a stated number of bytes within a stated window;
  under it the push stops with a message.
  The two values are measured on real pushes and put to the owner before a release carries them.

#### How long the commit service remembers a finished request (item 3.6)

- Asked:
  `README.md` says the records are pruned by count,
  `SPEC.md` says they are kept by age,
  and an audit landed one commit twice by resending a request after a thousand other landings.
- The owner:
  "Age and count together (Recommended)".
- So:
  a record is pruned only when it is both older than a stated age and beyond a stated count;
  the code and both documents follow that.

#### A way back from a bad wrapper release (item 3.3)

- Asked:
  whether the previous release stays on disk.
- The owner,
  in their own words:
  "Leave as it is.
  A way back is intentionally not given,
  to avoid agents from becoming lazy."
- So:
  nothing is kept and no test of a reinstall is added;
  a bad release is answered by a fixed release.
  No agent is briefed to build or to rely on a way back.

#### A subvolume someone else put inside a worktree being removed (item 3.1)

- Asked:
  what follows from "make the wrapper work in that case."
  for a subvolume or mount the wrapper did not make.
- The owner:
  "Remove around it (Recommended)".
- So:
  everything else in the worktree is removed and Git forgets the worktree;
  the foreign subvolume or mount is left where it stands,
  with the folders that lead to it,
  and the wrapper says so.
  The path stays occupied until the item is moved.
  This was not built or tried when it was offered,
  and the question said so.

From lines 26041 to 26050 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The owner on it

- Typed while this was being routed,
  between 12:00:11 and 12:04:49:
  "Yes the wrapper is supposed to do auto merge,
  but it should definitely have done it better."
- So the merge is wanted,
  also for a push a person typed and for a branch that is not checked out.
  What changes is how it is done:
  the names,
  the local branch advanced to the merge published for it and no other branch ever,
  the wording,
  a branch checked out in another worktree,
  and what else "better" asks for on that path.
- Brief:
  `~/temp/agent/pi-runs/push-of-a-branch-that-is-not-checked-out-is-handled-as-that-branch-prompt.md`,
  round 305.

From lines 26124 to 26189 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The owner's answers of 2026-11-12T11:58:45-05:00, through the question tool: removal and ignored files, a kept commit, the LFS record, local-only branches

The dialog was opened after 11:46:35 and the answers were read at 11:58:45.
Each pick is quoted as the owner gave it.

#### Worktree removal and ignored files (item 1.2 of the consolidated list)

- Asked:
  an unforced `git worktree remove`,
  and the automatic removal of a worktree whose branch has landed,
  delete files that exist only in that worktree and that Git ignores,
  as real Git does,
  and remove a worktree while a process works inside it;
  which behaviour is wanted under the rule restated at 10:33.
- The owner:
  "Delete ignored,
  spare busy (Recommended)".
- So:
  ignored files are deleted as real Git does and each deletion is logged
  (the same answer as "Keep allow and log" for `git clean -fX`);
  a worktree with a live process inside is refused,
  naming the process,
  with a `--no-enforce-*` opt-out.
- Not answered by this pick:
  the unforced removal that deletes a nested worktree with its untracked file.
  The restated rule decides it (an untracked file is a person's data:
  refuse,
  name it,
  opt-out),
  adopted here,
  open to veto.

#### A commit the commit service still holds when its worktree is removed

- Asked:
  whether the reference `refs/cli-git/kept/...`,
  written to keep such a commit before the removal,
  is a saved copy the restated rule rules out.
- The owner,
  in their own words:
  "Keep the reference,
  auto prune by size and age."
- So:
  the reference stays as built,
  and the wrapper prunes `refs/cli-git/kept/` by itself,
  by size and by age.
  The owner named no values;
  the agent that builds the pruning proposes them from a measurement of this repository's kept references,
  and the values are put to the owner before a release carries them.

#### The LFS push fix and its record file

- Asked:
  what may happen to `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND`,
  which the fix writes into `push-programs.json` in the repository's Git directory.
- The owner:
  "Memory only (Recommended)".
- So:
  the two command lines go to the commit service with each request and are held in memory,
  never written;
  a push the service must retry after its own restart,
  without them,
  stops with a message that says what to run.

#### Branches that exist only on this machine (item 1.4)

- The owner:
  "Push them as they are (Recommended)".
- The dry run of `~/temp/agent/scratch-scripts/push-branches-the-remote-lacks.ts` at about 11:59 named exactly eleven
  (log `~/temp/agent/gate/branches-the-remote-lacks-2026-11-12T1159.log`):
  `e2e-trace-diagnosis-20261101`,
  `feat/cli-git-worktree-copy` (38 commits past the remote's copy),
  `issue-150-isolated-dts`,
  `research/depot-platform-buildkit-20261018`,
  `research/depot-runtime-base-20261018`,
  `research/depot-runtime-orchestration-20261018`,
  `research/depot-sdk-custom-runtime-20261018`,
  `spike/linter-closure-static`,
  `translation-repair-prerebase-backup`,
  `trial/depot-logger-remainder-20261016`,
  `trial/depot-pi-host-20261016`.
- The push,
  never forced,
  was started at about 12:00
  (log `~/temp/agent/gate/push-local-only-branches-2026-11-12T1200.log`);
  its result is in the section
  "A typed push of a branch that was not checked out published a merge labelled with the checked-out branch".

From lines 27740 to 27753 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### A wrapper killed for memory in a 512 MB scope: the holder scan's read of a descriptor record, issue 6287 (2026-11-12T09:50:16-05:00)

#### What the owner said

- About 09:41,
  quoting a desktop notification:
  "The background service review208-rec5.scope has been terminated by the Linux kernel
  because the system is low on memory.
  Consider closing unused applications or browser tabs."
  "Is this yours?
  And also make sure this doesn't happen again."
- About 09:43:
  "I don't want you to stop the notification.
  I want you to fix the root cause."

The notification stays as it is.
Nothing may hide it,
filter it or change the unit's out-of-memory policy to avoid it.

From lines 27961 to 27968 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### After the restart: 0.6.1-alpha.13 published, the agent for issue 6286 started, the fleet held until the install (2026-11-12T08:39:44-05:00)

The machine was restarted at 04:03:18.
The owner,
at about 08:25:
"I've restarted and GPT has finished its investigation."
The other session's records are `doc/handover/plasma-session-reload-investigation.md`,
`doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md` and `doc/planning/git-wrapper-reload-remedy.md`.
Its proposal names three directions and chooses none;
it records that the owner was not sure and handed the work on.

From lines 28007 to 28014 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The hourly job

The owner,
at about 08:35:
"Re-arm the hourly fill and cull.
This time don't set an expiry date."
Cron `c0352b43`,
every hour at :41,
first fire 09:04,
with no end date in its prompt.
It is session-only,
and the harness deletes a recurring job after seven days,
so it is re-created about 2026-11-19 if the session still runs.
Each fire culls or reframes superseded work,
reads what ended,
routes findings,
and starts the next tracked work to keep the pools filled;
it starts nothing while free space is under 80 GiB.

From lines 28015 to 28028 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The desktop session was lost at 01:24:54; state before the owner's restart of the machine (2026-11-12T03:32:55-05:00)

#### What happened

The graphical session ended at 01:24:54 after a reload of the user service manager that took 139,770 ms,
requested by the wrapper from inside an agent's unit.
The record is `doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md` (issue 6286),
written by another session;
that session is investigating,
and this one changed nothing in the wrapper.
The coordinating session ended with the desktop;
it was started again at about 03:25.
The machine itself has been up since 2026-11-05.

The owner,
at about 03:25:
"There was a kinda serious issue in which the wrapper contributed to a whole desktop crash.
GPT is investigating it.
I need to reboot anyway.
Please tell me when is a good time to reboot."

From lines 28098 to 28103 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### After the restart, in this order

The owner was asked at 03:45 which comes first,
with three options (fix issue 6286 first and ship it in alpha.13,
recommended;
release alpha.13 as it stands;
hold everything until the other session reports).
The owner's answer:
"Release alpha.13 as it stands".
So the fix for issue 6286 follows in the release after it,
and agents run under alpha.13 meanwhile.
