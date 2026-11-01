import {
  type ContestLedger,
  readContestLedger,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { STRAGGLER_GRACE_MS, } from './stage-round.ts';
import {
  readWriterGrace,
  type WriterGrace,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Stage settings
// WHAT A LAUNCH FIXES FOR EVERY STAGE IT RUNS, read once by the entry file and
// handed down as a value: how long each round waits on stragglers after
// quorum, and where judged contests are kept.
//
// EACH WAS READ DEEP IN THE PIPELINE until 2026-10-10. Every round read
// `TRANSLATION_REPAIR_STRAGGLER_GRACE_MS` in its own default, every writer
// gather read both window dials, and every recorded contest read
// `TRANSLATION_REPAIR_RUNS_DIR`, so a test chose its windows and its ledger by
// writing the process's environment, a command's dependence on them was
// visible nowhere in its signatures, and the editor calibration adopted its
// own window by writing the variable back for its rounds to find. The entry
// files now read them through the same readers, once, inside their refusal
// boundary, and every stage on the way to a round takes them as a required
// value.

/**
 Settings a launch fixes for every stage round it runs.

 @example
 ```ts
 const stageSettings: StageSettings = { grace: { writerMs: 180_000, roundMs: 120_000, source: 'built-in', }, contestLedger: { kind: 'unkept', }, };
 ```
 */
export type StageSettings = {
  /**
   Window a writer round waits on stragglers after quorum, the window every
   other round waits, and where the writers' came from.
   */
  readonly grace: WriterGrace;

  /**
   Where judged contests are kept.
   */
  readonly contestLedger: ContestLedger;

  /**
   The seat the editor bench's rotation starts at, in place of the prompt's own (`rotatedBench`, `stage-fanout-window.ts`): a whole
   number below the size of the bench. A fixture or a calibration that names which editor seats ship pins it, since the seats asked
   otherwise move whenever a sheet's wording does (n161: the house tense rule), and the cases that named them had to stop (n166,
   Z3). Absent in every production launch, where the prompt decides. A start that is not a whole number from zero to below the size of the
   bench is refused where the rotation is made, with `StageBenchStartError`, which names the value and the size of the bench.
   */
  readonly editorBenchStart?: number;
};

/**
 Reads the settings of a launch whose rounds run under the built-in round
 window unless a dial says otherwise: every command but the editor
 calibration, which reads its windows with its own round fallback.

 @param env - environment of the launch, which the entry file hands in

 @returns Both windows and the contest ledger

 @throws StatedRefusalError when either window dial is set to something
 that is not a whole number of milliseconds a timer can hold, the round dial
 named first when both are

 @example
 ```ts
 const stageSettings = readStageSettings({ env: process.env, },);
 ```
 */
export function readStageSettings(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): StageSettings {
  return {
    grace: readWriterGrace({
      env,
      roundFallback: STRAGGLER_GRACE_MS,
    },),
    contestLedger: readContestLedger({ env, },),
  };
}

//endregion Stage settings
