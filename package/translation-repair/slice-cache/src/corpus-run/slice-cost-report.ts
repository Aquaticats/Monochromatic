import { reportingRefusals, } from '@monochromatic-dev/translation-repair-run/ts';
import { reportSliceCost, } from './slice-cost-report-run.ts';

//region Slice cost report
// The per-slice cost report, as a command: the wiring only. What it reads and
// prints is in `slice-cost-report-run.ts`, `slice-cost-bands.ts`,
// `slice-cost-spread.ts` and `slice-cost-lanes.ts`.

if (import.meta.main)
  await reportingRefusals({
    what: 'slice-cost-report',
    argv: process.argv,
    env: process.env,
    run: reportSliceCost,
  },);

//endregion Slice cost report
