# Cli git wrapper owner answers 2026 11 13

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 23831 to 23853 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

##### The author pool is full, and the owner keeps it at one

- A run that names no model goes to the first default pool with room,
  and one is open,
  `neuralwatt-flex`,
  full at 20 of 20 since about 17:35.
  The launcher then waits,
  asking every thirty seconds,
  and gives up after two hours.
  At 17:51 seven launchers waited,
  the archive run since 17:35:39 and pass 24 since 17:46:00 among them.
- Asked through the question tool at about 17:56 where author runs go when that pool is full
  (reopen standard Neuralwatt,
  overflow to the Kimi flex pool,
  keep one pool),
  the owner answered:
  "Keep it.
  I suspect any more would blow up the disk anyway."
- So about twenty-five agents is the most that run at once
  (twenty authors,
  five reviews),
  and the level of thirty-two is not reached until a pool changes.
  `doc/handover/agents-at-once-on-this-machine.md` says so under "Level 32".

From lines 24372 to 24387 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The owner: one pass carries every reviewed branch; pass 21 has the jam fix, file mode and kept-references (2026-11-13T16:14:18-05:00)

Not complete, and written by the coordinating session: passes 20 and 21 are going and have reported nothing.

- Asked at about 16:11 whether one integration pass may carry several reviewed branches, with one pass per branch as the other reading of "one agent, one thing",
  the owner chose **"One pass, all reviewed"** (answered by 16:12:49).
  A pass's one thing is the next release branch, made from the branches that passed review and built and tested together before `main`.
- **Pass 21** (`integrate/git-policy-cli-pass-21-for-alpha-19`, going since 16:13:21) carries three, in this order:
  the jam fix (`3382d69ea0`), file mode through a merge (`f89786b089`), kept references pruned by size and age (`e5e75025ed`).
  Its brief was rewritten nine seconds before its run began, and its session holds the new brief (the owner's words are in it).
- **Pass 20** (finished-records alone) was already going and stays as it is.
- Passes 22 and 23, queued with one branch each before the answer, are gone:
  23 was taken off the queue; 22 had been started by the filling unit while the question was open, forty seconds before it was stopped at 16:14:03,
  with nothing committed (`git log main..` empty, the worktree clean). Its units were stopped, its worktree removed through the wrapper, its branch deleted, and both briefs deleted.
  This is the one run stopped today, and not to get under the level: its whole task had moved into pass 21.

From lines 24489 to 24496 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The owner: one agent, one thing; pass 20 started with the first branch whose review found nothing (2026-11-13T15:38:11-05:00)

Not complete, and written by the coordinating session: pass 20 is queued and has merged nothing yet.

- The owner, between 15:35 and 15:37, as the brief for integration pass 20 was being written with a merge, a test fix and a possible second branch in it:
  "Avoid giving any one agent 2 things at once, or giving them one different thing after another."
  It is rule `A1T` in `AGENTS.md` (`1d826d9b38`): a brief holds a single task, and a different task goes to a fresh agent, never into a running or resumed one.
  A resumed author answering the review of its own change is the same thing going on.

From lines 24516 to 24527 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### Four more runs ended by 15:31:30; the upstream project already has the rate-limit classification reported (2026-11-13T15:34:05-05:00)

Not complete, and written by the coordinating session: every branch named here waits for a review or for its author.

- **Upstream.** Asked whether the classification is reported to Pi's upstream project, the owner chose "Draft only, I post it" (answered between 15:30 and 15:32).
  The search made for the draft found it reported already: issue 10656 in `earendil-works/pi`,
  with a pull request (10677) scoped to the provider that the project's contributor gate closed unmerged.
  So the draft is a comment with this machine's counts, not a new issue:
  `~/temp/agent/runs-of-2026-11-12-ended-by-the-full-drive/keep/draft-for-the-owner-to-post-comment-on-upstream-pi-issue-10656.md`. Nothing was posted.
  That thread names answers of the same provider that must stay final
  (`PrepaidBillOverdue`, `PostpaidBillOverdue`, and the message "Free allocated quota exceeded", which holds the words of the passing one);
  the run writing the plugin fix was given that as a note at 15:32:32.

From lines 24740 to 24753 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

##### The owner's decisions of 2026-11-13, answered between 13:56 and 14:37

- **An ignored file in the way of a discarding command** (`git reset --hard <other>`, `git checkout --force` and the like, where the file exists nowhere else):
  "Refuse, with opt-out (Recommended)".
  The command is refused and names the file; `--no-enforce-worktree` runs it as real Git would.
  The other options offered were "Allow and log" (the decision of 2026-11-12 for `git clean -fX`) and "Allow silently, as Git"
  (Git's documented default, `--overwrite-ignore`).
  This is what the discarding-commands branch does at `26834c0f32`; nothing changes there.
- **An abort in the main worktree**: "Same rule everywhere (Recommended)".
  In the main worktree too, `git merge --abort` and the rebase, cherry-pick, revert and am aborts run when they discard nothing,
  and are refused, naming the files, when they would discard work.
  The older rule (every abort refused in the main worktree) is ended.
  The abort-and-skip branch's author is told in its brief.

From lines 25207 to 25213 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The number of agents is a level, not a wall.** This session stopped a run of its own at 10:49:59 because another session's start had made five.
  The owner: "\"Forced you stop yours\"? Didn't I say bisect up until there's a problem?"
  No going run is stopped to hold the number; a count above it with no symptom is a reading at that count.
  The canonical record of the level, the criterion and the readings is now `doc/handover/agents-at-once-on-this-machine.md`,
  which every session reads (`AGENTS.md`, rule `MXA`).
  The five-unit readings of 10:45:25 to 10:49:25: write-and-sync 2 to 3 ms on both drives.
