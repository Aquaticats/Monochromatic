/**
 Tests for the contest driver: which slices it asks about, what it resumes,
 and what it refuses to write down.

 WHAT IS UNDER TEST IS SPENDING. Every case here is about a call that must or
 must not be made, which is the one property a driver has that its stage does
 not: the stage answers whatever it is handed, and the driver decides what it
 is worth handing over.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact';
import type {
  BenchSeating,
  SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor';
import type {
  ArtifactComparisonRow,
  ArtifactDeliveryRow,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  type ArtifactContestSlice,
  type LaneContestOutcome,
  persistLaneContestOutcome,
} from '@monochromatic-dev/translation-repair-lane-contest';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createSyntheticClient,
  DEFAULT_RETRY_POLICY,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { contestDocumentLanes, } from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { ARCHIVE_NAP, catRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/two-lane-comparison-row.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  TENSE_EXPLANATION,
  tenseBlocksOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/tense-sheet.test-fixture.ts';
import {
  BENCH_UNREACHABLE,
  stopLandingHook,
  throwBenchUnreachable,
  UNASKED_CLIENT,
} from './stop-landing-hook.test-fixture.ts';

/**
 Roster of three, the smallest that can produce a two-to-one split.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Logger for the driver under test.
 */
const l = tagged({ tag: 'lane-contest-driver-test', },);

/**
 Exact caller abort reason used by driver guard case.
 */
const CONTEST_ABORT = new Error('caller stopped lane contest',);

/**
 Successful contest calls in flight and peak observed by fixture, the start
 positions in the order their calls finished, and, where a case supplies one,
 the gate the first slice's calls wait on until every call of the second
 slice has finished.
 */
type ContestConcurrency = {
  now: number;
  peak: number;
  started: number;
  readonly finished: number[];
  readonly secondFinished?: PromiseWithResolvers<undefined>;
};

/**
 Original of the slice the two lanes disagree about.
 */
const SOURCE_NAP = '猫猫在书店的阁楼里睡觉。';

/**
 Wording the repair lane left.
 */
const REPAIR_NAP = 'The cat naps in the bookshop attic.';

/**
 Wording the translate lane left.
 */
const TRANSLATE_NAP = 'The cat dozes in the attic of the bookshop.';

/**
 Source metadata repeating one identity as visible name and alias.
 */
const METADATA_SOURCE = '---\nname: 猫猫\ninfo:\n  alias: 猫猫\n---\n';

/**
 Archive metadata retaining entry id beside translated alias.
 */
const METADATA_ARCHIVE = '---\nname: CatEntry\ninfo:\n  alias: Maomao\n---\n';

/**
 Translate lane metadata preserving source identity relation.
 */
const METADATA_TRANSLATED = '---\nname: Maomao\ninfo:\n  alias: Maomao\n---\n';

/**
 Two-line verse original, each line its own unit.
 */
const VERSE_SOURCE = '猫在窗台上，\n狗在门口边。';

/**
 Archive's English for the verse, one line per line.
 */
const VERSE_ARCHIVE = 'A cat on the windowsill,\na dog beside the door.';

/**
 Repair lane's verse, keeping both lines.
 */
const VERSE_KEPT = 'A cat upon the windowsill,\na dog beside the door.';

/**
 Translate lane's verse, merging both lines into one.
 */
const VERSE_MERGED = 'A cat upon the windowsill and a dog beside the door.';

/**
 Builds one ledger row, which is where the driver reads the original.

 @param sliceIndex - slice this row names

 @param shippedText - wording this lane`s document carries

 @returns Version 2 delivery row

 @example
 ```ts
 const row = catLedgerRow({ sliceIndex: 0, shippedText: REPAIR_NAP, },);
 ```
 */
function catLedgerRow(
  {
    sliceIndex,
    shippedText,
  }: {
    readonly sliceIndex: number;
    readonly shippedText: string;
  },
): ArtifactDeliveryRow {
  return {
    sliceIndex,
    sourceText: SOURCE_NAP,
    incumbentKind: 'present',
    incumbentText: ARCHIVE_NAP,
    outcome: {
      kind: 'decided',
      acceptedText: shippedText,
    },
    shippedText,
    delivery: { kind: 'replacement-shipped', },
  };
}

/**
 Builds both lanes as version 2 rows over a list of wording pairs.

 @param pairs - wording each lane left, slice by slice

 @returns Projection the driver reads

 @example
 ```ts
 const projected = catProjection({ pairs: [[REPAIR_NAP, TRANSLATE_NAP,],], },);
 ```
 */
function catProjection(
  {
    pairs,
  }: {
    readonly pairs: readonly (readonly [
      string,
      string,
    ])[];
  },
): ProjectedLanes {
  return {
    delivery: {
      repair: pairs.map(function toRepairRow(
        pair,
        sliceIndex,
      ): ArtifactDeliveryRow {
        return catLedgerRow({
          sliceIndex,
          shippedText: pair[0],
        },);
      },),
      translate: pairs.map(function toTranslateRow(
        pair,
        sliceIndex,
      ): ArtifactDeliveryRow {
        return catLedgerRow({
          sliceIndex,
          shippedText: pair[1],
        },);
      },),
    },
    comparison: pairs.map(function toComparisonRow(
      pair,
      sliceIndex,
    ): ArtifactComparisonRow {
      return catRow({
        sliceIndex,
        repairText: pair[0],
        translateText: pair[1],
      },);
    },),
  };
}

/**
 Builds one archive-absent insertion the translate lane filled, beside the
 slices of `catProjection`: the repair lane has no translation there to mend,
 so its ledger row is `not-applicable` and ships nothing.

 @param translateText - wording the translate lane shipped at the insertion

 @returns Projection whose slice 0 is the insertion

 @example
 ```ts
 const projected = insertionProjection({ translateText: TRANSLATE_NAP, },);
 ```
 */
function insertionProjection(
  { translateText, }: { readonly translateText: string; },
): ProjectedLanes {
  return {
    delivery: {
      repair: [
        {
          sliceIndex: 0,
          sourceText: SOURCE_NAP,
          incumbentKind: 'absent',
          incumbentText: '',
          outcome: { kind: 'not-applicable', },
          shippedText: '',
          delivery: { kind: 'gap-remains', },
        },
      ],
      translate: [
        {
          sliceIndex: 0,
          sourceText: SOURCE_NAP,
          incumbentKind: 'absent',
          incumbentText: '',
          outcome: {
            kind: 'decided',
            acceptedText: translateText,
          },
          shippedText: translateText,
          delivery: { kind: 'replacement-shipped', },
        },
      ],
    },
    comparison: [
      {
        sliceIndex: 0,
        incumbentKind: 'absent',
        incumbentText: '',
        repairText: '',
        translateText,
        laneRelation: 'translate-only',
        repairOutcome: { kind: 'not-applicable', },
        translateOutcome: {
          kind: 'decided',
          acceptedText: translateText,
        },
        decisionComparison: {
          kind: 'not-comparable',
          undecidedLanes: ['repair',],
        },
        repairDelivery: { kind: 'gap-remains', },
        translateDelivery: { kind: 'replacement-shipped', },
      },
    ],
  };
}

/**
 Builds one front matter question repeated at every requested position.

 BUILT FROM WHOLE ROWS, like every other projection here, rather than cast
 from the few fields the driver was known to read (ledger B63): a cast
 fixture keeps compiling when the driver starts reading a field it lacks.

 @param sliceCount - document positions carrying the same question

 @returns Projection whose position-free keys match

 @example
 ```ts
 const projected = metadataProjection({ sliceCount: 2, },);
 ```
 */
function metadataProjection(
  { sliceCount, }: { readonly sliceCount: number; },
): ProjectedLanes {
  /**
   Cat projection over the metadata wordings, before the metadata original
   and archive replace the nap ones.
   */
  const base = catProjection({
    pairs: Array.from({ length: sliceCount, }, function samePair(): readonly [
      string,
      string,
    ] {
      return [
        METADATA_ARCHIVE,
        METADATA_TRANSLATED,
      ];
    },),
  },);

  /**
   Ledger row carrying the metadata original and archive.

   @param row - cat row at this position

   @returns Same row over the metadata slice
   */
  function metadataRow(row: ArtifactDeliveryRow,): ArtifactDeliveryRow {
    return {
      ...row,
      sourceText: METADATA_SOURCE,
      incumbentText: METADATA_ARCHIVE,
    };
  }
  return {
    delivery: {
      repair: base.delivery
        .repair
        .map(function toMetadata(row,): ArtifactDeliveryRow {
          return metadataRow(row,);
        },),
      translate: base.delivery
        .translate
        .map(function toMetadata(row,): ArtifactDeliveryRow {
          return metadataRow(row,);
        },),
    },
    comparison: base.comparison
      .map(function toMetadata(row,): ArtifactComparisonRow {
        return {
          ...row,
          incumbentText: METADATA_ARCHIVE,
        };
      },),
  };
}

/**
 Builds one verse slice whose repair lane keeps the lines and whose translate
 lane merges them.

 @returns Projection carrying the verse original and its archive English

 @example
 ```ts
 const projected = verseProjection();
 ```
 */
function verseProjection(): ProjectedLanes {
  /**
   Cat projection over the two verse wordings, before the verse original and
   archive replace the nap ones.
   */
  const base = catProjection({
    pairs: [
      [
        VERSE_KEPT,
        VERSE_MERGED,
      ],
    ],
  },);
  return {
    delivery: {
      repair: base.delivery
        .repair
        .map(function versed(row,): ArtifactDeliveryRow {
          return {
            ...row,
            sourceText: VERSE_SOURCE,
            incumbentText: VERSE_ARCHIVE,
          };
        },),
      translate: base.delivery
        .translate
        .map(function versed(row,): ArtifactDeliveryRow {
          return {
            ...row,
            sourceText: VERSE_SOURCE,
            incumbentText: VERSE_ARCHIVE,
          };
        },),
    },
    comparison: base.comparison
      .map(function versed(row,): ArtifactComparisonRow {
        return {
          ...row,
          incumbentText: VERSE_ARCHIVE,
        };
      },),
  };
}

/**
 Client and cache a case drives the driver with, plus what each recorded.
 */
type CatRig = {
  /**
   Calls the transport served, including provider retries.
   */
  readonly calls: readonly string[];

  /**
   Model calls admitted by contest stage before provider retries.
   */
  readonly admitted: number;

  /**
   Keys the driver asked to persist.
   */
  readonly persisted: readonly string[];

  /**
   What every judge was sent, messages joined as the judge reads them.
   */
  readonly sheets: readonly string[];

  /**
   Records the driver produced.
   */
  readonly slices: readonly ArtifactContestSlice[];
};

/**
 Drives the contest over one projection, counting what it spent.

 @param pairs - wording each lane left, slice by slice

 @param answering - whether the transport serves ballots or fails every call

 @param resumed - ballots an earlier run already bought

 @param projected - optional explicit lane projection

 @param frontMatterSlices - syntax-bearing positions

 @param lineStructuredSlices - positions the line-structure rule governs

 @param answerChoice - lane every scripted ballot selects

 @param overlap - most contested slices in flight

 @param activity - optional successful-call overlap instrument

 @param abortOnCall - one-based admitted call that aborts caller signal

 @param lifeEnded - whether the entry records a death, which shows the judges
 the tense candidates of each text on offer

 @param subjectNames - names the translation declares for its subject

 @returns What the driver called, persisted and recorded

 @example
 ```ts
 const rig = await drive({ pairs, answering: true, },);
 ```
 */
async function drive(
  {
    pairs,
    answering,
    resumed = new Map<string, LaneContestOutcome>(),
    projected,
    frontMatterSlices = new Set(),
    lineStructuredSlices = new Set(),
    answerChoice = 'translate',
    overlap = 1,
    activity,
    abortOnCall,
    lifeEnded,
    subjectNames,
  }: {
    readonly pairs: readonly (readonly [
      string,
      string,
    ])[];
    readonly answering: boolean;
    readonly resumed?: ReadonlyMap<string, LaneContestOutcome>;
    readonly projected?: ProjectedLanes;
    readonly frontMatterSlices?: ReadonlySet<number>;
    readonly lineStructuredSlices?: ReadonlySet<number>;
    readonly answerChoice?: LaneContestOutcome['choice'];
    readonly overlap?: number;
    readonly activity?: ContestConcurrency;
    readonly abortOnCall?: number;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
  },
): Promise<CatRig> {
  /**
   Calls the transport served.
   */
  const calls: string[] = [];

  /**
   Keys the driver asked to persist.
   */
  const persisted: string[] = [];

  /**
   What every judge was sent.
   */
  const sheets: string[] = [];

  /**
   Cache recording what it was asked to keep.
   */
  const cache: SliceCache<LaneContestOutcome> = {
    resumed,
    persist: async function record({ key, },): Promise<void> {
      persisted.push(key,);
    },
  };

  /**
   Client answering every judge the same way, or failing every call.
   */
  const inner = createSyntheticClient({
    apiKey: 'test-key',
    // THE PRODUCTION RETRY COUNT WITHOUT ITS WAITS (ledger T5): the failing
    // fixture answered 500 and slept through the real backoff, 27.4 s a run.
    retryPolicy: {
      limit: DEFAULT_RETRY_POLICY.limit,
      baseMs: 1,
    },
    transport: async function cannedTransport(exchange,) {
      calls.push(exchange.label,);
      if (!answering) {
        return {
          status: 500,
          bodyText: 'the bookshop is closed',
        };
      }
      return {
        status: 200,
        bodyText: `data: ${
          JSON.stringify({
            choices: [
              {
                index: 0,
                delta: {
                  content: JSON.stringify({
                    verdict: answerChoice,
                    unsupported: [],
                    dropped: [],
                    assessment: 'the original supports it',
                  },),
                },
              },
            ],
          },)
        }\n\ndata: [DONE]\n\n`,
      };
    },
  },);

  /**
   Caller signal a fixture may abort after enough contest calls were admitted.
   */
  const controller = new AbortController();

  /**
   Calls admitted at client boundary, outside provider slot limiting so it
   measures slices admitted by the driver rather than transport concurrency.
   */
  const admitted = { count: 0, };

  /**
   Client-level fixture outside provider slot limiting.
   */
  const client: ModelClient = {
    chatText: inner.chatText,
    chatJson: async (request) => {
      admitted.count += 1;
      sheets.push(request.messages
        .map(function contentOf(message,): string {
          return messageText({ message, },);
        },)
        .join('\n',),);
      if (admitted.count === abortOnCall)
        controller.abort(CONTEST_ABORT,);
      if (activity !== undefined) {
        /**
         Start position making second slice finish before first under overlap.
         */
        const startPosition = activity.started;
        activity.started += 1;
        activity.now += 1;
        activity.peak = Math.max(
          activity.peak,
          activity.now,
        );
        // ORDERED BY A GATE where the case supplies one, as in the
        // consolidation driver's twin of this fixture: the head start alone (20
        // ms against 5) let a stall of 15 ms between the two slices' asks end
        // the first slice first, and the case passed without the order its
        // name states. A driver that never overlaps leaves the first slice's
        // calls held, and the file fails on the unsettled wait rather than
        // passing.
        await (((startPosition < ROSTER.length) && (activity.secondFinished !== undefined))
          ? activity.secondFinished.promise
          : wait(startPosition < ROSTER.length ? 20 : 5,));
        activity.now -= 1;
        activity.finished.push(startPosition,);
        if (activity.finished.filter(function ofSecondSlice(position,): boolean {
          return position >= ROSTER.length;
        },).length === ROSTER.length)
          activity.secondFinished?.resolve(undefined,);
      }
      return await inner.chatJson(request,);
    },
    quotas: inner.quotas,
  };

  /**
   Projection supplied by syntax case or ordinary cat fixture.
   */
  const askedProjection = projected ?? catProjection({ pairs, },);

  /**
   Records the driver produced.
   */
  const slices = await contestDocumentLanes({
    client,
    projected: askedProjection,
    modelIds: ROSTER,
    frontMatterSlices,
    lineStructuredSlices,
    sheetFacts: {
      ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
      ...((subjectNames === undefined) ? {} : { subjectNames, }),
    },
    cache,
    signal: (abortOnCall === undefined)
      ? AbortSignal.timeout(HANG_STOP_MS,)
      : controller.signal,
    perCallTimeoutMs: HANG_STOP_MS,
    overlap,
    l,
    // Whole bench, one round: these cases count the calls a memo saves, over
    // a bench every seat of which is scripted the same way.
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    calls,
    admitted: admitted.count,
    persisted,
    sheets,
    slices,
  };
}

/**
 Judges a hook hands back after a dry-out, none of them the driver's own.
 */
const RESEATED_JUDGES: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
];

/**
 Builds a client that records which seat every call asked and answers each
 with a ballot for the repair lane.

 @param asked - sink for the seat of every call, in order

 @returns Client to drive with

 @example
 ```ts
 const client = judgeRecordingClient({ asked: [], },);
 ```
 */
function judgeRecordingClient(
  { asked, }: { readonly asked: RosterModelId[]; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      /**
       Ballot every judge casts.
       */
      const rawText = JSON.stringify({
        verdict: 'repair',
        unsupported: [],
        dropped: [],
        assessment: 'the original supports it',
      },);
      /**
       That ballot as the sheet parses it.
       */
      const value: unknown = JSON.parse(rawText,);
      return request.validate(value,)
        ? { kind: 'ok', value, rawText, }
        : { kind: 'schema-mismatch', rawText, detail: 'fixture answers another sheet', };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Contests one slice the lanes worded differently on the fixture roster.

 @param client - client the judges are asked through

 @param beforeSlice - hook handing the slice its judges, none for a driver
 with no hook

 @param looked - sink for every key the driver looked up

 @returns Records the driver produced

 @example
 ```ts
 await contestOneSlice({ client, looked: [], },);
 ```
 */
async function contestOneSlice(
  {
    client,
    beforeSlice,
    looked = [],
  }: {
    readonly client: ModelClient;
    readonly beforeSlice?: () => Promise<BenchSeating>;
    readonly looked?: string[];
  },
): Promise<readonly ArtifactContestSlice[]> {
  /**
   Cache holding nothing, recording every key the driver looked up.
   */
  const recordedMiss = {
    get: function recordKey(key: string,): undefined {
      looked.push(key,);
      return undefined;
    },
  };
  return await contestDocumentLanes({
    client,
    projected: catProjection({ pairs: [[REPAIR_NAP, TRANSLATE_NAP,],], },),
    modelIds: ROSTER,
    frontMatterSlices: new Set(),
    lineStructuredSlices: new Set(),
    cache: {
      resumed: recordedMiss as unknown as ReadonlyMap<string, LaneContestOutcome>,
      persist: async function keepNothing(): Promise<void> {},
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    fanOut: 'whole-bench',
    ...((beforeSlice === undefined) ? {} : { beforeSlice, }),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Claim a probe corroborated against the repair lane's wording.
 */
const DAMAGE_CLAIM = 'the repair lane adds a second cat the original never mentions';

/**
 Accepted addition against the archive rendering.
 */
const DISPUTE_NOTE = 'the archive adds a garden the original never mentions';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: contestDocumentLanes.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'BUYS NOTHING where the two lanes left the same wording, which is most of most documents: a '
              + 'contest between two identical candidates has no question to put',
          fn: async () => {
            /**
             Two slices both lanes agree on.
             */
            const rig = await drive({
              pairs: [
                [
                  ARCHIVE_NAP,
                  ARCHIVE_NAP,
                ],
                [
                  REPAIR_NAP,
                  REPAIR_NAP,
                ],
              ],
              answering: true,
            },);
            expect(rig.calls,).toEqual([],);
            expect(rig.slices,).toEqual([],);
          },
        },),
        it({
          name: 'SHOWS EVERY JUDGE the tense candidates of the texts on offer where the entry records a death and names its subject, and none where it records none',
          fn: async () => {
            /**
             One contested slice whose repair wording follows the declared name in the present.
             */
            const pairs = [
              [
                'Mittens naps in the bookshop attic.',
                TRANSLATE_NAP,
              ],
            ] as const;
            /**
             Blocks each judge of an entry that records a death was sent.
             */
            const recorded = await drive({
              pairs,
              answering: true,
              lifeEnded: true,
              subjectNames: ['Mittens',],
            },);
            expect(recorded.sheets.map(function blocksOfSheet(sheet,): readonly string[] {
              return tenseBlocksOf({ content: sheet, },);
            },),).toEqual(ROSTER.map(function sameBlockFor(): readonly string[] {
              return [
                [
                  '===== TENSE CANDIDATES =====',
                  TENSE_EXPLANATION,
                  'CANDIDATE "repair":',
                  '- "Mittens naps in the bookshop attic." (matched "Mittens naps", rule s-ending-verb)',
                  '=====',
                ].join('\n',),
              ];
            },),);
            /**
             Blocks each judge of an entry that records none was sent.
             */
            const unrecorded = await drive({
              pairs,
              answering: true,
              subjectNames: ['Mittens',],
            },);
            expect(unrecorded.sheets.map(function blocksOfSheet(sheet,): readonly string[] {
              return tenseBlocksOf({ content: sheet, },);
            },),).toEqual(ROSTER.map(function noBlockFor(): readonly string[] {
              return [];
            },),);
          },
        },),
        it({
          name: 'KEYS THE PURCHASE by the death and the names the sheet shows, so ballots bought for other names are never '
            + 'served, and keys an entry that records no death the same whatever names it carries',
          fn: async () => {
            /**
             One contested slice, which every drive buys once.
             */
            const pairs = [
              [
                'Mittens naps in the bookshop attic.',
                TRANSLATE_NAP,
              ],
            ] as const;
            /**
             Keys the driver persisted for an entry that records a death and names one subject.
             */
            const oneName = await drive({
              pairs,
              answering: true,
              lifeEnded: true,
              subjectNames: ['Mittens',],
            },);
            /**
             Keys it persisted for the same entry once its translation declares an alias.
             */
            const withAlias = await drive({
              pairs,
              answering: true,
              lifeEnded: true,
              subjectNames: [
                'Mittens',
                'Whiskers',
              ],
            },);
            /**
             Keys it persisted for an entry that records a death and names no subject.
             */
            const noName = await drive({
              pairs,
              answering: true,
              lifeEnded: true,
            },);
            /**
             Keys it persisted for an entry that records no death and is given no names.
             */
            const unrecorded = await drive({
              pairs,
              answering: true,
            },);
            /**
             Keys it persisted for an entry that records no death and is given a name all the same.
             */
            const unrecordedWithName = await drive({
              pairs,
              answering: true,
              subjectNames: ['Mittens',],
            },);
            /**
             The key each drive bought its one slice under.
             */
            const keys = [
              oneName,
              withAlias,
              noName,
              unrecorded,
            ].map(function keyOf(rig,): string {
              return rig.persisted.join('\n',);
            },);
            expect(keys.every(function holdsOneKey(key,): boolean {
              return (key !== '') && (!key.includes('\n',));
            },),).toBe(true,);
            expect(new Set(keys,).size,).toBe(keys.length,);
            expect(unrecordedWithName.persisted,).toEqual(unrecorded.persisted,);
          },
        },),
        it({
          name: 'ASKS ONLY the slices that differ, leaving the agreed ones out of the record entirely',
          fn: async () => {
            /**
             Three slices, of which the middle one differs.
             */
            const rig = await drive({
              pairs: [
                [
                  ARCHIVE_NAP,
                  ARCHIVE_NAP,
                ],
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
                [
                  ARCHIVE_NAP,
                  ARCHIVE_NAP,
                ],
              ],
              answering: true,
            },);
            expect(rig.calls
              .length,).toBe(ROSTER.length,);
            expect(rig.slices
              .map(function nameSlice(slice,): number {
                return slice.sliceIndex;
              },),).toEqual([1,],);
            expect(rig.slices
              .at(0,)
              ?.verdict,).toEqual({
                kind: 'lane-won',
                lane: 'translate',
              },);
          },
        },),
        it({
          name: 'SHIPS THE TRANSLATE WORDING AT AN ARCHIVE-ABSENT INSERTION WITHOUT ASKING THE ROSTER, '
            + 'since the repair lane has no translation there to mend and the question has one applicable lane',
          fn: async () => {
            /**
             One insertion the translate lane filled and the repair lane could not mend.
             */
            const rig = await drive({
              pairs: [
                [
                  '',
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
              projected: insertionProjection({ translateText: TRANSLATE_NAP, },),
            },);
            expect(rig.calls,).toEqual([],);
            expect(rig.persisted,).toEqual([],);
            expect(rig.slices,).toEqual([
              {
                sliceIndex: 0,
                verdict: {
                  kind: 'sole-lane',
                  lane: 'translate',
                },
                ballots: [],
                usable: 0,
              },
            ],);
          },
        },),
        it({
          name: 'SHOWS NO TENSE BLOCK AND RECORDS NO CANDIDATES AT AN ARCHIVE-ABSENT INSERTION of an entry that records a death, '
            + 'since a sole-lane slice sends no sheet to any judge',
          fn: async () => {
            /**
             One insertion worded in the present after a pronoun, which a sheet would list as a candidate.
             */
            const rig = await drive({
              pairs: [
                [
                  '',
                  'She naps in the attic of the bookshop.',
                ],
              ],
              answering: true,
              projected: insertionProjection({ translateText: 'She naps in the attic of the bookshop.', },),
              lifeEnded: true,
            },);
            expect(rig.calls,).toEqual([],);
            expect(rig.sheets,).toEqual([],);
            expect(rig.persisted,).toEqual([],);
            expect(rig.slices.map(function hasField(slice,): boolean {
              return 'tenseCandidates' in slice;
            },),).toEqual([false,],);
          },
        },),
        it({
          name: 'ASKS THE ROSTER ONLY about the contested slice when an insertion sits beside it, '
            + 'so a one-lane slice never spends a roster read',
          fn: async () => {
            /**
             Insertion at slice 0, and slice 1 where both lanes worded the passage differently.
             */
            const insertion = insertionProjection({ translateText: TRANSLATE_NAP, },);
            const rig = await drive({
              pairs: [
                [
                  '',
                  TRANSLATE_NAP,
                ],
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
              projected: {
                delivery: {
                  repair: [
                    ...insertion.delivery.repair,
                    catLedgerRow({
                      sliceIndex: 1,
                      shippedText: REPAIR_NAP,
                    },),
                  ],
                  translate: [
                    ...insertion.delivery.translate,
                    catLedgerRow({
                      sliceIndex: 1,
                      shippedText: TRANSLATE_NAP,
                    },),
                  ],
                },
                comparison: [
                  ...insertion.comparison,
                  catRow({
                    sliceIndex: 1,
                    repairText: REPAIR_NAP,
                    translateText: TRANSLATE_NAP,
                  },),
                ],
              },
            },);
            expect(rig.calls
              .length,).toBe(ROSTER.length,);
            expect(rig.slices
              .map(function nameSlice(slice,): number {
                return slice.sliceIndex;
              },),).toEqual([
                0,
                1,
              ],);
          },
        },),
        it({
          name: 'KEEPS THE ROSTER at an archive-absent insertion that is front matter, since the syntax admission '
            + 'rule judges its candidates and records them, so no lane is named by rule alone',
          fn: async () => {
            /**
             One insertion the translate lane filled, marked as the syntax-bearing slice.
             */
            const rig = await drive({
              pairs: [
                [
                  '',
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
              projected: insertionProjection({ translateText: TRANSLATE_NAP, },),
              frontMatterSlices: new Set([0,]),
            },);
            expect(rig.calls
              .length,).toBe(ROSTER.length,);
            expect(rig.slices
              .map(function nameSlice(slice,): number {
                return slice.sliceIndex;
              },),).toEqual([0,],);
            expect(rig.slices
              .at(0,)
              ?.ballots
              .length,).toBe(ROSTER.length,);
          },
        },),
        it({
          name: 'runs two contested slices at once at overlap 2, after a serial positive '
            + 'control proves the successful-call instrument distinguishes one from two, '
            + 'and returns records in comparison order when the second slice finishes first',
          fn: async () => {
            /**
             Distinct questions preventing cache-key aliasing from affecting order.
             */
            const pairs = [
              [
                REPAIR_NAP,
                TRANSLATE_NAP,
              ],
              [
                `${REPAIR_NAP} Again.`,
                `${TRANSLATE_NAP} Again.`,
              ],
            ] as const;

            /**
             Serial positive-control activity.
             */
            const serial: ContestConcurrency = {
              now: 0,
              peak: 0,
              started: 0,
              finished: [],
            };
            await drive({
              pairs,
              answering: true,
              overlap: 1,
              activity: serial,
            },);

            /**
             Two-slice activity, gated so the second slice finishes first.
             */
            const overlapped: ContestConcurrency = {
              now: 0,
              peak: 0,
              started: 0,
              finished: [],
              secondFinished: Promise.withResolvers<undefined>(),
            };
            const rig = await drive({
              pairs,
              answering: true,
              overlap: 2,
              activity: overlapped,
            },);
            expect(serial.peak,).toBe(ROSTER.length,);
            expect(overlapped.peak,).toBe(ROSTER.length * 2,);
            // THE SECOND SLICE DID FINISH FIRST: every call of the second slice
            // ended before any call of the first, so the check that the records
            // come back in comparison order is exercised.
            expect(overlapped.finished.map(function sliceOf(position,): number {
              return (position < ROSTER.length) ? 0 : 1;
            },),).toEqual([
              ...ROSTER.map(function second(): number {
                return 1;
              },),
              ...ROSTER.map(function first(): number {
                return 0;
              },),
            ],);
            expect(rig.slices.map(function toIndex(slice,) {
              return slice.sliceIndex;
            },),).toEqual([
              0,
              1,
            ],);
          },
        },),

        it({
          name: 'ASKS ONCE for two contested slices carrying the same question at overlap 1 and 2, '
            + 'so a cold run cannot settle contradictory ballots where a warm run resumes one record',
          fn: async () => {
            await Promise.all(([1, 2,] as const).map(async function atOverlap(
              overlap,
            ): Promise<void> {
              const rig = await drive({
                pairs: [
                  [
                    REPAIR_NAP,
                    TRANSLATE_NAP,
                  ],
                  [
                    REPAIR_NAP,
                    TRANSLATE_NAP,
                  ],
                ],
                answering: true,
                overlap,
              },);
              expect(rig.calls.length,).toBe(ROSTER.length,);
              expect(rig.persisted.length,).toBe(1,);
              expect(rig.slices.map(function toIndex(slice,) {
                return slice.sliceIndex;
              },),).toEqual([
                0,
                1,
              ],);
              expect(rig.slices.map(function toVerdict(slice,) {
                return slice.verdict;
              },),).toEqual([
                {
                  kind: 'lane-won',
                  lane: 'translate',
                },
                {
                  kind: 'lane-won',
                  lane: 'translate',
                },
              ],);
            },),);
          },
        },),

        it({
          name: 'ASKS AGAIN for a twin whose contest roster was unheard at overlap 1 and 2, '
            + 'because the in-run memo may hold only what a warm run can resume',
          fn: async () => {
            await Promise.all(([1, 2,] as const).map(async function atOverlap(
              overlap,
            ): Promise<void> {
              const single = await drive({
                pairs: [
                  [
                    REPAIR_NAP,
                    TRANSLATE_NAP,
                  ],
                ],
                answering: false,
                overlap,
              },);
              const twin = await drive({
                pairs: [
                  [
                    REPAIR_NAP,
                    TRANSLATE_NAP,
                  ],
                  [
                    REPAIR_NAP,
                    TRANSLATE_NAP,
                  ],
                ],
                answering: false,
                overlap,
              },);
              expect(single.admitted,).toBe(ROSTER.length,);
              expect(twin.admitted,).toBe(single.admitted * 2,);
              expect(twin.persisted,).toEqual([],);
            },),);
          },
        },),

        it({
          name: 'REBUYS IDENTICAL FRONT MATTER winner that cannot ship instead of persisting or twin-memoizing it',
          fn: async () => {
            /**
             One unsafe contest as purchase positive control.
             */
            const single = await drive({
              pairs: [],
              projected: metadataProjection({ sliceCount: 1, },),
              frontMatterSlices: new Set([0,]),
              answerChoice: 'repair',
              answering: true,
              overlap: 2,
            },);
            /**
             Same unsafe question repeated at two positions.
             */
            const twin = await drive({
              pairs: [],
              projected: metadataProjection({ sliceCount: 2, },),
              frontMatterSlices: new Set([
                0,
                1,
              ],),
              answerChoice: 'repair',
              answering: true,
              overlap: 2,
            },);
            expect(single.admitted,).toBe(ROSTER.length,);
            expect(single.persisted,).toEqual([],);
            expect(single.slices[0]?.verdict,).toEqual({ kind: 'settled-neither', },);
            expect(single.slices[0]?.eligibility,).toEqual({
              syntax: 'front-matter',
              sourceText: METADATA_SOURCE,
              archive: 'ineligible',
              repair: 'ineligible',
              translate: 'eligible',
            },);
            expect(twin.admitted,).toBe(single.admitted * 2,);
            expect(twin.persisted,).toEqual([],);
          },
        },),

        it({
          name: 'REBUYS A WINNER MERGING THE LINES OF A GOVERNED SLICE instead of persisting it (ledger H2), '
            + 'while the same winner persists where no line rule governs and the lane keeping the lines '
            + 'persists where one does',
          fn: async () => {
            /**
             Merged winner where the line-structure rule governs the slice.
             */
            const governed = await drive({
              pairs: [],
              projected: verseProjection(),
              lineStructuredSlices: new Set([0,],),
              answerChoice: 'translate',
              answering: true,
            },);

            /**
             Same merged winner where no line rule governs.
             */
            const free = await drive({
              pairs: [],
              projected: verseProjection(),
              answerChoice: 'translate',
              answering: true,
            },);

            /**
             Line-keeping winner of the governed slice.
             */
            const kept = await drive({
              pairs: [],
              projected: verseProjection(),
              lineStructuredSlices: new Set([0,],),
              answerChoice: 'repair',
              answering: true,
            },);
            expect(governed.admitted,).toBe(ROSTER.length,);
            expect(governed.persisted,).toEqual([],);
            expect(free.persisted
              .length,).toBe(1,);
            expect(kept.persisted
              .length,).toBe(1,);
          },
        },),

        it({
          name: 'PERSISTS a settled verdict, since ballots are the purchased thing and the next resume must not re-buy them',
          fn: async () => {
            /**
             One contested slice, answered.
             */
            const rig = await drive({
              pairs: [
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
            },);
            expect(rig.persisted
              .length,).toBe(1,);
          },
        },),
        it({
          name: 'REFUSES TO PERSIST winner that final publication cannot ship',
          fn: async () => {
            /**
             Cache writes attempted by unsafe winner.
             */
            const persisted: string[] = [];
            await persistLaneContestOutcome({
              key: 'unsafe-front-matter-winner',
              outcome: {
                choice: 'repair',
                ballots: [
                  {
                    choice: 'repair',
                    unsupported: [],
                    unsupportedRaw: [],
                    dropped: [],
                    droppedRaw: [],
                    reason: 'first vote',
                  },
                  {
                    choice: 'repair',
                    unsupported: [],
                    unsupportedRaw: [],
                    dropped: [],
                    droppedRaw: [],
                    reason: 'second vote',
                  },
                ],
                usable: 2,
                findings: [],
              },
              choiceMayShip: false,
              cache: {
                resumed: new Map<string, LaneContestOutcome>(),
                persist: async ({ key, },) => {
                  persisted.push(key,);
                },
              },
              signal: new AbortController().signal,
            },);
            expect(persisted,).toEqual([],);
          },
        },),

        it({
          name: 'THROWS the caller abort reason while the roster is in flight, even after '
            + 'the first voices answered',
          fn: async () => {
            await expect(drive({
              pairs: [
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
              abortOnCall: ROSTER.length,
            },),)
              .rejects
              .toBe(CONTEST_ABORT,);
          },
        },),

        it({
          name: 'REFUSES persistence under an already aborted caller after a quorum-complete '
            + 'outcome returned, preserving the final pre-write defense',
          fn: async () => {
            /**
             Exact caller reason helper must surface.
             */
            const stopped = new Error('caller abandoned completed contest',);
            const controller = new AbortController();
            controller.abort(stopped,);

            /**
             Writes attempted after abort.
             */
            const persisted: string[] = [];
            await expect(persistLaneContestOutcome({
              key: 'lane-contest-persistence-guard-fixture',
              outcome: {
                choice: 'repair',
                ballots: [
                  {
                    choice: 'repair',
                    unsupported: [],
                    unsupportedRaw: [],
                    dropped: [],
                    droppedRaw: [],
                    reason: 'first corroborating ballot',
                  },
                  {
                    choice: 'repair',
                    unsupported: [],
                    unsupportedRaw: [],
                    dropped: [],
                    droppedRaw: [],
                    reason: 'second corroborating ballot',
                  },
                ],
                usable: 2,
                findings: [],
              },
              cache: {
                resumed: new Map<string, LaneContestOutcome>(),
                persist: async ({ key, },) => {
                  persisted.push(key,);
                },
              },
              signal: controller.signal,
            },),)
              .rejects
              .toBe(stopped,);
            expect(persisted,).toEqual([],);
          },
        },),

        it({
          name:
            'REFUSES TO PERSIST an unheard roster, because a provider down for one night is not a property '
              + 'of the question and caching it would freeze that night into every later resume',
          fn: async () => {
            /**
             One contested slice nobody answered.
             */
            const rig = await drive({
              pairs: [
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
              ],
              answering: false,
            },);
            expect(rig.persisted,).toEqual([],);
            expect(rig.slices
              .at(0,)
              ?.verdict,).toEqual({ kind: 'quorum-not-met', },);
          },
        },),
        it({
          name:
            'RESUMES a slice off the cache without calling anything, and does not write back what it just '
              + 'read: a re-persisted resume is a write per slice per run for nothing',
          fn: async () => {
            /**
             Ballots an earlier run bought, under the key this run derives.
             */
            const bought: LaneContestOutcome = {
              choice: 'repair',
              ballots: [
                {
                  choice: 'repair',
                  unsupported: [],
                  unsupportedRaw: [],
                  dropped: [],
                  droppedRaw: [],
                  reason: 'bought earlier',
                },
                {
                  choice: 'repair',
                  unsupported: [],
                  unsupportedRaw: [],
                  dropped: [],
                  droppedRaw: [],
                  reason: 'bought earlier',
                },
              ],
              usable: 2,
              findings: [],
            };

            // THROUGH A FRESH RUN FIRST, to learn the key rather than to spell it
            // out here: a fixture that wrote the key itself would keep passing after
            // the two derivations diverged, which is the defect the pinned key test
            // exists for and the one a resumption test must not repeat.
            const learned = await drive({
              pairs: [
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
            },);

            /**
             Same slice, with those ballots already on disk.
             */
            const rig = await drive({
              pairs: [
                [
                  REPAIR_NAP,
                  TRANSLATE_NAP,
                ],
              ],
              answering: true,
              resumed: new Map([
                [
                  learned.persisted
                    .at(0,) ?? '',
                  bought,
                ],
              ],),
            },);
            expect(rig.calls,).toEqual([],);
            expect(rig.persisted,).toEqual([],);
            expect(rig.slices
              .at(0,)
              ?.verdict,).toEqual({
                kind: 'lane-won',
                lane: 'repair',
              },);
          },
        },),

        it({
          name: 'mixes one resumed row with one fresh row at overlap 2, buying and persisting '
            + 'only the fresh question while returning both in comparison order',
          fn: async () => {
            /**
             Two distinct contest questions.
             */
            const pairs = [
              [
                REPAIR_NAP,
                TRANSLATE_NAP,
              ],
              [
                `${REPAIR_NAP} Again.`,
                `${TRANSLATE_NAP} Again.`,
              ],
            ] as const;

            /**
             Fresh pass used only to derive both production keys.
             */
            const learned = await drive({
              pairs,
              answering: true,
            },);

            /**
             Quorum-complete first-row outcome already on disk.
             */
            const bought: LaneContestOutcome = {
              choice: 'repair',
              ballots: [
                {
                  choice: 'repair',
                  unsupported: [],
                  unsupportedRaw: [],
                  dropped: [],
                  droppedRaw: [],
                  reason: 'first stored ballot',
                },
                {
                  choice: 'repair',
                  unsupported: [],
                  unsupportedRaw: [],
                  dropped: [],
                  droppedRaw: [],
                  reason: 'second stored ballot',
                },
              ],
              usable: 2,
              findings: [],
            };
            const rig = await drive({
              pairs,
              answering: true,
              overlap: 2,
              resumed: new Map([
                [
                  learned.persisted.at(0,) ?? '',
                  bought,
                ],
              ],),
            },);
            expect(rig.admitted,).toBe(ROSTER.length,);
            expect(rig.persisted.length,).toBe(1,);
            expect(rig.slices.map(function toIndex(slice,) {
              return slice.sliceIndex;
            },),).toEqual([
              0,
              1,
            ],);
            expect(rig.slices.map(function toVerdict(slice,) {
              return slice.verdict;
            },),).toEqual([
              {
                kind: 'lane-won',
                lane: 'repair',
              },
              {
                kind: 'lane-won',
                lane: 'translate',
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${contestDocumentLanes.name} re-seated under a hold (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS A SLICE ON THE JUDGES ITS HOOK RETURNS, as the lanes and the consolidation seat theirs, so '
            + 'judges re-read after a provider dry-out are the ones the slice asks rather than those read before it',
          fn: async () => {
            /**
             Seat of every call a driver with no hook made.
             */
            const control: RosterModelId[] = [];
            await contestOneSlice({ client: judgeRecordingClient({ asked: control, },), },);
            /**
             Seat of every call the re-seated driver made.
             */
            const asked: RosterModelId[] = [];
            await contestOneSlice({
              client: judgeRecordingClient({ asked, },),
              beforeSlice: async (): Promise<BenchSeating> => ({ modelIds: RESEATED_JUDGES, }),
            },);
            expect({
              controlOnRoster: (control.length > 0) && control.every(function onRoster(seat,): boolean {
                return (ROSTER as readonly RosterModelId[]).includes(seat,);
              },),
              reseatedAskedAny: asked.length > 0,
              outsideReseated: asked.filter(function outside(seat,): boolean {
                return !RESEATED_JUDGES.includes(seat,);
              },),
            },).toEqual({
              controlOnRoster: true,
              reseatedAskedAny: true,
              outsideReseated: [],
            },);
          },
        },),
        it({
          name: 'KEYS A RE-SEATED SLICE BY THE JUDGES IT RUNS ON, so ballots the judges read before the dry-out '
            + 'cast are never resumed for it, while a hook handing back the starting judges keys the slice as a '
            + 'driver with no hook does',
          fn: async () => {
            /**
             Keys a driver with no hook looks up.
             */
            const starting: string[] = [];
            await contestOneSlice({ client: judgeRecordingClient({ asked: [], },), looked: starting, },);
            /**
             Keys looked up when the hook re-seats the slice elsewhere.
             */
            const moved: string[] = [];
            await contestOneSlice({
              client: judgeRecordingClient({ asked: [], },),
              looked: moved,
              beforeSlice: async (): Promise<BenchSeating> => ({ modelIds: RESEATED_JUDGES, }),
            },);
            /**
             Keys looked up when the hook hands back the judges the driver started on.
             */
            const kept: string[] = [];
            await contestOneSlice({
              client: judgeRecordingClient({ asked: [], },),
              looked: kept,
              beforeSlice: async (): Promise<BenchSeating> => ({ modelIds: ROSTER, }),
            },);
            expect({
              lookups: [starting.length, moved.length, kept.length,],
              movedDiffers: moved[0] !== starting[0],
              keptMatches: kept[0] === starting[0],
            },).toEqual({
              lookups: [1, 1, 1,],
              movedDiffers: true,
              keptMatches: true,
            },);
          },
        },),
        it({
          name: 'OPENS NO ROUND FOR A SLICE whose hook reports fewer reachable seats than the contest needs voices, '
            + 'so the slice keeps its incumbent as an unheard roster does and no call is made (XingZ6020: 63 rounds '
            + 'on one reachable seat of seven)',
          fn: async () => {
            /**
             Seat of every call the driver made under a bench below the floor.
             */
            const asked: RosterModelId[] = [];
            const slices = await contestOneSlice({
              client: judgeRecordingClient({ asked, },),
              beforeSlice: async (): Promise<BenchSeating> => ({ modelIds: ROSTER, reachable: 1, }),
            },);
            expect({
              calls: asked.length,
              verdict: slices.at(0,)?.verdict,
              usable: slices.at(0,)?.usable,
              ballots: slices.at(0,)?.ballots.length,
            },).toEqual({
              calls: 0,
              verdict: { kind: 'quorum-not-met', },
              usable: 0,
              ballots: 0,
            },);
          },
        },),
        it({
          name: 'ASKS A SLICE AS BEFORE once its hook reports the contest\'s floor reachable',
          fn: async () => {
            /**
             Seat of every call the driver made with the floor met.
             */
            const asked: RosterModelId[] = [];
            const slices = await contestOneSlice({
              client: judgeRecordingClient({ asked, },),
              beforeSlice: async (): Promise<BenchSeating> => ({ modelIds: ROSTER, reachable: 2, }),
            },);
            expect({
              asked: asked.length > 0,
              verdict: slices.at(0,)?.verdict.kind,
            },).toEqual({
              asked: true,
              verdict: 'lane-won',
            },);
          },
        },),
        it({
          name: 'WRITES A LATER SLICE\'S OWN FAILURE UNDER THE CALLER\'S STOP, its hook refused by an unreachable '
            + 'bench as the stop landed while the first slice ended in the caller\'s reason, and refuses with that '
            + 'reason',
          fn: async () => {
            /**
             Every line the driver wrote.
             */
            const messages: string[] = [];
            /**
             The caller's steering, which the later slice's hook stops.
             */
            const steering = new AbortController();
            /**
             What the driver refused with.
             */
            const refusal = await rejectionOf(async function contestsUnderTheStop(): Promise<unknown> {
              return await contestDocumentLanes({
                client: UNASKED_CLIENT,
                projected: catProjection({
                  pairs: [
                    [REPAIR_NAP, TRANSLATE_NAP,],
                    [REPAIR_NAP, TRANSLATE_NAP,],
                  ],
                },),
                modelIds: ROSTER,
                frontMatterSlices: new Set(),
                lineStructuredSlices: new Set(),
                cache: {
                  resumed: new Map(),
                  persist: async function keepNothing(): Promise<void> {},
                },
                signal: steering.signal,
                perCallTimeoutMs: HANG_STOP_MS,
                overlap: 2,
                l: capturingLogger({ messages, },),
                beforeSlice: stopLandingHook<BenchSeating>({
                  steering,
                  reason: CONTEST_ABORT,
                  afterStop: throwBenchUnreachable,
                },),
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);
            expect(refusal,).toBe(CONTEST_ABORT,);
            expect(messages,).toEqual([
              '[contestDocumentLanes] lane contest: slices that differ: 2 of 2',
              `[contestDocumentLanes] [mapOverlapped] slice 1 failed (${BENCH_UNREACHABLE.message}), and the caller `
                + 'stopped the work',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${contestDocumentLanes.name} with evidence against a lane (T8 batch 13)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE JUDGES a corroborated damage claim and a dispute note, and logs the claim count in words '
            + 'that agree with it (ledger B66)',
          fn: async () => {
            /**
             Every request the judges were sent, as text.
             */
            const prompts: string[] = [];
            /**
             Lines the driver logged.
             */
            const messages: string[] = [];
            /**
             Client casting the one ballot every judge casts, recording each request.
             */
            const recording = judgeRecordingClient({ asked: [], },);
            await contestDocumentLanes({
              client: {
                ...recording,
                chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
                  prompts.push(JSON.stringify(request,),);
                  return await recording.chatJson(request,);
                },
              },
              projected: catProjection({ pairs: [[REPAIR_NAP, TRANSLATE_NAP,],], },),
              modelIds: ROSTER,
              frontMatterSlices: new Set(),
              lineStructuredSlices: new Set(),
              cache: {
                resumed: new Map(),
                persist: async function keepNothing(): Promise<void> {},
              },
              damageClaimsBySlice: new Map([[0, [DAMAGE_CLAIM,],],],),
              disputeNotesBySlice: new Map([[0, DISPUTE_NOTE,],],),
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: capturingLogger({ messages, },),
              fanOut: 'whole-bench',
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              asked: prompts.length > 0,
              everyPromptCarriesClaim: prompts.every(function carriesClaim(prompt,): boolean {
                return prompt.includes(DAMAGE_CLAIM,);
              },),
              everyPromptCarriesNote: prompts.every(function carriesNote(prompt,): boolean {
                return prompt.includes(DISPUTE_NOTE,);
              },),
              claimLine: messages.some(function namesClaim(line,): boolean {
                return line.includes('slice 0: 1 corroborated damage claim shown to the judges',);
              },),
            },).toEqual({
              asked: true,
              everyPromptCarriesClaim: true,
              everyPromptCarriesNote: true,
              claimLine: true,
            },);
          },
        },),
      ],
    },),
  ],
},);
