/**
 Guards the hand-off of the translate lane's ballots to the consolidation (owner, 2026-10-29, "Ship the two-ballot
 text (Recommended)"): a slice whose archive the repair panel disputed has no eligible standing, the translate lane
 holds a text two judges chose that passes the rule, and the slate bench reads one judge reachable. The consolidation
 ships that text and buys nothing (XingZ6022: sixteen slices, 65 calls, 10.679 Wh). A slice whose translate text drew
 no ballot count asks as before.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ArchiveDispute, } from '@monochromatic-dev/translation-repair-archive';
import type { ConsolidateSliceSeating, ProjectedLanes, } from '@monochromatic-dev/translation-repair-artifact';
import type { ConsolidationSettlement, } from '@monochromatic-dev/translation-repair-consolidate';
import type { ArtifactContestSlice, } from '@monochromatic-dev/translation-repair-lane-contest';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import { consolidateDocument, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the driver writes through, whose output is not under test.
 */
const l = tagged({ tag: 'consolidate-driver-translate-winner-test', },);

/**
 Writers and slate judges of the cases.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 The archive's rendering, which the repair panel disputed: it adds a purr the original never states.
 */
const ARCHIVE = 'The cat dozed on the windowsill and purred for you.';

/**
 What the translate lane would ship, which two of its judges chose.
 */
const TRANSLATE_TEXT = 'The cat naps on the sill, tail beside the radiator.';

/**
 The one slice: both lanes word it, the repair lane left the archive's wording, the translate lane wrote its own.
 */
const PROJECTED = {
  comparison: [{
    sliceIndex: 0,
    incumbentKind: 'present',
    incumbentText: ARCHIVE,
    repairText: ARCHIVE,
    translateText: TRANSLATE_TEXT,
  },],
  delivery: {
    repair: [{
      sliceIndex: 0,
      sourceText: '猫猫在窗台上打盹，尾巴垂在暖气片旁边。',
    },],
    translate: [],
  },
} as unknown as ProjectedLanes;

/**
 The contest backed the repair lane, whose text is the archive the panel disputed.
 */
const CONTESTS: readonly ArtifactContestSlice[] = [{
  sliceIndex: 0,
  verdict: {
    kind: 'lane-won',
    lane: 'repair',
  },
  ballots: [],
  usable: ROSTER.length,
},];

/**
 The dispute: accepted claims against the archive, with no repair that the checkers confirmed, so the archive and the
 repair lane's text are wordings the slice refuses.
 */
const DISPUTES: ReadonlyMap<number, ArchiveDispute> = new Map([[
  0,
  {
    sliceIndex: 0,
    standIn: ARCHIVE,
    standInEligible: false,
    standInRefusal: 'archive-wording',
    acceptedClaims: ['accuracy/addition major: the cat does not purr.',],
  },
],],);

/**
 Drives the slice with a stated ballot count, and counts what it asked of the providers.

 @param translateWinnerBallots - per-slice ballots of the translate lane, none left out

 @returns Calls made, the settled records, and what the cache was asked to keep

 @example
 ```ts
 const { calls, slices, } = await drive({ translateWinnerBallots: new Map([[0, 2,],],), },);
 ```
 */
async function drive(
  { translateWinnerBallots, }: { readonly translateWinnerBallots?: ReadonlyMap<number, number>; },
): Promise<{
  readonly calls: number;
  readonly terminal?: string;
  readonly shippedKind?: string;
  readonly shippedText?: string;
  readonly written: number;
}> {
  /**
   Calls the refusing client received.
   */
  const asked = { calls: 0, };
  /**
   Client that refuses every call and counts them.
   */
  const client: ModelClient = {
    chatText: async () => {
      asked.calls += 1;
      throw new Error('scripted refusal',);
    },
    chatJson: async <ValueT,>(_request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      asked.calls += 1;
      throw new Error('scripted refusal',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the consolidation',);
    },
  };
  /**
   Keys the driver decided were worth resuming later.
   */
  const written: string[] = [];
  /**
   Cache standing in for the entry store, recording every write.
   */
  const cache: SliceCache<ConsolidationSettlement> = {
    resumed: new Map(),
    persist: async function recordWrite({ key, }: { readonly key: string; },): Promise<void> {
      written.push(key,);
    },
  };
  try {
    const slices = await consolidateDocument({
      client,
      projected: PROJECTED,
      contests: CONTESTS,
      modelIds: ROSTER,
      frontMatterSlices: new Set(),
      lineStructuredSlices: new Set(),
      pictureContextBySlice: new Map(),
      neighbourContextBySlice: new Map(),
      cache,
      signal: AbortSignal.timeout(HANG_STOP_MS,),
      perCallTimeoutMs: HANG_STOP_MS,
      beforeSlice: async (): Promise<ConsolidateSliceSeating> => ({ slateReachable: 1, }),
      archiveDisputes: DISPUTES,
      ...((translateWinnerBallots === undefined) ? {} : { translateWinnerBallots, }),
      l,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
    /**
     The one slice's record.
     */
    const [slice,] = slices;
    return {
      calls: asked.calls,
      ...((slice === undefined) ? {} : { terminal: slice.terminal, shippedKind: slice.shipped.kind, }),
      ...((slice?.shipped.kind === 'translate-winner') ? { shippedText: slice.shipped.text, } : {}),
      written: written.length,
    };
  }
  catch (error) {
    // A purchase whose producers all refuse ends in the stage's interruption, which is the ask this counts.
    if (!(error instanceof TranslationRepairInterruptedError))
      throw error;
    return {
      calls: asked.calls,
      written: written.length,
    };
  }
}

await describe({
  name: consolidateDocument.name,
  children: [
    it({
      name: 'SHIPS THE TRANSLATE LANE\'S TWO-BALLOT TEXT over a disputed archive and buys nothing where one slate judge '
        + 'is reachable, recording the shipped kind and keeping nothing for a later run',
      fn: async () => {
        expect(await drive({ translateWinnerBallots: new Map([[0, 2,],],), },),).toEqual({
          calls: 0,
          terminal: 'incumbent-only',
          shippedKind: 'translate-winner',
          shippedText: TRANSLATE_TEXT,
          written: 0,
        },);
      },
    },),
    it({
      name: 'ASKS AS BEFORE where the translate lane drew one ballot, or where the caller counted none',
      fn: async () => {
        /**
         Ballots of one: one judge alone never decides.
         */
        const oneBallot = await drive({ translateWinnerBallots: new Map([[0, 1,],],), },);
        /**
         No count at all.
         */
        const uncounted = await drive({},);
        expect({
          oneBallotAsked: oneBallot.calls > 0,
          uncountedAsked: uncounted.calls > 0,
          oneBallotShipped: oneBallot.shippedKind,
          uncountedShipped: uncounted.shippedKind,
        },).toEqual({
          oneBallotAsked: true,
          uncountedAsked: true,
          oneBallotShipped: undefined,
          uncountedShipped: undefined,
        },);
      },
    },),
  ],
},);
