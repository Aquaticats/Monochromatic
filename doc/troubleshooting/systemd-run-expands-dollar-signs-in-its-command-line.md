# Systemd run expands dollar signs in its command line

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 38681 to 38683 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

    Note:
    `systemd-run` expands `$VAR` in its command line,
    so a literal `$?` must be written `$$?`;
