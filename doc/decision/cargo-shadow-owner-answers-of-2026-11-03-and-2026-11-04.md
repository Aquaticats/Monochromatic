# Cargo shadow owner answers of 2026 11 03 and 2026 11 04

## Status

Moved on 2026-11-23 from the old handover,
verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 46961 to 46976 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  cargo-shadow:
  owner answered "Yes,
  add the wiring (Recommended)" for the three release-wiring edits the permission check had refused the agent
  (`releasable-crates.json`,
  the crate list in `crate-release.yml`,
  `release` and `release:wait` tasks),
  and "Done."
  for the first-publish token (`cargo login` in the owner's own terminal,
  per `doc/runbook/publish-crate-first-time.md`);
  the branch (`39b67c8748`,
  main merged,
  610 passed) merges after 0.5.32 is built;
  its one `doc/decision/rust-dependency-catalog.md` hunk (`c3602e69f1`,
  the update now living in the cargo shadow):
  owner "Approve the edit (Recommended)" (question tool,
  2026-11-04).

From lines 46984 to 46989 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

  Done 2026-11-04:
  0.5.32 installed;
  cargo-shadow merged to main (`624e4ea5d2`,
  without waiting for 0.5.32,
  owner:
  "you don't have to wait for that.");
