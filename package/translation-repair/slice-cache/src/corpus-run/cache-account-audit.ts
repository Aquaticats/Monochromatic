import { auditCacheAccounts, } from './cache-account-run.ts';
import {
  type CommandLineOf,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Cache account audit
// The pre-launch cache version check (ledger M28), as a command: the wiring
// only. It runs from the package directory, which the mise task does, and
// names the runs directory a pass would use now. What it reads and prints is
// in `cache-account-run.ts`, `cache-account-git.ts`,
// `cache-account-setting.ts`, `cache-account-print.ts` and
// `cache-account-slice-report.ts`.

/**
 Runs the audit from where the process stands, over the runs directory the
 environment names.

 @param line - the command line, read whole by `reportingRefusals`

 @example
 ```ts
 await auditHere({ line, },);
 ```
 */
async function auditHere({ line, }: { readonly line: CommandLineOf<'cache-account-audit'>; },): Promise<void> {
  await auditCacheAccounts({
    line,
    packageDirectory: process.cwd(),
    runsDir: await resolveRunsDir(),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'cache-account-audit',
    argv: process.argv,
    env: process.env,
    run: auditHere,
  },);

//endregion Cache account audit
