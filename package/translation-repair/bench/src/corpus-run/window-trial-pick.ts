import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { TrialSlice, } from './window-trial-draw.ts';
import type { WindowTrialRow, } from './window-trial-ledger.ts';
import { runSliceArms, } from './window-trial-slice.ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region Window trial pick
// One drawn slice, with the difference between a slice that cannot be tried and
// a run that cannot continue.
//
// A SLICE CAN REFUSE FOR REASONS THAT ARE ITS OWN. One with no neighbouring
// section has no wide arm; one with no incumbent can have every judge decline.
// Both raise from the stage, and neither says anything about the trial.
//
// LETTING EITHER END THE WALK WOULD WEDGE THE RUN PERMANENTLY. The ledger
// records arms that completed, and a slice that refused completes none, so a
// resumed run redraws it, walks to it, and dies there again. Every restart would
// stop at the same slice and none would ever reach the slices behind it.

/**
 What one drawn slice yielded.

 @example
 ```ts
 const outcome: PickOutcome = { kind: 'refused', };
 ```
 */
export type PickOutcome = {
  /**
   Slice ran, whether or not it owed anything.
   */
  readonly kind: 'bought';

  /**
   Arms appended, empty when the ledger already held them.
   */
  readonly rows: readonly WindowTrialRow[];
} | {
  /**
   Slice refused, and the walk continues past it.
   */
  readonly kind: 'refused';
};

/**
 Buys one slice's arms, reporting a refusal rather than raising it.

 THE LIVE WINDOW CHECK IS NOT CAUGHT HERE, because it does not run here: it
 reads the witness after this returns, so its refusal propagates out of the
 walk on its own. A guard for it inside this catch would be a branch nothing
 reaches.

 @param client - injected model client

 @param slices - every prepared slice of this entry, for the window

 @param pick - slice to buy, with its class label

 @param entryId - entry it belongs to

 @param protocol - digest this run buys under

 @param ledgerPath - where completed arms are appended

 @param done - arms already bought, as keys

 @param models - translator and judge rosters

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - run logger

 @param sheetFacts - what the entry's preparation tells every judging sheet of
 the page, handed to the arms

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Arms bought, or the fact that this slice refused

 @example
 ```ts
 const outcome = await runPick({ client, slices, pick, ... },);
 ```
 */
export async function runPick(
  {
    client,
    slices,
    pick,
    entryId,
    protocol,
    ledgerPath,
    done,
    models,
    signal,
    perCallTimeoutMs,
    l,
    sheetFacts,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly slices: readonly ChunkPair[];
    readonly pick: TrialSlice;
    readonly entryId: string;
    readonly protocol: string;
    readonly ledgerPath: string;
    readonly done: ReadonlySet<string>;
    readonly models: {
      readonly translatorModelIds: readonly RosterModelId[];
      readonly judgeModelIds: readonly RosterModelId[];
    };
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly sheetFacts: PreparedSheetFacts;
    readonly stageSettings: StageSettings;
  }>,
): Promise<PickOutcome> {
  try {
    return {
      kind: 'bought',
      rows: await runSliceArms({
        client,
        slices,
        sliceIndex: pick.sliceIndex,
        sliceClass: pick.sliceClass,
        entryId,
        protocol,
        ledgerPath,
        done,
        models,
        signal,
        perCallTimeoutMs,
        l,
        sheetFacts,
        stageSettings,
      },),
    };
  }
  catch (error) {
    l.warn(
      `${entryId}/${String(pick.sliceIndex,)} (${pick.sliceClass}): refused, ${
        exchangeFailureLogText({ error, },)
      }`,
    );
    return { kind: 'refused', };
  }
}

//endregion Window trial pick
