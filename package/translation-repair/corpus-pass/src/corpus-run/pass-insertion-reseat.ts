import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import type {
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { sliceTagged, } from '@monochromatic-dev/translation-repair-typography/ts';
import { decidePassInsertionAdmission, } from './pass-insertion-admission.ts';
import {
  type Reseated,
  reseatHookFor,
} from './pass-reseat-hook.ts';
import {
  type JudgeSeats,
  RUN_PER_CALL_TIMEOUT_MS,
  type RunClient,
  type SeatReadingClient,
} from '@monochromatic-dev/translation-repair-run/ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Insertion admission re-seating
// THE INSERTION ADMISSION'S PER-CANDIDATE HOOK (ledger X12), beside the other
// phases' hooks, so a test can drive it with the reading rig they are tested
// with.

/**
 Hook the insertion admission calls before each candidate's coverage question.

 @example
 ```ts
 const hooks: InsertionHooks = insertionHooksFor({ client, signal, l, },);
 ```
 */
export type InsertionHooks = {
  readonly beforeCandidate: () => Promise<BenchSeating>;
};

/**
 What the insertion admission takes from a seat reading: the roster, as
 `runPassEntry` seats it from the lanes' reading.

 @param seats - benches as of this candidate

 @returns Roster the candidate is asked of, beside the line naming it

 @example
 ```ts
 const { seating, line, } = coverageSeatingOf({ seats, },);
 ```
 */
function coverageSeatingOf(
  { seats, }: { readonly seats: JudgeSeats; },
): Reseated<BenchSeating> {
  /**
   Roster the candidate is asked of, for the line.
   */
  const roster = seats.roster
    .join(',',);
  return {
    seating: { modelIds: seats.roster, },
    line: `candidate re-seated under a hold: roster=${roster}`,
  };
}

/**
 Builds the insertion admission's per-candidate hook.

 @param client - run client whose dryness view and holds are the router's own

 @param signal - entry abort the readings honour

 @param l - entry logger

 @returns Per-candidate reader, handing each candidate the roster it is asked of

 @example
 ```ts
 const hooks = insertionHooksFor({ client, signal, l, },);
 ```
 */
export function insertionHooksFor(
  {
    client,
    signal,
    l,
  }: {
    readonly client: SeatReadingClient;
    readonly signal: AbortSignal;
    readonly l: Logger;
  },
): InsertionHooks {
  return {
    beforeCandidate: reseatHookFor<BenchSeating>({
      client,
      signal,
      phase: 'insertion admission',
      unseated: {},
      seatingOf: coverageSeatingOf,
      l,
    },),
  };
}

/**
 Runs the insertion admission for one entry with its per-candidate hook
 wired, the seam `runPassEntry` calls (ledger X12 and X14: split out so a
 test can drive the wiring, as it drives the contest's and the
 consolidation's).

 @param client - run client whose dryness view and holds the hook reads

 @param prepared - entry's preparation carrying its insertion slices

 @param modelIds - roster the lanes' reading seated, which each candidate is
 asked of until a hold re-seats it

 @param overlap - most coverage questions in flight

 @param signal - entry deadline and caller abort

 @param entryId - entry the log lines are tagged with

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 the window is rotated by the sheet text and a sheet edit elsewhere moves
 which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Admitted positions and count-only evidence for every candidate

 @example
 ```ts
 const admission = await admitPassInsertions({ client, prepared, modelIds, overlap, signal, entryId, stageSettings, },);
 ```
 */
export async function admitPassInsertions(
  {
    client,
    prepared,
    modelIds,
    overlap,
    signal,
    entryId,
    fanOut,
    stageSettings,
  }: {
    readonly client: RunClient;
    readonly prepared: PreparedDocumentPair;
    readonly modelIds: readonly RosterModelId[];
    readonly overlap: number;
    readonly signal: AbortSignal;
    readonly entryId: string;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  },
): Promise<InsertionAdmission> {
  /**
   Entry logger the admission and its hook write to, which names the attempt inside one (ledger B963).
   */
  const l = sliceTagged({ l: tagged({ tag: entryId, },), },);
  return await decidePassInsertionAdmission({
    client,
    prepared,
    modelIds,
    overlap,
    signal,
    perCallTimeoutMs: RUN_PER_CALL_TIMEOUT_MS,
    l,
    // EVERY CANDIDATE RE-SEATS UNDER A HOLD (ledger X12).
    beforeCandidate: insertionHooksFor({
      client,
      signal,
      l,
    },)
      .beforeCandidate,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    stageSettings,
  },);
}

//endregion Insertion admission re-seating
