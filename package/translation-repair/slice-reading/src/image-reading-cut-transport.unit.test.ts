/**
 Tests for a cut picture reading whose re-ask of another model fails (ledger B1200): a transport failure of one model
 moves the order on to the next, as the provider's own re-ask does, and the claim on a model the router refused before
 any exchange is released.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { NoProviderForModelError, } from '@monochromatic-dev/translation-repair-markup';
import {
  type ChatTextReply,
  type ChatTextRequest,
  createReaskClaims,
  type ModelClient,
  type ReaskClaims,
} from '@monochromatic-dev/translation-repair-provider/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { readPastRefusal, } from '../dist/final/node/index.mjs';
import {
  SEAT_BEDROCK_ONLY_VISION_UNSEATED,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';

//region Cut reader whose stand-in fails tests

/**
 The reader whose reply is cut.
 */
const READER: RosterModelId = SEAT_SYNTHETIC_VISION_WITHHELD;

/**
 The model the order asks first.
 */
const FIRST_STAND_IN: RosterModelId = SEAT_HYPER_TEXT_BEDROCK;

/**
 The model the order asks second.
 */
const SECOND_STAND_IN: RosterModelId = SEAT_BEDROCK_ONLY_VISION_UNSEATED;

/**
 Reply the completion cap ended while the model was still reasoning.
 */
const CUT: ChatTextReply = { text: '', finishReason: 'length', servedBy: 'neuralwatt', };

/**
 What a model that reads returns.
 */
const READING = '走失猫咪 Mittens，虎斑，请电 555 0134。';

/**
 What the cut reader's re-ask did.
 */
type CutRun = {
  /**
   Kind of the reading the reader came to.
   */
  readonly kind: string;

  /**
   Models asked, in the order the asks arrived.
   */
  readonly asks: readonly RosterModelId[];

  /**
   Record of the models asked for the readers of the picture, as the run left it.
   */
  readonly claims: ReaskClaims;
};

/**
 Reads the picture with a reader whose reply is cut and whose stand-ins fail as scripted.

 @param refused - model the router refuses before any exchange

 @param failing - model whose exchange fails in transport

 @returns What the reader came to, the models asked and the record of the models taken

 @example
 ```ts
 const run = await cutReaderWithFailures({ refused: undefined, failing: FIRST_STAND_IN, },);
 ```
 */
async function cutReaderWithFailures(
  {
    refused,
    failing,
  }: {
    readonly refused?: RosterModelId;
    readonly failing?: RosterModelId;
  },
): Promise<CutRun> {
  /**
   Models asked, in the order the asks arrived.
   */
  const asks: RosterModelId[] = [];

  /**
   Client whose reader is cut and whose every other model reads, bar the ones scripted to fail.
   */
  const client: ModelClient = {
    chatText: async (request,): Promise<ChatTextReply> => {
      /**
       Request as this package's own contract describes it.
       */
      const { modelId, } = request as ChatTextRequest;
      asks.push(modelId,);
      await Promise.resolve();
      if (modelId === READER)
        return CUT;
      if (modelId === refused)
        throw new NoProviderForModelError({ modelId, reason: 'the run withheld this seat', },);
      if (modelId === failing)
        throw new Error('connection reset while the stand-in was answering',);
      return { text: READING, servedBy: 'hyper', };
    },
    chatJson: async () => {
      throw new Error('chatJson unused by the reading stage',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the reading stage',);
    },
  };

  /**
   Record the readers of the picture share.
   */
  const claims = createReaskClaims();

  /**
   What the reader made of the picture.
   */
  const reading = await readPastRefusal({
    client,
    modelId: READER,
    bytes: new Uint8Array(64,),
    assetName: 'noticeboard.webp',
    benchModelIds: [READER,],
    claims,
    reask: 'other-model',
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: tagged({ tag: 'cut-transport-test', },),
  },);
  return {
    kind: reading.kind,
    asks,
    claims,
  };
}

await describe({
  name: `${readPastRefusal.name} (a stand-in that fails)`,
  concurrency: 1,
  children: [
    it({
      name: 'MOVES ON TO THE NEXT MODEL WHEN THE ONE ASKED FAILS IN TRANSPORT (ledger B1200), and returns the reading of '
        + 'the model that took it, never asking the cut reader again',
      fn: async () => {
        const run = await cutReaderWithFailures({ failing: FIRST_STAND_IN, },);
        expect(run.kind,).toBe('read',);
        expect(run.asks,).toEqual([READER, FIRST_STAND_IN, SECOND_STAND_IN,],);
      },
    },),

    it({
      name: 'RELEASES THE CLAIM OF A MODEL THE ROUTER REFUSED BEFORE ANY EXCHANGE (ledger B1200) and keeps the claim of one '
        + 'whose exchange failed, since only the second was asked',
      fn: async () => {
        const run = await cutReaderWithFailures({ refused: FIRST_STAND_IN, failing: SECOND_STAND_IN, },);
        expect(run.kind,).toBe('read',);
        expect(run.claims.list(),).not.toContain(FIRST_STAND_IN,);
        expect(run.claims.list(),).toContain(SECOND_STAND_IN,);
      },
    },),
  ],
},);

//endregion Cut reader whose stand-in fails tests
