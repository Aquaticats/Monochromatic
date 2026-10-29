import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  buildConsistencyMessages,
  canonicalBallotOf,
  type ConsistencyBallot,
  type ConsistencyBallotWire,
  consistencyLabels,
  consistencyResponseFormat,
  type ConsistencySubject,
  type ConsistencyVoice,
  readConsistencyBallotWire,
  usableConsistencyBallotFor,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type {
  HeardVoice,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type ConsistencyCacheValue,
  consistencyKey,
  type ConsistencyRecord,
} from './pass-consistency-cache.ts';
import { gatherThroughCache, } from './pass-consistency-gather.ts';

//region Pass consistency ask
// ASKING THE BENCH ONCE ABOUT ONE REPEATED WORDING, or resuming what it said,
// through the one cache protocol every question of the pass shares
// (`pass-consistency-gather.ts`): the quorum helper is `gatherStageVoices`, the
// one the page title stage asks through, with its seats, quorum and retry
// rounds. A gather that reached its quorum is stored; one that did not is asked
// again on a resume, since a short bench may come back.

/**
 What asking produced: the ballots, the quorum and what the gather reported.

 @example
 ```ts
 const asked: ConsistencyAsked = { quorum: 2, voices: [], findings: [], resumed: false, };
 ```
 */
export type ConsistencyAsked = ConsistencyRecord & {
  /**
   Whether the record was read back from an earlier run rather than bought.
   */
  readonly resumed: boolean;
};

/**
 One heard ballot as the pass keeps it: the seat the roster asked and the ballot's
 own fields, nothing else a reply carried.

 The places the ballot names are stored as the sheet showed them (its choice, its kept places and the label of each quote), whatever
 form the judge wrote them in (the lead's ruling of 2026-10-15).

 @param seat - roster seat that answered, which no reply field can replace

 @param ballot - reply, of the right shape

 @param labels - labels the sheet showed

 @returns Ballot with its seat

 @example
 ```ts
 const voice = voiceOf({ seat: 'glm-5.3', ballot, labels: consistencyLabels({ count: 2, },), },);
 ```
 */
function voiceOf(
  {
    seat,
    ballot,
    labels,
  }: {
    readonly seat: string;
    readonly ballot: ConsistencyBallotWire;
    readonly labels: readonly string[];
  },
): ConsistencyVoice {
  /**
   The reply as the ballot the cache stores, which keeps the spelling it was stored under (ledger B762).
   */
  const heard = readConsistencyBallotWire({ wire: ballot, },);
  /**
   The ballot with the shown labels; the ballot as heard when it names none, which the gather's guard has already refused.
   */
  const [shown,] = canonicalBallotOf({
    ballot: heard,
    labels,
  },);
  /**
   The ballot to keep.
   */
  const kept = shown ?? heard;
  return {
    seat,
    reason: kept.reason,
    quotes: kept.quotes
      .map(function quoteOf(entry,): ConsistencyBallot['quotes'][number] {
        return {
          label: entry.label,
          quote: entry.quote,
          ...((entry.slot === undefined) ? {} : { slot: entry.slot, }),
        };
      },),
    choice: kept.choice,
    keep: [...kept.keep,],
  };
}

/**
 Reads a stored value as a first-round record, which is the one kind that has
 no `kind`.

 @param value - what the cache holds under a key

 @returns Record as a list of one, none when the value is another kind

 @example
 ```ts
 const [record,] = storedBallots(value,);
 ```
 */
function storedBallots(value: ConsistencyCacheValue,): readonly ConsistencyRecord[] {
  return ('kind' in value) ? [] : [value,];
}

/**
 Builds the record of a first-round gather, voices in roster order, each ballot
 copied field by field with the roster's seat (ledger B443).

 @param quorum - voices the gather closed on

 @param voices - ballots heard with their seats

 @param findings - what the gather reported

 @param labels - labels the sheet showed, which each ballot's places are stored as

 @returns Record, as the cache keeps it

 @example
 ```ts
 const record = ballotRecordOf({ quorum: 2, voices, findings: [], labels: consistencyLabels({ count: 2, },), },);
 ```
 */
function ballotRecordOf(
  {
    quorum,
    voices,
    findings,
    labels,
  }: {
    readonly quorum: number;
    readonly voices: readonly HeardVoice<ConsistencyBallotWire>[];
    readonly findings: readonly string[];
    readonly labels: readonly string[];
  },
): ConsistencyRecord {
  return {
    quorum,
    voices: voices.map(function toVoice({
      modelId,
      value,
    },): ConsistencyVoice {
      return voiceOf({
        seat: modelId,
        ballot: value,
        labels,
      },);
    },),
    findings,
  };
}

/**
 Asks the bench about one repeated wording, or reads its earlier answer.

 @param client - provider client

 @param modelIds - bench asked, in roster order

 @param subject - repeated wording and every place it stands

 @param cache - store an earlier answer is read from and a settled one kept in

 @param signal - entry deadline and caller abort

 @param exchangeTimeoutMs - per-call bound

 @param l - logger the stage writes to

 @param stageSettings - straggler windows the entry file read for this launch

 @returns Ballots heard, quorum the gather closed on and its findings

 @throws {@link StageRosterRepeatError} when the bench seats one model twice

 @example
 ```ts
 const asked = await askConsistencyBench({ client, modelIds, subject, cache, signal, exchangeTimeoutMs, l, stageSettings, },);
 ```
 */
export async function askConsistencyBench(
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
    readonly subject: ConsistencySubject;
    readonly cache: SliceCache<ConsistencyCacheValue>;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  },
): Promise<ConsistencyAsked> {
  /**
   Logger pre-tagged with this function's name.
   */
  const al = tagged({
    tag: askConsistencyBench.name,
    l,
  },);
  /**
   The sheet every judge reads.
   */
  const messages = buildConsistencyMessages({ subject, },);
  /**
   Labels the sheet showed, the only labels a reply may use.
   */
  const labels = consistencyLabels({ count: subject.occurrences
    .length, },);
  /**
   The ballots' record, and whether the cache supplied it.
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
    responseFormat: consistencyResponseFormat({ labels, },),
    // A reply that cannot be counted against the places the sheet showed is
    // asked again like one that is not a ballot.
    validate: usableConsistencyBallotFor({ labels, },),
    stage: 'page-consistency',
    graceMs: stageSettings
      .grace
      .roundMs,
    cache,
    key: consistencyKey({
      stage: 'ballots',
      messages,
      modelIds,
      ...((subject.lifeEnded === undefined) ? {} : { lifeEnded: subject.lifeEnded, }),
      ...((subject.subjectNames === undefined) ? {} : { subjectNames: subject.subjectNames, }),
    },),
    readStored: storedBallots,
    recordOf: function recordFor(gathered,): ConsistencyRecord {
      return ballotRecordOf({
        ...gathered,
        labels,
      },);
    },
    l: al,
  },);
  return {
    ...record,
    resumed,
  };
}

//endregion Pass consistency ask
