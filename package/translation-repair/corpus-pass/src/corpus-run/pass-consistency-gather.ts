import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type {
  JsonSchemaResponseFormat,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  gatherStageVoices,
  type HeardVoice,
  type StageRosterRepeatError,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ConsistencyCacheValue, } from './pass-consistency-cache.ts';

//region Pass consistency gather
// ONE QUESTION PUT TO A BENCH THROUGH THE CACHE, the way every stage of the pass
// asks: a gather that reached its quorum is stored under the question's key and
// read back on a resume; one that did not is not stored, since a short bench
// may come back. The kinds of answer differ in the shape of a voice, so the
// caller says how to read a stored record and how to build one.

/**
 What a stored gather holds of every kind: the quorum, the voices with their
 seats and what the gather reported.
 */
type StoredGather = {
  /**
   Voices the gather closed on.
   */
  readonly quorum: number;

  /**
   Every voice heard, with its seat.
   */
  readonly voices: readonly { readonly seat: string; }[];

  /**
   What the gather reported.
   */
  readonly findings: readonly string[];
};

/**
 Whether the voices of a stored gather are ones a fresh gather over this bench
 could have heard: each from a seat of the bench and each seat once, which
 also bounds them by the bench's size.

 THE QUORUM IS NOT THIS CHECK'S QUESTION. A stored record has already passed
 the cache's own check (`pass-consistency-cache.ts`: a quorum of one at least,
 and no fewer voices than it), and no bound on the quorum can be sharper than
 that: a gather closes on one voice when the bench seats one or two, or when
 the run withheld all but one or two of its seats (the key names the whole
 bench, not the seats left), so a floor of two would ask again a gather this
 pass made and spent paid calls on.

 @param record - stored gather

 @param modelIds - bench asked

 @returns Whether it can be counted again as the answer to this question

 @example
 ```ts
 const fits = recordFits({ record, modelIds, },);
 ```
 */
export function recordFits(
  {
    record,
    modelIds,
  }: {
    readonly record: StoredGather;
    readonly modelIds: readonly RosterModelId[];
  },
): boolean {
  /**
   Seats heard.
   */
  const seats = record.voices
    .map(function seatOf(voice,): string {
      return voice.seat;
    },);
  return (new Set(seats,).size === seats.length)
    && seats.every(function isSeated(seat,): boolean {
      return modelIds.some(function isSeat(modelId,): boolean {
        return modelId === seat;
      },);
    },);
}

/**
 Asks a bench one question, or reads its earlier answer.

 @param client - provider client

 @param modelIds - bench asked, in roster order

 @param messages - sheet every seat reads

 @param signal - entry deadline and caller abort

 @param exchangeTimeoutMs - per-call bound

 @param maxAnswerChars - bound on one answer, when the caller knows how large its input was

 @param responseFormat - structured-output constraint

 @param validate - client-side guard, so a reply that cannot be counted is asked again

 @param stage - stage label for logging and findings

 @param graceMs - window a straggler gets after quorum

 @param cache - store an earlier answer is read from and a settled one kept in

 @param key - question's identity

 @param readStored - reads a stored value as this question's record, as a list of zero or one: none for another kind

 @param recordOf - builds the record of a fresh gather from its voices in roster order

 @param l - logger the stage writes to

 @returns Record and whether it was read back

 @throws {@link StageRosterRepeatError} when the bench seats one model twice

 @example
 ```ts
 const { record, resumed, } = await gatherThroughCache({ client, modelIds, messages, signal, exchangeTimeoutMs, responseFormat, validate, stage, graceMs, cache, key, readStored, recordOf, l, },);
 ```
 */
export async function gatherThroughCache<WireT, RecordT extends StoredGather,>(
  {
    client,
    modelIds,
    messages,
    signal,
    exchangeTimeoutMs,
    maxAnswerChars,
    responseFormat,
    validate,
    stage,
    graceMs,
    cache,
    key,
    readStored,
    recordOf,
    l,
  }: {
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly messages: readonly ChatMessage[];
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly maxAnswerChars?: number;
    readonly responseFormat: JsonSchemaResponseFormat;
    readonly validate: (value: unknown,) => value is WireT;
    readonly stage: string;
    readonly graceMs: number;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly key: string;
    readonly readStored: (value: ConsistencyCacheValue,) => readonly RecordT[];
    readonly recordOf: (
      gathered: {
        readonly quorum: number;
        readonly voices: readonly HeardVoice<WireT>[];
        readonly findings: readonly string[];
      },
    ) => RecordT;
    readonly l: Logger;
  },
): Promise<{
  readonly record: RecordT;
  readonly resumed: boolean;
}> {
  /**
   Logger pre-tagged with this function's name.
   */
  const gl = tagged({
    tag: gatherThroughCache.name,
    l,
  },);
  /**
   The value an earlier run stored under this question's key.
   */
  const value = cache.resumed
    .get(key,);
  /**
   The stored answer as this question's record, none when the value is another kind.
   */
  const [stored,] = (value === undefined) ? [] : readStored(value,);
  /**
   The stored answer, when it is one this bench could have given.
   */
  const cached = ((stored !== undefined) && recordFits({
    record: stored,
    modelIds,
  },)) ? stored : undefined;
  if ((value !== undefined) && (cached === undefined)) {
    gl.warn(
      `${stage}: the stored answer cannot be an answer of this bench (another kind, a seat heard twice, or a seat the bench does not have), so the question is asked again`,
    );
  }
  if (cached !== undefined) {
    gl.info(
      `${stage}: resumed ${String(cached.voices
        .length,)} ${
        wordForCount({
          count: cached.voices
            .length,
          one: 'voice',
          many: 'voices',
        },)
      }`,
    );
    return {
      record: cached,
      resumed: true,
    };
  }
  /**
   Quorum-bounded replies, a reply that cannot be counted asked again.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds,
    messages,
    signal,
    exchangeTimeoutMs,
    ...((maxAnswerChars === undefined) ? {} : { maxAnswerChars, }),
    responseFormat,
    validate,
    stage,
    l: gl,
    graceMs,
  },);
  /**
   What this gather comes to, voices in roster order so a stored record does not depend on arrival.
   */
  const record = recordOf({
    quorum: gather.quorum,
    voices: gather.voices
      .toSorted(function byRoster(
        left,
        right,
      ): number {
        return modelIds.indexOf(left.modelId,) - modelIds.indexOf(right.modelId,);
      },),
    findings: gather.findings,
  },);
  // A GATHER THAT MISSED ITS QUORUM IS NOT AN ANSWER and is not stored: the
  // bench was short, not undecided, and storing that would keep the page
  // unsettled on every resume.
  if (gather.quorumMet) {
    try {
      await cache.persist({
        key,
        serialized: JSON.stringify(record,),
      },);
    }
    catch (error) {
      // A STORE THAT REFUSES COSTS THE RESUME, NOT THE PAGE: the voices in
      // hand are counted either way.
      gl.warn(`${stage}: the answer could not be stored (${exchangeFailureLogText({ error, },)}); it is counted all the same`,);
    }
  }
  return {
    record,
    resumed: false,
  };
}

//endregion Pass consistency gather
