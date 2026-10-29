import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import { endsTheRun, } from '@monochromatic-dev/translation-repair-stage/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { ImageReading, } from './image-reading-stage.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Image reading contain
// WHAT EVERY READER OF ONE PICTURE MADE OF IT, once their exchanges have settled: a reader that failed outright is
// contained as an unavailable reading, and the one failure that is not a reader's own, every provider dry, ends the run.
//
// SPLIT OUT OF `image-reading-pair.ts` at its line budget, with the block that stood there.

/**
 One reader's reading, labelled by the model that made it.
 */
export type LabelledReading = {
  /**
   Model that read the picture.
   */
  readonly modelId: RosterModelId;

  /**
   What it made of the picture.
   */
  readonly reading: ImageReading;
};

/**
 Reads every reader's settled exchange into a labelled reading, a reader that failed outright carrying its failure
 rather than taking the others down with it.

 A READING IS EVIDENCE, NEVER A GATE, so a reader that throws costs its own reading and nothing else. Measured on
 `wangzihao980` before this changed: one reader looped on `picture1.webp`, the client's runaway guard ended the call,
 `Promise.all` rejected, and the rejection travelled through `readDocumentPictures` into `settleEntry`. The entry
 finished `status=ERROR` with `processed=0`, so BOTH LANES were lost to a transcription nobody needed. Nothing downstream
 reads a picture as required, which is what makes containing it here correct rather than merely convenient.

 EVERY PROVIDER DRY IS NOT A READER THAT FAILED OUTRIGHT, and ends the run. A router with nothing left to buy serves no
 other reader and no other picture either, so it is raised here, after every reader has settled, and a client built
 without the pass guard still ends the run instead of recording each reader as failed and going on with no reader
 available.

 @param settled - how each reader's exchange ended, in roster order

 @param readerModelIds - roster the exchanges were asked of, in the same order

 @param assetName - picture being read, for the lines

 @param signal - caller abort, under which a failed reader's line says the caller stopped the reading

 @param l - logger the failed readers' lines are written on

 @returns One labelled reading per reader, in roster order

 @throws Whatever refusal ends the run, when a reader met it

 @example
 ```ts
 const outcomes = containFailedReaders({ settled, readerModelIds, assetName, signal, l, },);
 ```
 */
export function containFailedReaders(
  {
    settled,
    readerModelIds,
    assetName,
    signal,
    l,
  }: {
    readonly settled: readonly PromiseSettledResult<LabelledReading>[];
    readonly readerModelIds: readonly RosterModelId[];
    readonly assetName: string;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): readonly LabelledReading[] {
  for (const result of settled) {
    if ((result.status === 'rejected') && endsTheRun({ error: result.reason, },))
      throw result.reason;
  }

  return settled.map(function contained(
    result,
    index,
  ): LabelledReading {
    if (result.status === 'fulfilled') {
      return result.value;
    }

    /**
     Reader whose exchange threw, taken by position because a rejected
     settlement carries no label of its own. Present at every position:
     `settled` is the roster mapped one ask per slot, so the settlement at
     `index` is the ask the slot at `index` made.
     */
    const modelId = readerModelIds[index];
    if (modelId === undefined)
      throw new Error(
        `unreachable: readImagePair settled ${String(settled.length,)} ${
          wordForCount({
            count: settled.length,
            one: 'reader',
            many: 'readers',
          },)
        } for ${assetName} and then could not name the one at index ${
          String(index,)
        }, though each settlement is the ask the roster slot at its own index made`,
      );
    /**
     The reader's own failure as a log line carries it: a refusal with its
     status and the provider's words.
     */
    const failure = exchangeFailureLogText({ error: result.reason, },);
    // UNDER THE CALLER'S STOP the reader's own failure is still written, here,
    // since the stop is raised once every settlement is read and no catch past
    // it sees this one; the caller's own reason is the stop itself and is not
    // written. Written from the settlements rather than inside each reader as
    // it ends, so the lines follow roster order whichever reader ended first
    // (`mistake-prevention.md`, "Output decided by arrival order"), and a
    // failure that ended before the stop arrived is written as well.
    if (signal.aborted && (result.reason !== signal.reason))
      l.warn(`${assetName}: ${modelId} failed outright (${failure}), and the caller stopped the reading`,);
    if (!signal.aborted)
      l.warn(`${assetName}: ${modelId} failed outright, so it contributes no reading (${failure})`,);
    return {
      modelId,
      reading: {
        kind: 'unavailable',
        reason: 'reader-failed',
      },
    };
  },);
}

//endregion Image reading contain
