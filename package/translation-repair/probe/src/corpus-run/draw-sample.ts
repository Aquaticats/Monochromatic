import { readCorpusFile, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { readPoolPolicy, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  type CommandLineOf,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run/ts';
import { drawGradingSample, } from './draw-sample-run.ts';

//region Draw sample
// Draws the stratified precision sample over every settled artifact and writes
// the grading sheets outside the repo: `draw-sample-pool.ts` reads the pool,
// `draw-sample-bands.ts` says how it divides, `draw-sample-sheets.ts` writes
// the three files and `draw-sample-run.ts` is the procedure.

/**
 Draws over the runs directory and the pool's generation policy of the
 environment, and the pinned corpus.

 @param line - the draw's command line, read whole by `reportingRefusals`

 @example
 ```ts
 await drawSample({ line, },);
 ```
 */
async function drawSample({ line, }: { readonly line: CommandLineOf<'draw-sample'>; },): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
  await drawGradingSample({
    line,
    runsDir: await resolveRunsDir(),
    pin,
    readSource: readCorpusFile,
    policy: readPoolPolicy({ env: process.env, },),
    // The checkout this command's build stands in, where a required commit is
    // resolved and placed; the pool took it from its own location until
    // 2026-10-10.
    repository: import.meta.dirname,
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'draw-sample',
    argv: process.argv,
    env: process.env,
    run: drawSample,
  },);

//endregion Draw sample
