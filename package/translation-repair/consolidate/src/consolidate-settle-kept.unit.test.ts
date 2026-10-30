/**
 Tests for the exit of a settlement whose slate judges left the standing in place: a standing the rule refused gives way
 to the translate lane's two-ballot text or to the archive (owner, 2026-09-27 and 2026-10-29, "Ship the two-ballot
 text"), an eligible standing the contest did not endorse gives way to that text where the slate heard fewer than two
 judges and the tense detector flags the standing and not the text (owner, 2026-10-29, "Swap only where flagged"), and
 anything else goes to the final polish.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { TranslateStageResult, } from '@monochromatic-dev/translation-repair-translate-stage';
import {
  type ConsolidationSettlement,
  settleKeptStanding,
} from '../dist/final/node/index.mjs';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the exit writes through, whose output is not under test.
 */
const l = tagged({ tag: 'consolidate-settle-kept-test', },);

/**
 Invented original: the cat was a tabby with a white chin.
 */
const SOURCE = '猫猫是一只下巴雪白的虎斑猫。';

/**
 A standing the detector flags: a copula in the present directly after the subject's name.
 */
const FLAGGED_STANDING = 'Mittens is a tabby with a white chin.';

/**
 A text the detector does not flag: the same sentence told in the past.
 */
const PAST_TEXT = 'Mittens was a tabby with a white chin.';

/**
 What the floor made of a slate with nothing valid on it.
 */
const FLOOR = {
  kind: 'incumbent-only',
  refusedModelIds: [],
} as const;

/**
 The translate lane's text as the lane offer hands it over once it passed the rule.
 */
const TRANSLATE_LANE: LaneText = {
  lane: 'translate',
  text: PAST_TEXT,
};

/**
 Client that fails the case if any exchange is made: the exits under test ask nobody.
 */
const SILENT_CLIENT: ModelClient = {
  chatText: async () => {
    throw new Error('no exchange expected',);
  },
  quotas: async () => {
    throw new Error('no exchange expected',);
  },
  chatJson: async () => {
    throw new Error('no exchange expected',);
  },
};

/**
 Settles a slate that left the standing in place.

 @param standingEligible - whether the standing passed the deterministic gate
 @param standingMayShip - whether the contest endorsed the standing
 @param heard - judges the slate round heard
 @param laneTexts - lane texts the slate offered

 @returns Settlement

 @example
 ```ts
 const settled = await keptWith({ standingEligible: true, standingMayShip: false, heard: 1, laneTexts: [TRANSLATE_LANE,], });
 ```
 */
async function keptWith(
  {
    standingEligible,
    standingMayShip,
    heard,
    laneTexts,
  }: {
    readonly standingEligible: boolean;
    readonly standingMayShip: boolean;
    readonly heard: number;
    readonly laneTexts: readonly LaneText[];
  },
): Promise<ConsolidationSettlement> {
  return await settleKeptStanding({
    client: SILENT_CLIENT,
    decided: {
      tally: { ballots: heard, },
      findings: [],
    } as unknown as TranslateStageResult,
    terminal: 'slate-declined-standing',
    standingText: FLAGGED_STANDING,
    floor: FLOOR,
    verdicts: [],
    standingEligible,
    standingMayShip,
    laneTexts,
    translateWinnerBallots: 2,
    subject: {
      sourceText: SOURCE,
      incumbentText: FLAGGED_STANDING,
      lifeEnded: true,
      subjectNames: ['Mittens',],
    },
    lineStructured: false,
    sliceIndex: 5,
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 What a case reads off a settlement.

 @param settled - settlement

 @returns Fields the owner's rulings decide

 @example
 ```ts
 const read = readOf({ settled, });
 ```
 */
function readOf({ settled, }: { readonly settled: ConsolidationSettlement; },): Readonly<Record<string, unknown>> {
  return {
    text: settled.text,
    archiveReason: settled.archiveReason,
    translateWinnerShipped: settled.translateWinnerShipped,
    translateWinnerReason: settled.translateWinnerReason,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: settleKeptStanding.name,
      children: [
        it({
          name: 'SHIPS THE TRANSLATE TEXT over a standing the rule refused, after a slate that heard one judge, '
            + 'and keeps the archive with the reason where the offer holds no such text or the slate heard two',
          fn: async () => {
            expect({
              shipped: readOf({
                settled: await keptWith({
                  standingEligible: false,
                  standingMayShip: false,
                  heard: 1,
                  laneTexts: [TRANSLATE_LANE,],
                },),
              },),
              notOffered: readOf({
                settled: await keptWith({
                  standingEligible: false,
                  standingMayShip: false,
                  heard: 1,
                  laneTexts: [],
                },),
              },),
              seated: readOf({
                settled: await keptWith({
                  standingEligible: false,
                  standingMayShip: false,
                  heard: 2,
                  laneTexts: [TRANSLATE_LANE,],
                },),
              },),
            },).toEqual({
              shipped: {
                text: PAST_TEXT,
                archiveReason: undefined,
                translateWinnerShipped: true,
                translateWinnerReason: 'refused-standing',
              },
              notOffered: {
                text: FLAGGED_STANDING,
                archiveReason: 'final-round-unseated',
                translateWinnerShipped: undefined,
                translateWinnerReason: undefined,
              },
              seated: {
                text: FLAGGED_STANDING,
                archiveReason: 'slate-declined',
                translateWinnerShipped: undefined,
                translateWinnerReason: undefined,
              },
            },);
          },
        },),
        it({
          name: 'SHIPS THE TRANSLATE TEXT over an eligible standing the contest did not endorse where the slate heard '
            + 'one judge and the detector flags the standing and not the text, and goes on to the polish otherwise',
          fn: async () => {
            expect({
              swapped: readOf({
                settled: await keptWith({
                  standingEligible: true,
                  standingMayShip: false,
                  heard: 1,
                  laneTexts: [TRANSLATE_LANE,],
                },),
              },),
              seated: readOf({
                settled: await keptWith({
                  standingEligible: true,
                  standingMayShip: false,
                  heard: 2,
                  laneTexts: [TRANSLATE_LANE,],
                },),
              },),
              endorsed: readOf({
                settled: await keptWith({
                  standingEligible: true,
                  standingMayShip: true,
                  heard: 1,
                  laneTexts: [],
                },),
              },),
            },).toEqual({
              swapped: {
                text: PAST_TEXT,
                archiveReason: undefined,
                translateWinnerShipped: true,
                translateWinnerReason: 'present-tense-standing',
              },
              seated: {
                text: FLAGGED_STANDING,
                archiveReason: undefined,
                translateWinnerShipped: undefined,
                translateWinnerReason: undefined,
              },
              endorsed: {
                text: FLAGGED_STANDING,
                archiveReason: undefined,
                translateWinnerShipped: undefined,
                translateWinnerReason: undefined,
              },
            },);
          },
        },),
      ],
    },),
  ],
},);
