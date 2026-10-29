# Task-slice adoption acceptance

Purpose:
 the reopened vet for Pants 2.33.1 is closed
 with no disqualifying finding,
 so the last gate before the `doc/decision/` record
 and the 1,811-name migration
 is your explicit adoption acceptance.

What changed since the contract answer:
 [`doc/audit/tech-task-slice-owner-vet-2026-10-29.md`](../audit/tech-task-slice-owner-vet-2026-10-29.md)
 is complete,
 filled from two 2026-10-29 upstream audits
 (maintenance and release provenance,
 license and dependency surface,
 production source for the consumed boundary,
 human auditability).

What to inspect:
 the vet report sections
 "Adoption-depth audit for Pants 2.33.1",
 "Confidence and evidence limits",
 and "Recommendation".

## Vet verdict

No hard gate fails.
The material findings:

- License and provenance pass.
  Apache-2.0,
  GitHub release assets only
  (PyPI is dead-ended at 2.17.1),
  every asset carries a GitHub sha256 digest,
  and `.github/workflows/release.yaml`
  runs `actions/attest-build-provenance`
  with a published sigstore bundle.
  No detached signatures,
  so offline verification is weaker
  than the GitHub-hosted path.
- Maintenance is alive but concentrated.
  Median merge latency 0.24 days
  and 26 of 30 recent merges land within 8 days,
  yet `benjyw` merged 24 of those 30
  and external work waits 47 to 421 days.
  `MAINTAINERS.md` lists 18 active maintainers.
  Stable release `2.33.1` is 63 days old,
  with newer work in prereleases and cherry-picks.
- The consumed boundary is exactly the slice
  and is inspectable:
   `adhoc_tool` argv is a plain list,
  exec-spawned with `env_clear()`;
  `peek` emits JSON with expanded dependency edges;
  `--loop` reruns on `notify` invalidation;
  cache hits are labeled `ran`,
  `hit_locally`,
  `hit_remotely`,
  `memoized`.
  Caveat:
   the default configuration wraps argv
  in `bash -c "cd <workdir> && <shlex.join(argv)>"`,
  so a `bash` is in the tree
  though every element is POSIX-quoted;
  a local-workspace environment target
  removes the wrapper entirely.
- The measured summary omission
  and the fail-fast raised path
  are confirmed in source
  (`test.py:1026` `continue`s past
  `result_metadata=None` results;
  `task.rs:181` `future::try_join_all`
  aborts the goal on the first raised failure).
  Neither is acknowledged upstream,
  and the 30 second default timeout
  (`AdhocToolTimeoutField` `default = 30`)
  is not reported as a defect either.
  All three stay unacknowledged,
  which is the trust 1 of 4
  already deducted in the scoring.
- Fuzzing and mutation testing are absent
  from the Pants tree.
  Reported,
  not scored.

## What adoption must carry in

Upstream does not provide these,
 so the migration builds them:

- a summary line
  for the dependents of a failed build;
- an aggregate exit path
  that survives a raised failure;
- a task timeout above 30 seconds;
- watch and cache configuration
  the repo owns
  (the launcher,
  Python Build Standalone,
  and anonymous telemetry
  are external or on by default).

## Answer 2026-10-29

The user wrote "2,
and we're still going to migrate off pants to meow later when we have capacity."

So the plan is written first
 ([`doc/planning/task-slice-pants-migration.md`](../planning/task-slice-pants-migration.md)),
 and adoption follows the read.
The scope statement is new and load-bearing:
 Pants is an interim owner,
 `meow` stays the endpoint,
 and hop 1 is written for cheap reversal
 and cheap second migration.

## How to respond

Options,
 ranked
 (ranking 1 > 2 > 3,
 because the vet is clean and the owner is already chosen,
 so the decision record can be written now
 while the migration plan is the natural next artifact;
 option 2 delays the record without adding evidence;
 option 3 discards a settled choice).

1.  Adopt.
    Write `doc/decision/task-slice-owner-pants.md`
    (adopted candidate and version,
    adoption date and authorizing request,
    linked vet report and governing skill revision,
    hard constraints and frozen weights,
    complete ranking and rejected alternatives,
    integration boundary,
    migration,
    exit and rollback,
    revisit triggers),
    record the scope carve
    (the task-execution slice leaves `meow`'s scope
    from `doc/decision/monorepo-manager-all-rust.md`
    and goes to Pants),
    then start the migration plan
    for 1,811 task names,
    603 edges,
    and the `//package/<path>:<task>` call sites.
2.  Adopt the owner,
    but write the migration plan first
    and adopt after reading it.
3.  Hold.
    Keep the question open
    and change no decision record.

Example response:

```text
1
```
