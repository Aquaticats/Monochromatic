import { reportingRefusals, } from './cli-refusal.ts';
import { resolveRunsDir, } from './run-config.ts';
import { verifyPublishedRun, } from './verify-published-run.ts';

//region Verify published
// Checks a run's PUBLISHED TREE against the artifacts that produced it: the
// wiring only. It spends no quota and touches no model. What it reads and
// prints is in `verify-published-run.ts` and `verify-published-entry.ts`, and
// what a pass does about a disagreement is in `page-agreement.ts` and
// `published-page-check.ts`.
//
// A RUN ALWAYS SHIPS (the owner, 2026-09-27; ledger A16b). Every finding is
// printed and the exit is 0; only a run that could not be read at all exits
// otherwise, since that run was never examined.

/**
 Verifies the runs directory the environment names, with this build as the
 reader, and leaves the exit code the verification returns.

 @example
 ```ts
 await verifyHere();
 ```
 */
async function verifyHere(): Promise<void> {
  process.exitCode = await verifyPublishedRun({
    runsDir: await resolveRunsDir(),
    pipelineDir: import.meta.dirname,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'verify-published',
    argv: process.argv,
    env: process.env,
    run: verifyHere,
  },);

//endregion Verify published
