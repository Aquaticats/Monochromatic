# Owner instructions for nights away and reboots 2026 11 15 to 2026 11 19

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 44778 to 44786 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### The owner is away overnight; shut down when nothing is left (2026-11-18T23:25:00-05:00)

The owner,
plain text:
"I'm gonna go to bed.
You keep working and try not to block on me.
Whenever you run out of
things you can do w/o me,
shutdown the entire system."
So:
every run going finishes and is handled (reviews,
merges
of reviewed non-wrapper work,
the next step of each item);
questions for the owner are written here,
ranked,
and asked
when the owner is back,
never waited on;
and once no run is going and nothing more can move without the owner,
everything is committed and pushed (rule `RBT`),
every unit and worktree this session started is stopped or removed,
and the machine is powered off.
The queue stays stopped (no heavy worktree use before the ZFS route).

From lines 45216 to 45216 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

**State at the overnight poweroff (owner:
"Shut down when done"),
2026-11-19:**
`main` equals `origin/main`;
no local

From lines 45232 to 45235 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


**Owner to bed,
2026-11-19 about 01:05:
"Shut down when done (Recommended)"** (asked:
does last night's "shut the whole
machine down once I ran out of work" hold again).
Before any power-off:
every run ended and recorded,
work committed
and pushed,
main == origin/main,
no `pi-*`,
`reviewer-*` or other unit of this session left,
then poweroff.

From lines 46029 to 46029 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The owner then wrote:
  "I'm going to be away for a few hours.
  Don't block on me."
  No question is put while the owner is away;
  what needs the owner goes into the kept list.

From lines 46072 to 46077 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The session stood in a question dialog for ten hours and twenty minutes (written here at 2026-11-16T10:20:24-05:00;
an earlier wording of this line gave a time four minutes ahead of the clock):

- **What happened.**
  At about 23:55 on 2026-11-15 this session's last message left two choices in prose for the owner,
  who had written "going back to sleep now".
  The stop hook required them to be put through the question tool,
  the session did so,
  and the dialog stood until the owner answered at about 10:17 on 2026-11-16.
  While it stood the session read nothing and started nothing:
  59 runs ended unread,
  pass 57 ended at 02:42 with nobody to merge it,
  the filling unit ran out at about 07:46 and from then nothing was started (9 Pi units were going at 10:18),
  the drive probe ended at 01:14 and the rotation watch died at about the same time on a `systemctl` that did not answer.
- **The rule this session takes from it,
  open to veto:**
  while the owner is away or asleep,
  a final message leaves no choice in prose and offers nothing ("say if you …");
  what needs the owner goes into the kept list,
  stated as a fact,
  and the question is asked when the owner is next at the keyboard.
  A question dialog is a wait with no bound.

From lines 46460 to 46461 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The owner has the new SSD and asked for the machine to be made ready for a reboot (about 17:58):
  "Let everything already running run to their conclusion and sort out the system until it's good to reboot.
  Tell me when is a good time to reboot."**
  So nothing new starts:
  no Pi run,
  no review,
  no probe,
  no filling unit.
  The filling unit was stopped between two of its rounds at 18:01:34 and none was started in its place;
  thirteen Pi runs and five reviews are going and end by themselves.
  Entries stay in the resume queue in their order,
  the queue author's sixth run and the retire author's second among those the filling unit had not started.
  Read at 18:00:
  `main` equals `origin/main` (`93f91a5dff`);
  no container is running.
  **Corrected after 18:03:57:
  `~/temp` is a link to `/var/mnt/encrypted/temp`,
  so the scratch `~/temp/agent` (the resume queue,
  every brief,
  every review file,
  every report) stands on the repository's SATA drive beside the repository,
  and not on the home folder's NVMe drive as first written here** (the first reading asked which drive holds `/var/home` and did not follow the link;
  `realpath` gave the answer).
  So one drive holds both,
  and the copy of the scratch's text goes to the home drive.
  Before the reboot:
  each run and review read and recorded here as it ends;
  every worktree checked for files not committed and commits `origin` does not hold;
  a copy of the scratch's irreplaceable text on the other drive;
  how to start again written here;
  this session's own units stopped.
  The scripts and their outputs are in `~/temp/agent/getting-the-machine-ready-for-a-reboot-2026-11-16/`
  (scratch,
  deleted on 2026-11-27;
  not kept).

From lines 1967 to 1982 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Stopped for the owner's shutdown at night (2026-11-17T19:28:50-05:00)

Not final and not approved:
this session's own account.
The owner:
"shutdown the entire system after you finish A THOROUGH RESEARCH.
I'm going to bed."

- **ZFS fast route (task 379):**
  the research the owner asked for twice ("Dig even deeper.")
  is written up in
  `doc/planning/cli-git-zfs-worktree.md`,
  with every mechanism tried on this machine as root on throwaway datasets
  (the owner:
  "You have passwordless sudo."),
  every system file put back and checked,
  and probe scripts and outputs under `~/temp/agent/zfs-fast-route-design-facts-2026-11-17/`
  (scratch,
  deleted on 2026-11-27;
  what this record uses from it is stated here).
  Findings that changed the earlier questions:
  an anchored-regex sudoers rule (sudo 1.9.17p2 here) allows exactly
  mount and unmount of a child of the parent and refused every variant tried,
  so it needs no root code of ours;
  a wildcard sudoers rule lets the user turn setuid back on;
  a polkit rule for systemd mount units lets the user
  mount anything at the allowed path (tmpfs and `suid` were accepted),
  so it is unfit to ship;
  fstab `user` entries fail for ZFS in both forms;
  a ZED handler works;
  fuse-overlayfs needs no root and costs 2.2× to 3.4× for the worktree's life.
  The next step moved to the entry of 2026-11-18T17:15:00-05:00.

From lines 2021 to 2029 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### Stopped for a reboot with the new SSD in hand: what is saved where, and how to start again (2026-11-16T19:22:33-05:00)

Not final and not approved:
this is the session's own account of what it checked.
Every run and review of this effort has ended;
the last review had returned by 19:28:45 and its result is in the round list under "Open work".

#### What the owner asked

At about 17:58 on 2026-11-16:
"I just received the new SSD.
Let everything already running run to their conclusion and sort out the system until it's good to reboot.
Tell me when is a good time to reboot."

From lines 2692 to 2740 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

#### The owner is away for six to ten hours from 2026-11-13T18:48:09-05:00

The owner,
in the terminal at about 18:47:
"I'm going to be away for 6 to 10h.
You keep working."

What that means for the coordinating session until the owner writes again,
by the rules that already stand:

- The queue goes on:
  each run that ends is read and routed
  (a review for an author's answer,
  the findings back to the author,
  a pass for a branch whose review found nothing),
  and each pass that reports fit is merged into `main`,
  released,
  installed and sample-committed,
  in the order they finish.
  Publishing alpha releases of the wrapper stands authorized from earlier in the effort.
- Nothing is asked through the question tool while nobody is there to answer.
  A choice that a recorded decision settles is adopted,
  written under "Adopted while the owner was away" in the section it belongs to,
  and is open to veto.
  A choice no record and no measurement settles is not made:
  the work that needs it waits,
  everything else goes on,
  and the question is kept in the list at the end of this section for the owner's return.
- Nothing destructive beyond what the owner already ordered
  (the deletion of the two old toolchains' build folders,
  going since 18:30:32),
  nothing sent outside this repository other than its own issues and releases,
  and no change to the pools:
  about twenty-five agents stays the most.
- The units this depends on and when each ends by itself:
  the filling unit `fill-free-places-and-put-ended-reviews-back-at-level-24-20261114T085433`
  (started 2026-11-14T08:54:33-05:00 for eight hours,
  so it is started again by 16:54;
  the one at level sixteen was stopped in the same second,
  on the owner's answer that the cap resumes at twenty-four),
  the drive probe at level twenty-four
  (a unit named `disk-ceiling-probe-level-24-` with its start second,
  started within a minute of the filling unit for eight hours;
  it ends with status 3 on two slow home-drive readings in a row,
  and the rule then gives twenty),
  and the waiter of the moment
  (eight hours from its start,
  started again each time it reports).

From lines 10136 to 10146 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

##### The owner sleeps and the CPU is held under 80 degrees; six runs are routed, among them two reviews that send mpm's stale launchers and the killed-Git lock back (2026-11-15T05:08:56-05:00)

Not final and not approved:
an instruction of the owner's,
what three reviews and four authors report,
how the coordinating session weighed them,
and what it queued.

- **The owner's words,
  about 04:51**:
  "I need to go to sleep for a few hours.
  Keep CPU temp lower than 80 degrees until I say I've finished sleeping for today."

From lines 46590 to 46590 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **"Add the rule (Recommended)"**:

From lines 46625 to 46627 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner wrote "I've finished sleeping now and you can lift the CPU temp limits.",
and then answered four questions through the question tool,
between 10:46:27 and 10:48.

From lines 46629 to 46630 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The CPU temperature**:
  "Leave lifted".

From lines 46635 to 46636 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The cap on agents**:
  "Step to 20 with the probe (Recommended)".

From lines 46684 to 46685 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The briefing rule in `AGENTS.md`**:
  "Name both folders (Recommended)".
