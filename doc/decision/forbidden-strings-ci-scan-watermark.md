# Forbidden-strings CI scan scope: validated-tree watermark

Adopted 2026-10-20 while fixing issue `#993`,
 under maintainer direction to prefer performance and correctness
 even at the cost of more code,
 and to reject full-tree scans on every push as too expensive.
 Regression tests:
 `package/forbidden-strings/cli/tests/ci_scan_scope.rs`.

## Problem

The push scan scope was the push event's own `before..after` range.
 Any run that did not complete successfully
(canceled by `cancel-in-progress`,
 failed,
 or timed out)
 left its range unscanned,
 and no later run rescanned it unless a later push touched those files again.
 Issue `#993` reports the canceled case;
 a failed run leaves the same hole.

## What was adopted

Push runs scan the tree-to-tree delta from a validated-tree watermark:

- The watermark is the head commit of the newest checked-out-history ancestor that is also
  the head of a successful `push` run of this exact workflow on the same branch.
- A run's own head becomes a watermark candidate only when that run completes successfully,
  so cancellation, failure, and timeout can never advance the baseline past unvalidated
  changes.
- Candidate heads must carry the `validated-tree-watermark-v1` marker in their committed
  workflow file.
  Only revisions whose push runs prove their head tree this way count;
  earlier revisions scanned only their own push range.
- With no suitable candidate the run scans the current tree with `--all`, the only
  coverage-safe scope when validation state is unknown, and that head becomes the first
  watermark for later runs.
- Pull requests and merge groups keep their cumulative `origin/main...HEAD` comparison,
  which already survives cancellation,
  and never establish watermarks:
  a pull-request head can sit in `main`'s ancestry after a merge while having only been
  validated for its own pull-request diff.

## Invariant

A successful marker-era push run leaves its head tree fully validated:
 every file in it was either scanned by that run or is identical to a file in the baseline
 tree,
 which was validated by induction down to a full-tree fallback run.
 The next run's tree-to-tree delta therefore covers every still-present change that any
 earlier run failed to validate.

## Rejected alternatives

- **Full-tree scan on every push**.
  Measured 29.9 s warm and 51.2 s cold on 2026-10-20 over 17,803 tracked files and 785 MB,
  growing with the corpus;
  rejected by the maintainer.
  It stays only as the rare fallback above.
- **Commit-status or movable-tag watermark**.
  A stamp needs `statuses: write`, which can forge required checks,
  and a tag needs `contents: write`, which can rewrite branches;
  both escalate a secret-bearing workflow.
  Listing runs is read-only (`actions: read`).
- **Watermark without the version marker**.
  Would trust pre-fix runs whose scans covered only their own push range,
  recreating the issue `#993` hole at bootstrap.
- **Fallback to changes since the newest old full-tree-era commit**.
  That era ended at `6eae389841` (2026-07-30);
  its delta is 14,800 files and 732 MB today,
  the same cost as a full scan.
- **Per-file scanned-blob manifest**.
  Selects the same scan sets as the watermark at every step except files reverted to a
  previously scanned blob,
  and needs durable storage the run list does not offer.
- **Cross-run cache of the compiled rule artifact**.
  The artifact retains exact rule bytes (README security model),
  so caching it across runs would leak the secret rule set to any later run.
- **Dropping `cancel-in-progress`**.
  A failed run leaves the same hole;
  cancellation is one instance of "run did not complete successfully".

## Limits

- Rule-set changes do not rescan untouched files;
  a stricter rule set applies only to files the next scans select,
  as before this change.
- Watermark lookup depends on the Actions run list;
  when listing fails the run falls back to the current tree,
  which is sound but slower.
- The marker is a version handshake:
  any change to scan-scope semantics must bump it in the same commit.
