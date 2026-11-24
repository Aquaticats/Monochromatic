# Stale cargo lock file fails the lint through the closure check owner answer 2026 11 13

## Status

Moved on 2026-11-23 from the old handover, verbatim;
not yet edited into a record of its own.

## Moved from the old handover

From lines 24031 to 24045 of the old handover
(`doc/handover/old-git-wrapper-handover-being-sorted-into-records-and-issues.md` at `c84e2f1dbf`),
moved verbatim on 2026-11-23:

- That branch changes what a stale lock file does:
  with every crate cached,
  the fetch step used to fail on it and now leaves it alone,
  and the closure check names it as a warning that does not fail the lint.
  Asked through the question tool at about 17:42 with three options
  (fail through the closure check,
  a warning only,
  keep the old fetch),
  the owner chose "Fail via closure check (Recommended)".
  A run for it is queued:
  `stale-cargo-lock-file-fails-the-lint-through-the-closure-check`,
  branch `fix/linter-monochromatic-lint-stale-cargo-lock-file-fails-the-lint-through-the-closure-check`.
  The cargo fetch branch reaches `main` with that fix or after it,
  never before,
  so that no lint in between lets a stale lock file through.
