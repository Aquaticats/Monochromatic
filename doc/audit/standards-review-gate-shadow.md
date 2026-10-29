# Standards-review gate shadow log

Running record of shadow runs of the gate in `doc/runbook/standards-review-gate.md`.
One entry per run,
appended as runs happen.

An empty log means no shadow run has happened yet,
and never means there was nothing to catch.
The research baseline these runs extend is the 6 of 8 recall measurement
under the heading `Recall measurement results`
in `doc/handover/superlinear-cost-dismissal-gate.md`.

## Entry format

- Date:
- Subject:
- Input size:
- Reviewer route:
- Wall-clock cost:
- Named superlinear cost:
  yes,
  no,
  or unjudged
  (the runner writes `unjudged`;
  a scorer reads the findings and settles it)
- Findings kept:
- Skipped: 12c25207857b docs(handover): record the finished tool-image migration and the pre-existing failures it surfaced (no code files)
- Skipped: 17241cbad51b feat(mise): run the standards-review gate shadow from a post-commit hook (no code files)
