import { reportingRefusals, } from './cli-refusal.ts';
import { reportRunTiming, } from './run-timing-report-run.ts';

//region Run timing report
// WHERE A RUN'S HOURS WENT, read back off its own log. Spends no quota and
// touches no model.
//
// THE TIMING WORK OPENED ON A LOG THAT COULD NOT ANSWER THIS.
// `doc/audit/every-volume-guard-is-blind-to-one-model.md` had to bound the
// straggler cost from above, at the grace window times the number of cut
// events, and recorded that confirming it "needs the dispatch timestamps the
// run does not currently record". Two lines now record them, and this reads
// them back.
//
// A LOG WITH NO TIMING LINES AND A RUN THAT WAITED ON NOTHING ARE DIFFERENT
// ANSWERS, and this says which. Every log written before the timing work carries no
// round line and no `elapsed`, so silence is the ordinary case for the archive.
//
// PRINTS IDS, COUNTS AND DURATIONS. Never a passage: a run log holds
// unlicensed corpus wording, and this reads run logs.
//
// THIS FILE IS ONLY THE WIRING. `run-timing-report-run.ts` holds the procedure
// and `run-timing-report-print.ts` the printers.

if (import.meta.main)
  await reportingRefusals({
    what: 'run-timing-report',
    argv: process.argv,
    env: process.env,
    run: reportRunTiming,
  },);

//endregion Run timing report
