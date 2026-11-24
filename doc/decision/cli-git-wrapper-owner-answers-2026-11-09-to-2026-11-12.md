# Cli git wrapper owner answers 2026 11 09 to 2026 11 12

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 29324 to 29325 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Whether refusing a foreign nested subvolume by name meets "make the wrapper work in that case.",
  or whether it is set aside under a kept name.

From lines 29468 to 29484 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### A nested subvolume that the wrapper did not make

Branch `fix/git-policy-cli-worktree-removal-finishes-with-a-nested-subvolume`,
tip `34af1df537`,
not merged.
A removal now takes the pools the wrapper itself made and finishes
(exit status 0 where `0.6.1-alpha.10` ended with 2 and a registration left behind),
and a killed removal is finished by the next `git worktree list`.
For a subvolume or mount someone else put inside the worktree it refuses before anything is destroyed and names it.
The agent laid out the alternatives and built the first:
refuse and name it;
set it aside under a kept name (cannot cover a mount point,
breaks an inner worktree's pointers);
delete it (discards data,
never);
copy it out and delete (doubles the disk,
unbounded time).
The owner's instruction was "make the wrapper work in that case.";
whether refusing a foreign subvolume by name meets it is a question for the owner's return.

From lines 30197 to 30208 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner's answer,
2026-11-11T19:12-05:00,
through the question tool,
to "Who should do the rename?"
(options:
the service hands out a tree,
a mark inside the member,
leave it as it is):
"Service hands out a tree (Recommended)".
So the commit service renames a pool member into place ahead of time and hands the commit its path,
and the commit still runs every check on the tree it is handed (the record's option A,
checked form).
The record's other two questions were picked under the owner's delegation of non-destructive choices,
open to veto:
every check the claim makes stays (options C and D,
the claim trusting the fill,
are not taken:
the saving is 0.82 ms at the median and the tail does not move);
and the smaller skeleton with one listing (option E) is built if its probe passes
(Git and `write_shadow_refs` in a shadow without `objects/pack`,
`refs/heads` and `refs/tags`),
since it changes no guarantee.
Round 237 builds all of it (`perf/git-policy-cli-commit-service-hands-a-commit-its-shadow`,
on `integrate/git-policy-cli-pass-14-for-alpha-11`,
brief `commit-service-hands-a-commit-its-shadow-prompt.md`).

From lines 30369 to 30380 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

A jam met during the first pass,
and the owner's instruction on it.
The worktree of the finished round 228 run (`pi-clippy-passes-after-the-daily-update-of-2026-11-11`) could not be removed:
its Git directory held the wrapper's own snapshot taken ahead (`monochromatic-git-policy-cli/ahead/.cli-git-worktree-ahead-*`),
the wrapper's removal refused it as "a nested subvolume or another filesystem",
and every Git command in the main checkout printed "The interrupted removal of ... cannot be finished"
until the coordinator ran `btrfs subvolume delete` on that snapshot and `git worktree prune` at about 18:45.
The owner,
on reading "because of a nested subvolume":
"make the wrapper work in that case."
Round 234 (`fix/git-policy-cli-worktree-removal-finishes-with-a-nested-subvolume`,
on top of `fix/git-policy-cli-sonnet-findings-removal-with-a-nested-subvolume`,
whose answer was to refuse)
makes the removal delete every subvolume the wrapper itself made and finish an interrupted removal unaided;
for a subvolume that is not the wrapper's own it reports the ways to remove the worktree without losing that data and builds one only if it is shown safe.

From lines 33241 to 33252 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

### The merge carry loses edits on a kill or a save (2026-11-10T05:53:55-05:00)

`~/temp/agent/merge-carry-script-on-main/reproductions/carry-interruption.ts`,
run under the owner's "Run that script here (Recommended)",
reproduced both cases on `b73a61f9f0`.
A `SIGKILL` between the staged and unsaved layers loses the unsaved bytes for good:
the next command drops the capture ref and the record,
and says the file is the person's.
A save between the carry's check and its write is overwritten,
and the merge exits 0.
The Pi agent `merge-carry-keeps-edits-through-a-kill-and-a-save` owns the fix.

From lines 34002 to 34016 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  The owner answered the moved-path-note question on 2026-11-10T08:08:51-05:00,
  "Yes,
  notes only (Recommended)",
  for the twelve issues that question named (4365,
  5929,
  6201,
  3279,
  5926,
  5788,
  3191,
  3275,
  5765,
  5766,
  6101 and 4357):
  the note adds a line beside a stale path and changes no decision.
  Issues 4408 and 3258 were not among them.
