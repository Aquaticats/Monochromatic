# Systemd unit writable path that does not exist when the unit starts

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 15111 to 15126 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A Pi run cannot write into the main checkout** (`0a9d5c3c93`),
  third review,
  ended 19:55:10,
  four things.
  Reproduced with a real capped transient unit:
  a `.cache` that is absent when the unit starts can never be made inside it,
  since systemd drops a writable path that is not there,
  so a checkout whose first run must make it fails with a read-only file system.
  That one would stop real runs.
  Reproduced for the read:
  a FIFO where the checkout's marker or a run's summary stands blocks the launcher before the unit exists or after it has ended,
  where the unit's own time limit covers neither.
  And two new messages say nothing of the work is lost,
  where a removed worktree's ignored files are.
  The author is resumed
  (`…/pi-run-cannot-write-into-the-main-checkout-answers-its-third-review-of-2026-11-14-prompt.md`).

From lines 17663 to 17676 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- **A Pi run cannot write into the main checkout** (tip `ca137eb2bf`, ended 13:54:18):
  the run's unit gets the main checkout read-only (`ReadOnlyPaths=`),
  with its own worktree,
  the common Git directory,
  the checkout's `.cache`,
  its scratch and its run folder writable (`ReadWritePaths=`).
  By its controls on a disposable repository:
  before the change a write and a delete under the checkout's `package/` succeed;
  after it both are refused with `Read-only file system`,
  while writes into the worktree,
  the run folder,
  scratch and `.cache` succeed and a commit lands.
  The user manager of this machine (systemd 262) honours both properties for an unprivileged unit;
  a writable path that does not exist fails the unit unless it carries the `-` prefix.
