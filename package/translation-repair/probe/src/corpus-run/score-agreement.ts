import {
  type CommandLineOf,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { printGradeReport, } from './score-agreement-run.ts';

//region Score agreement
// Wiring only: the runs directory comes from the environment here, and the
// report itself is `score-agreement-run.ts`.

/**
 Prints precision and, when pre-grades exist, agreement against them.

 @param line - the report's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await reportGrades({ line, },);
 ```
 */
async function reportGrades({ line, }: { readonly line: CommandLineOf<'score-agreement'>; },): Promise<void> {
  await printGradeReport({
    runsDir: await resolveRunsDir(),
    line,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'score-agreement',
    argv: process.argv,
    env: process.env,
    run: reportGrades,
  },);

//endregion Score agreement
