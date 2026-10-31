import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import {
  runLogger,
  sliceTagged,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type PassPreparation,
  preparePassEntry,
} from './pass-prepare.ts';
import {
  type Reseated,
  reseatHookFor,
} from './pass-reseat-hook.ts';
import type { PassOutsideReads, } from './pass-outside-reads.ts';
import type { PassVisualEvidenceReader, } from './pass-visual-evidence.ts';
import type { PipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type JudgeSeats,
  readJudgeSeats,
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
  type SeatReadingClient,
} from '@monochromatic-dev/translation-repair-run/ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Preparation re-seating
// THE PREPARATION'S PER-ITEM HOOK (ledger X12), beside the other phases'
// hooks, so a test can drive it with the reading rig they are tested with.
// One hook serves every stage that asks the roster: the attestation, each
// section and block pairing round, and each archive block's review.

/**
 Hook the preparation calls before each item it asks the roster about.

 @example
 ```ts
 const hooks: PreparationHooks = preparationHooksFor({ client, signal, l, },);
 ```
 */
export type PreparationHooks = {
  readonly beforeItem: () => Promise<BenchSeating>;
};

/**
 What the preparation takes from a seat reading: the roster, as
 `runPassPreparation` seats it.

 @param seats - benches as of this item

 @returns Roster the item is asked of, beside the line naming it

 @example
 ```ts
 const { seating, line, } = preparationSeatingOf({ seats, },);
 ```
 */
function preparationSeatingOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Reseated<BenchSeating> {
  /**
   Roster the item is asked of, for the line.
   */
  const roster = seats.roster
    .join(',',);
  return {
    seating: { modelIds: seats.roster, },
    line: `stage re-seated under a hold: roster=${roster}`,
  };
}

/**
 Builds the preparation's per-item hook.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param l - entry logger

 @returns Per-item reader, handing each item the roster it is asked of

 @example
 ```ts
 const hooks = preparationHooksFor({ client, signal, l, },);
 ```
 */
export function preparationHooksFor(
  {
    client,
    signal,
    l,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): PreparationHooks {
  return {
    beforeItem: reseatHookFor<BenchSeating>({
      client,
      signal,
      phase: 'preparation',
      unseated: {},
      seatingOf: preparationSeatingOf,
      l,
    },),
  };
}

/**
 Prepares one entry on the roster the meters seat, with its per-item hook
 wired: the seam `runPassEntry` calls (ledger X12 and X14: split out so a
 test can drive the wiring, as it drives the other phases').

 @param client - run client whose dryness view seats the roster and whose
 chat surface asks it

 @param entryId - corpus entry prepared, which tags the log lines

 @param entryCacheDir - per-entry cache root

 @param pipelineDigest - generation stamp for the caches

 @param readPictures - shared entry reader supplying picture support

 @param sourceText - original

 @param targetText - archive as the corpus holds it

 @param signal - entry deadline and caller abort

 @param outsideReads - what the preparation reads from outside the pipeline
 (ledger X19): the readers the pass's entry file builds over its environment
 in a run

 @param fanOut - seats a pairing round asks: the window of quorum plus one
 by default, or the whole bench a fixture scripting every seat asks for,
 since the window is rotated by the sheet text and a sheet edit elsewhere
 moves which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Prepared slices, every finding on their `alignmentFindings`

 @example
 ```ts
 const paired = await runPassPreparation({ client, entryId, entryCacheDir, pipelineDigest, readPictures, sourceText, targetText, signal, outsideReads, stageSettings, },);
 ```
 */
export async function runPassPreparation(
  {
    client,
    entryId,
    entryCacheDir,
    pipelineDigest,
    readPictures,
    sourceText,
    targetText,
    signal,
    outsideReads,
    fanOut,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly entryId: string;
    readonly entryCacheDir: string;
    readonly pipelineDigest: PipelineDigest;
    readonly readPictures: PassVisualEvidenceReader;
    readonly sourceText: string;
    readonly targetText: string;
    readonly signal: AbortSignal;
    readonly outsideReads: PassOutsideReads;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  },
): Promise<PassPreparation> {
  /**
   Entry logger the preparation and its hook write to, which names the attempt inside one (ledger B963).
   */
  const l = sliceTagged({ l: tagged({ tag: entryId, l: runLogger, },), },);
  /**
   The roster preparation asks, read off the meters first of all
   (`run-seats.ts`): a withheld model pairs no blocks and reviews no
   archive either, and the pairing round is the entry's first purchase.
   */
  const seats = await readJudgeSeats({
    client,
    phase: 'preparation',
    signal,
    l,
  },);
  return await preparePassEntry({
    client,
    entryId,
    entryCacheDir,
    pipelineDigest,
    modelIds: seats.roster,
    readPictures,
    sourceText,
    targetText,
    signal,
    exchangeTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    // EVERY ITEM RE-SEATS UNDER A HOLD (ledger X12).
    beforeItem: preparationHooksFor({
      client,
      signal,
      l,
    },)
      .beforeItem,
    outsideReads,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);
}

//endregion Preparation re-seating
