import { reportingRefusals, } from './cli-refusal.ts';
import { reportSpendCost, } from './spend-report-run.ts';

//region Spend report
// WHAT A RUN COST, read back off its own log. Spends no quota and touches no
// model.
//
// A LOG WITH NO RECORDS AND A RUN THAT SPENT NOTHING ARE DIFFERENT ANSWERS, and
// this says which. Every log this project wrote before `spend-line.ts` landed
// carries no `SPEND` line at all, so silence is the ordinary case for the
// archive and reporting it as a zero total would be a lie about every one of
// them. `NOTHING RECORDED` names it.
//
// PRINTS IDS, COUNTS AND CREDITS. Never a passage: a run log holds unlicensed
// corpus wording, and this reads run logs.
//
// THIS FILE IS ONLY THE WIRING. `spend-report-run.ts` holds the procedure,
// `spend-report-print.ts` the printing of a priced tally and
// `spend-report-line.ts` the line of each seat.

if (import.meta.main)
  await reportingRefusals({
    what: 'spend-report',
    argv: process.argv,
    env: process.env,
    run: reportSpendCost,
  },);

//endregion Spend report
