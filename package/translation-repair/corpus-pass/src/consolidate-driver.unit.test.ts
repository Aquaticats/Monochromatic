/**
 Tests for the consolidation over one document.

 WHAT THESE PIN is the driver's own reasoning, which is the part no stage test
 reaches: which slices it asks about at all, what it resumes rather than
 rebuys, what it is willing to write to the cache, and what it refuses to
 proceed past.

 ALMOST EVERY CASE HERE BUYS NOTHING. The client those hand over throws on any
 call, which is the assertion: a driver that reached the roster on a resumed
 slice, or on a slice the contest never settled, fails loudly instead of
 quietly costing a run its budget. The rounds themselves are covered by
 `consolidate-settle.unit.test.ts`.

 THE TWO SHEET CASES ARE THE EXCEPTION, and deliberately so. They let the calls
 through to a client that records what it was sent and answers each with content
 no sheet can parse, because what they ask is what the driver SENT rather than
 what a roster would say back. Every voice is lost, which the validity floor
 already handles, so the driver settles on the standing text having bought
 nothing usable.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { SliceNeighbourContext, } from '@monochromatic-dev/translation-repair-repair';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest';
import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive';
import {
  type TranslateDecision,
  type TranslateStageResult,
  TranslationRepairInterruptedError,
} from '@monochromatic-dev/translation-repair-translate-stage';
import {
  SLICE_COST_MARKER,
  SLICE_START_MARKER,
  type SliceCache,
  TRANSLATE_LINE_STRUCTURE_RULE,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { firstRoundWindow, } from '@monochromatic-dev/translation-repair-stage';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createSyntheticClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  type ConsolidationPolishConfig,
  type ConsolidationSettlement,
  type ConsolidationTerminal,
  consolidationWorthResuming,
  persistConsolidationSettlement,
} from '@monochromatic-dev/translation-repair-consolidate';
import {
  consolidateRunShape,
  consolidateSliceKey,
  type ConsolidateSliceSeating,
  type ProjectedLanes,
} from '@monochromatic-dev/translation-repair-artifact';
import { consolidateDocument, } from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_DECISIONS,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  BENCH_UNREACHABLE,
  stopLandingHook,
  throwBenchUnreachable,
  UNASKED_CLIENT,
} from './stop-landing-hook.test-fixture.ts';

/**
 Logger the driver writes through, whose output is not under test.
 */
const l = tagged({ tag: 'consolidate-driver-test', },);

/**
 Roster this run seats.
 */
const ROSTER = [SEAT_HYPER_OPENROUTER_VISION_EDITOR,] as const;

/** Roster wide enough for independent selection. */
const WIDE_ROSTER = [
  ...ROSTER,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

/**
 What a picture near a slice was read to say, offered to the driver in the
 picture cases and looked for on the sheets it sends, so the reading offered
 and the reading sought cannot drift apart.
 */
const TABBY_READING = 'the photograph shows a tabby asleep on a stack of library books';

/**
 Successful consolidation calls in flight and peak observed by fixture.
 */
type ConsolidationConcurrency = {
  now: number;
  peak: number;
  started: number;

  /**
   Start positions in the order their calls finished, so a test can show a
   later call answering first rather than assume it (ledger T6).
   */
  readonly finished: number[];

  /**
   Gate the first-started call holds on until the second-started call has
   finished, where a case needs the second to answer first and both to be in
   flight at once whatever the machine's load.
   */
  readonly secondFinished?: PromiseWithResolvers<undefined>;
};

/**
 A client that refuses to be used, so any call at all is a failure rather
 than a slow test.

 NO RETRIES since 2026-09-27 (ledger T5): two positive controls that expect
 this refusal passed only by outlasting five production backoffs, and the file
 took 10.24 s; 0.77 s without them. The refusal still propagates unchanged.
 */
const REFUSING_CLIENT = createSyntheticClient({
  apiKey: 'test-key',
  retryPolicy: {
    limit: 0,
    baseMs: 1,
  },
  transport: async function refusingTransport() {
    throw new Error('the driver bought a call it should not have',);
  },
},);

/**
 A refusing client that counts the calls it refused, for a positive control
 that has to show the roster was reached.

 THE COUNT IS THE EVIDENCE (ledger T5). The controls read the refusal off a
 thrown error until 2026-09-27, and that error only reached them because five
 production backoffs outlasted the driver's 5 s signal: with the backoff gone
 the refusal is a lost voice, the slice exits `failed`, and nothing is thrown.

 @returns Client beside the number of calls it refused

 @example
 ```ts
 const { client, calls, } = countingRefusingClient();
 ```
 */
function countingRefusingClient(): {
  readonly client: ModelClient;
  readonly calls: { count: number; };
} {
  /**
   Calls refused so far.
   */
  const calls = { count: 0, };
  return {
    client: createSyntheticClient({
      apiKey: 'test-key',
      retryPolicy: {
        limit: 0,
        baseMs: 1,
      },
      transport: async function refuseAndCount() {
        calls.count += 1;
        throw new Error('the driver bought a call it should not have',);
      },
    },),
    calls,
  };
}

/**
 Builds a client that records every request body and answers none of them.

 ANSWERS WITH UNREADABLE CONTENT rather than throwing, which matters more than
 it looks. A thrown transport error is retried five times and then propagates
 out of the driver, so a case built that way spends seconds failing on the
 refusal instead of reading the sheet. A well-formed reply carrying content no
 sheet can parse loses the voice instead, which is an outcome the validity floor
 is already built for.

 @returns Client to drive with, beside the bodies it recorded

 @example
 ```ts
 const { client, bodies, } = recordingClient();
 ```
 */
function recordingClient(): {
  readonly client: ModelClient;
  readonly bodies: readonly string[];
} {
  /**
   Every request body the driver sent, in order.
   */
  const bodies: string[] = [];

  return {
    client: createSyntheticClient({
      apiKey: 'test-key',
      transport: async function recordThenSayNothingUsable(exchange,) {
        bodies.push(exchange.bodyJson ?? '',);
        return {
          status: 200,
          bodyText: `data: ${
            JSON.stringify({
              choices: [
                {
                  index: 0,
                  delta: { content: 'no sheet can read this', },
                },
              ],
            },)
          }\n\ndata: [DONE]\n\n`,
        };
      },
    },),
    bodies,
  };
}

/**
 Marker the gate's own sheet carries and no other round does.

 READ OFF `consolidate-gate-wire.ts`. The gate and the slate judges share the
 ballot schema, so the schema name alone cannot tell them apart and a case
 reading `the judges' sheet` would otherwise be reading the gate's half the time.
 */
const GATE_MARKER = 'unsupported and dropped each a list';

/**
 Builds a client that records every request and answers each role usefully.

 WHY NOT {@link recordingClient}. That one answers content no sheet can parse,
 so every voice is lost and the driver settles on the standing text having
 bought nothing. That is exactly right for reading what a PRODUCER was sent,
 since the producer round happens before anything can be lost, and useless for
 reading a judge's sheet, because no judging round ever runs.

 @returns Client to drive with, beside the judge requests it recorded

 @example
 ```ts
 const { client, judgeSheets, } = answeringClient();
 ```
 */
function answeringClient(): {
  readonly client: ModelClient;
  readonly judgeSheets: readonly string[];
} {
  /**
   Requests the SLATE judges received, gate rounds excluded.
   */
  const judgeSheets: string[] = [];

  return {
    client: createSyntheticClient({
      apiKey: 'test-key',
      transport: async function answerByRole(exchange,) {
        /**
         Everything this call is sending, which is where the sheet lives.
         */
        const sent = exchange.bodyJson ?? '';

        /**
         Which of the three rounds this call is, decided once so the answer and
         the recording cannot disagree about it.

         THE GATE IS CHECKED BEFORE THE BALLOT, because both ask for the same
         schema and only the gate's own wording separates them.
         */
        const role = sent.includes('translation_report',)
          ? 'produce'
          : (sent.includes(GATE_MARKER,) ? 'gate' : 'judge');

        /**
         Reply each round is given.
         */
        const answers: Record<string, string> = {
          produce: JSON.stringify({ translation: 'A cat asleep in the sun.', },),
          gate: JSON.stringify({
            verdict: 'standing',
            unsupported: [],
            dropped: [],
            assessment: 'the original supports it',
          },),
          judge: JSON.stringify({
            best: 1,
            assessment: 'it says what the original says',
          },),
        };

        /**
         Reply body for whichever role asked.
         */
        const answer = answers[role] ?? '';

        if (role === 'judge')
          judgeSheets.push(sent,);

        return {
          status: 200,
          bodyText: `data: ${
            JSON.stringify({
              choices: [
                {
                  index: 0,
                  delta: { content: answer, },
                },
              ],
            },)
          }\n\ndata: [DONE]\n\n`,
        };
      },
    },),
    judgeSheets,
  };
}

/**
 Finds numbered candidate carrying scripted wording in serialized request.

 @param sent - serialized request body

 @param needle - candidate wording marker

 @returns One-based candidate index, or zero when absent

 @example
 ```ts
 const best = candidateCarrying({ sent, needle: 'rests naturally', });
 ```
 */
function candidateCarrying(
  {
    sent,
    needle,
  }: {
    readonly sent: string;
    readonly needle: string;
  },
): number {
  /** Candidate heading marker. */
  const marker = 'CANDIDATE ';
  for (let cursor = sent.indexOf(marker,); cursor >= 0;) {
    /** Start of next candidate block, or request end. */
    const next = sent.indexOf(marker, cursor + marker.length,);
    /** Current serialized candidate block. */
    const block = sent.slice(cursor, next === (-1) ? sent.length : next,);
    if (block.includes(needle,)) {
      return Number(sent.charAt(cursor + marker.length,),);
    }
    cursor = next;
  }
  return 0;
}

/**
 Reads only messages from serialized provider request.

 @param body - serialized provider request

 @returns Serialized messages without provider model field
 */
function requestMessages({ body, }: { readonly body: string; }): string {
  /** Parsed request envelope. */
  const parsed = JSON.parse(body,) as { readonly messages?: unknown; };
  return JSON.stringify(parsed.messages ?? [],);
}

/**
 Builds a client whose producers write one text, whose judges prefer a
 second text when a sheet carries it, and whose gate keeps the standing
 unless a sheet carries that second text.

 A STAGE-LOCAL RECOVERY once re-asked the producers with the failed
 strategy's evidence, and this client answered that second ask differently;
 the single consolidation attempt (1ba8f713a) ended the re-ask, and its
 branches here went on 2026-09-29 with the evidence builder (ledger B30).

 @returns Client plus producer sheets and full payloads, and every request
 body whatever its role, so a case can pin that nothing was bought

 @example
 ```ts
 const { client, producerSheets, } = scriptedClient();
 ```
 */
function scriptedClient(
  {
    initialText = 'A cat asleep in the sun.',
    endorsedText = 'The cat rests naturally in the sunlight.',
  }: {
    readonly initialText?: string;
    readonly endorsedText?: string;
  } = {},
): {
  readonly client: ModelClient;
  readonly producerSheets: readonly string[];
  readonly producerPayloads: readonly string[];
  readonly requests: readonly string[];
} {
  /** Producer requests in dispatch order. */
  const producerSheets: string[] = [];
  /** Full producer payloads retaining model identity for uniqueness checks. */
  const producerPayloads: string[] = [];
  /** Every request body served, producers', judges' and the gate's alike. */
  const requests: string[] = [];
  return {
    client: createSyntheticClient({
      apiKey: 'test-key',
      transport: async function answerByScript(exchange,) {
        /** Complete request body carrying stage sheet. */
        const sent = exchange.bodyJson ?? '';
        requests.push(sent,);
        /** Current role inferred from schema and gate marker. */
        const role = sent.includes('translation_report',)
          ? 'produce'
          : (sent.includes(GATE_MARKER,) ? 'gate' : 'judge');
        /** Structured answer for current role. */
        const answer = (function answerByRole(): string {
          if (role === 'produce') {
            producerSheets.push(requestMessages({ body: sent, }),);
            producerPayloads.push(sent,);
            return JSON.stringify({ translation: initialText, },);
          }
          if (role === 'judge') {
            return JSON.stringify({
              best: candidateCarrying({
                sent,
                needle: sent.includes(endorsedText,)
                  ? endorsedText
                  : initialText,
              },),
              assessment: 'best supported rendering',
            },);
          }
          return JSON.stringify({
            verdict: sent.includes(endorsedText,) ? 'consolidated' : 'standing',
            unsupported: [],
            dropped: [],
            assessment: 'the consolidation improves fidelity',
          },);
        })();
        return {
          status: 200,
          bodyText: `data: ${JSON.stringify({
            choices: [{
              index: 0,
              delta: { content: answer, },
            },],
          },)}\n\ndata: [DONE]\n\n`,
        };
      },
    },),
    producerSheets,
    producerPayloads,
    requests,
  };
}

/**
 Builds both ledgers for a document of two slices.

 @returns Projection shaped as the lanes leave one

 @example
 ```ts
 const projected = twoSliceDocument();
 ```
 */
function twoSliceDocument(): ProjectedLanes {
  /**
   One comparison row per slice, both lanes wording them differently.
   */
  const comparison = [0, 1,].map(function toRow(sliceIndex,) {
    return {
      sliceIndex,
      incumbentKind: 'present',
      incumbentText: `archive wording for slice ${String(sliceIndex,)}`,
      repairText: `repair wording for slice ${String(sliceIndex,)}`,
      translateText: `translate wording for slice ${String(sliceIndex,)}`,
    };
  },);

  return {
    comparison,
    delivery: {
      repair: comparison.map(function toDelivery(row,) {
        return {
          sliceIndex: row.sliceIndex,
          sourceText: `原文${String(row.sliceIndex,)}`,
        };
      },),
      translate: [],
    },
  } as unknown as ProjectedLanes;
}

/**
 Builds one archive-absent insertion, which only the translate lane applies to.

 @returns Projection whose one slice the repair lane marked not-applicable

 @example
 ```ts
 const projected = insertionDocument();
 ```
 */
function insertionDocument(): ProjectedLanes {
  /**
   Translate lane's wording at the insertion, which is the only wording there.
   */
  const translateText = 'translate wording for slice 0';
  return {
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
    delivery: {
      repair: [
        {
          sliceIndex: 0,
          sourceText: '原文0',
        },
      ],
      translate: [],
    },
  } as unknown as ProjectedLanes;
}

/**
 Builds two rows asking an identical consolidation question.

 @returns Projection whose position-free consolidation keys match

 @example
 ```ts
 const projected = twinSliceDocument();
 ```
 */
function twinSliceDocument(): ProjectedLanes {
  /**
   Same question stamped at both document positions.
   */
  const comparison = [0, 1,].map(function toRow(sliceIndex,) {
    return {
      sliceIndex,
      incumbentKind: 'present',
      incumbentText: 'The archive says the cat naps.',
      repairText: 'The cat naps.',
      translateText: 'The cat is napping.',
    };
  },);
  return {
    comparison,
    delivery: {
      repair: comparison.map(function toDelivery(row,) {
        return {
          sliceIndex: row.sliceIndex,
          sourceText: '猫猫在打盹。',
        };
      },),
      translate: [],
    },
  } as unknown as ProjectedLanes;
}

/**
 Builds one contest record, as the contest wrote it for the artifact.

 @param sliceIndex - slice this answers

 @param lane - lane the contest backed

 @returns Record shaped as the contest stage produces one

 @example
 ```ts
 const record = contestSettling({ sliceIndex: 0, lane: 'repair', },);
 ```
 */
function contestSettling(
  {
    sliceIndex,
    lane,
  }: {
    readonly sliceIndex: number;
    readonly lane: 'repair' | 'translate';
  },
): ArtifactContestSlice {
  return {
    sliceIndex,
    verdict: {
      kind: 'lane-won',
      lane,
    },
    ballots: [],
    usable: ROSTER.length,
  };
}

/**
 Builds a settlement as the stage returns one, for the cache to hand back.

 @param terminal - how the slice left the stage

 @returns Settlement shaped as `settleConsolidation` returns one

 @example
 ```ts
 const settled = settlementReaching({ terminal: 'incumbent-only', },);
 ```
 */
function settlementReaching(
  { terminal, }: { readonly terminal: ConsolidationTerminal; },
): ConsolidationSettlement {
  return {
    terminal,
    text: 'whatever this slice keeps',
    floor: {
      kind: 'incumbent-only',
      refusedModelIds: [...ROSTER,],
    },
    verdicts: [],
    rewrapped: false,
    demoted: false,
    findings: [],
  };
}

/**
 Runs the driver over a document, collecting what it tried to persist.

 @param contests - what the contest settled, keyed by slice

 @param resumed - settlements an earlier run already bought

 @param projected - both ledgers, overridable to test a ledger gap

 @param frontMatterSlices - syntax-bearing metadata positions

 @param judgeModelIds - starting slate judges, the writers when not given

 @param polishConfig - starting naturalness roles, none when not given

 @param overlap - most contested slices in flight

 @param activity - optional successful-call overlap instrument

 @param messages - optional destination for operational logging

 @param writes - optional external persistence capture

 @param signal - optional caller cancellation

 @returns Records the driver produced beside the keys it wrote

 @example
 ```ts
 const { slices, written, } = await driveWith({ contests: [], },);
 ```
 */
async function driveWith(
  {
    contests,
    resumed = new Map(),
    projected = twoSliceDocument(),
    client = REFUSING_CLIENT,
    lineStructuredSlices = new Set(),
    frontMatterSlices = new Set(),
    pictureContextBySlice = new Map(),
    neighbourContextBySlice = new Map(),
    modelIds = ROSTER,
    judgeModelIds,
    polishConfig,
    overlap = 1,
    beforeSlice,
    activity,
    messages,
    writes,
    // The bound on a whole drive when a case passes no signal of its own. At
    // 5,000 ms a loaded machine outran it, and the case that drives two slices
    // serially and then two at once failed with a `TimeoutError` its name never
    // states (commit `5086b6c6f`). A case that needs the run stopped brings its
    // own controller.
    signal = AbortSignal.timeout(HANG_STOP_MS,),
    archiveDisputes,
    acceptedRepairClaimNotes,
    referenceContext,
    sheetFacts,
  }: {
    readonly contests: readonly ArtifactContestSlice[];
    readonly archiveDisputes?: ReadonlyMap<number, ArchiveDispute>;
    /**
     The claims the repair panel accepted per slice, for the cases reading
     what the key and the polish gate are given.
     */
    readonly acceptedRepairClaimNotes?: ReadonlyMap<number, string>;
    /**
     What the pages the original cites say, for the cases reading whether
     each round was shown it.
     */
    readonly referenceContext?: string;
    /**
     The page-level facts the document carries, for the cases reading what each
     record keeps of them; wins over `referenceContext` when both are given.
     */
    readonly sheetFacts?: PreparedSheetFacts;
    readonly resumed?: ReadonlyMap<string, ConsolidationSettlement>;
    readonly projected?: ProjectedLanes;
    readonly client?: ModelClient;
    readonly lineStructuredSlices?: ReadonlySet<number>;
    readonly frontMatterSlices?: ReadonlySet<number>;
    readonly pictureContextBySlice?: ReadonlyMap<number, string>;
    readonly neighbourContextBySlice?: ReadonlyMap<number, SliceNeighbourContext>;
    readonly modelIds?: readonly RosterModelId[];
    readonly judgeModelIds?: readonly RosterModelId[];
    readonly polishConfig?: ConsolidationPolishConfig;
    readonly overlap?: number;
    readonly beforeSlice?: () => Promise<ConsolidateSliceSeating>;
    readonly activity?: ConsolidationConcurrency;
    readonly messages?: string[];
    readonly writes?: string[];
    readonly signal?: AbortSignal;
  },
) {
  /**
   Keys the driver decided were worth resuming later.
   */
  const written: string[] = writes ?? [];

  /**
   Cache standing in for the entry store, recording every write.
   */
  const cache: SliceCache<ConsolidationSettlement> = {
    resumed,
    persist: async function recordWrite({ key, }: { readonly key: string; },): Promise<void> {
      written.push(key,);
    },
  };

  /**
   Client-level activity instrument outside provider slot limiting.
   */
  const measuredClient: ModelClient = (activity === undefined)
    ? client
    : {
      chatText: client.chatText,
      chatJson: async (request) => {
        /**
         Start position making second slice answer before first under overlap.
         */
        const startPosition = activity.started;
        activity.started += 1;
        activity.now += 1;
        activity.peak = Math.max(
          activity.peak,
          activity.now,
        );
        // ORDERED BY A GATE where the case supplies one: a 20 ms against 5 ms
        // head start let the first call finish first 1 time in 8 at 0.2 CPU
        // (2026-09-27). A driver that never overlaps leaves the first call held
        // and the file exits 13 on the unsettled wait rather than passing
        // (measured with the driver forced serial; the run's timeout signal
        // does not hold the process open).
        await (((startPosition === 0) && (activity.secondFinished !== undefined))
          ? activity.secondFinished.promise
          : wait(startPosition === 0 ? 20 : 5,));
        activity.now -= 1;
        activity.finished.push(startPosition,);
        if (startPosition === 1)
          activity.secondFinished?.resolve(undefined,);
        return await client.chatJson(request,);
      },
      quotas: client.quotas,
    };

  const slices = await consolidateDocument({
    client: measuredClient,
    projected,
    contests,
    modelIds,
    ...((judgeModelIds === undefined) ? {} : { judgeModelIds, }),
    ...((polishConfig === undefined) ? {} : { polishConfig, }),
    frontMatterSlices,
    cache,
    signal,
    perCallTimeoutMs: HANG_STOP_MS,
    overlap,
    ...((beforeSlice === undefined) ? {} : { beforeSlice, }),
    lineStructuredSlices,
    pictureContextBySlice,
    neighbourContextBySlice,
    ...((archiveDisputes === undefined) ? {} : { archiveDisputes, }),
    ...((acceptedRepairClaimNotes === undefined) ? {} : { acceptedRepairClaimNotes, }),
    ...((sheetFacts !== undefined)
      ? { sheetFacts, }
      : ((referenceContext === undefined)
        ? {}
        : {
          sheetFacts: {
            lifeEnded: false,
            referenceContext,
          },
        })),
    l: (messages === undefined) ? l : capturingLogger({ messages, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  return {
    slices,
    written,
  };
}

/**
 Builds a judged round that settled the way a resume case needs.

 WHOLE AND HONEST rather than cast, because the predicate reads a field OFF
 this object and a fixture narrowed to that field would stop the compiler
 noticing if the field moved or was renamed. Everything else is the emptiest
 value its type admits.

 @param decision - what the judges settled on

 @returns Round shaped as the judge returns one

 @example
 ```ts
 const decided = judgedAs({ decision: 'judged', },);
 ```
 */
function judgedAs(
  { decision, }: { readonly decision: TranslateDecision; },
): TranslateStageResult {
  return {
    text: 'The cat naps in the window.',
    origin: 'incumbent',
    producer: {
      kind: 'incumbent',
      matched: [],
    },
    decision,
    voteWeight: 0,
    tally: {
      judgesAvailable: 0,
      ballots: 0,
      abstentions: 0,
      selfVotes: 0,
    },
    ballots: [],
    heardTranslators: 0,
    candidateCount: 1,
    findings: [],
    slate: [],
    selectedIndex: 0,
    shippedIndex: 0,
    perCandidate: [],
  };
}

/**
 Builds a settlement that left the stage the way a resume case needs.

 ONLY THE FIELDS THE PREDICATE READS are real here. It looks at the terminal,
 at how many ballots the gate could read, and at what the judges decided;
 everything else on a settlement is carried for the record rather than for
 this decision.

 @param terminal - how the slice left the stage

 @param usable - ballots the gate could read, absent where it never ran

 @param decision - what the judges decided, absent where none were asked

 @returns Settlement shaped as the stage returns one

 @example
 ```ts
 const settlement = settlementFor({ terminal: 'consolidated', usable: 4, },);
 ```
 */
function settlementFor(
  {
    terminal,
    usable,
    decision,
  }: {
    readonly terminal: ConsolidationTerminal;
    readonly usable?: number;
    readonly decision?: TranslateDecision;
  },
): ConsolidationSettlement {
  return {
    terminal,
    text: 'The cat naps in the window.',
    floor: {
      kind: 'proposals',
      validModelIds: ['hf:cat/Cat-A',],
    },
    verdicts: [],
    rewrapped: false,
    demoted: false,
    findings: [],
    ...((usable === undefined)
      ? {}
      : {
        gate: {
          choice: 'consolidated',
          ships: 'consolidated',
          ballots: [],
          usable,
          findings: [],
        },
      }),
    ...((decision === undefined) ? {} : { decided: judgedAs({ decision, },), }),
  } as ConsolidationSettlement;
}

/**
 Builds a client that records which roster seat every call asked and answers
 each with nothing a sheet can read, so every voice is lost after it was asked.

 @param asked - sink for the seat of every call, in order

 @returns Client to drive with

 @example
 ```ts
 const client = seatRecordingClient({ asked: [], },);
 ```
 */
function seatRecordingClient(
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
      return {
        kind: 'schema-mismatch',
        rawText: 'no sheet can read this',
        detail: 'fixture answers nothing usable',
      };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Writers and judges a hook hands back after a dry-out, none of them the
 driver's own; three, so a refiner among them can still be selected.
 */
const RESEATED: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 Naturalness roles seated on the given seats, with no guard facts: the first
 refines, and all of them judge and gate, which lets the refiner's text reach
 the selection minimum however it grades its own.

 @param seats - seats the roles sit on

 @returns Polish configuration for a roster

 @example
 ```ts
 const polish = polishOn({ seats: RESEATED, },);
 ```
 */
function polishOn({ seats, }: { readonly seats: readonly RosterModelId[]; },): ConsolidationPolishConfig {
  return {
    refinerModelIds: seats.slice(0, 1,),
    judgeModelIds: seats,
    gateModelIds: seats,
    declaredNames: [],
    definitions: '',
  };
}

/**
 Builds a client that records which seat every call asked and answers each
 producer, slate judge and gate usefully, so the slate is judged and the
 winner gated and polished. Rounds whose sheet takes another answer lose the
 voice after it was asked.

 @param asked - sink for the seat of every call, in order

 @param sheets - sink for everything each call sent, for the cases reading
 what a round was shown

 @returns Client to drive with

 @example
 ```ts
 const client = roleAnsweringClient({ asked: [], },);
 ```
 */
function roleAnsweringClient(
  {
    asked,
    sheets,
  }: {
    readonly asked: RosterModelId[];
    readonly sheets?: string[];
  },
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
       Everything the call sends, which is where the sheet lives.
       */
      const sent = JSON.stringify(request,);
      sheets?.push(sent,);
      /**
       Reply this round is given, told apart as `answeringClient` tells it.
       */
      const rawText = sent.includes('translation_report',)
        ? JSON.stringify({ translation: 'A cat asleep in the sun.', },)
        : (sent.includes(GATE_MARKER,)
          ? JSON.stringify({ verdict: 'standing', unsupported: [], dropped: [], assessment: 'the original supports it', },)
          : JSON.stringify({ best: 1, assessment: 'it says what the original says', },));
      /**
       That reply as the sheet parses it.
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
 Builds a client that answers every stage by its reply schema, so a standing
 reaches the final polish: the refiner rewrites, the selection backs the
 rewrite, and the gates approve it.

 @param sheets - sink for everything each call sent

 @returns Client to drive with

 @example
 ```ts
 const client = polishingClient({ sheets: [], },);
 ```
 */
function polishingClient({ sheets, }: { readonly sheets: string[]; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      sheets.push(JSON.stringify(request,),);
      /**
       Stage the call belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Reply this stage is given.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: 'A cat asleep in the sun.', }
        : (schema === 'refine_report')
          ? { rewrites: [{ paragraph: 1, newText: 'a repair wording for slice 0', },], }
          : (schema === 'consolidation_polish_gate')
            ? { verdict: 'polished', unsupported: [], dropped: [], assessment: 'equally faithful', }
            : (schema === 'absolute_naturalness_review')
              ? { isAcceptable: true, findings: [], assessment: 'publication-ready', }
              : (schema === 'candidate_ballot')
                ? { best: 1, assessment: 'it says what the original says', }
                : { verdict: 'standing', unsupported: [], dropped: [], assessment: 'the original supports it', };
      return request.validate(value,)
        ? { kind: 'ok', value, rawText: JSON.stringify(value,), }
        : { kind: 'schema-mismatch', rawText: JSON.stringify(value,), detail: `fixture does not answer ${schema}`, };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Whether a request is a ballot of the judges who choose among the final polish
 writers' rewrites, whose sheet is the only one that heads the base "CURRENT
 English translation" (ledger B805).

 @param sheet - a request as the client recorded it

 @returns Whether it is such a ballot

 @example
 ```ts
 const judged = sheets.filter(isRewriteJudgeSheet,);
 ```
 */
function isRewriteJudgeSheet(sheet: string,): boolean {
  return sheet.includes('candidate_ballot',) && sheet.includes('CURRENT English translation',);
}

/**
 Drives one contested slice against a cache that answers every key, and
 records the key the driver looked up.

 @param beforeSlice - hook handing the slice its seating, none for a driver
 with no hook

 @returns Keys the driver looked up, in order

 @example
 ```ts
 const keys = await keysLookedUp({},);
 ```
 */
async function keysLookedUp(
  { beforeSlice, }: { readonly beforeSlice?: () => Promise<ConsolidateSliceSeating>; },
): Promise<readonly string[]> {
  /**
   Every key the driver asked the cache for.
   */
  const looked: string[] = [];
  /**
   Cache answering every key with a settlement, so nothing is bought.
   */
  const everyKeyRecorded = {
    get: function recordKey(key: string,): ConsolidationSettlement {
      looked.push(key,);
      return settlementReaching({ terminal: 'incumbent-only', },);
    },
  };
  await driveWith({
    contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
    resumed: everyKeyRecorded as unknown as ReadonlyMap<string, ConsolidationSettlement>,
    ...((beforeSlice === undefined) ? {} : { beforeSlice, }),
  },);
  return looked;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: consolidateDocument.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS NOTHING WHERE THE CONTEST NEVER RAN, which is the majority of most documents: a '
            + 'slice both lanes worded identically has nothing to consolidate, because a third rendering '
            + 'would be competing against their agreement rather than resolving a difference',
          fn: async () => {
            const { slices, written, } = await driveWith({ contests: [], },);

            expect(slices.length,).toBe(0,);
            expect(written.length,).toBe(0,);
          },
        },),

        it({
          name: 'RESUMES AUDITABLE CHANGED POLISH without rebuying any stage',
          fn: async () => {
            /**
             Exact first-slice question driver derives.
             */
            const key = consolidateSliceKey({
              runShape: consolidateRunShape({
                modelIds: ROSTER,
                declaredNamePairs: [],
              },),
              sourceText: '原文0',
              incumbentText: 'archive wording for slice 0',
              repairText: 'repair wording for slice 0',
              translateText: 'translate wording for slice 0',
              standingText: 'repair wording for slice 0',
              ballots: [],
              lineStructured: false,
              pictureContext: '',
              neighbouringSourceText: '',
              neighbouringIncumbentText: '',
            },);
            /**
             Cached settlement whose final polish replaced approved base.
             */
            const settled: ConsolidationSettlement = {
              ...settlementReaching({ terminal: 'gate-kept-standing', },),
              polish: {
                kind: 'settled',
                baseText: 'repair wording for slice 0',
                proposedText: 'polished wording for slice 0',
                text: 'polished wording for slice 0',
                changed: true,
                refinersHeard: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                rounds: [],
                review: {
                  correctionCount: 0,
                  corrections: [],
                  rounds: [],
                  confirmations: [],
                },
                findings: [],
              },
            };
            const messages: string[] = [];
            /**
             How often the driver asked before a slice, which it does even for a
             slice the cache then answers: the wait belongs to the bench, not
             to the purchase.
             */
            const before = { calls: 0, };
            const resumed = await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              resumed: new Map<string, ConsolidationSettlement>([[key, settled,],]),
              messages,
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => {
                before.calls += 1;
                return {};
              },
            },);
            expect(before.calls,).toBe(1,);
            expect(resumed.written,).toEqual([],);
            expect(messages.some(function started(line,): boolean {
              return line.includes(
                `${SLICE_START_MARKER} lane=consolidation chunk=0 sourceChars=3`,
              );
            },),).toBe(true,);
            expect(messages.some(function finished(line,): boolean {
              return line.includes(
                `${SLICE_COST_MARKER} lane=consolidation chunk=0 sourceChars=3`,
              ) && line.endsWith('exit=resumed',);
            },),).toBe(true,);
            expect(resumed.slices[0]?.polish?.kind,).toBe('settled',);
            expect(resumed.slices[0]?.polish?.kind === 'settled'
              ? resumed.slices[0].polish.changed
              : false,).toBe(true,);
          },
        },),

        it({
          name: 'REACHES THE ROSTER FOR A SLICE NOTHING HAS SETTLED, which is the positive control for '
            + 'the "RESUMES AUDITABLE CHANGED POLISH" case. A cache test that never proves the uncached path buys anything '
            + 'would pass just as well against a driver that had stopped calling the roster at all',
          fn: async () => {
            /**
             Operational messages naming unfinished slice exit.
             */
            const messages: string[] = [];
            /**
             Roster that refuses every call and counts it.
             */
            const { client, calls, } = countingRefusingClient();
            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              messages,
            },);

            expect(calls.count,).toBeGreaterThan(0,);
            // BOUGHT, NOT RESUMED: every voice refused, the slice still computed.
            expect(messages.some(function namesComputedExit(line,): boolean {
              return line.includes(`${SLICE_COST_MARKER} lane=consolidation chunk=0`,)
                && line.endsWith('exit=computed',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'REACHES CONSOLIDATION AFTER CONTEST SETTLES NEITHER OR MISSES QUORUM, using archive only as comparison baseline rather than silently reviving it',
          fn: async () => {
            await Promise.all(([
              { kind: 'settled-neither', },
              { kind: 'quorum-not-met', },
            ] as const).map(async function driveVerdict(verdict,): Promise<void> {
              /**
               Roster that refuses every call and counts it.
               */
              const { client, calls, } = countingRefusingClient();
              /**
               What the driver threw, if anything: a roster that answers nothing
               interrupts the entry as an outage.
               */
              const raised: unknown = await (async function captureOutage(): Promise<unknown> {
                try {
                  await driveWith({
                    client,
                    contests: [{
                      sliceIndex: 0,
                      verdict,
                      ballots: [],
                      usable: ROSTER.length,
                    },],
                  },);
                }
                catch (error) {
                  return error;
                }
                return 'no interruption';
              })();
              expect(calls.count,).toBeGreaterThan(0,);
              expect(String(raised,),).toContain('provider-unavailable',);
            },),);
          },
        },),
        it({
          name: 'SETTLES A SOLE-LANE SLICE EXACTLY AS A TRANSLATE WIN, asking the same prompts and shipping the same '
            + 'slice, since the consolidation and its polish run on the lane that applies (ledger L6)',
          fn: async () => {
            /**
             Consolidation over the insertion with the translate lane named by a sole-lane record.
             */
            const soleClient = scriptedClient();
            const sole = await driveWith({
              client: soleClient.client,
              modelIds: WIDE_ROSTER,
              projected: insertionDocument(),
              contests: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'sole-lane',
                  lane: 'translate',
                },
                ballots: [],
                usable: 0,
              },],
            },);
            /**
             Consolidation over the same insertion with the translate lane named by a roster win.
             */
            const won = scriptedClient();
            const wonRun = await driveWith({
              client: won.client,
              modelIds: WIDE_ROSTER,
              projected: insertionDocument(),
              contests: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'lane-won',
                  lane: 'translate',
                },
                ballots: [],
                usable: WIDE_ROSTER.length,
              },],
            },);
            expect(sole.slices,).toEqual(wonRun.slices,);
            expect(soleClient.requests,).toEqual(won.requests,);
            expect(soleClient.requests.length,).toBeGreaterThan(0,);
          },
        },),
        it({
          name: 'SHIPS archive-declined standing after the single attempt, recording non-endorsement without a recovery round',
          fn: async () => {
            const {
              client,
              producerSheets,
              producerPayloads,
            } = scriptedClient();
            const { slices, written, } = await driveWith({
              client,
              modelIds: WIDE_ROSTER,
              contests: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'settled-neither',
                  archive: 'declined',
                },
                ballots: [],
                usable: ROSTER.length,
              },],
            },);

            // One producer round (every producer read the one sheet, so no
            // second strategy was asked for), and the honest gate-kept terminal
            // ships with the non-endorsement recorded.
            expect(new Set(producerSheets,).size,).toBe(1);
            expect(new Set(producerPayloads,).size,).toBe(producerPayloads.length);
            expect(slices[0]?.terminal,).toBe('gate-kept-standing');
            // An unendorsed standing settlement is not worth resuming, so nothing
            // is persisted and a warm run asks again.
            expect(written,).toHaveLength(0);
          },
        },),
        it({
          name: 'SHIPS THE REPAIR LANE\'S TEXT AS THE STANDING at a settled-neither contest on a disputed slice, so the '
            + 'archive rendering the adjudicators found to invent a detail never stands (class one hundred seven, '
            + 'CuspariaKLSY10 slice 3, 2026-09-24)',
          fn: async () => {
            /**
             Repair lane's text for slice 0, standing in for the archive.
             */
            const standIn = 'repair wording for slice 0';
            const { client, } = scriptedClient();
            const { slices, } = await driveWith({
              client,
              modelIds: WIDE_ROSTER,
              contests: [{
                sliceIndex: 0,
                verdict: { kind: 'settled-neither', },
                ballots: [],
                usable: WIDE_ROSTER.length,
              },],
              archiveDisputes: new Map([[0, { sliceIndex: 0, standIn, standInEligible: true, acceptedClaims: ['accuracy/addition major: The cat did not swallow pills.',], },],],),
            },);
            expect(slices[0]?.terminal,).toBe('gate-kept-standing',);
            expect(slices[0]?.shipped,).toEqual({
              kind: 'incumbent',
              text: standIn,
            },);
          },
        },),
        it({
          name: 'SHIPS unjudged settled-neither standing after the single attempt under the same recording',
          fn: async () => {
            const { client, } = scriptedClient();
            const { slices, written, } = await driveWith({
              client,
              modelIds: WIDE_ROSTER,
              contests: [{
                sliceIndex: 0,
                verdict: { kind: 'settled-neither', },
                ballots: [],
                usable: WIDE_ROSTER.length,
              },],
            },);

            expect(slices[0]?.terminal,).toBe('gate-kept-standing');
            expect(written,).toHaveLength(0);
          },
        },),
        it({
          name: 'SHIPS THE SLATE\'S VALID CHOICE when the repair-lane standing dropped a source destination and '
            + 'the gate preferred it: the publisher refuses such a standing (DroppedDestinationError), and the '
            + 'owner\'s 2026-09-04 rule prefers the best valid proposal, failing the slice only where there is '
            + 'none (class one hundred eighty-five)',
          fn: async () => {
            const destination = 'https://example.test/cat-record';
            const sourceText = `[猫猫的记录](${destination})。`;
            const initialText = `[The cat record](${destination}).`;
            const endorsedText = `[The cat's final record](${destination}).`;
            const { client, } = scriptedClient({ initialText, endorsedText, });
            const projected = {
              comparison: [{
                sliceIndex: 0,
                incumbentKind: 'present',
                incumbentText: 'The archive mentions the cat record.',
                repairText: 'The repair lane mentions the cat record.',
                translateText: initialText,
              },],
              delivery: {
                repair: [{ sliceIndex: 0, sourceText, },],
                translate: [],
              },
            } as unknown as ProjectedLanes;
            const { slices, } = await driveWith({
              client,
              modelIds: WIDE_ROSTER,
              projected,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', }),],
            },);
            expect(slices[0]?.terminal,).toBe('consolidated',);
          },
        },),
        it({
          name: 'KEEPS A VALID INCUMBENT where the repair-lane standing dropped a source destination and '
            + 'the archive carried it: the incumbent stands in as the standing, the gate keeps it, and the '
            + 'artifact says the incumbent ships instead of stopping the entry (owner, 2026-09-09, the '
            + 'sixth Mio at slice 3)',
          fn: async () => {
            const destination = 'https://example.test/cat-record';
            const sourceText = `[猫猫的记录](${destination})。`;
            const incumbentText = `[The cat record](${destination}).`;
            const initialText = `[The cat's record](${destination}).`;
            const endorsedText = `[The cat's final record](${destination}).`;
            const { client, } = scriptedClient({ initialText, endorsedText, });
            const projected = {
              comparison: [{
                sliceIndex: 0,
                incumbentKind: 'present',
                incumbentText,
                repairText: 'The repair lane mentions the cat record.',
                translateText: initialText,
              },],
              delivery: {
                repair: [{ sliceIndex: 0, sourceText, },],
                translate: [],
              },
            } as unknown as ProjectedLanes;
            const { slices, } = await driveWith({
              client,
              modelIds: WIDE_ROSTER,
              projected,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', }),],
            },);
            expect(slices[0]?.terminal,).toBe('gate-kept-standing',);
            // THE ARTIFACT SAYS THE INCUMBENT SHIPS. A bare "unchanged" here would
            // have the page assembled from the lane's wording, which is the text
            // the gate refused.
            expect(slices[0]?.shipped,).toEqual({
              kind: 'incumbent',
              text: incumbentText,
            },);
          },
        },),
        it({
          name: 'SHIPS THE ARCHIVE where the floor can compare nothing, an original the strict grammar cannot read, '
            + 'whichever lane the contest backed: both lanes\' texts and the archive are refused alike, the archive '
            + 'stands, and no writer, judge or gate is asked (ledger B43, B45)',
          fn: async () => {
            /**
             Original with an expression the strict grammar never closes.
             */
            const sourceText = '猫猫在{窗台上打盹。';
            /**
             Every settlement, one per lane the contest backed.
             */
            const settled = await Promise.all((['repair', 'translate',] as const).map(async function settleFor(lane,) {
              const {
                client,
                requests,
              } = scriptedClient({
                initialText: 'The cat dozes on the windowsill.',
                endorsedText: 'The cat naps on the windowsill.',
              },);
              const messages: string[] = [];
              const projected = {
                comparison: [{
                  sliceIndex: 0,
                  incumbentKind: 'present',
                  incumbentText: 'The cat is doing the dozing on the windowsill.',
                  repairText: 'The cat dozed on the sill.',
                  translateText: 'The cat dozes on the windowsill.',
                },],
                delivery: {
                  repair: [{ sliceIndex: 0, sourceText, },],
                  translate: [],
                },
              } as unknown as ProjectedLanes;
              const { slices, } = await driveWith({
                client,
                modelIds: WIDE_ROSTER,
                projected,
                contests: [contestSettling({ sliceIndex: 0, lane, }),],
                messages,
              },);
              return {
                terminal: slices[0]?.terminal,
                shipped: slices[0]?.shipped,
                requests: requests.length,
                refusedBoth: messages.some(function refusesBoth(line,): boolean {
                  return line.includes('withheld from the slate: no comparison was possible: original could not be read',)
                    && line.includes('the incumbent fails it too: no comparison was possible',);
                },),
                askedNobody: messages.some(function namesTheReason(line,): boolean {
                  return line.includes('the floor can compare nothing here (original could not be read',)
                    && line.includes('so no writer is asked',);
                },),
              };
            },),);
            expect(settled,).toEqual([
              {
                terminal: 'incumbent-only',
                shipped: {
                  kind: 'archive',
                  reason: 'no-wording-passed',
                },
                requests: 0,
                refusedBoth: true,
                askedNobody: true,
              },
              {
                terminal: 'incumbent-only',
                shipped: {
                  kind: 'archive',
                  reason: 'no-wording-passed',
                },
                requests: 0,
                refusedBoth: true,
                askedNobody: true,
              },
            ],);
          },
        },),
        it({
          name: 'ASKS identical unsafe twins once and ships both with the recorded non-endorsement',
          fn: async () => {
            const {
              client,
              producerPayloads,
            } = scriptedClient();
            const { slices, written, } = await driveWith({
              client,
              modelIds: WIDE_ROSTER,
              projected: twinSliceDocument(),
              contests: [
                {
                  sliceIndex: 0,
                  verdict: { kind: 'settled-neither', archive: 'declined', },
                  ballots: [],
                  usable: WIDE_ROSTER.length,
                },
                {
                  sliceIndex: 1,
                  verdict: { kind: 'settled-neither', archive: 'declined', },
                  ballots: [],
                  usable: WIDE_ROSTER.length,
                },
              ],
              overlap: 2,
            },);

            // Unsafe questions are never twin-memoized, so each twin buys its
            // own single attempt, one window of producers, and both ship the
            // recorded outcome.
            expect(producerPayloads,).toHaveLength(firstRoundWindow({ benchSize: WIDE_ROSTER.length, },) * 2,);
            expect(slices,).toHaveLength(2);
            expect(slices.every(function keptWithRecord(slice,): boolean {
              return slice.terminal === 'gate-kept-standing';
            },),).toBe(true,);
            expect(written,).toHaveLength(0);
          },
        },),
        it({
          name: 'RECORDS a declined archive that the single attempt kept, never pausing the entry',
          fn: async () => {
            const { client, judgeSheets, } = answeringClient();
            const writes: string[] = [];
            const { slices, } = await driveWith({
              client,
              writes,
              contests: [{
                sliceIndex: 0,
                verdict: {
                  kind: 'settled-neither',
                  archive: 'declined',
                },
                ballots: [],
                usable: ROSTER.length,
              },],
            },);

            expect(slices[0]?.terminal,).toBe('slate-declined-standing');
            expect(judgeSheets.every(function carriesArchiveBaseline(sheet,): boolean {
              return sheet.includes('archive wording for slice 0',);
            },),).toBe(true,);
            expect(writes,).toEqual([]);
          },
        },),
        it({
          name: 'PAUSES provider-silent unsafe standing as unavailable without persisting it',
          fn: async () => {
            const { client, } = recordingClient();
            const writes: string[] = [];
            let thrown: unknown;
            try {
              await driveWith({
                client,
                writes,
                contests: [{
                  sliceIndex: 0,
                  verdict: { kind: 'settled-neither', archive: 'declined', },
                  ballots: [],
                  usable: ROSTER.length,
                },],
              },);
            }
            catch (error) {
              thrown = error;
            }

            expect(thrown,).toBeInstanceOf(TranslationRepairInterruptedError,);
            expect((thrown as TranslationRepairInterruptedError).reason,).toBe('provider-unavailable');
            expect(writes,).toEqual([]);
          },
        },),
        it({
          name: 'REFUSES A CONTESTED SLICE MISSING FROM THE REPAIR LEDGER rather than consolidating '
            + 'against no original, because the comparison and the ledger disagreeing about which slices '
            + 'exist is a defect upstream and settling one of them anyway would hide it',
          fn: async () => {
            /**
             A document whose comparison names a slice the ledger does not.
             */
            const gapped = {
              ...twoSliceDocument(),
              delivery: {
                repair: [],
                translate: [],
              },
            };

            /**
             What the driver did instead of returning.
             */
            let raised: unknown;
            try {
              await driveWith({
                contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
                projected: gapped,
              },);
            } catch (error: unknown) {
              raised = error;
            }

            expect(raised,).toBeInstanceOf(Error,);
            expect(String(raised,).includes('does not appear in the repair ledger',),).toBe(true,);
          },
        },),

        it({
          name: 'RETURNS ONE RECORD PER CONSOLIDATED SLICE IN COMPARISON ORDER, resuming both without '
            + 'buying, so a reader of the artifact can line records up against the comparison rows they '
            + 'answer rather than re-deriving the order',
          fn: async () => {
            /**
             Both slices settled by an earlier run, keyed by whatever the driver
             asks for: the cache here answers every key.
             */
            const everyKey = {
              get: function answerAnyKey() {
                return settlementReaching({ terminal: 'incumbent-only', },);
              },
            };

            const { slices, written, } = await driveWith({
              contests: [
                contestSettling({ sliceIndex: 0, lane: 'repair', },),
                contestSettling({ sliceIndex: 1, lane: 'translate', },),
              ],
              resumed: everyKey as unknown as ReadonlyMap<string, ConsolidationSettlement>,
            },);

            expect(slices.length,).toBe(2,);
            expect(slices[0]?.sliceIndex,).toBe(0,);
            expect(slices[1]?.sliceIndex,).toBe(1,);
            expect(slices[0]?.terminal,).toBe('incumbent-only',);
            expect(slices[0]?.gate.kind,).toBe('not-asked',);
            expect(written.length,).toBe(0,);
          },
        },),

        it({
          name: 'RECORDS THE TENSE CANDIDATES OF THE WORDING A SLICE SHIPPED when the entry records a death, '
            + 'and records none where it records none or where the slice ships no wording of its own',
          fn: async () => {
            /**
             Both slices resumed as shipping the present-tense consolidation.
             */
            const shipping = {
              get: function answerAnyKey() {
                return {
                  ...settlementReaching({ terminal: 'consolidated', },),
                  text: 'She naps in the window.',
                };
              },
            };
            /**
             What the driver recorded for an entry that records a death.
             */
            const recorded = await driveWith({
              contests: [
                contestSettling({ sliceIndex: 0, lane: 'repair', },),
                contestSettling({ sliceIndex: 1, lane: 'translate', },),
              ],
              resumed: shipping as unknown as ReadonlyMap<string, ConsolidationSettlement>,
              sheetFacts: {
                lifeEnded: true,
                subjectNames: ['The cat',],
              },
            },);
            expect(recorded.slices.map(function findingOf(slice,) {
              return slice.tenseCandidates;
            },),).toEqual([
              [
                {
                  label: 'CANDIDATE "consolidated"',
                  found: [
                    {
                      sentence: 'She naps in the window.',
                      matched: 'She naps',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
              [
                {
                  label: 'CANDIDATE "consolidated"',
                  found: [
                    {
                      sentence: 'She naps in the window.',
                      matched: 'She naps',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
            ],);
            /**
             What the driver recorded for an entry that records none.
             */
            const unrecorded = await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              resumed: shipping as unknown as ReadonlyMap<string, ConsolidationSettlement>,
              sheetFacts: { lifeEnded: false, },
            },);
            expect(unrecorded.slices.map(function hasField(slice,): boolean {
              return 'tenseCandidates' in slice;
            },),).toEqual([false,],);
          },
        },),

        it({
          name: 'REFUSES persistence under an already aborted caller after a stable '
            + 'consolidation returned, preserving the final pre-write defense',
          fn: async () => {
            /**
             Exact caller reason helper must surface.
             */
            const stopped = new Error('caller abandoned completed consolidation',);
            const controller = new AbortController();
            controller.abort(stopped,);

            /**
             Writes attempted after abort.
             */
            const written: string[] = [];
            await expect(persistConsolidationSettlement({
              standingMayShip: true,
              key: 'consolidation-persistence-guard-fixture',
              settlement: settlementReaching({ terminal: 'incumbent-only', },),
              cache: {
                resumed: new Map<string, ConsolidationSettlement>(),
                persist: async ({ key, },) => {
                  written.push(key,);
                },
              },
              signal: controller.signal,
            },),)
              .rejects
              .toBe(stopped,);
            expect(written,).toEqual([],);
          },
        },),

        it({
          name: 'runs two consolidation slices at once at overlap 2, after a serial '
            + 'positive control proves the successful-call instrument distinguishes one from two, '
            + 'and returns records in comparison order when the second producer answers first',
          fn: async () => {
            /**
             Serial positive-control activity.
             */
            const serial: ConsolidationConcurrency = {
              now: 0,
              peak: 0,
              started: 0,
              finished: [],
            };
            const serialClient = recordingClient();
            await driveWith({
              client: serialClient.client,
              contests: [
                contestSettling({ sliceIndex: 0, lane: 'repair', },),
                contestSettling({ sliceIndex: 1, lane: 'translate', },),
              ],
              overlap: 1,
              activity: serial,
            },);

            /**
             Two-slice activity, gated so the second call answers first.
             */
            const overlapped: ConsolidationConcurrency = {
              now: 0,
              peak: 0,
              started: 0,
              finished: [],
              secondFinished: Promise.withResolvers<undefined>(),
            };
            const overlapClient = recordingClient();
            const { slices, } = await driveWith({
              client: overlapClient.client,
              contests: [
                contestSettling({ sliceIndex: 0, lane: 'repair', },),
                contestSettling({ sliceIndex: 1, lane: 'translate', },),
              ],
              overlap: 2,
              activity: overlapped,
            },);
            expect(serial.peak,).toBe(1,);
            expect(overlapped.peak,).toBe(2,);
            // THE SECOND PRODUCER DID ANSWER FIRST: the call started first is not
            // the first to finish, so the check that the slices come back in
            // comparison order is exercised.
            expect(overlapped.finished,).toContain(0,);
            expect(overlapped.finished[0],).not.toBe(0,);
            expect(slices.map(function toIndex(slice,) {
              return slice.sliceIndex;
            },),).toEqual([
              0,
              1,
            ],);
          },
        },),

        it({
          name: 'mixes one resumed row with one fresh row at overlap 2, buying and persisting '
            + 'only the fresh consolidation while returning both in comparison order',
          fn: async () => {
            /**
             Fresh pass used only to derive production keys for both rows.
             */
            const learningClient = recordingClient();
            const learned = await driveWith({
              client: learningClient.client,
              contests: [
                contestSettling({ sliceIndex: 0, lane: 'repair', },),
                contestSettling({ sliceIndex: 1, lane: 'translate', },),
              ],
            },);

            /**
             Second pass resuming first row and buying second.
             */
            const freshClient = recordingClient();
            const mixed = await driveWith({
              client: freshClient.client,
              contests: [
                contestSettling({ sliceIndex: 0, lane: 'repair', },),
                contestSettling({ sliceIndex: 1, lane: 'translate', },),
              ],
              resumed: new Map([
                [
                  learned.written.at(0,) ?? '',
                  settlementReaching({ terminal: 'incumbent-only', },),
                ],
              ],),
              overlap: 2,
            },);
            expect(freshClient.bodies.length,).toBeGreaterThan(0,);
            expect(mixed.written.length,).toBe(1,);
            expect(mixed.slices.map(function toIndex(slice,) {
              return slice.sliceIndex;
            },),).toEqual([
              0,
              1,
            ],);
          },
        },),

        it({
          name: 'NAMES THE SLICE on every line the settlement writes (class one hundred eighty-five: '
            + 'TianqiChen66619 consolidated slices in parallel and its slate and gate ballots named no slice, '
            + 'so slice 9\'s ballots could not be told from its neighbours\')',
          fn: async () => {
            /**
             Operational messages of one bought slice.
             */
            const messages: string[] = [];
            await driveWith({
              client: recordingClient().client,
              projected: twinSliceDocument(),
              contests: [contestSettling({ sliceIndex: 1, lane: 'repair', },),],
              messages,
            },);
            /**
             Lines the slice's purchase wrote: its producers, slate, gate and
             settlement.
             */
            const settling = messages.filter(function fromPurchase(line,): boolean {
              return [
                '[settleConsolidation]',
                '[produceConsolidations]',
                '[judgeTranslateSlate]',
                '[gateConsolidatedSlice]',
              ].some(function carries(tag,): boolean {
                return line.includes(tag,);
              },);
            },);
            expect(settling.length,).toBeGreaterThan(0,);
            expect(settling.every(function namesSlice(line,): boolean {
              return line.includes('[slice 1]',);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'ASKS ONCE for two slices carrying the same consolidation question at overlap 1 and 2, '
            + 'so cold and warm runs settle one reusable third rendering',
          fn: async () => {
            await Promise.all(([1, 2,] as const).map(async function atOverlap(
              overlap,
            ): Promise<void> {
              /**
               Calls one copy of this question costs, measured rather than assumed
               because malformed structured replies are retried by transport.
               */
              const singleFixture = recordingClient();
              const single = await driveWith({
                client: singleFixture.client,
                projected: twinSliceDocument(),
                contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
                overlap,
              },);

              /**
               Same question stamped at both positions.
               */
              const twinFixture = recordingClient();
              const messages: string[] = [];
              const { slices, written, } = await driveWith({
                client: twinFixture.client,
                projected: twinSliceDocument(),
                contests: [
                  contestSettling({ sliceIndex: 0, lane: 'repair', },),
                  contestSettling({ sliceIndex: 1, lane: 'repair', },),
                ],
                overlap,
                messages,
              },);
              expect(singleFixture.bodies.length,).toBeGreaterThan(0,);
              expect(twinFixture.bodies.length,).toBe(singleFixture.bodies.length,);
              expect(single.written.length,).toBe(1,);
              expect(written.length,).toBe(1,);
              expect(messages.filter(function computed(line,): boolean {
                return line.includes(`${SLICE_COST_MARKER} lane=consolidation`,)
                  && line.endsWith('exit=computed',);
              },),).toHaveLength(1,);
              expect(messages.filter(function reused(line,): boolean {
                return line.includes(`${SLICE_COST_MARKER} lane=consolidation`,)
                  && line.endsWith('exit=reused',);
              },),).toHaveLength(1,);
              expect(slices.map(function toIndex(slice,) {
                return slice.sliceIndex;
              },),).toEqual([
                0,
                1,
              ],);
            },),);
          },
        },),

        it({
          name: 'ASKS AGAIN for a twin whose consolidation gate did not reach quorum at overlap 1 and 2, '
            + 'because the in-run memo may hold only what a warm run can resume',
          fn: async () => {
            await Promise.all(([1, 2,] as const).map(async function atOverlap(
              overlap,
            ): Promise<void> {
              /**
               One unsettled consolidation as asks positive control.
               */
              const singleActivity: ConsolidationConcurrency = {
                now: 0,
                peak: 0,
                started: 0,
                finished: [],
              };
              const singleClient = answeringClient();
              const single = await driveWith({
                client: singleClient.client,
                projected: twinSliceDocument(),
                contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
                overlap,
                activity: singleActivity,
              },);

              /**
               Same unsettled question at both positions.
               */
              const twinActivity: ConsolidationConcurrency = {
                now: 0,
                peak: 0,
                started: 0,
                finished: [],
              };
              const twinClient = answeringClient();
              const twin = await driveWith({
                client: twinClient.client,
                projected: twinSliceDocument(),
                contests: [
                  contestSettling({ sliceIndex: 0, lane: 'repair', },),
                  contestSettling({ sliceIndex: 1, lane: 'repair', },),
                ],
                overlap,
                activity: twinActivity,
              },);
              expect(singleActivity.started,).toBeGreaterThan(0,);
              expect(single.written,).toEqual([],);
              expect(twinActivity.started,).toBe(singleActivity.started * 2,);
              expect(twin.written,).toEqual([],);
            },),);
          },
        },),

        it({
          name: 'SHOWS A GOVERNED SLICE\'S PRODUCERS THE RULE AGAINST MERGING LINES, which is the only '
            + 'reason this driver is handed the governed set at all. The subject field carrying the fact '
            + 'was born optional on 2026-08-21 and no caller ever set it, so until 2026-08-22 every '
            + 'consolidating producer was told its passage was prose, verse included',
          fn: async () => {
            const { client, bodies, } = recordingClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              lineStructuredSlices: new Set([0,],),
            },);

            expect(bodies.length,).toBeGreaterThan(0,);
            expect(bodies.some(function carriesRule(body,): boolean {
              return body.includes(TRANSLATE_LINE_STRUCTURE_RULE,);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES THE RULE OUT OF A SLICE IT DOES NOT GOVERN, which is what makes the "SHOWS A GOVERNED '
            + 'SLICE\'S PRODUCERS THE RULE AGAINST MERGING LINES" case evidence. A sheet carrying the rule unconditionally would satisfy that one just as well, '
            + 'and would mean the governed set was never read',
          fn: async () => {
            const { client, bodies, } = recordingClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              lineStructuredSlices: new Set(),
            },);

            expect(bodies.length,).toBeGreaterThan(0,);
            expect(bodies.some(function carriesRule(body,): boolean {
              return body.includes(TRANSLATE_LINE_STRUCTURE_RULE,);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'SHOWS A PRODUCER WHAT THE PICTURES NEAR ITS SLICE WERE READ TO SAY, which the sheet has '
            + 'rendered since it was written and no caller ever supplied. A passage whose meaning leans '
            + 'on an image was consolidated blind until 2026-08-22, while the translate lane that wrote '
            + 'one of the candidates had been shown the same reading all along',
          fn: async () => {
            const { client, bodies, } = recordingClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              pictureContextBySlice: new Map([[0, TABBY_READING,],],),
            },);

            expect(bodies.length,).toBeGreaterThan(0,);
            expect(bodies.some(function carriesReading(body,): boolean {
              return body.includes(TABBY_READING,);
            },),).toBe(true,);
          },
        },),

        it({
          name: 'LEAVES A SLICE THE MAP NEVER MENTIONS UNILLUSTRATED, which is what makes the "SHOWS A '
            + 'PRODUCER WHAT THE PICTURES NEAR ITS SLICE WERE READ TO SAY" case evidence rather than a sheet that always carries a picture heading. It pins the lookup too, '
            + 'which is why the map here holds a reading for a slice this contest never settles: a driver '
            + 'reading a neighbouring slice\'s entry finds that reading and puts it on the sheet, and both '
            + 'picture cases fail together. A slice index naming three different things is why that is '
            + 'the defect worth pinning',
          fn: async () => {
            const { client, bodies, } = recordingClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              pictureContextBySlice: new Map([[1, TABBY_READING,],],),
            },);

            expect(bodies.length,).toBeGreaterThan(0,);
            expect(bodies.some(function carriesReading(body,): boolean {
              return body.includes(TABBY_READING,);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'SHOWS THE SLATE JUDGES WHAT THE PICTURES NEAR THEIR SLICE SAY. One change gave the readings to '
            + 'this stage\'s PRODUCERS and stopped, so for one day the judges weighed proposals written '
            + 'against evidence they could not see, which is worse than both halves being blind: a producer '
            + 'that used a picture correctly looked to its judge like one inventing detail. Read off the '
            + 'judges\' own request, which is the only place the wiring is visible',
          fn: async () => {
            const { client, judgeSheets, } = answeringClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              pictureContextBySlice: new Map([[0, 'The photograph shows a tortoiseshell asleep in a sunlit doorway.',],],),
            },);

            expect(judgeSheets.length,).toBeGreaterThan(0,);
            expect(
              judgeSheets.every(function carriesReading(sheet,): boolean {
                return sheet.includes('tortoiseshell asleep in a sunlit doorway',);
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'SHOWS THEM THE PASSAGES EITHER SIDE, which neither half of this stage has ever been given. '
            + 'The translate lane\'s judges have had the window since the relocation finding, and a consolidation judge '
            + 'without it cannot tell a passage the archive moved next door from one a candidate invented. '
            + 'Kept apart from the "SHOWS THE SLATE JUDGES WHAT THE PICTURES NEAR THEIR SLICE SAY" case so a '
            + 'break in one is not read as a break in the other',
          fn: async () => {
            const { client, judgeSheets, } = answeringClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              neighbourContextBySlice: new Map([[
                0,
                {
                  sourceText: '她把窗户推开了一条缝。',
                  incumbentText: 'She pushed the window open a crack.',
                },
              ],],),
            },);

            expect(judgeSheets.length,).toBeGreaterThan(0,);
            expect(
              judgeSheets.every(function carriesWindow(sheet,): boolean {
                return sheet.includes('她把窗户推开了一条缝',)
                  && sheet.includes('She pushed the window open a crack',);
              },),
            ).toBe(true,);
          },
        },),

        it({
          name: 'SHOWS THEM NEITHER WHEN THE DRIVER HOLDS NEITHER, which is the control the "SHOWS THEM THE '
            + 'PASSAGES EITHER SIDE" case needs. '
            + 'A sheet that rendered these blocks unconditionally would satisfy that one just as well and would '
            + 'mean the two maps were never read, and a slice near no readable picture would be shown a '
            + 'heading promising readings it does not have',
          fn: async () => {
            const { client, judgeSheets, } = answeringClient();

            await driveWith({
              client,
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
            },);

            expect(judgeSheets.length,).toBeGreaterThan(0,);
            expect(
              judgeSheets.some(function carriesEvidence(sheet,): boolean {
                return sheet.includes('tortoiseshell',)
                  || sheet.includes('她把窗户推开了一条缝',)
                  || sheet.includes('She pushed the window open a crack',);
              },),
            ).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: consolidationWorthResuming.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES TO CACHE A GATE TOO THIN TO SETTLE, which is the whole reason this predicate '
            + 'exists: a night when one voice of six answered is a fact about a provider, not about the '
            + 'question, and writing it would answer every later resume of the entry with that night. The '
            + 'gate refuses to act below its quorum, so a cache that kept the result would preserve a '
            + 'verdict the gate itself declined to reach',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'gate-kept-standing', usable: 1, },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'CACHES A GATE THAT REACHED ITS QUORUM, which is the positive control: a predicate that '
            + 'refused everything would pass the "REFUSES TO CACHE A GATE TOO THIN TO SETTLE" case while making '
            + 'every run re-buy every slice it '
            + 'had already settled',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'consolidated', usable: 2, },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES TO CACHE UNCHANGED UNENDORSED BASELINE while retaining fresh consolidated result, so retry can recover instead of replaying final-selection failure',
          fn: async () => {
            expect(consolidationWorthResuming({
              settlement: settlementFor({ terminal: 'gate-kept-standing', usable: 2, },),
              standingMayShip: false,
            },),).toBe(false,);
            expect(consolidationWorthResuming({
              settlement: settlementFor({ terminal: 'consolidated', usable: 2, },),
              standingMayShip: false,
            },),).toBe(true,);
            expect(consolidationWorthResuming({
              settlement: {
                ...settlementFor({ terminal: 'gate-kept-standing', usable: 2, },),
                polish: {
                  kind: 'settled',
                  baseText: 'The cat faced life proactively.',
                  proposedText: 'The cat maintained a positive outlook on life.',
                  text: 'The cat maintained a positive outlook on life.',
                  changed: true,
                  refinersHeard: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                  contributors: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                  rounds: [],
                  review: {
                    correctionCount: 0,
                    corrections: [],
                    rounds: [],
                    confirmations: [],
                  },
                  findings: [],
                },
              },
              standingMayShip: false,
            },),).toBe(true,);
          },
        },),

        it({
          name: 'REFUSES TO CACHE A GATE\'S TERMINAL WITH NO GATE RECORDED, which the settlement type admits and no '
            + 'settlement builder produces: without the ballots there is no quorum to trust',
          fn: async () => {
            expect((['consolidated', 'gate-kept-standing', 'wrap-erased-difference',] as const).map(
              function worthResuming(terminal,): boolean {
                return consolidationWorthResuming({
                  standingMayShip: true,
                  settlement: settlementFor({ terminal, },),
                },);
              },
            ),).toEqual([false, false, false,],);
          },
        },),

        it({
          name: 'REFUSES TO CACHE A DECLINED SLATE WITH NO DECISION RECORDED, which the settlement type admits and no '
            + 'settlement builder produces (the absence exit ledger B51 removed built one, marked archive-kept): '
            + 'without the decision there is no telling a settled decline from a pending one',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'slate-declined-standing', },),
            },),).toBe(false,);
          },
        },),

        it({
          name: 'CACHES A SLICE STOPPED BEFORE THE GATE BY THE SLATE OR THE CONTEST, because neither is a '
            + 'fact about who answered. A floor that refused every proposal read the structural guard, and '
            + 'a contest that named neither lane left nothing to improve on; both hold on any night',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'incumbent-only', },),
            },),).toBe(true,);
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'no-standing-text', },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'NEVER CACHES A SLICE THAT KEPT THE ARCHIVE because no wording passed the rule (owner, '
            + '2026-09-27, "Keep archive, ship"). The mark saying so lives on the settlement a run bought, so '
            + 'a copy resumed without it would ship the lane text the rule refused. Refused even where the '
            + 'standing claims endorsement, so this rests on no coupling two files away',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: {
                ...settlementFor({ terminal: 'incumbent-only', },),
                archiveKept: true,
              },
            },),).toBe(false,);
            expect(consolidationWorthResuming({
              settlement: {
                ...settlementFor({ terminal: 'no-standing-text', },),
                archiveKept: true,
              },
              standingMayShip: false,
            },),).toBe(false,);
          },
        },),

        it({
          name: 'REFUSES TO CACHE JUDGES THAT DECLINED TO SETTLE, and caches judges that decided, which is '
            + 'the same distinction one stage earlier: translate-retry.ts buys a second judging for '
            + 'exactly declined-indecision and declined-rejection and records the settled decline under a '
            + 'different name. A predicate keyed on which text won rather than on whether they settled '
            + 'would freeze an undecided panel',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'slate-declined-standing', decision: 'declined-indecision', },),
            },),).toBe(false,);
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'slate-declined-standing', decision: 'declined-rejection', },),
            },),).toBe(false,);
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'slate-endorsed-standing', decision: 'judged', },),
            },),).toBe(true,);

            // THE SETTLED DECLINE, which is why the decision read survived the
            // terminal split: it shares its `slate-declined-standing` terminal with
            // the `declined-indecision` and `declined-rejection` settlements and gets
            // the opposite answer, so the name alone cannot decide this one.
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'slate-declined-standing', decision: 'no-candidate-backed', },),
            },),).toBe(true,);
          },
        },),

        it({
          name: 'CACHES A SLATE NO JUDGE WAS ASKED ABOUT, which is the state the merged name hid '
            + 'worst. A slate carrying one candidate measures the PRODUCING half and says nothing '
            + 'about the roster, so counting it beside an endorsement of the archive reported a '
            + 'working panel where no panel spoke',
          fn: async () => {
            expect(consolidationWorthResuming({
              standingMayShip: true,
              settlement: settlementFor({ terminal: 'slate-unjudged-standing', decision: 'sole-candidate', },),
            },),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: `${consolidateDocument.name} re-seated under a hold (ledger H5)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEATS A SLICE ON THE ROSTER ITS HOOK RETURNS, as both lanes seat theirs, so writers re-read '
            + 'after a provider dry-out are the ones the slice asks rather than the roster read before it',
          fn: async () => {
            /**
             Seat of every call the driver made.
             */
            const asked: RosterModelId[] = [];
            await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              client: seatRecordingClient({ asked, },),
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({
                roster: {
                  modelIds: RESEATED,
                  judgeModelIds: RESEATED,
                },
              }),
            },);
            expect(asked.length,).toBeGreaterThan(0,);
            expect(asked.filter(function outsideRoster(seat,): boolean {
              return !RESEATED.includes(seat,);
            },),).toEqual([],);
          },
        },),

        it({
          name: 'SEATS THE SLATE JUDGES AND NATURALNESS ROLES ITS HOOK RETURNS TOO, after a control proves a '
            + 'slice with no hook asks the starting judges and polish roles, each on a seat no other role holds',
          fn: async () => {
            /**
             Starting roster: writers, judges and polish roles each on their own seat.
             */
            const starting = {
              judgeModelIds: [SEAT_HYPER_ONLY,],
              polishConfig: polishOn({
                seats: [
                  SEAT_OPENROUTER_ONLY,
                  SEAT_OPENROUTER_DECISIONS,
                  SEAT_OPENROUTER_ONLY_CHECKER,
                ],
              },),
            };
            /**
             Seat of every call a driver with no hook made.
             */
            const control: RosterModelId[] = [];
            await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              client: roleAnsweringClient({ asked: control, },),
              ...starting,
            },);
            /**
             Seat of every call the re-seated driver made.
             */
            const asked: RosterModelId[] = [];
            await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              client: roleAnsweringClient({ asked, },),
              ...starting,
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({
                roster: {
                  modelIds: RESEATED,
                  judgeModelIds: RESEATED,
                  polishConfig: polishOn({ seats: RESEATED, },),
                },
              }),
            },);
            expect({
              controlAskedJudges: control.includes(SEAT_HYPER_ONLY,),
              controlAskedPolish: control.includes(SEAT_OPENROUTER_ONLY,),
              reseatedAskedAny: asked.length > 0,
              outsideRoster: asked.filter(function outsideRoster(seat,): boolean {
                return !RESEATED.includes(seat,);
              },),
            },).toEqual({
              controlAskedJudges: true,
              controlAskedPolish: true,
              reseatedAskedAny: true,
              outsideRoster: [],
            },);
          },
        },),

        it({
          name: 'KEYS A RE-SEATED SLICE BY THE ROSTER IT RUNS ON, so a settlement the roster read before the '
            + 'dry-out reached is never resumed for it, while a hook handing back the starting roster keys '
            + 'the slice as a driver with no hook does',
          fn: async () => {
            /**
             Keys a driver with no hook looks up.
             */
            const starting = await keysLookedUp({},);
            /**
             Keys looked up when the hook re-seats the slice elsewhere.
             */
            const moved = await keysLookedUp({
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({
                roster: {
                  modelIds: RESEATED,
                  judgeModelIds: RESEATED,
                },
              }),
            },);
            /**
             Keys looked up when the hook hands back the roster the driver started on.
             */
            const kept = await keysLookedUp({
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({
                roster: {
                  modelIds: ROSTER,
                  judgeModelIds: ROSTER,
                },
              }),
            },);
            /**
             Keys looked up when the hook keeps the writers and judges but seats
             naturalness roles the driver started without.
             */
            const polished = await keysLookedUp({
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({
                roster: {
                  modelIds: ROSTER,
                  judgeModelIds: ROSTER,
                  polishConfig: polishOn({ seats: RESEATED, },),
                },
              }),
            },);
            expect({
              lookups: [starting.length, moved.length, kept.length, polished.length,],
              movedDiffers: moved[0] !== starting[0],
              keptMatches: kept[0] === starting[0],
              polishedDiffers: polished[0] !== starting[0],
            },).toEqual({
              lookups: [1, 1, 1, 1,],
              movedDiffers: true,
              keptMatches: true,
              polishedDiffers: true,
            },);
          },
        },),

        it({
          name: 'BUYS NOTHING FOR A SLICE whose hook reports fewer reachable slate judges than the ballots a '
            + 'winner needs, keeps the standing as the incumbent-only exit does, and writes nothing to the '
            + 'cache, so an unsettled entry asks the slice again in a later pass (XingZ6020: 54 slices, 164 calls)',
          fn: async () => {
            /**
             Seat of every call the driver made under a bench below the floor.
             */
            const asked: RosterModelId[] = [];
            const { slices, written, } = await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              client: seatRecordingClient({ asked, },),
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({ slateReachable: 1, }),
            },);
            expect({
              calls: asked.length,
              terminal: slices[0]?.terminal,
              written: written.length,
            },).toEqual({
              calls: 0,
              terminal: 'incumbent-only',
              written: 0,
            },);
          },
        },),

        it({
          name: 'ASKS A SLICE AS BEFORE once its hook reports the slate floor reachable',
          fn: async () => {
            /**
             Seat of every call the driver made with the floor met.
             */
            const asked: RosterModelId[] = [];
            await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              client: seatRecordingClient({ asked, },),
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({ slateReachable: 2, }),
            },);
            expect(asked.length,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'STILL POLISHES AN ENDORSED STANDING when the slate bench is below the floor, since the final '
            + 'naturalness check refuses a body slice with no settled polish: the writers and the slate judges '
            + 'are not asked, and the naturalness roles are',
          fn: async () => {
            /**
             Seat of every call the driver made.
             */
            const asked: RosterModelId[] = [];
            await driveWith({
              contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
              client: roleAnsweringClient({ asked, },),
              judgeModelIds: [SEAT_HYPER_ONLY,],
              polishConfig: polishOn({
                seats: [
                  SEAT_OPENROUTER_ONLY,
                  SEAT_OPENROUTER_DECISIONS,
                  SEAT_OPENROUTER_ONLY_CHECKER,
                ],
              },),
              beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({ slateReachable: 1, }),
            },);
            /**
             Writers the driver started on, as a plain list of seats.
             */
            const writers: readonly RosterModelId[] = ROSTER;
            expect({
              askedWriter: asked.some(function isWriter(seat,): boolean {
                return writers.includes(seat,);
              },),
              askedSlateJudge: asked.includes(SEAT_HYPER_ONLY,),
              askedPolish: asked.includes(SEAT_OPENROUTER_ONLY,),
            },).toEqual({
              askedWriter: false,
              askedSlateJudge: false,
              askedPolish: true,
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
             Why the caller stopped the consolidation.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             What the driver refused with.
             */
            const refusal = await rejectionOf(async function consolidatesUnderTheStop(): Promise<unknown> {
              return await driveWith({
                contests: [
                  contestSettling({
                    sliceIndex: 0,
                    lane: 'repair',
                  },),
                  contestSettling({
                    sliceIndex: 1,
                    lane: 'translate',
                  },),
                ],
                client: UNASKED_CLIENT,
                overlap: 2,
                beforeSlice: stopLandingHook<ConsolidateSliceSeating>({
                  steering,
                  reason,
                  afterStop: throwBenchUnreachable,
                },),
                messages,
                signal: steering.signal,
              },);
            },);
            expect(refusal,).toBe(reason,);
            expect(messages,).toEqual([
              '[consolidateDocument] consolidation: 2 contested slices to settle',
              `[consolidateDocument] [mapOverlapped] slice 1 failed (${BENCH_UNREACHABLE.message}), and the caller stopped `
                + 'the work',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${consolidateDocument.name} showing the cited references (class forty-one)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SHOWS THE PRODUCERS AND THE SLATE JUDGES what the pages the original cites say, when the document '
            + 'carries them; a run carrying none is the control, whose sheets never mention them (the gate, which '
            + 'these judges never reach by keeping the standing, is pinned in consolidate-settle.unit.test.ts)',
          fn: async () => {
            /** What the cited page says, marked so a sheet carrying it is plain. */
            const references = 'CITED PAGE SAYS: the cat naps in the sun every afternoon.';
            /**
             Every producer and slate judge sheet one run sent.

             @param referenceContext - the document's references, none left out

             @returns Producer and slate judge sheets
             */
            async function sheetsOf(
              { referenceContext, }: { readonly referenceContext?: string; },
            ): Promise<{
              readonly producers: readonly string[];
              readonly judges: readonly string[];
            }> {
              /** Everything each call sent. */
              const sheets: string[] = [];
              await driveWith({
                contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
                client: roleAnsweringClient({ asked: [], sheets, },),
                ...((referenceContext === undefined) ? {} : { referenceContext, }),
              },);
              return {
                producers: sheets.filter(function isProducer(sheet,): boolean {
                  return sheet.includes('translation_report',);
                },),
                judges: sheets.filter(function isJudge(sheet,): boolean {
                  return (!sheet.includes('translation_report',)) && (!sheet.includes(GATE_MARKER,));
                },),
              };
            }
            /**
             For each round, whether every sheet it was sent carries the
             references, none when the round was never asked.

             @param rounds - one run's sheets by round

             @returns Producer and judge answers in that order
             */
            function carriedBy(
              rounds: Awaited<ReturnType<typeof sheetsOf>>,
            ): readonly string[] {
              return [rounds.producers, rounds.judges,].map(function carried(sheets,): string {
                if (sheets.length === 0)
                  return 'never asked';
                return sheets.every(function carries(sheet,): boolean {
                  return sheet.includes(references,);
                },)
                  ? 'all'
                  : 'not all';
              },);
            }
            expect([
              carriedBy(await sheetsOf({},),),
              carriedBy(await sheetsOf({ referenceContext: references, },),),
            ],).toEqual([
              [ 'not all', 'not all', ],
              [ 'all', 'all', ],
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${consolidateDocument.name} given the claims the repair panel accepted (CuspariaKLSY21 slice 0)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEYS A SLICE THE PANEL ACCEPTED CLAIMS ON under those claims, and every polish gate sheet and '
            + 'every polish writer sheet and every sheet of the judges of the writers\' rewrites carries them, where a '
            + 'slice with none keys as before and none mentions them (ledger B805)',
          fn: async () => {
            /** Note the panel's accepted claim is written as. */
            const note = 'REPAIR PANEL CLAIMS ACCEPTED: (1) accuracy/mistranslation minor: the nap is softened.';
            /**
             Keys one drive looked up and the polish gate sheets it sent.

             @param notes - accepted claims per slice, none left out
             @returns Keys and gate sheets
             */
            async function drive(
              { notes, }: { readonly notes?: ReadonlyMap<number, string>; },
            ): Promise<{
              readonly keys: readonly string[];
              readonly gateSheets: readonly string[];
              readonly writerSheets: readonly string[];
              readonly rewriteJudgeSheets: readonly string[];
            }> {
              /** Every key the driver asked the cache for. */
              const keys: string[] = [];
              /** Everything each call sent. */
              const sheets: string[] = [];
              await driveWith({
                contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
                client: polishingClient({ sheets, },),
                polishConfig: polishOn({ seats: RESEATED, },),
                resumed: {
                  get: function recordKey(key: string,): undefined {
                    keys.push(key,);
                    return undefined;
                  },
                } as unknown as ReadonlyMap<string, ConsolidationSettlement>,
                ...((notes === undefined) ? {} : { acceptedRepairClaimNotes: notes, }),
              },);
              return {
                keys,
                gateSheets: sheets.filter(function isGate(sheet,): boolean {
                  return sheet.includes('consolidation_polish_gate',);
                },),
                writerSheets: sheets.filter(function isWriter(sheet,): boolean {
                  return sheet.includes('refine_report',);
                },),
                // The refine selection's evidence is the only sheet that heads the base "CURRENT English translation".
                rewriteJudgeSheets: sheets.filter(isRewriteJudgeSheet,),
              };
            }
            const withClaims = await drive({ notes: new Map([[0, note,],],), },);
            const without = await drive({},);
            expect(withClaims.keys.length,).toBeGreaterThan(0,);
            expect(withClaims.keys,).not.toEqual(without.keys,);
            expect(withClaims.gateSheets.length,).toBeGreaterThan(0,);
            expect(withClaims.gateSheets.every(function carries(sheet,): boolean {
              return sheet.includes(note,);
            },),).toBe(true,);
            expect(withClaims.writerSheets.length,).toBeGreaterThan(0,);
            expect(withClaims.writerSheets.every(function carries(sheet,): boolean {
              return sheet.includes(note,);
            },),).toBe(true,);
            expect(withClaims.rewriteJudgeSheets.length,).toBeGreaterThan(0,);
            expect(withClaims.rewriteJudgeSheets.every(function carries(sheet,): boolean {
              return sheet.includes(note,);
            },),).toBe(true,);
            // THE CONTROL REACHES THE REWRITE JUDGES TOO, or "never mentions" would hold of no sheet at all.
            expect(without.rewriteJudgeSheets.length,).toBeGreaterThan(0,);
            expect(without.rewriteJudgeSheets.some(function carries(sheet,): boolean {
              return sheet.includes('REPAIR PANEL CLAIMS ACCEPTED',);
            },),).toBe(false,);
            // THE CONTROL REACHES THE WRITER TOO, or "never mentions" would hold of no sheet at all.
            expect(without.writerSheets.length,).toBeGreaterThan(0,);
            expect(without.writerSheets.some(function carries(sheet,): boolean {
              return sheet.includes('REPAIR PANEL CLAIMS ACCEPTED',);
            },),).toBe(false,);
            // THE CONTROL REACHES THE GATE TOO, or "never mentions" would hold of no sheet at all.
            expect(without.gateSheets.length,).toBeGreaterThan(0,);
            expect(without.gateSheets.some(function carries(sheet,): boolean {
              return sheet.includes('REPAIR PANEL CLAIMS ACCEPTED',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'SHOWS THE POLISH WRITERS AND THE JUDGES OF THEIR REWRITES the dispute note of a disputed slice, as it '
            + 'shows the polish gate, where an undisputed slice shows none of them a dispute heading (ledger B805)',
          fn: async () => {
            /**
             Wording of the note, as the archive package writes one for the claim.
             */
            const dispute: ArchiveDispute = {
              sliceIndex: 0,
              standIn: 'a repair wording for slice 0',
              standInEligible: true,
              acceptedClaims: ['accuracy/addition major: the tabby purrs in the sun.',],
            };
            /**
             Heading the dispute note opens with.
             */
            const heading = 'ARCHIVE RENDERING DISPUTED';
            /**
             Every sheet one drive sent.

             @param disputes - disputes per slice, none left out
             @returns Gate, writer and rewrite judge sheets
             */
            async function drive(
              { disputes, }: { readonly disputes?: ReadonlyMap<number, ArchiveDispute>; },
            ): Promise<{
              readonly gateSheets: readonly string[];
              readonly writerSheets: readonly string[];
              readonly rewriteJudgeSheets: readonly string[];
            }> {
              /** Everything each call sent. */
              const sheets: string[] = [];
              await driveWith({
                contests: [contestSettling({ sliceIndex: 0, lane: 'repair', },),],
                client: polishingClient({ sheets, },),
                polishConfig: polishOn({ seats: RESEATED, },),
                ...((disputes === undefined) ? {} : { archiveDisputes: disputes, }),
              },);
              return {
                gateSheets: sheets.filter(function isGate(sheet,): boolean {
                  return sheet.includes('consolidation_polish_gate',);
                },),
                writerSheets: sheets.filter(function isWriter(sheet,): boolean {
                  return sheet.includes('refine_report',);
                },),
                rewriteJudgeSheets: sheets.filter(isRewriteJudgeSheet,),
              };
            }
            const disputed = await drive({ disputes: new Map([[0, dispute,],],), },);
            const undisputed = await drive({},);
            for (const role of ['gateSheets', 'writerSheets', 'rewriteJudgeSheets',] as const) {
              expect(disputed[role].length,).toBeGreaterThan(0,);
              expect(disputed[role].every(function carries(sheet,): boolean {
                return sheet.includes(dispute.acceptedClaims[0] ?? '',);
              },),).toBe(true,);
              // THE CONTROL REACHES THE ROLE TOO, or "shows none" would hold of no sheet at all.
              expect(undisputed[role].length,).toBeGreaterThan(0,);
              expect(undisputed[role].some(function carries(sheet,): boolean {
                return sheet.includes(heading,);
              },),).toBe(false,);
            }
          },
        },),
      ],
    },),
  ],
},);
