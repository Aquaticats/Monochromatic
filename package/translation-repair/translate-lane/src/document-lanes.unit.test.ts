/**
 Tests for running both lanes over one preparation.

 What this covers that neither lane's own tests can: that the two outputs come
 back side by side with nothing merged or preferred, that the preparation's
 alignment findings are reported once rather than per lane, that the lanes run
 in the stated order, and that a failure in the first lane stops the second
 from spending anything.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { wait, } from '@monochromatic-dev/module-async-time/ts';
import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type {
  ChunkRepairOutcome,
  RepairModels,
  RepairSliceSeating,
} from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  runDocumentLanes,
  type TranslateModels,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { candidateCarrying, } from '@monochromatic-dev/translation-repair-test-fixture/ts/translate-ballot.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the driver under test.
 */
const l = tagged({ tag: 'document-lanes-test', },);

/**
 Original document: two sections, each one paragraph.
 */
const SOURCE_TEXT = `## 第一节

猫猫在窗台上打盹。

## 第二节

窗台上有一只鸟。
`;

/**
 Translation as it stands, awkward but complete.
 */
const TARGET_TEXT = `## Section one

The cat is doing the sleeping on the windowsill.

## Section two

On the windowsill there is being a bird.
`;

/**
 Sentence every translator returns for the first section.
 */
const FRESH = 'The cat naps on the windowsill.';

/**
 Models that produce and judge.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
];

/**
 Translate lane roster.
 */
const TRANSLATE_MODELS: TranslateModels = {
  translatorModelIds: ROSTER.slice(
    0,
    3,
  ),
  judgeModelIds: ROSTER,
};

/**
 Repair lane roster. The critics here raise nothing, so the stages after them
 never seat anyone.
 */
const REPAIR_MODELS: RepairModels = {
  criticModelIds: ROSTER.slice(
    0,
    3,
  ),
  panelModelIds: ROSTER.slice(
    3,
    6,
  ),
  editorModelIds: ROSTER.slice(
    0,
    3,
  ),
  judgeModelIds: ROSTER,
  checkerModelIds: ROSTER.slice(
    3,
    6,
  ),
  refinerModelIds: ROSTER.slice(
    0,
    3,
  ),
};

/**
 Every schema the script served, in order, so a case can say which lane ran
 first without reading a clock.
 */
type SchemaLog = string[];

/**
 Every call the script served with the seat that asked, so a case can say
 which roster a stage ran on.
 */
type AskedLog = { readonly schema: string; readonly modelId: RosterModelId; }[];

/**
 Successful model calls in flight for one stage.
 */
type StageConcurrency = {
  now: number;
  peak: number;
};

/**
 Repair and translate activity observed at client boundary.
 */
type LaneConcurrency = {
  readonly critic: StageConcurrency;
  readonly translate: StageConcurrency;
};

/**
 Client serving both lanes from one script.

 @param served - schema names appended in call order

 @param controller - abort the script may fire, standing in for the entry
 deadline

 @param abortAfterCriticCalls - critic calls served before the script aborts;
 absent means it never does

 @param activity - optional overlap instrument for both lane entry stages

 @returns Client honoring the script

 @example
 ```ts
 const client = lanesClient({ served, controller, },);
 ```
 */
function lanesClient(
  {
    served,
    controller,
    abortAfterCriticCalls,
    activity,
    askedBy,
  }: {
    readonly served: SchemaLog;
    readonly controller: AbortController;
    readonly abortAfterCriticCalls?: number;
    readonly activity?: LaneConcurrency;
    readonly askedBy?: AskedLog;
  },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by either lane',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Schema the caller asked for, which names the stage.
       */
      const schema = request.responseFormat
        ?.json_schema
        .name
        ?? 'unnamed';

      /**
       Everything the caller sent, which carries the slice original.
       */
      const content = request.messages
        .map(function toContent(message,) {
          return messageText({ message, },);
        },)
        .join('\n',);
      served.push(schema,);
      askedBy?.push({
        schema,
        modelId: request.modelId,
      },);

      /**
       Instrument for this lane's entry stage, when requested.
       */
      const stageActivity = schema === 'critic_report'
        ? activity?.critic
        : (schema === 'translation_report' ? activity?.translate : undefined);
      if (stageActivity !== undefined) {
        stageActivity.now += 1;
        stageActivity.peak = Math.max(
          stageActivity.peak,
          stageActivity.now,
        );
        await wait(10,);
        stageActivity.now -= 1;
      }

      /**
       Critic calls served so far, which is what the script counts down to
       its abort: it stands in for an entry deadline landing mid-lane.
       */
      const criticCalls = served.filter(function isCritic(name,) {
        return name === 'critic_report';
      },)
        .length;
      if ((schema === 'critic_report')
        && (abortAfterCriticCalls !== undefined)
        && (criticCalls > abortAfterCriticCalls))
        controller.abort(new Error('entry deadline reached',),);
      if (request.signal
        .aborted)
        throw new Error('exchange torn down by abort',);

      /**
       Reply for whichever stage asked, keyed by its schema.
       */
      const value: unknown = replyFor({
        schema,
        content,
      },);
      if (!request.validate(value,))
        throw new Error(`scripted ${schema} failed the wire guard`,);
      return {
        kind: 'ok',
        value,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by either lane',);
    },
  };
}

/**
 Scripted reply for one stage.

 The repair lane's critics find nothing, so that lane settles every slice
 unchanged and the naturalness lane leaves each paragraph alone. The translate
 lane renders each slice afresh and its judges pick that rendering. So the two
 lanes disagree about the document by construction, which is what makes "both
 outputs, neither chosen" a testable claim.

 @param schema - schema name the stage asked for

 @param content - everything the stage sent

 @returns Wire value for that stage

 @throws {@link Error} when a stage this script does not serve asks

 @example
 ```ts
 const value = replyFor({ schema: 'critic_report', content, },);
 ```
 */
function replyFor(
  {
    schema,
    content,
  }: {
    readonly schema: string;
    readonly content: string;
  },
): unknown {
  if (schema === 'critic_report')
    return { issues: [], };
  if (schema === 'refine_report')
    return { rewrites: [], };
  if (schema === 'translation_report')
    return { translation: renderingFor({ content, },), };
  if (schema === 'candidate_ballot') {
    return {
      best: candidateCarrying({
        content,
        needle: FRESH,
      },),
      assessment: 'scripted',
    };
  }
  throw new Error(`no script for ${schema}`,);
}

/**
 Renders one slice the way a translator that respected block structure would.

 @param content - translator prompt, which carries the slice original

 @returns Rendering for that slice

 @example
 ```ts
 const rendering = renderingFor({ content, },);
 ```
 */
function renderingFor({ content, }: { readonly content: string; },): string {
  if (content.includes('## 第一节',))
    return `## Section one\n\n${FRESH}`;
  if (content.includes('## 第二节',))
    return '## Section two\n\nA bird sits on the windowsill.';
  return FRESH;
}

/**
 Runs both lanes over the fixture pair.

 @param served - schema log, passed in so a case expecting a REJECTION can
 still read what was bought

 @param abortAfterCriticCalls - critic calls served before the script aborts

 @param overlap - most slices each lane keeps in flight

 @param activity - optional overlap instrument for both lane entry stages

 @param prepared - preparation to run over, the cat document pair unless a case supplies its own

 @param logger - logger the driver writes to, `l` unless a case reads its warnings

 @returns Both lane results

 @example
 ```ts
 const lanes = await runLanes({ served: [], },);
 ```
 */
async function runLanes(
  {
    served,
    abortAfterCriticCalls,
    repairSliceCache,
    overlap = 1,
    activity,
    reseatTranslate,
    beforeSlice,
    askedBy,
    prepared = prepareDocumentPair({
      sourceText: SOURCE_TEXT,
      targetText: TARGET_TEXT,
    },),
    logger = l,
  }: {
    readonly served: SchemaLog;
    readonly abortAfterCriticCalls?: number;
    readonly repairSliceCache?: SliceCache<ChunkRepairOutcome>;
    readonly overlap?: number;
    readonly activity?: LaneConcurrency;
    readonly reseatTranslate?: () => Promise<TranslateModels>;
    readonly beforeSlice?: (
      args: { readonly lane: 'repair' | 'translate'; },
    ) => Promise<RepairSliceSeating>;
    readonly askedBy?: AskedLog;
    readonly prepared?: ReturnType<typeof prepareDocumentPair>;
    readonly logger?: Logger;
  },
) {
  /**
   Run steering, which the script may abort part way through.
   */
  const controller = new AbortController();
  return await runDocumentLanes({
    client: lanesClient({
      served,
      controller,
      ...((askedBy === undefined) ? {} : { askedBy, }),
      ...((abortAfterCriticCalls === undefined)
        ? {}
        : { abortAfterCriticCalls, }),
      ...((activity === undefined) ? {} : { activity, }),
    },),
    prepared,
    repairModels: REPAIR_MODELS,
    translateModels: TRANSLATE_MODELS,
    signal: controller.signal,
    perCallTimeoutMs: HANG_STOP_MS,
    overlap,
    ...((repairSliceCache === undefined)
      ? {}
      : { repairSliceCache, }),
    ...((reseatTranslate === undefined)
      ? {}
      : { reseatTranslate, }),
    ...((beforeSlice === undefined)
      ? {}
      : { beforeSlice, }),
    l: logger,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

await describe({
  name: runDocumentLanes.name,
  children: [
    it({
      name: 'returns BOTH documents and prefers neither, which is the whole '
        + 'contract: which one should ship is the open question, and a driver '
        + 'that answered it here would answer it invisibly for every count '
        + 'downstream',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];

        const lanes = await runLanes({ served, },);
        // The repair lane found nothing to repair, so it hands back the archive.
        expect(lanes.repair
          .status,).toBe('unchanged',);
        expect(lanes.translate
          .translatedText,).toContain(FRESH,);
      },
    },),
    it({
      name: 'HANDS THE REPAIR LANE\'S DISPUTES TO THE TRANSLATE LANE, so a slice whose archive rendering the '
        + 'repair lane\'s adjudicators disputed reads as a dispute there, and a slice nobody disputed does not',
      fn: async () => {
        /**
         Slices an undisputed run settled, as the repair lane stored them under their keys.
         */
        const settled: { readonly key: string; readonly serialized: string; }[] = [];
        const undisputed = await runLanes({
          served: [],
          repairSliceCache: {
            resumed: new Map<string, ChunkRepairOutcome>(),
            persist: async (slice,) => {
              settled.push(slice,);
            },
          },
        },);
        /**
         Key the undisputed run stored the first slice under.
         */
        const { key, } = nonNullishOrThrow(settled[0],);
        /**
         That slice as a run whose adjudicators accepted an addition claim against the archive would have
         stored it, the addition mended and the checkers confirming it.
         */
        const mended: ChunkRepairOutcome = {
          ...nonNullishOrThrow(undisputed.repair
            .chunks[0],),
          repairedText: '## Section one\n\nThe cat sleeps on the windowsill.',
          changed: true,
          accuracyPatchSelected: true,
          issues: [
            {
              issueId: 'adjudicated/sleeping',
              status: 'accepted',
              severity: 'major',
              claims: [
                {
                  claimId: 'issue/sleeping',
                  claim: {
                    category: 'accuracy/addition',
                    severity: 'major',
                    summary: 'The translation adds that the cat is doing the sleeping, which the original never says.',
                    spans: [],
                  },
                },
              ],
              tallies: {},
            },
          ],
          resolvedIssueIds: ['adjudicated/sleeping',],
        };
        const disputed = await runLanes({
          served: [],
          repairSliceCache: {
            resumed: new Map<string, ChunkRepairOutcome>([[key, mended,],],),
            persist: async () => {},
          },
        },);
        /**
         Dispute findings of the first slice's translate record, in each run.
         */
        const disputesOfFirst = [
          undisputed,
          disputed,
        ].map(function disputesOf(lanes,): readonly string[] {
          return nonNullishOrThrow(lanes.translate
            .slices[0],)
            .findings
            .filter(function isDispute(finding,): boolean {
              return finding.startsWith('translate-archive-disputed',);
            },);
        },);
        expect(disputesOfFirst,).toEqual([
          [],
          [
            'translate-archive-disputed (slice 0): the repair lane\'s adjudicators accepted 1 disputing claim(s) '
              + '(accuracy/addition at any severity, any other accuracy claim the panel settled at major or worse) '
              + 'against the archive rendering, so the repair lane\'s text stands in for it (classes one hundred seven '
              + 'and one hundred seventy-six)',
          ],
        ],);
      },
    },),
    it({
      name: 'RE-SEATS THE TRANSLATE LANE when it is about to start, after the repair lane has spent its '
        + 'calls, and writes with the roster the re-seating returns: the thirteenth class, where the '
        + 'lanes reading was minutes old by the time the writers were asked',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];
        /**
         How many calls the repair lane had made when the translate lane re-seated, and how often.
         */
        const reseated = {
          times: 0,
          callsBefore: -1,
        };
        const lanes = await runLanes({
          served,
          reseatTranslate: async (): Promise<TranslateModels> => {
            reseated.times += 1;
            reseated.callsBefore = served.length;
            return TRANSLATE_MODELS;
          },
        },);
        expect(reseated.times,).toBe(1,);
        expect(reseated.callsBefore,).toBeGreaterThan(0,);
        expect(lanes.repair
          .status,).toBe('unchanged',);
      },
    },),
    it({
      name: 'ASKS BEFORE EVERY SLICE OF EITHER LANE, naming the lane, so a caller can hold a slice back '
        + 'while a named provider hold keeps that lane\'s bench from quorum: the thirteenth class\'s '
        + 'second face, a hold two minutes into a phase',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];
        /**
         Lanes asked, in the order asked.
         */
        const asked: string[] = [];
        const lanes = await runLanes({
          served,
          beforeSlice: async ({ lane, },): Promise<RepairSliceSeating> => {
            asked.push(lane,);
            return {};
          },
        },);
        /**
         Slices both lanes ran over.
         */
        const sliceCount = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },).slices
          .length;
        expect(asked.filter(function isRepair(lane,): boolean {
          return lane === 'repair';
        },).length,).toBe(sliceCount,);
        expect(asked.filter(function isTranslate(lane,): boolean {
          return lane === 'translate';
        },).length,).toBe(sliceCount,);
        expect(asked.lastIndexOf('repair',),).toBeLessThan(asked.indexOf('translate',),);
        expect(lanes.translate
          .translatedText,).toContain(FRESH,);
        expect(lanes.repair
          .repairedText,).toBe(TARGET_TEXT,);
        // The translate lane rewrote both slices, so its document differs.
        expect(lanes.translate
          .translatedText,).toContain(FRESH,);
        expect(lanes.translate
          .translatedText,).not.toBe(TARGET_TEXT,);
        // Nothing merged the two, and nothing named a winner. The two ledgers
        // are per lane and DESCRIBE rather than decide: each says what its own
        // lane's document carries, and neither mentions the other lane, so the
        // question of which document ships is still unanswered here.
        expect(Object.keys(lanes,)
          .toSorted(),).toEqual([
            'alignmentFindings',
            'repair',
            'repairDelivery',
            'translate',
            'translateDelivery',
          ],);
      },
    },),

    it({
      name: 'SEATS THE REPAIR LANE ON THE ROSTER ITS HOOK RETURNS, so a bench re-seated after a provider '
        + 'dry-out inside the lane is the one the next chunk asks (class one hundred three, zheermao7, '
        + '2026-09-23)',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];
        /**
         Every call with the seat that asked.
         */
        const askedBy: AskedLog = [];
        /**
         Roster the repair lane's hook hands back: critics re-seated, since
         the critics are the one repair stage this script always asks.
         */
        const reseated: RepairModels = {
          ...REPAIR_MODELS,
          criticModelIds: ROSTER.slice(
            3,
            6,
          ),
        };
        await runLanes({
          served,
          askedBy,
          beforeSlice: async ({ lane, },): Promise<RepairSliceSeating> =>
            (lane === 'repair') ? { repairModels: reseated, } : {},
        },);
        /**
         Seats the critic stage ran on.
         */
        const critics = askedBy
          .filter(function isCritic(call,): boolean {
            return call.schema === 'critic_report';
          },)
          .map(function toSeat(call,): RosterModelId {
            return call.modelId;
          },);
        expect(critics.length,).toBeGreaterThan(0,);
        for (const seat of critics)
          expect(reseated.criticModelIds,).toContain(seat,);
      },
    },),

    it({
      name: 'SEATS THE TRANSLATE LANE ON THE ROSTER ITS HOOK RETURNS, as it seats the repair lane, so a bench '
        + 're-seated after a provider dry-out inside the lane is the one the next slice asks (ledger H5)',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];
        /**
         Every call with the seat that asked.
         */
        const askedBy: AskedLog = [];
        /**
         Roster the translate lane's hook hands back: writers re-seated.
         */
        const reseated: TranslateModels = {
          ...TRANSLATE_MODELS,
          translatorModelIds: ROSTER.slice(
            3,
            6,
          ),
        };
        await runLanes({
          served,
          askedBy,
          beforeSlice: async ({ lane, },): Promise<RepairSliceSeating & { readonly translateModels?: TranslateModels; }> =>
            (lane === 'translate') ? { translateModels: reseated, } : {},
        },);
        /**
         Seats the translate stage asked to write.
         */
        const writers = askedBy
          .filter(function isWriter(call,): boolean {
            return call.schema === 'translation_report';
          },)
          .map(function toSeat(call,): RosterModelId {
            return call.modelId;
          },);
        expect(writers.length,).toBeGreaterThan(0,);
        for (const seat of writers)
          expect(reseated.translatorModelIds,).toContain(seat,);
      },
    },),

    it({
      name: 'threads one overlap through BOTH repair and translate lanes, after serial '
        + 'positive controls prove each stage instrument distinguishes one slice from two',
      fn: async () => {
        /**
         Activity under default sequential admission.
         */
        const serial: LaneConcurrency = {
          critic: {
            now: 0,
            peak: 0,
          },
          translate: {
            now: 0,
            peak: 0,
          },
        };
        await runLanes({
          served: [],
          overlap: 1,
          activity: serial,
        },);

        /**
         Activity under two slices admitted per lane.
         */
        const overlapped: LaneConcurrency = {
          critic: {
            now: 0,
            peak: 0,
          },
          translate: {
            now: 0,
            peak: 0,
          },
        };
        await runLanes({
          served: [],
          overlap: 2,
          activity: overlapped,
        },);
        expect(serial.critic.peak,).toBeGreaterThan(0,);
        expect(overlapped.critic.peak,).toBeGreaterThan(serial.critic.peak,);
        expect(serial.translate.peak,).toBeGreaterThan(0,);
        expect(overlapped.translate.peak,).toBeGreaterThan(serial.translate.peak,);
      },
    },),

    it({
      name: 'hands back one delivery row per prepared slice FOR EACH LANE, and each ledger describes '
        + 'its own lane`s document: the rows a lane ships are the slices that lane`s result names, and '
        + 'writing them over the archive reproduces exactly that lane`s text',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];

        const lanes = await runLanes({ served, },);
        expect(lanes.repairDelivery
          .records
          .length,).toBe(lanes.repair
            .sliceCount,);
        expect(lanes.translateDelivery
          .records
          .length,).toBe(lanes.translate
            .sliceCount,);

        // Each ledger carries the slicing it was built over, and both lanes ran
        // over one preparation, so the two names agree. A consumer holding two
        // ledgers cannot establish that for itself, which is why they travel
        // named rather than being stamped where they are read.
        expect(lanes.repairDelivery
          .preparationIdentity,).toBe(lanes.translateDelivery
            .preparationIdentity,);
        expect(lanes.repairDelivery
          .preparationIdentity,).toMatch('sha256-preparation-v2:',);

        /**
         Slices the translate lane's document carries a change for, read off
         its ledger rather than off its index set.
         */
        const translateShipped = lanes.translateDelivery
          .records
          .filter(function carriesAChange(record,): boolean {
            return record.delivery
              .kind === 'replacement-shipped';
          },)
          .map(function toIndex(record,): number {
            return record.sliceIndex;
          },);
        expect(translateShipped,).toEqual(lanes.translate
          .changedSliceIndices,);

        /**
         Whether any repair row claims the document carries a change, which
         this fixture's repair lane found nothing to make.
         */
        const repairShipped = lanes.repairDelivery
          .records
          .some(function carriesAChange(record,): boolean {
            return record.delivery
              .kind === 'replacement-shipped';
          },);
        expect(repairShipped,).toBe(false,);
      },
    },),

    it({
      name: 'runs the REPAIR lane first, since its naturalness phase settles '
        + 'after the slice loop and nothing persists what that phase produced, '
        + 'while every translate slice is cached as it finishes',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];

        await runLanes({ served, },);

        /**
         First critic call, which only the repair lane makes.
         */
        const firstRepair = served.indexOf('critic_report',);

        /**
         First translator call, which only the translate lane makes.
         */
        const firstTranslate = served.indexOf('translation_report',);
        expect(firstRepair,).toBeGreaterThanOrEqual(0,);
        expect(firstTranslate,).toBeGreaterThanOrEqual(0,);
        expect(firstRepair,).toBeLessThan(firstTranslate,);
      },
    },),

    it({
      name: 'reports the preparation`s alignment findings ONCE, at the top '
        + 'level. Both lanes ran over one preparation, so counting them per '
        + 'lane would count one defect in the archive twice',
      fn: async () => {
        /**
         Schemas the run served, in order.
         */
        const served: SchemaLog = [];

        /**
         Preparation the assertion compares against, made the same way the
         driver makes it.
         */
        const prepared = prepareDocumentPair({
          sourceText: SOURCE_TEXT,
          targetText: TARGET_TEXT,
        },);

        const lanes = await runLanes({ served, },);
        expect(lanes.alignmentFindings,).toEqual(prepared.alignmentFindings,);
      },
    },),

    it({
      name: 'WRITES THE STRUCTURE AND SECTION-PAIRING FINDINGS to the run log as warn lines, once each, '
        + 'and leaves the findings other lines already report out of it',
      fn: async () => {
        /**
         Findings the preparation recorded: two the run log never showed, and two it shows elsewhere.
         */
        const structure = 'alignment structure-mismatch (source section 13: source-only (roster-unpaired); anchored '
          + 'for insertion at offset 17)';
        const sections = 'section-pairing paired 1 of 2 original and 1 translation sections, from 4 usable voices of 4 heard';
        const blocks = 'block-pairing section 0 paired 2 of 2 original and 2 of 2 translation blocks across 2 '
          + 'relations, from 4 usable voices of 4 heard';
        const lost = 'stage-voice-lost (block-pairing minimax-m3)';
        /**
         Warnings the driver wrote, in order.
         */
        const warnings: string[] = [];
        /**
         Logger keeping the warnings.
         */
        const capturing: Logger = {
          ...l,
          warn: function keep(message: string,): void {
            warnings.push(message,);
          },
        };
        await runLanes({
          served: [],
          prepared: {
            ...prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },),
            alignmentFindings: [
              structure,
              sections,
              blocks,
              lost,
            ],
          },
          logger: capturing,
        },);
        expect(warnings.filter(function recorded(message,): boolean {
          return message.includes('alignment finding recorded: ',);
        },),).toEqual([
          `[${runDocumentLanes.name}] alignment finding recorded: ${structure}`,
          `[${runDocumentLanes.name}] alignment finding recorded: ${sections}`,
        ],);
      },
    },),

    it({
      name: 'STOPS at a first-lane failure the signal knows nothing about, and '
        + 'buys nothing for the second. This is the case an abort cannot pin: '
        + 'an aborted second lane refuses on its own, so a driver that swallowed '
        + 'the first lane`s failure would still look right. Here the signal '
        + 'stays live, and only the driver`s own stopping keeps the translate '
        + 'lane from spending a whole document',
      fn: async () => {
        /**
         Schemas the run served, read after the rejection.
         */
        const served: SchemaLog = [];

        /**
         Failure the run raised.

         The repair lane's own cache store fails on the first settled slice,
         which is a failure inside the first lane with nothing aborted
         anywhere: the signal stays live throughout, so only the driver's own
         stopping can keep the translate lane from spending a whole document.
         */
        let caught: unknown;
        try {
          await runLanes({
            served,
            repairSliceCache: {
              resumed: new Map<string, ChunkRepairOutcome>(),
              persist: async () => {
                throw new Error('scripted cache write failure',);
              },
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('scripted cache write failure',);
        expect(served.includes('translation_report',),).toBe(false,);
        expect(served.includes('candidate_ballot',),).toBe(false,);
      },
    },),

    it({
      name: 'STOPS at the first lane`s failure and buys nothing for the '
        + 'second. An abort inside repair must not be followed by a whole '
        + 'translate lane over the same document, which is the failure a '
        + 'driver that caught and continued would produce',
      fn: async () => {
        /**
         Schemas the run served, in order, read after the rejection.
         */
        const served: SchemaLog = [];

        /**
         Failure the run raised.
         */
        let caught: unknown;
        try {
          await runLanes({
            served,
            abortAfterCriticCalls: 1,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(served.includes('critic_report',),).toBe(true,);
        expect(served.includes('translation_report',),).toBe(false,);
        expect(served.includes('candidate_ballot',),).toBe(false,);
      },
    },),

    it({
      name: 'names EVERY empty role across BOTH lanes in one refusal, rather '
        + 'than stopping at the repair lane. An operator with a role empty in '
        + 'each lane would otherwise pay a preflight per lane to learn that, '
        + 'which is the cost this check exists to spare them',
      fn: async () => {
        /**
         Failure the driver raised before either lane started.
         */
        let caught: unknown;
        try {
          await runDocumentLanes({
            client: {
              chatText: async () => {
                throw new Error('an unconfigured run must ask nobody anything',);
              },
              chatJson: async () => {
                throw new Error('an unconfigured run must ask nobody anything',);
              },
              quotas: async () => {
                throw new Error('quotas unused by either lane',);
              },
            },
            prepared: prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },),
            repairModels: {
              ...REPAIR_MODELS,
              panelModelIds: [],
            },
            translateModels: {
              ...TRANSLATE_MODELS,
              translatorModelIds: [],
            },
            signal: new AbortController().signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        // Both lanes, in one message, each role named with the lane it belongs
        // to: the two lanes share a `judgeModelIds`, so an unprefixed name
        // would say nothing about which one is empty.
        expect(String(caught,),).toContain('repair.panelModelIds',);
        expect(String(caught,),).toContain('translate.translatorModelIds',);
      },
    },),

    it({
      name: 'REFUSES AN EMPTY CRITIC ROSTER, which until Question 3 was answered had to be '
        + 'tolerated in case dropping the stage was the shape chosen. Answer B keeps critics as '
        + 'evidence for the judges, so an empty list is a misconfiguration, and the failure it '
        + 'used to produce was the expensive kind: a whole corpus pass of settled, unchanged '
        + 'documents that later analysis reads as pages needing no repair',
      fn: async () => {
        /**
         Failure the driver raised before either lane started.
         */
        let caught: unknown;
        try {
          await runDocumentLanes({
            client: {
              chatText: async () => {
                throw new Error('an unconfigured run must ask nobody anything',);
              },
              chatJson: async () => {
                throw new Error('an unconfigured run must ask nobody anything',);
              },
              quotas: async () => {
                throw new Error('quotas unused by either lane',);
              },
            },
            prepared: prepareDocumentPair({
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
            },),
            repairModels: {
              ...REPAIR_MODELS,
              criticModelIds: [],
            },
            translateModels: TRANSLATE_MODELS,
            signal: new AbortController().signal,
            perCallTimeoutMs: HANG_STOP_MS,
            l,
            stageSettings: SCRIPTED_STAGE_SETTINGS,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(String(caught,),).toContain('repair.criticModelIds',);
      },
    },),
  ],
},);
