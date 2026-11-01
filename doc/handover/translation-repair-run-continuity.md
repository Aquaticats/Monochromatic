# Keeping the corpus pass running across agent sessions

A corpus pass outlives the session that started it,
and the API failures that
 end an agent session mid-work do not end the pass.
This records what is
 supervising the `pass13` run,
why there are two supervisors rather than one,
 and how to stop them.

Nothing here lives in the repository.
The scripts,
logs and the runs directory
 are all outside git,
because the runs directory holds corpus text from an
 unlicensed source.

## Current status, 2026-09-06

Passes on the legacy pipeline are authorized again since the owner's direction decision of 2026-09-01
and the OpenRouter order of 2026-09-03;
the current handover is [`translation-repair.md`](translation-repair.md).
Every pass runs under the owner's rule of 2026-09-06,
ALWAYS KILL AND RELAUNCH:
when source changes while a pass is running,
kill the pass by pid,
build,
and relaunch the same entry into a fresh runs dir on the new build.
A supervisor that resumes a pass across a rebuild would break that rule,
so no supervisor is used;
the launch is one detached process per entry.

## Superseded status, 2026-09-01

No corpus pass or continuity supervisor is running for the current redesign work.
This is historical operating evidence for the legacy pipeline,
not authorization to relaunch a run.
The finite replacement effort stopped after Candidate M failed.
Read
[`translation-repair-redesign-failure-2026-09-01.md`](../audit/translation-repair-redesign-failure-2026-09-01.md)
and the current section of [`translation-repair.md`](translation-repair.md) first.

Do not recreate supervisors,
resume a historical root,
or launch a fresh pass until the owner explicitly authorizes a new phase.
If that happens,
revalidate every command and commit identity against current source.

## Historical parts

- [Supervisor design and verification](translation-repair-run-continuity-supervision.md)
- [Second-run restart measurement](translation-repair-run-continuity-measurement.md)
- [Naturalness, refinement, and pairing cache fixes](translation-repair-run-continuity-cache-fixes.md)

These files are historical evidence,
not authorization to relaunch a pass.
