import { reportCapCensus, } from './cap-census-report.ts';
import { reportingRefusals, } from './cli-refusal.ts';

//region Cap census
// The pre-launch reading of the completion caps (ledger P10), as a command:
// the wiring only. The report is `cap-census-report.ts`; what it reads is in
// `cap-census-walk.ts`, `cap-census-pass-run.ts` and `cap-census-read.ts`, and
// what it prints in `cap-census-print.ts`.

if (import.meta.main)
  await reportingRefusals({
    what: 'cap-census',
    argv: process.argv,
    env: process.env,
    run: reportCapCensus,
  },);

//endregion Cap census
