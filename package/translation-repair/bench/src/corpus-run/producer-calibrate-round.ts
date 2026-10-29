import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { producerModelIds, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { SelectionRound, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { runTranslateStage, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { BenchSlice, } from './bench-sample.ts';
import { RUN_PER_CALL_TIMEOUT_MS, } from '@monochromatic-dev/translation-repair-run/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Producer calibrate round
// ONE SLICE OF THE PRODUCER CALIBRATION, every model writing and every model
// judging, and what it leaves behind for the standing to be summed from.
//
// SPLIT OUT OF `producer-calibrate.ts` when that file became wiring only. The
// client is handed in, so the round buys nothing the caller did not give it
// and a case can script every reply.

/**
 What one slice produced, with everyone who wrote on it.

 THE AUTHORS ARE CARRIED APART FROM THE ROUND, because a standing is summed
 from ballots and a slate can hold a candidate no ballot ever named. Without
 this list a model its provider refused and a model whose wording every peer
 proposed word for word are both simply absent from the table, and the two
 call for opposite readings.

 @example
 ```ts
 const { round, authors, } = await runCalibrationRound({ slice, roster, client, },);
 ```
 */
export type ProducerCalibrateRound = {
  /**
   Slate and ballots, in the shape a standing is summed from.
   */
  readonly round: SelectionRound;

  /**
   Every model holding a stake in any candidate on that slate, including one
   whose text was collapsed into an identical peer's.
   */
  readonly authors: readonly RosterModelId[];
};

/**
 Runs one slice with every model writing and every model judging.

 @param slice - passage to translate

 @param roster - every model that writes and judges it

 @param client - client this slice's calls go through, built by the caller so
 a case supplies one that answers from a script

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Slate, ballots and authors of that round

 @example
 ```ts
 const round = await runCalibrationRound({ slice, roster, client, stageSettings, },);
 ```
 */
export async function runCalibrationRound(
  {
    slice,
    roster,
    client,
    stageSettings,
  }: {
    readonly slice: BenchSlice;
    readonly roster: readonly RosterModelId[];
    readonly client: ModelClient;
    readonly stageSettings: StageSettings;
  },
): Promise<ProducerCalibrateRound> {
  /**
   Logger tagged for this slice.
   */
  const l = tagged({ tag: `calibrate-${slice.entryId}-${String(slice.index,)}`, },);

  /**
   What the stage decided, with every seat filled.
   */
  const result = await runTranslateStage({
    client,
    translatorModelIds: roster,
    judgeModelIds: roster,
    sourceText: slice.sourceText,
    incumbentText: slice.incumbentText,
    // Every drawn slice comes from a pair the archive HAS translated, so there
    // is always something to fall back on.
    incumbentKind: 'present',
    lineStructured: slice.lineStructured,
    signal: new AbortController().signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    stageSettings,
  },);

  /**
   Provenance of every candidate the judges were shown, in slate order.
   */
  const producers = result
    .slate
    .map(function toProducer(entry,) {
      return entry.producer;
    },);

  return {
    round: {
      producers,
      ballots: result.ballots,
    },
    authors: producers.flatMap(function stakeholders(producer,): readonly RosterModelId[] {
      return producerModelIds(producer,);
    },),
  };
}

//endregion Producer calibrate round
