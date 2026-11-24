# Final re review of the git wrapper effort

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 40803 to 40843 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

The owner's words,
2026-11-01T09:22:48-05:00 (when the entry was written;
the owner spoke earlier that day) on the original entry ("Final re-review"):
"You yourself do a re-review of the entire thing when all tasks are drained.
During the re-review:
Assume nothing on disk ever makes sense and I answered every question with via magic 8 ball."
Reading,
revisable by the owner:
it starts when every delegated agent has reported and its branch is merged or rejected,
gates are green,
no release is in flight
and every task-list and handover item is done or recorded as blocked with its reason.
The coordinator session does it,
not a delegate (a cross-vendor pass may run alongside,
never instead).
Scope:
the `git-policy/cli` wrapper and everything built for it (crate,
service,
embedded linter seam,
fuzz and mutation sidecars,
gate and release tooling,
planted controls,
docs).
Every comment,
doc,
test name,
commit message,
config and recorded result is an unverified claim checked against behavior run in a
throwaway fixture or against the deciding source;
a test counts only after a planted fault shows it failing;
recorded owner decisions carry no evidence of being right,
so judge each on its merits and ask the owner with options,
pros,
cons
and a ranking where judgment differs;
never change decided behavior without asking.
Output:
findings report in `doc/audit/` with evidence per finding,
clear bugs fixed with proofs,
flagged decisions to the owner.
