import { createHash, } from 'node:crypto';

import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import {
  CONSISTENCY_CACHE_VERSION,
  type ConsistencyChoiceVoice,
  type ConsistencyVoice,
  isConsistencyBallot,
  isConsistencyChoiceBallot,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  hashContent,
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  openNamespacedCache,
  PAGE_CONSISTENCY_NAMESPACE,
} from '@monochromatic-dev/translation-repair-slice-cache/ts';

//region Pass consistency cache
// THE ANSWERS OF THE PAGE CONSISTENCY PASS, kept so a resumed page counts the
// same answers and lands on the same page. Four kinds are stored in the one
// cache, each under a key that names its kind: the ballots of the first round,
// the choices of a run-off, the paragraphs the writers wrote and the choices
// among a place's candidates. What is stored is what the bench or the writers
// said, with the quorum the gather closed on; the counting against the English
// is not stored, since it is deterministic and moves with the English it reads.

/**
 What one settled first-round gather stores. It has no `kind`, which is what
 tells it from the other three.

 @example
 ```ts
 const record: ConsistencyRecord = { quorum: 2, voices: [], findings: [], };
 ```
 */
export type ConsistencyRecord = {
  /**
   Voices the gather closed on.
   */
  readonly quorum: number;

  /**
   Every ballot heard, with its seat.
   */
  readonly voices: readonly ConsistencyVoice[];

  /**
   What the gather reported: a short bench, a seat that stayed unheard.
   */
  readonly findings: readonly string[];
};

/**
 What one settled gather of choices stores: a run-off's, or a place's choice
 among its candidates.

 @example
 ```ts
 const record: ConsistencyChoiceRecord = { kind: 'choice', quorum: 2, voices: [], findings: [], };
 ```
 */
export type ConsistencyChoiceRecord = {
  /**
   Tells this record from the others.
   */
  readonly kind: 'choice';

  /**
   Voices the gather closed on.
   */
  readonly quorum: number;

  /**
   Every choice heard, with its seat.
   */
  readonly voices: readonly ConsistencyChoiceVoice[];

  /**
   What the gather reported.
   */
  readonly findings: readonly string[];
};

/**
 One writer's paragraph as stored: the reply as the model wrote it.

 @example
 ```ts
 const voice: ConsistencyWriterVoice = { seat: 'glm-5.3', translation: 'The cat naps.', };
 ```
 */
export type ConsistencyWriterVoice = {
  /**
   Seat that answered.
   */
  readonly seat: string;

  /**
   The paragraph the seat wrote.
   */
  readonly translation: string;
};

/**
 What one settled gather of writers stores.

 @example
 ```ts
 const record: ConsistencyWritersRecord = { kind: 'writers', quorum: 2, voices: [], findings: [], };
 ```
 */
export type ConsistencyWritersRecord = {
  /**
   Tells this record from the others.
   */
  readonly kind: 'writers';

  /**
   Voices the gather closed on.
   */
  readonly quorum: number;

  /**
   Every paragraph heard, with its seat.
   */
  readonly voices: readonly ConsistencyWriterVoice[];

  /**
   What the gather reported.
   */
  readonly findings: readonly string[];
};

/**
 Anything the cache holds.
 */
export type ConsistencyCacheValue = ConsistencyRecord | ConsistencyChoiceRecord | ConsistencyWritersRecord;

/**
 Whether a stored value carries the gather's quorum and findings, with a
 quorum that no gather closes below one and no fewer voices than it.

 @param value - parsed stored record

 @returns Whether the quorum, the voice list and the findings are the shapes a gather stores

 @example
 ```ts
 carriesGather({ value: { quorum: 2, voices: [], findings: [], }, },);
 ```
 */
function carriesGather({ value, }: { readonly value: Readonly<Record<string, unknown>>; },): boolean {
  return ((typeof value.quorum) === 'number')
    && Number.isInteger(value.quorum,)
    && (value.quorum >= 1)
    && isJsonArray(value.voices,)
    && (value.voices
      .length
      >= value.quorum)
    && isJsonArray(value.findings,)
    && value.findings
      .every(function isFinding(finding,): boolean {
        return (typeof finding) === 'string';
      },);
}

/**
 Whether a stored value is a gather's record.

 A STORED RECORD IS AN OUTSIDE READ, so it must be one the persistence rule
 writes: a gather is stored only once it reached its quorum, and a quorum is
 at least one (a negative one is refused by the artifact reader, which would
 leave the run's own artifact unreadable), so a record of a quorum below one,
 or of fewer voices than its quorum, is no gather's and is asked again. A
 quorum beyond what JSON carries exactly is refused by the voice count.

 @param value - parsed stored record

 @returns Whether it can be counted again

 @example
 ```ts
 isConsistencyCacheValue({ quorum: 2, voices: [], findings: [], },);
 ```
 */
function isConsistencyCacheValue(value: unknown,): value is ConsistencyCacheValue {
  if (!isJsonRecord(value,))
    return false;
  if (!carriesGather({ value, },))
    return false;
  /**
   The voices as the JSON holds them.
   */
  const voices = isJsonArray(value.voices,) ? value.voices : [];
  if (value.kind === undefined) {
    return voices.every(function isVoice(voice,): boolean {
      return isJsonRecord(voice,) && ((typeof voice.seat) === 'string')
        && isConsistencyBallot(voice,);
    },);
  }
  if (value.kind === 'choice') {
    return voices.every(function isVoice(voice,): boolean {
      return isJsonRecord(voice,) && ((typeof voice.seat) === 'string')
        && isConsistencyChoiceBallot(voice,);
    },);
  }
  return (value.kind === 'writers') && voices.every(function isVoice(voice,): boolean {
    return isJsonRecord(voice,) && ((typeof voice.seat) === 'string')
      && ((typeof voice.translation) === 'string');
  },);
}

/**
 Opens an entry's page consistency cache.

 @param dir - per-entry slice-cache directory

 @param generation - digest of the built pipeline this pass runs

 @returns Cache resuming settled gathers and persisting new ones

 @example
 ```ts
 const cache = await openConsistencyCache({ dir: entryCacheDir, generation, },);
 ```
 */
export async function openConsistencyCache(
  {
    dir,
    generation,
  }: {
    readonly dir: string;
    readonly generation: string;
  },
): Promise<SliceCache<ConsistencyCacheValue>> {
  return await openNamespacedCache({
    dir,
    generation,
    namespace: PAGE_CONSISTENCY_NAMESPACE,
    isValue: isConsistencyCacheValue,
  },);
}

/**
 Which of the pass's questions a key names.
 */
export type ConsistencyStage = 'ballots' | 'run-off' | 'writers' | 'choice' | 'choice-run-off';

/**
 Names one question: the version, the stage, the whole sheet the bench or the
 writers are shown (which holds the wording, every place's paragraph and
 English, the declared identity and the cited pages) and the bench that answers,
 as one JSON value so no two questions share a key. An entry that records a
 death adds the death and the names its tense candidates read, since the sheet
 then shows a block built from them (the lead's decision of 2026-10-12); an
 entry that records none keys as before.

 @param stage - question's kind

 @param messages - sheet

 @param modelIds - bench asked

 @param lifeEnded - whether the entry records a death, folded in only when it does

 @param subjectNames - names the translation declares for its subject, folded in
 with a recorded death, since the tense candidates read each as a subject

 @returns Cache key

 @example
 ```ts
 const key = consistencyKey({ stage: 'ballots', messages, modelIds, },);
 ```
 */
export function consistencyKey(
  {
    stage,
    messages,
    modelIds,
    lifeEnded,
    subjectNames,
  }: {
    readonly stage: ConsistencyStage;
    readonly messages: readonly ChatMessage[];
    readonly modelIds: readonly RosterModelId[];
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
  },
): string {
  return createHash('sha256',)
    .update(
    JSON.stringify({
      version: CONSISTENCY_CACHE_VERSION,
      stage,
      sheet: hashContent({ content: JSON.stringify(messages,), },),
      roster: modelIds,
      ...((lifeEnded === true)
        ? {
          tense: [
            'life-ended',
            subjectNames ?? [],
          ],
        }
        : {}),
    },),
    'utf8',
    )
    .digest('hex',);
}

//endregion Pass consistency cache
