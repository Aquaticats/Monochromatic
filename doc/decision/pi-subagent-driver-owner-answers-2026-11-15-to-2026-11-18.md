# Pi subagent driver owner answers 2026 11 15 to 2026 11 18

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 45830 to 45840 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **Keeping a Pi run out of the main checkout**: "Protect, containers via manager (Recommended)".
  The checkout stays read-only for a run through systemd's path properties;
  a run's containers are started outside the run by the account's service manager.
  The owner was told, and accepted by choosing it:
  a container started that way can mount the checkout writable, one more stated hole beside the transient-service one,
  since the protection guards against accidents and not against a run that tries to get out;
  `link` and `rename` across the run's folders end `EXDEV` and that stays a stated limit;
  and `podman` had not yet been run through the route when the question was asked.
  So `EXDEV` no longer stops the driver branch as a finding it brings.
  The options not taken: hold the branch for a protection without mounts (Landlock, unmeasured);
  the protection with no containers in a run; no protection.

From lines 45947 to 45954 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:


The Pi driver and reviews:

- A review that does not cover its change: **"Restart once by itself ON A DIFFERENT MODEL"**, and which model: **"The other of the pair; Sonnet while Qwen is dry (Recommended)"**.
- A review's unit and a read-only worktree: **"Authors only"** (against the session's recommendation).
- A worktree at a run's name with no proof of who made it: **"Remove after a bound"** (against the session's recommendation).
- A wrapper process in a run's worktree: **"Do not count the commit service (Recommended)"**.
- A run kept because its scan cannot be read: **"A shorter limit, then end as failed (Recommended)"**.

From lines 45989 to 45990 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Pi driver, the queue of runs: **"Folder, one file per entry (Recommended)"**. The lock file over one JSON file is set aside; the rule on lock-free concurrency decides it.
  The scratch queue of this session is rewritten with it at the changeover.

From lines 46064 to 46064 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  The owner's earlier answer for a thin review ("Restart once by itself ON A DIFFERENT MODEL", "The other of the pair; Sonnet while Qwen is dry") has no other model left to go to while Sonnet is the only reviewer:

From lines 46078 to 46079 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **The owner's two answers (2026-11-16, about 10:17).** A thin Sonnet review: **"Kimi for these only"**: the Kimi pool is reopened for one command for each thin review (`PI_RUNS_POOL_LIMITS=neuralwatt-kimi-flex=20` on that start alone); the pool stays closed for everything else, and the adoption recorded on 2026-11-15T23:42 (hold a thin review) is replaced by this.
  The open LFS audit of `0.6.1-alpha.14`: **"Sonnet, current code (Recommended)"**: it is run as a Sonnet audit of the LFS path in today's `main`.

From lines 46508 to 46508 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **How much Sonnet review: "All queued, three at once (Recommended)".** Asked with the measurement beside it (four reviews used 223,867 to 582,762 subagent tokens, mean about 372,000, 17 to 27 minutes each; about fifty tips queued, on the order of 19 million tokens).

From lines 46516 to 46518 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

One more answer, at about 11:34 on 2026-11-16, asked with the morning's measurements beside it (reviews finish at up to about 7 an hour at three places, author runs end at about 6 to 7 an hour, so 49 queued reviews do not shrink):

- **How the 18 agent places are split: "5 reviews, 13 Pi runs (Recommended)".** Five Sonnet reviews at once; the unit that fills Pi places runs at level 13 since 11:34:59 (no going run is stopped). The cap of 18 is unchanged.
