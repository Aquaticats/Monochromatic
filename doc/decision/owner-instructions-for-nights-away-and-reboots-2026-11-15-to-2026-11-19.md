# Owner instructions for nights away and reboots 2026 11 15 to 2026 11 19

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 44778 to 44786 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


### The owner is away overnight; shut down when nothing is left (2026-11-18T23:25:00-05:00)

The owner, plain text: "I'm gonna go to bed. You keep working and try not to block on me. Whenever you run out of
things you can do w/o me, shutdown the entire system." So: every run going finishes and is handled (reviews, merges
of reviewed non-wrapper work, the next step of each item); questions for the owner are written here, ranked, and asked
when the owner is back, never waited on; and once no run is going and nothing more can move without the owner,
everything is committed and pushed (rule `RBT`), every unit and worktree this session started is stopped or removed,
and the machine is powered off. The queue stays stopped (no heavy worktree use before the ZFS route).

From lines 45216 to 45216 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

**State at the overnight poweroff (owner: "Shut down when done"), 2026-11-19:** `main` equals `origin/main`; no local

From lines 45232 to 45235 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


**Owner to bed, 2026-11-19 about 01:05: "Shut down when done (Recommended)"** (asked: does last night's "shut the whole
machine down once I ran out of work" hold again). Before any power-off: every run ended and recorded, work committed
and pushed, main == origin/main, no `pi-*`, `reviewer-*` or other unit of this session left, then poweroff.

From lines 46029 to 46029 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- The owner then wrote: "I'm going to be away for a few hours. Don't block on me." No question is put while the owner is away; what needs the owner goes into the kept list.

From lines 46072 to 46077 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The session stood in a question dialog for ten hours and twenty minutes (written here at 2026-11-16T10:20:24-05:00; an earlier wording of this line gave a time four minutes ahead of the clock):

- **What happened.** At about 23:55 on 2026-11-15 this session's last message left two choices in prose for the owner, who had written "going back to sleep now". The stop hook required them to be put through the question tool, the session did so, and the dialog stood until the owner answered at about 10:17 on 2026-11-16.
  While it stood the session read nothing and started nothing: 59 runs ended unread, pass 57 ended at 02:42 with nobody to merge it, the filling unit ran out at about 07:46 and from then nothing was started (9 Pi units were going at 10:18), the drive probe ended at 01:14 and the rotation watch died at about the same time on a `systemctl` that did not answer.
- **The rule this session takes from it, open to veto:** while the owner is away or asleep, a final message leaves no choice in prose and offers nothing ("say if you …"); what needs the owner goes into the kept list, stated as a fact, and the question is asked when the owner is next at the keyboard. A question dialog is a wait with no bound.

From lines 46460 to 46461 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The owner has the new SSD and asked for the machine to be made ready for a reboot (about 17:58): "Let everything already running run to their conclusion and sort out the system until it's good to reboot. Tell me when is a good time to reboot."** So nothing new starts: no Pi run, no review, no probe, no filling unit. The filling unit was stopped between two of its rounds at 18:01:34 and none was started in its place; thirteen Pi runs and five reviews are going and end by themselves. Entries stay in the resume queue in their order, the queue author's sixth run and the retire author's second among those the filling unit had not started.
  Read at 18:00: `main` equals `origin/main` (`93f91a5dff`); no container is running. **Corrected after 18:03:57: `~/temp` is a link to `/var/mnt/encrypted/temp`, so the scratch `~/temp/agent` (the resume queue, every brief, every review file, every report) stands on the repository's SATA drive beside the repository, and not on the home folder's NVMe drive as first written here** (the first reading asked which drive holds `/var/home` and did not follow the link; `realpath` gave the answer). So one drive holds both, and the copy of the scratch's text goes to the home drive. Before the reboot: each run and review read and recorded here as it ends; every worktree checked for files not committed and commits `origin` does not hold; a copy of the scratch's irreplaceable text on the other drive; how to start again written here; this session's own units stopped. The scripts and their outputs are in `~/temp/agent/getting-the-machine-ready-for-a-reboot-2026-11-16/`.
