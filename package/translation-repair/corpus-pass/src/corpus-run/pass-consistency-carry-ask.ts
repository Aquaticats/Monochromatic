import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  buildCarryChoiceMessages,
  buildCarryWriterMessages,
  type CarryChoiceSubject,
  type CarryWriterSubject,
  consistencyChoiceResponseFormat,
  usableChoiceFor,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { codePointLength, } from '@monochromatic-dev/translation-repair-markup/ts';
import {
  isTranslateReportWire,
  producedVolumeBound,
  type SliceCache,
  TRANSLATE_RESPONSE_FORMAT,
  type TranslateReportWire,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type {
  StageRosterRepeatError,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ConsistencyCacheValue,
  type ConsistencyChoiceRecord,
  consistencyKey,
  type ConsistencyWriterVoice,
  type ConsistencyWritersRecord,
} from './pass-consistency-cache.ts';
import { gatherThroughCache, } from './pass-consistency-gather.ts';
import {
  choiceRecordOf,
  storedChoices,
} from './pass-consistency-run-off.ts';

//region Pass consistency carry ask
// THE TWO QUESTIONS OF A CARRY, asked through the cache: the translate stage's
// writer bench is asked for the receiving place's paragraph, and the judge bench
// is asked to choose among the candidates. A gather that reached its quorum is
// stored under the question's key, so a resumed page carries the same words.

/**
 What a gather of writers produced.

 @example
 ```ts
 const asked: CarryWritersAsked = { kind: 'writers', quorum: 2, voices: [], findings: [], resumed: false, };
 ```
 */
export type CarryWritersAsked = ConsistencyWritersRecord & {
  /**
   Whether the record was read back from an earlier run rather than bought.
   */
  readonly resumed: boolean;
};

/**
 What a gather of choices among candidates produced.

 @example
 ```ts
 const asked: CarryChoiceAsked = { kind: 'choice', quorum: 2, voices: [], findings: [], resumed: false, };
 ```
 */
export type CarryChoiceAsked = ConsistencyChoiceRecord & {
  /**
   Whether the record was read back from an earlier run rather than bought.
   */
  readonly resumed: boolean;
};

/**
 Reads a stored value as a writers' record.

 @param value - what the cache holds under a key

 @returns Record as a list of one, none when the value is another kind

 @example
 ```ts
 const [record,] = storedWriters(value,);
 ```
 */
function storedWriters(value: ConsistencyCacheValue,): readonly ConsistencyWritersRecord[] {
  return (('kind' in value) && (value.kind === 'writers')) ? [value,] : [];
}

/**
 Builds the stored record of a writers' gather.

 @param quorum - voices the gather closed on

 @param voices - paragraphs heard, with their seats

 @param findings - what the gather reported

 @returns Record, as the cache keeps it

 @example
 ```ts
 const record = writersRecordOf({ quorum: 2, voices: [], findings: [], },);
 ```
 */
function writersRecordOf(
  {
    quorum,
    voices,
    findings,
  }: {
    readonly quorum: number;
    readonly voices: readonly {
      readonly modelId: RosterModelId;
      readonly value: TranslateReportWire;
    }[];
    readonly findings: readonly string[];
  },
): ConsistencyWritersRecord {
  return {
    kind: 'writers',
    quorum,
    voices: voices.map(function voiceOf({
      modelId,
      value,
    },): ConsistencyWriterVoice {
      return {
        seat: modelId,
        translation: value.translation,
      };
    },),
    findings,
  };
}

/**
 Asks the writer bench to carry the chosen rendering to one place, or reads
 its earlier answer.

 @param client - provider client

 @param modelIds - writer bench asked, in roster order

 @param subject - repeated wording, chosen place and receiving place

 @param cache - store an earlier answer is read from and a settled one kept in

 @param signal - entry deadline and caller abort

 @param exchangeTimeoutMs - per-call bound

 @param l - logger the stage writes to

 @param stageSettings - straggler windows the entry file read for this launch

 @returns Paragraphs heard, quorum the gather closed on and its findings

 @throws {@link StageRosterRepeatError} when the bench seats one model twice

 @example
 ```ts
 const asked = await askCarryWriters({ client, modelIds, subject, cache, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function askCarryWriters(
  {
    client,
    modelIds,
    subject,
    cache,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
  }: {
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly subject: CarryWriterSubject;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<CarryWritersAsked> {
  /**
   The sheet every writer reads.
   */
  const messages = buildCarryWriterMessages({ subject, },);
  /**
   The writers' record, and whether the cache supplied it.
   */
  const {
    record,
    resumed,
  } = await gatherThroughCache({
    client,
    modelIds,
    messages,
    signal,
    exchangeTimeoutMs,
    // A TRANSLATION IS ABOUT THREE TIMES ITS CHINESE SOURCE, so the original
    // blocks of the receiving places bound what a paragraph written for them can
    // legitimately be (`produced-volume-bound.ts`), counted in code points as the
    // bound's ratio was measured; places that share one original block count it once.
    maxAnswerChars: producedVolumeBound({
      materialChars: [
        ...new Set(subject.receiving
          .map(function paragraphOf({ view, },): string {
            return view.paragraph;
          },),),
      ]
        .reduce(
        function summed(
          total: number,
          paragraph,
        ): number {
          return total + codePointLength({ text: paragraph, },);
        },
        0,
        ),
    },),
    responseFormat: TRANSLATE_RESPONSE_FORMAT,
    validate: isTranslateReportWire,
    stage: 'page-consistency-writers',
    // A WRITER ROUND: every voice the window cuts is one fewer paragraph on the
    // slate, so it waits under the writer window.
    graceMs: stageSettings
      .grace
      .writerMs,
    cache,
    key: consistencyKey({
      stage: 'writers',
      messages,
      modelIds,
      ...((subject.lifeEnded === undefined) ? {} : { lifeEnded: subject.lifeEnded, }),
      ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
    },),
    readStored: storedWriters,
    recordOf: writersRecordOf,
    l,
  },);
  return {
    ...record,
    resumed,
  };
}

/**
 Asks the bench to choose among a place's candidates, or reads its earlier answer.

 @param client - provider client

 @param modelIds - judge bench asked, in roster order

 @param subject - repeated wording, receiving place and its candidates

 @param cache - store an earlier answer is read from and a settled one kept in

 @param signal - entry deadline and caller abort

 @param exchangeTimeoutMs - per-call bound

 @param l - logger the stage writes to

 @param stageSettings - straggler windows the entry file read for this launch

 @param runOff - whether this is the run-off among the finalists, whose sheet shows those candidates only and says a first round
 was split between them, so that it differs from the first ask's sheet even when every candidate is a finalist, and whose answer
 is kept under a key of its own; the first ask among all of them where absent

 @returns Choices heard, quorum the gather closed on and its findings

 @throws {@link StageRosterRepeatError} when the bench seats one model twice

 @example
 ```ts
 const asked = await askCarryChoice({ client, modelIds, subject, cache, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function askCarryChoice(
  {
    client,
    modelIds,
    subject,
    cache,
    signal,
    exchangeTimeoutMs,
    l,
    stageSettings,
    runOff = false,
  }: {
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly subject: CarryChoiceSubject;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
    readonly runOff?: boolean;
  },
): Promise<CarryChoiceAsked> {
  /**
   The sheet every judge reads; a run-off's rules say a first round split between its candidates, so its messages are never
   the first ask's, which a client that replays a repeated model and prompt would answer from the first round.
   */
  const messages = buildCarryChoiceMessages({
    subject,
    runOff,
  },);
  /**
   Labels the sheet shows, the only labels a reply may use.
   */
  const labels = subject.candidates
    .map(function labelOf({ label, },): string {
      return label;
    },);
  /**
   The choices' record, and whether the cache supplied it.
   */
  const {
    record,
    resumed,
  } = await gatherThroughCache({
    client,
    modelIds,
    messages,
    signal,
    exchangeTimeoutMs,
    responseFormat: consistencyChoiceResponseFormat({ labels, },),
    validate: usableChoiceFor({ labels, },),
    stage: runOff ? 'page-consistency-choice-run-off' : 'page-consistency-choice',
    graceMs: stageSettings
      .grace
      .roundMs,
    cache,
    key: consistencyKey({
      stage: runOff ? 'choice-run-off' : 'choice',
      messages,
      modelIds,
      ...((subject.lifeEnded === undefined) ? {} : { lifeEnded: subject.lifeEnded, }),
      ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
    },),
    readStored: storedChoices,
    recordOf: function recordFor(gathered,): ConsistencyChoiceRecord {
      return choiceRecordOf({
        ...gathered,
        labels,
      },);
    },
    l,
  },);
  return {
    ...record,
    resumed,
  };
}

//endregion Pass consistency carry ask
