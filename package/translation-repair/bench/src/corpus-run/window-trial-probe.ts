import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import {
  createProbeRunClient,
  readHeadSha,
  readRunCorpusPinSetting,
  reportingRefusals,
  resolveRunsDir,
  RUN_PER_CALL_TIMEOUT_MS,
  RUN_ROSTER,
} from '@monochromatic-dev/translation-repair-run/ts';
import { runPick, } from './window-trial-pick.ts';
import { runWindowTrial, } from './window-trial-probe-run.ts';
import { readStageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Window trial probe
// The window trial, run end to end: the wiring only. It asks models and writes
// a trial ledger, so it SPENDS QUOTA: point `TRANSLATION_REPAIR_RUNS_DIR` at a
// throwaway directory. What the walk does is in `window-trial-probe-run.ts`
// with `window-trial-probe-draw.ts`, `window-trial-probe-tally.ts`,
// `window-trial-probe-check.ts` and `window-trial-probe-lines.ts`; the ledger,
// the draw, the per-slice arms and the report each have their own file and tests.

/**
 Logger the run writes under.
 */
const l = contextRoot({ tag: 'window-trial', },);

/**
 Runs the trial over the pinned corpus, with the runs directory the
 environment names and the head this checkout stands at.

 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Corpus clone and commit this run reads, after any environment
   override, read inside the refusal boundary (ledger B302).
   */
  const { pin, } = readRunCorpusPinSetting({ env: process.env, },);

  /**
   Run directory the ledger lives under.
   */
  const runsDir = await resolveRunsDir();
  return runWindowTrial({
    runsDir,
    headSha: await readHeadSha(),
    makeClient: createProbeRunClient,
    pin,
    listPeople: listCorpusPeople,
    readPage: readCorpusFile,
    pickSlice: runPick,
    roster: RUN_ROSTER,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings: readStageSettings({ env: process.env, },),
  },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'window-trial-probe',
    argv: process.argv,
    env: process.env,
    run: main,
  },);

//endregion Window trial probe
