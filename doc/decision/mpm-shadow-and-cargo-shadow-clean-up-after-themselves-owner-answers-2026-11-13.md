# Mpm shadow and cargo shadow clean up after themselves owner answers 2026 11 13

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 23727 to 23747 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

##### The owner's answers

- Asked at about 18:28 whether the folders of 2026-11-10 and 2026-11-11 may be deleted
  (both watched,
  the older alone,
  or wait for the new drive):
  "Delete both,
  watched.
  and make our mpm-shadow auto cleanup old toolchains by size + last modified time."
- Asked which of the two tools should own that cleanup
  (the session offered mpm-shadow or cargo-shadow's cleaner as alternatives,
  which they are not):
  "What?
  Shouldn't it be both?
  mpm-shadow cleans up after itself,
  cargo-shadow also cleans up after itself."
- So each tool removes,
  by itself,
  what it itself made,
  chosen by size and last-modified time:
  mpm-shadow the old versions it installed,
  cargo-shadow the build folders of old toolchains.
