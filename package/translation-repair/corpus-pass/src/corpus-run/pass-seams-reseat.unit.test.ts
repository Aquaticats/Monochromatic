/**
 Guards ledger X14: the pass seams wire each phase's re-seat hook into its
 driver. The hooks and the drivers each have their own cases, but no case drove
 a seam, so a seam that dropped its hook survived every guard. Each case here
 runs a seam against a client whose dryness view changes after the phase's own
 reading, under a hold: only a wired hook seats the bench the second view
 seats. A control whose view never changes shows the seat the change removes
 would otherwise have been asked.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary';
import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  makeInsertionChunk,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  judgeSeatsFor,
  type RunClient,
} from '@monochromatic-dev/translation-repair-run';
import type { DocumentLanesResult, } from '@monochromatic-dev/translation-repair-translate-lane';
import type {
  BudgetView,
  ChatJsonOutcome,
  ChatJsonRequest,
} from '@monochromatic-dev/translation-repair-provider';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest';
import type {
  ArtifactComparisonRow,
  ArtifactDeliveryRow,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import { ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-archive';
import {
  admitPassInsertions,
  createPassPictureReader,
  readLanesSeats,
  runPassConsolidation,
  runPassContest,
  runPassLanes,
  runPassPreparation,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { insertionRow, } from '@monochromatic-dev/translation-repair-artifact-pool/ts/sole-lane-comparison-row.test-fixture.ts';

import { NO_OUTSIDE_READS, } from './pass-outside-reads.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import { metersAllWet, } from '@monochromatic-dev/translation-repair-provider/ts/meter-states.test-fixture.ts';

/**
 Logger the seams write to, whose lines are not under test.
 */
const l = tagged({ tag: 'pass-seams-reseat-test', },);

/**
 Every provider wet, the Claude key included: the view each phase reads at its start. The Claude provider read dry here
 until the review of the Claude route, which made the name untrue and left every pass-level path under a wet Claude
 key untested; Haiku 5.5 now takes its seats in these cases like any wet provider's models.
 */
const ALL_WET: BudgetView = {
  neuralwatt: false,
  synthetic: false,
  bedrock: false,
  deepseek: false,
  xiaomi: false,
  anthropic: false,
  hyper: false,
  openrouter: false,
};

/**
 Synthetic dry: the view a hook reads once the hold began, under which no
 phase finds a bench short, so no reading waits (measured 2026-09-28).
 */
const SYNTHETIC_DRY: BudgetView = {
  ...ALL_WET,
  neuralwatt: true,
  synthetic: true,
};

/**
 Synthetic and Hyper dry: the view the late-bench cases read once the hold
 began. Since Qwen3.8-27B left the late judges on 2026-10-22 (ledger B815),
 Synthetic dry alone takes no late judge. `hf:moonshotai/Kimi-K3` leaves the
 late judges only when Synthetic and Hyper read dry while OpenRouter, the one
 provider left to serve it and one it is withheld on, reads wet: of the 31
 views with a provider dry, measured on 2026-10-22, those four alone take a
 late judge.
 */
const SYNTHETIC_AND_HYPER_DRY: BudgetView = {
  ...SYNTHETIC_DRY,
  hyper: true,
};

/**
 Hold Synthetic's refusal registers, which is what makes a hook read again.
 */
const HOLD_MS = 40;

/**
 Original of the one contested slice.
 */
const SOURCE = '猫在窗边睡着了。';

/**
 Archive wording of it.
 */
const ARCHIVE = 'The cat slept by the window.';

/**
 Repair lane wording of it.
 */
const REPAIR = 'The cat fell asleep by the window.';

/**
 Translate lane wording of it.
 */
const TRANSLATE = 'The cat had fallen asleep beside the window.';

/**
 The one seat both benches under test lose when Synthetic and Hyper go dry:
 first of the late judges and of the slate judges, and among the writers,
 while every provider is wet.
 */
const LOST_SEAT = (function lostSeat(): RosterModelId {
  /**
   Late judges while every provider is wet.
   */
  const wet = judgeSeatsFor({ dry: ALL_WET, },).lateJudges;
  /**
   Late judges while Synthetic and Hyper are dry.
   */
  const dry = new Set(judgeSeatsFor({ dry: SYNTHETIC_AND_HYPER_DRY, },).lateJudges,);
  /**
   Seats the dry-out takes.
   */
  const lost = wet.filter(function gone(seat,): boolean {
    return !dry.has(seat,);
  },);
  if (lost.length === 0)
    throw new Error('the fixture views seat the same late judges, so no case here can tell a wired hook apart',);
  return lost[0] as RosterModelId;
})();

/**
 Builds a run client whose view is every provider wet on the first reading
 and the given view on every later one, with Synthetic held, recording every
 seat a structured call asked.

 @param later - view every reading after the first answers

 @param asked - sink for the seat of every call

 @param reviewed - sink for the seat of every call on the archive review
 sheet, for a case that must tell the review apart from the pairing

 @param answer - reply every call is given, which the sheet may refuse

 @param reading - what every picture reader transcribes, for a case that
 reads pictures; a text call throws without it

 @param held - whether Synthetic's hold runs, which is what makes a hook read
 again; a case without it asks the benches the phase first seated

 @returns Client to drive a seam with

 @example
 ```ts
 const client = viewChangingClient({ later: SYNTHETIC_DRY, asked: [], answer: '{}', },);
 ```
 */
function viewChangingClient(
  {
    later,
    asked,
    reviewed,
    answer,
    reading,
    held = true,
  }: {
    readonly later: BudgetView;
    readonly reading?: string;
    readonly held?: boolean;
    readonly asked: RosterModelId[];
    readonly reviewed?: RosterModelId[];
    readonly answer: string;
  },
): RunClient {
  /**
   Readings taken so far.
   */
  const counter = { reads: 0, };
  return {
    providerDryness: async (): Promise<BudgetView> => {
      counter.reads += 1;
      return (counter.reads === 1) ? ALL_WET : later;
    },
    providerMeterStates: metersAllWet,
    providerHolds: () => ({
      neuralwatt: 0,
      synthetic: held ? HOLD_MS : 0,
      bedrock: 0,
      deepseek: 0,
      xiaomi: 0,
      anthropic: 0,
      hyper: 0,
      openrouter: 0,
    }),
    chatText: async (request,) => {
      if (reading === undefined)
        throw new Error('chatText not used',);
      asked.push(request.modelId,);
      return { text: reading, };
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      if (request.responseFormat?.json_schema.name === ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT.json_schema.name)
        reviewed?.push(request.modelId,);
      /**
       That reply as the sheet parses it.
       */
      const value: unknown = JSON.parse(answer,);
      return request.validate(value,)
        ? { kind: 'ok', value, rawText: answer, }
        : { kind: 'schema-mismatch', rawText: answer, detail: 'fixture answers another sheet', };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
}

/**
 Both lanes over the one slice, worded differently.
 */
const PROJECTED: ProjectedLanes = (function projected(): ProjectedLanes {
  /**
   Delivery row a lane ships with its wording.
   */
  function deliveryOf({ shippedText, }: { readonly shippedText: string; },): ArtifactDeliveryRow {
    return {
      sliceIndex: 0,
      sourceText: SOURCE,
      incumbentKind: 'present',
      incumbentText: ARCHIVE,
      outcome: {
        kind: 'decided',
        acceptedText: shippedText,
      },
      shippedText,
      delivery: { kind: 'replacement-shipped', },
    };
  }
  /**
   Comparison row carrying the two wordings.
   */
  const comparison: ArtifactComparisonRow = {
    sliceIndex: 0,
    incumbentKind: 'present',
    incumbentText: ARCHIVE,
    repairText: REPAIR,
    translateText: TRANSLATE,
    laneRelation: 'both-differ',
    repairOutcome: {
      kind: 'decided',
      acceptedText: REPAIR,
    },
    translateOutcome: {
      kind: 'decided',
      acceptedText: TRANSLATE,
    },
    decisionComparison: {
      kind: 'comparable',
      verdict: 'different',
    },
    repairDelivery: { kind: 'replacement-shipped', },
    translateDelivery: { kind: 'replacement-shipped', },
  };
  return {
    delivery: {
      repair: [deliveryOf({ shippedText: REPAIR, },),],
      translate: [deliveryOf({ shippedText: TRANSLATE, },),],
    },
    comparison: [comparison,],
  };
})();

/**
 Generation stamp the caches are opened under.
 */
const DIGEST = 'pass-seams-reseat-test' as PipelineDigest;

/**
 Runs the contest seam over a slice, every judge voting for the repair lane.

 @param later - view every reading after the phase's own answers

 @param projected - both lanes over the slices the contest answers

 @param lifeEnded - whether the entry records a death

 @param subjectNames - names the translation declares for its subject

 @returns Seats the judges' calls asked, and the records the seam returned

 @example
 ```ts
 const { asked, slices, } = await contestRun({ later: SYNTHETIC_DRY, projected: PROJECTED, },);
 ```
 */
async function contestRun(
  {
    later,
    projected,
    lifeEnded,
    subjectNames,
  }: {
    readonly later: BudgetView;
    readonly projected: ProjectedLanes;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
  },
): Promise<{
  readonly asked: readonly RosterModelId[];
  readonly slices: readonly ArtifactContestSlice[];
}> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  await using cacheDir = await scratchDir({ prefix: 'pass-seams-reseat-', },);
  const slices = await runPassContest({
    client: viewChangingClient({
      later,
      asked,
      answer: JSON.stringify({
        verdict: 'repair',
        unsupported: [],
        dropped: [],
        assessment: 'the original supports it',
      },),
    },),
    // Only the repair lane's chunks are read, for damage claims and dispute
    // notes, and a lane with none carries neither.
    lanes: { repair: { chunks: [], }, } as unknown as DocumentLanesResult,
    projected,
    frontMatterSlices: new Set(),
    lineStructuredSlices: new Set(),
    sheetFacts: {
      ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
      ...((subjectNames === undefined) ? {} : { subjectNames, }),
    },
    entryCacheDir: cacheDir.path,
    pipelineDigest: DIGEST,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    overlap: 1,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
    // Every late judge must be asked, so the lost seat is, and the default
    // window is rotated by the sheet text.
    fanOut: 'whole-bench',
  },);
  return {
    asked,
    slices,
  };
}

/**
 Runs the contest seam over the one slice, every judge voting for the repair
 lane.

 @param later - view every reading after the phase's own answers

 @returns Seats the judges' calls asked

 @example
 ```ts
 const asked = await contestAsked({ later: SYNTHETIC_DRY, },);
 ```
 */
async function contestAsked({ later, }: { readonly later: BudgetView; },): Promise<readonly RosterModelId[]> {
  return (await contestRun({
    later,
    projected: PROJECTED,
  },)).asked;
}

/**
 Runs the consolidation seam over the one slice, which the contest gave the
 repair lane, every call answered with nothing a sheet can read.

 @param later - view every reading after the phase's own answers

 @returns Seats the consolidation's calls asked

 @example
 ```ts
 const asked = await consolidationAsked({ later: SYNTHETIC_DRY, },);
 ```
 */
async function consolidationAsked({ later, }: { readonly later: BudgetView; },): Promise<readonly RosterModelId[]> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   The contest's record for the slice.
   */
  const contest: ArtifactContestSlice = {
    sliceIndex: 0,
    verdict: {
      kind: 'lane-won',
      lane: 'repair',
    },
    ballots: [],
    usable: 3,
  };
  await using cacheDir = await scratchDir({ prefix: 'pass-seams-reseat-', },);
  await runPassConsolidation({
    client: viewChangingClient({
      later,
      asked,
      answer: '"no sheet can read this"',
    },),
    prepared: prepareDocumentPair({
      sourceText: SOURCE,
      targetText: ARCHIVE,
    },),
    projected: PROJECTED,
    contests: [contest,],
    frontMatterSlices: new Set(),
    pictureReadings: new Map(),
    entryCacheDir: cacheDir.path,
    pipelineDigest: DIGEST,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    overlap: 1,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asked;
}

/**
 The one seat the roster loses when Synthetic goes dry.
 */
const LOST_ROSTER_SEAT = (function lostRosterSeat(): RosterModelId {
  /**
   Roster while Synthetic is dry.
   */
  const dry = new Set(judgeSeatsFor({ dry: SYNTHETIC_DRY, },).roster,);
  /**
   Seats the dry-out takes from the roster.
   */
  const lost = judgeSeatsFor({ dry: ALL_WET, },).roster.filter(function gone(seat,): boolean {
    return !dry.has(seat,);
  },);
  if (lost.length === 0)
    throw new Error('the fixture views seat the same roster, so the admission case cannot tell a wired hook apart',);
  return lost[0] as RosterModelId;
})();

/**
 A page whose one slice is a source-only passage, over an archive long enough
 that page shortfall corroborates nothing.
 */
const GAP: PreparedDocumentPair = {
  sourceText: `${SOURCE}\n${'猫在窗台晒太阳。'.repeat(20,)}`,
  targetText: `## Cats\n\n${'The cat sleeps in warm sunlight. '.repeat(20,)}`,
  slices: [{
    source: {
      kind: 'content',
      sliceIndex: 0,
      nodes: [],
      startOffset: 0,
      endOffset: SOURCE.length,
      text: SOURCE,
    },
    target: makeInsertionChunk({
      sliceIndex: 0,
      offset: 0,
    },),
  },],
  lineStructuredSliceIndices: new Set(),
  declaredNames: [],
  alignmentFindings: [],
  unclaimedTargetBlocks: [],
  alignmentPairCount: 1,
};

/**
 Runs the insertion admission seam over the one gap, every seat answering that
 the archive carries none of it.

 @param later - view every reading after the lanes' own answers

 @returns Seats the coverage calls asked

 @example
 ```ts
 const asked = await admissionAsked({ later: SYNTHETIC_DRY, },);
 ```
 */
async function admissionAsked({ later, }: { readonly later: BudgetView; },): Promise<readonly RosterModelId[]> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   Client whose first reading stands for the lanes' own, which seated the
   roster the admission starts on.
   */
  const client = viewChangingClient({
    later,
    asked,
    answer: JSON.stringify({
      coverage: 'none',
      copiedSpan: '',
      assessment: 'the archive carries none of it',
    },),
  },);
  // THE LANES' READING, taken before the admission as `runPassEntry` takes it.
  await client.providerDryness({ signal: new AbortController().signal, },);
  await admitPassInsertions({
    client,
    prepared: GAP,
    modelIds: judgeSeatsFor({ dry: ALL_WET, },).roster,
    overlap: 1,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    entryId: 'CatEntry',
    // The control must ask the seat the dry-out takes, so the whole roster is asked, as in the contest case.
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asked;
}

/**
 Runs the preparation seam over a two-paragraph page, which the pairing's
 block round asks about, every seat pairing the blocks in order.

 @param later - view every reading after the preparation's own answers

 @returns Seats the preparation's calls asked

 @example
 ```ts
 const asked = await preparationAsked({ later: SYNTHETIC_DRY, },);
 ```
 */
async function preparationAsked({ later, }: { readonly later: BudgetView; },): Promise<readonly RosterModelId[]> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  await using cacheDir = await scratchDir({ prefix: 'pass-seams-reseat-', },);
  await runPassPreparation({
    client: viewChangingClient({
      later,
      asked,
      answer: JSON.stringify({ pairs: [{ source: 0, target: 0, }, { source: 1, target: 1, },], },),
    },),
    entryId: 'CatEntry',
    entryCacheDir: cacheDir.path,
    pipelineDigest: DIGEST,
    readPictures: async () => new Map(),
    sourceText: `${SOURCE}\n\n它梦见了鱼。`,
    targetText: `${ARCHIVE}\n\nIt dreamed of fish.`,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    outsideReads: NO_OUTSIDE_READS,
    // The control must ask the seat the dry-out takes, so the whole roster is asked in the pairing round, as in the contest case.
    fanOut: 'whole-bench',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asked;
}

/**
 Runs the preparation seam over a page whose archive carries a paragraph the
 original lacks, which the pairing leaves unclaimed and the archive review is
 asked about.

 @param later - view every reading after the preparation's own answers

 @returns Seats the archive review's calls asked

 @example
 ```ts
 const asked = await archiveReviewAsked({ later: SYNTHETIC_DRY, },);
 ```
 */
async function archiveReviewAsked({ later, }: { readonly later: BudgetView; },): Promise<readonly RosterModelId[]> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   Seat of every call on the archive review sheet.
   */
  const reviewed: RosterModelId[] = [];
  await using cacheDir = await scratchDir({ prefix: 'pass-seams-reseat-', },);
  await runPassPreparation({
    client: viewChangingClient({
      later,
      asked,
      reviewed,
      answer: JSON.stringify({ pairs: [{ source: 0, target: 0, }, { source: 1, target: 1, },], },),
    },),
    entryId: 'CatEntry',
    entryCacheDir: cacheDir.path,
    pipelineDigest: DIGEST,
    readPictures: async () => new Map(),
    sourceText: `${SOURCE}\n\n它梦见了鱼。`,
    targetText: `${ARCHIVE}\n\nIt dreamed of fish.\n\nThe cat won an award.`,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    outsideReads: NO_OUTSIDE_READS,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return reviewed;
}

/**
 Seats either lane's benches hold under a view.

 @param dry - which providers read dry

 @returns Every seat of the repair and translate benches

 @example
 ```ts
 const seats = laneSeatsUnder({ dry: ALL_WET, },);
 ```
 */
function laneSeatsUnder({ dry, }: { readonly dry: BudgetView; },): ReadonlySet<RosterModelId> {
  /**
   Both lanes' benches under that view.
   */
  const {
    repairModels,
    translateModels,
  } = judgeSeatsFor({ dry, },);
  return new Set([
    ...repairModels.criticModelIds,
    ...repairModels.panelModelIds,
    ...repairModels.editorModelIds,
    ...repairModels.judgeModelIds,
    ...translateModels.translatorModelIds,
    ...translateModels.judgeModelIds,
  ],);
}

/**
 Seats the dry-out takes from the lanes' benches.
 */
const LOST_LANE_SEATS: readonly RosterModelId[] = (function lostLaneSeats(): readonly RosterModelId[] {
  /**
   Lane seats while Synthetic is dry.
   */
  const dry = laneSeatsUnder({ dry: SYNTHETIC_DRY, },);
  /**
   Lane seats the dry-out takes.
   */
  const lost = [...laneSeatsUnder({ dry: ALL_WET, },),].filter(function gone(seat,): boolean {
    return !dry.has(seat,);
  },);
  if (lost.length === 0)
    throw new Error('the fixture views seat the same lane benches, so the lanes case cannot tell a wired hook apart',);
  return lost;
})();

/**
 Runs the lanes seam over the one slice, on the benches and hooks a lanes
 reading supplies.

 @param later - view every reading after the lanes' own answers

 @returns Seats the lanes' calls asked

 @example
 ```ts
 const asked = await lanesAsked({ later: SYNTHETIC_DRY, },);
 ```
 */
async function lanesAsked({ later, }: { readonly later: BudgetView; },): Promise<readonly RosterModelId[]> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   Client whose first reading is the lanes' own.
   */
  const client = viewChangingClient({
    later,
    asked,
    answer: '{}',
  },);
  /**
   Entry abort the readings and the lanes honour.
   */
  const signal = AbortSignal.timeout(HANG_STOP_MS,);
  await runPassLanes({
    client,
    prepared: prepareDocumentPair({
      sourceText: SOURCE,
      targetText: ARCHIVE,
    },),
    lanesSeating: await readLanesSeats({
      client,
      signal,
      entryId: 'CatEntry',
    },),
    pictureReadings: new Map(),
    signal,
    entryId: 'CatEntry',
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asked;
}

/**
 Lost lane seats a run asked.

 @param asked - seats the run asked

 @returns Those among them the dry-out takes

 @example
 ```ts
 const lostAsked = lostLaneSeatsIn({ asked, },);
 ```
 */
function lostLaneSeatsIn({ asked, }: { readonly asked: readonly RosterModelId[]; },): readonly RosterModelId[] {
  return asked.filter(function lost(seat,): boolean {
    return LOST_LANE_SEATS.includes(seat,);
  },);
}

/**
 The one picture the pictures case reads.
 */
const PICTURE = 'noticeboard.webp';

/**
 What the stand-in OCR and every reader find on that picture.
 */
const PICTURE_TEXT = '走失猫咪 Mittens，虎斑，请电 555 0134。';

/**
 The one seat the dry-out takes from the picture readers.
 */
const LOST_READER_SEAT = (function lostReaderSeat(): RosterModelId {
  /**
   Readers while Synthetic is dry.
   */
  const dry = new Set(judgeSeatsFor({ dry: SYNTHETIC_DRY, },).readers,);
  /**
   Readers the dry-out takes.
   */
  const lost = judgeSeatsFor({ dry: ALL_WET, },).readers.filter(function gone(seat,): boolean {
    return !dry.has(seat,);
  },);
  if (lost.length === 0)
    throw new Error('the fixture views seat the same readers, so the pictures case cannot tell a wired hook apart',);
  return lost[0] as RosterModelId;
})();

/**
 Reads the one picture through the entry's picture reader, the bytes and the
 OCR handed in so nothing reaches the corpus or the local OCR tools.

 @param later - view every reading after the pictures' own answers

 @param held - whether Synthetic's hold runs, so the hook reads again

 @returns Seats the readers' calls asked, beside how often the stand-in OCR read

 @example
 ```ts
 const { asked, ocrReads, } = await picturesAsked({ later: SYNTHETIC_DRY, held: true, },);
 ```
 */
async function picturesAsked(
  {
    later,
    held,
  }: {
    readonly later: BudgetView;
    readonly held: boolean;
  },
): Promise<{
  readonly asked: readonly RosterModelId[];
  readonly ocrReads: number;
}> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  /**
   Readings the stand-in OCR took, so a case can tell it was the one asked.
   */
  const ocr = { reads: 0, };
  /**
   Entry abort the readings honour.
   */
  const signal = AbortSignal.timeout(HANG_STOP_MS,);
  /**
   The entry's picture reader over stand-in sources.
   */
  const readPictures = createPassPictureReader({
    client: viewChangingClient({
      later,
      asked,
      answer: '{}',
      reading: PICTURE_TEXT,
      held,
    },),
    entryId: 'CatEntry',
    // Invented: the stand-in sources read no clone.
    pin: {
      cloneDir: '/cats/corpus/clone',
      commitSha: 'feedface'.repeat(5,),
    },
    cache: {
      resumed: new Map(),
      persist: async () => {},
    },
    signal,
    l,
    pictureSources: {
      gather: async () => new Map([[PICTURE, new Uint8Array([1, 2, 3,],),],],),
      readOcr: async () => {
        ocr.reads += 1;
        return {
          kind: 'read',
          text: PICTURE_TEXT,
        };
      },
    },
  },);
  await readPictures({
    slices: prepareDocumentPair({
      sourceText: `小猫在窗台上睡觉。\n\n<PhotoScroll photos={[ '\${path}/photos/${PICTURE}' ]} />\n`,
      targetText: ARCHIVE,
    },).slices,
  },);
  return {
    asked,
    ocrReads: ocr.reads,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'pass seams re-seat under a hold (ledger X14)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THE CONTEST SEAM WIRES ITS HOOK: a slice after a dry-out asks the judges the hook re-read, never '
            + 'the seat the dry-out took, which a contest whose view never changes does ask',
          fn: async () => {
            /**
             Seats asked while every provider stays wet.
             */
            const control = await contestAsked({ later: ALL_WET, },);
            /**
             Seats asked once Synthetic and Hyper read dry after the phase's own reading.
             */
            const moved = await contestAsked({ later: SYNTHETIC_AND_HYPER_DRY, },);
            expect({
              controlAskedLost: control.includes(LOST_SEAT,),
              movedAskedAny: moved.length > 0,
              movedAskedLost: moved.includes(LOST_SEAT,),
            },).toEqual({
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: false,
            },);
          },
        },),
        it({
          name: 'THE CONTEST SEAM RECORDS THE TENSE CANDIDATES the judges were shown, beside the verdict, where the entry '
            + 'records a death and names its subject, and records none where it records none',
          fn: async () => {
            /**
             Both lanes over a slice whose repair wording follows the declared name in the present.
             */
            const projected: ProjectedLanes = {
              delivery: {
                repair: PROJECTED.delivery
                  .repair
                  .map(function present(row,): ArtifactDeliveryRow {
                    return {
                      ...row,
                      shippedText: 'Mittens naps by the window.',
                    };
                  },),
                translate: PROJECTED.delivery.translate,
              },
              comparison: PROJECTED.comparison
                .map(function present(row,): ArtifactComparisonRow {
                  return {
                    ...row,
                    repairText: 'Mittens naps by the window.',
                  };
                },),
            };
            /**
             What the seam recorded for an entry that records a death.
             */
            const recorded = await contestRun({
              later: ALL_WET,
              projected,
              lifeEnded: true,
              subjectNames: ['Mittens',],
            },);
            /**
             What the seam recorded for an entry that records none.
             */
            const unrecorded = await contestRun({
              later: ALL_WET,
              projected,
              subjectNames: ['Mittens',],
            },);
            expect({
              recorded: recorded.slices.map(function findingOf(slice,) {
                return slice.tenseCandidates;
              },),
              unrecorded: unrecorded.slices.map(function hasField(slice,): boolean {
                return 'tenseCandidates' in slice;
              },),
            },).toEqual({
              recorded: [
                [
                  {
                    label: 'CANDIDATE "repair"',
                    found: [
                      {
                        sentence: 'Mittens naps by the window.',
                        matched: 'Mittens naps',
                        rule: 's-ending-verb',
                      },
                    ],
                  },
                ],
              ],
              unrecorded: [false,],
            },);
          },
        },),
        it({
          name: 'THE CONTEST SEAM RECORDS NO TENSE CANDIDATES FOR A SOLE-LANE SLICE, which asked no judge and showed no sheet, '
            + 'and records them for the contested slice beside it',
          fn: async () => {
            /**
             Wording that follows a pronoun in the present.
             */
            const presentWording = 'She naps by the window.';
            /**
             Slice 0 an archive-absent insertion only the translate lane filled, and slice 1 contested, both worded in the present.
             */
            const projected: ProjectedLanes = {
              delivery: {
                repair: [
                  {
                    sliceIndex: 0,
                    sourceText: SOURCE,
                    incumbentKind: 'absent',
                    incumbentText: '',
                    outcome: { kind: 'not-applicable', },
                    shippedText: '',
                    delivery: { kind: 'gap-remains', },
                  },
                  ...PROJECTED.delivery
                    .repair
                    .map(function contested(row,): ArtifactDeliveryRow {
                      return {
                        ...row,
                        sliceIndex: 1,
                        shippedText: presentWording,
                      };
                    },),
                ],
                translate: [
                  {
                    sliceIndex: 0,
                    sourceText: SOURCE,
                    incumbentKind: 'absent',
                    incumbentText: '',
                    outcome: {
                      kind: 'decided',
                      acceptedText: presentWording,
                    },
                    shippedText: presentWording,
                    delivery: { kind: 'replacement-shipped', },
                  },
                  ...PROJECTED.delivery
                    .translate
                    .map(function contested(row,): ArtifactDeliveryRow {
                      return {
                        ...row,
                        sliceIndex: 1,
                      };
                    },),
                ],
              },
              comparison: [
                insertionRow({
                  sliceIndex: 0,
                  translateText: presentWording,
                },),
                ...PROJECTED.comparison
                  .map(function contested(row,): ArtifactComparisonRow {
                    return {
                      ...row,
                      sliceIndex: 1,
                      repairText: presentWording,
                    };
                  },),
              ],
            };
            /**
             What the seam recorded for an entry that records a death.
             */
            const recorded = await contestRun({
              later: ALL_WET,
              projected,
              lifeEnded: true,
            },);
            expect(recorded.slices.map(function readOne(slice,) {
              return {
                sliceIndex: slice.sliceIndex,
                verdict: slice.verdict.kind,
                recordsTense: 'tenseCandidates' in slice,
              };
            },),).toEqual([
              {
                sliceIndex: 0,
                verdict: 'sole-lane',
                recordsTense: false,
              },
              {
                sliceIndex: 1,
                verdict: 'lane-won',
                recordsTense: true,
              },
            ],);
          },
        },),
        it({
          name: 'THE CONSOLIDATION SEAM WIRES ITS HOOK: a slice after a dry-out asks the writers, judges and '
            + 'naturalness roles the hook re-read, never the seat the dry-out took, which a consolidation whose '
            + 'view never changes does ask',
          fn: async () => {
            /**
             Seats asked while every provider stays wet.
             */
            const control = await consolidationAsked({ later: ALL_WET, },);
            /**
             Seats asked once Synthetic and Hyper read dry after the phase's own reading.
             */
            const moved = await consolidationAsked({ later: SYNTHETIC_AND_HYPER_DRY, },);
            expect({
              controlAskedLost: control.includes(LOST_SEAT,),
              movedAskedAny: moved.length > 0,
              movedAskedLost: moved.includes(LOST_SEAT,),
            },).toEqual({
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'insertion admission seam re-seats under a hold (ledger X14)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THE INSERTION SEAM WIRES ITS HOOK: a candidate after a dry-out is asked of the roster the hook '
            + 're-read, never the seat the dry-out took, which an admission whose view never changes does ask',
          fn: async () => {
            /**
             Seats asked while every provider stays wet.
             */
            const control = await admissionAsked({ later: ALL_WET, },);
            /**
             Seats asked once Synthetic reads dry after the lanes' reading.
             */
            const moved = await admissionAsked({ later: SYNTHETIC_DRY, },);
            expect({
              controlAskedLost: control.includes(LOST_ROSTER_SEAT,),
              movedAskedAny: moved.length > 0,
              movedAskedLost: moved.includes(LOST_ROSTER_SEAT,),
            },).toEqual({
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'preparation seam re-seats under a hold (ledger X14)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THE PREPARATION SEAM WIRES ITS HOOK: a pairing round after a dry-out asks the roster the hook '
            + 're-read, never the seat the dry-out took, which a preparation whose view never changes does ask',
          fn: async () => {
            /**
             Seats asked while every provider stays wet.
             */
            const control = await preparationAsked({ later: ALL_WET, },);
            /**
             Seats asked once Synthetic reads dry after the preparation's own reading.
             */
            const moved = await preparationAsked({ later: SYNTHETIC_DRY, },);
            expect({
              controlAskedLost: control.includes(LOST_ROSTER_SEAT,),
              movedAskedAny: moved.length > 0,
              movedAskedLost: moved.includes(LOST_ROSTER_SEAT,),
            },).toEqual({
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'preparation seam re-seats the archive review under a hold (ledger X12)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THE PREPARATION HANDS ITS HOOK TO THE ARCHIVE REVIEW: a review after a dry-out asks the roster '
            + 'the hook re-read, never the seat the dry-out took, which a review whose view never changes does ask',
          fn: async () => {
            /**
             Review seats asked while every provider stays wet.
             */
            const control = await archiveReviewAsked({ later: ALL_WET, },);
            /**
             Review seats asked once Synthetic reads dry after the preparation's own reading.
             */
            const moved = await archiveReviewAsked({ later: SYNTHETIC_DRY, },);
            expect({
              controlAskedLost: control.includes(LOST_ROSTER_SEAT,),
              movedAskedAny: moved.length > 0,
              movedAskedLost: moved.includes(LOST_ROSTER_SEAT,),
            },).toEqual({
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: false,
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'lanes seam re-seats under a hold (ledger X14)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THE LANES SEAM WIRES BOTH LANES\' HOOKS: a lane after a dry-out asks the benches the hooks re-read, '
            + 'never a seat the dry-out took, which lanes whose view never changes do ask',
          fn: async () => {
            /**
             Seats asked while every provider stays wet.
             */
            const control = await lanesAsked({ later: ALL_WET, },);
            /**
             Seats asked once Synthetic reads dry after the lanes' own reading.
             */
            const moved = await lanesAsked({ later: SYNTHETIC_DRY, },);
            expect({
              controlAskedLost: lostLaneSeatsIn({ asked: control, },).length > 0,
              movedAskedAny: moved.length > 0,
              movedAskedLost: lostLaneSeatsIn({ asked: moved, },),
            },).toEqual({
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: [],
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'pictures seam re-seats under a hold (ledger X14)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THE PICTURES SEAM WIRES ITS HOOK: a picture after a dry-out is read by the readers the hook re-read, '
            + 'never the seat the dry-out took, which a reading whose view never changes does ask',
          fn: async () => {
            /**
             Seats asked while every provider stays wet.
             */
            const control = await picturesAsked({ later: ALL_WET, held: true, },);
            /**
             Seats asked once Synthetic reads dry after the pictures' own reading.
             */
            const moved = await picturesAsked({ later: SYNTHETIC_DRY, held: true, },);
            expect({
              controlReadOcr: control.ocrReads > 0,
              controlAskedLost: control.asked.includes(LOST_READER_SEAT,),
              movedAskedAny: moved.asked.length > 0,
              movedAskedLost: moved.asked.includes(LOST_READER_SEAT,),
            },).toEqual({
              controlReadOcr: true,
              controlAskedLost: true,
              movedAskedAny: true,
              movedAskedLost: false,
            },);
          },
        },),
        it({
          name: 'THE PICTURES SEAM SEATS THE READERS while nothing is held: every picture is read by the readers '
            + 'the stage\'s own reading seated, and by no seat off that bench',
          fn: async () => {
            /**
             Seats asked with no hold running, so no hook reads again.
             */
            const unheld = await picturesAsked({ later: SYNTHETIC_DRY, held: false, },);
            /**
             Readers the stage's own reading seats.
             */
            const readers: readonly RosterModelId[] = judgeSeatsFor({ dry: ALL_WET, },).readers;
            expect({
              askedAny: unheld.asked.length > 0,
              offReaders: unheld.asked.filter(function offBench(seat,): boolean {
                return !readers.includes(seat,);
              },),
            },).toEqual({
              askedAny: true,
              offReaders: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
