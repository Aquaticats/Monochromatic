import {
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { readSettledRecipe, } from './settled-carve.ts';
import { reportSliceCensus, } from './slice-census-report.ts';

//region Slice census
// What the corpus looks like AFTER slicing, as a command: the wiring only. The
// report is `slice-census-report.ts`, which says what it answers; the pin and
// the runs directory are what only a real process has.

// Guarded so this runs only when INVOKED, never as an import side effect.
if (import.meta.main)
  await reportingRefusals({
    what: 'slice-census',
    argv: process.argv,
    env: process.env,
    run: function runSliceCensus(): Promise<void> {
      /**
       Corpus clone and commit this run reads, after any environment
       override, read inside the refusal boundary (ledger B302).
       */
      const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
      return reportSliceCensus({
        pin,
        resolveRuns: resolveRunsDir,
        readRecipe: readSettledRecipe,
      },);
    },
  },);

//endregion Slice census
