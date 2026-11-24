# Pi review run stalls before sending and times out at 3600 seconds

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 49657 to 49674 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- Cross-vendor reviews run through `pi` (see `CLAUDE.md`),
  each in its own scope (1 CPU,
  2 GiB),
  as many at once as there is work:
  the owner asked "make sure we're running everything that can be run in parallel in parallel".
  An earlier "keep at most three running" was the coordinator's own unmeasured guess at the cause of the timeouts
  (added 2026-11-02,
  `f0780fff47`),
  never an owner rule;
  removed 2026-11-04 after the owner asked who set it.
  Whether concurrency affects the timeouts is unmeasured;
  record each run's concurrency and outcome before limiting again.
  Several have timed out at 3600 s with no output and an empty session folder
  (`~/.pi/agent/sessions/--var-mnt-encrypted-temp-agent-review-*--/`):
  the run stalled inside `pi` before sending.
  A one-word probe answers in about 5 s;
  relaunch with the original question,
  narrower input and a word cap.
