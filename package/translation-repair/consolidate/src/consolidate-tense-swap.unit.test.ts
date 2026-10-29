/**
 Guards the owner's answer of 2026-10-29 to B1050, "Swap only where flagged (Recommended)": when no vote can seat two
 judges, the two-ballot translate text ships instead of an eligible standing the contest did not endorse, only where
 the tense detector flags present tense in the standing and does not flag the translate text. The owner accepts the
 detector deciding here, a narrow exception to the ruling of 2026-10-11 that made it advisory.

 XingZ6022 (2026-10-29) shipped a life that had ended in the present tense at six slices (11, 16, 42, 43, 77 and 80
 flagged the standing and not the translate text) because the slate heard one judge and kept the standing.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import type { LaneText, } from '@monochromatic-dev/translation-repair-translate-floor';
import {
  buyConsolidationSlice,
  type ConsolidationSettlement,
  consolidationWorthResuming,
  settleConsolidation,
  standingKeptUnendorsed,
} from '../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Invented original: the cat was a tabby with a white chin.
 */
const SOURCE = '猫猫是一只下巴雪白的虎斑猫。';

/**
 Name the translation declares for its subject, which the detector reads as a named subject beside a verb.
 */
const SUBJECT_NAMES = ['Mittens',] as const;

/**
 A standing the detector flags: a copula in the present directly after the subject's name.
 */
const FLAGGED_STANDING = 'Mittens is a tabby with a white chin.';

/**
 A text the detector does not flag: the same sentence told in the past.
 */
const PAST_TEXT = 'Mittens was a tabby with a white chin.';

/**
 A text the detector flags too: past tense followed by a present-tense sentence about the cat.
 */
const FLAGGED_TRANSLATE = 'Mittens was a tabby with a white chin. She naps on the sill.';

/**
 The flagged sentence inside quoted speech, which the detector blanks and so does not flag.
 */
const QUOTED_STANDING = 'The vet said, “Mittens is a tabby with a white chin.” Then she left.';

/**
 The words of the quoted standing with the marks taken away, the control that the quotation is what keeps it unflagged.
 */
const UNQUOTED_STANDING = 'The vet said, Mittens is a tabby with a white chin. Then she left.';

/**
 Writers and slate judges of the cases.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Counts of what a scripted client was asked.
 */
type Asked = {
  /**
   Every call of any kind.
   */
  calls: number;
};

/**
 Client under which every slate judge names no candidate.

 @param asked - where the calls are counted

 @returns Client over every stage these cases reach

 @example
 ```ts
 const client = rejectingClient({ asked: { calls: 0, }, });
 ```
 */
function rejectingClient({ asked, }: { readonly asked: Asked; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by these stages',);
    },
    quotas: async () => {
      throw new Error('quotas unused by these stages',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.calls += 1;
      /**
       Stage the request belongs to, by its reply schema.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Reply for the stage.
       */
      const value: unknown = (schema === 'candidate_ballot')
        ? {
          best: 0,
          assessment: 'No candidate renders the original without an addition.',
        }
        : (schema === 'refine_report')
          ? { rewrites: [], }
          : (schema === 'absolute_naturalness_review')
            ? {
              isAcceptable: true,
              findings: [],
              assessment: 'publication-ready',
            }
            : undefined;
      if (value === undefined)
        throw new Error(`no fixture reply for ${schema}`,);
      if (!request.validate(value,))
        throw new Error(`fixture ${schema} reply failed validation`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 What one case varies about the slice.
 */
type Case = {
  /**
   Wording in place, eligible for the deterministic rule.
   */
  readonly standing: string;

  /**
   The translate lane's text on the offer.
   */
  readonly translate: string;

  /**
   Slate judges seated, who all name no candidate.
   */
  readonly judgeCount: number;

  /**
   Ballots that named the translate lane's text in its own round.
   */
  readonly ballots?: number;

  /**
   Slate judges a wet provider would serve, absent where the caller did not count.
   */
  readonly slateReachable?: number;

  /**
   Whether the entry records a death.
   */
  readonly lifeEnded?: boolean;

  /**
   Whether the contest endorsed the standing.
   */
  readonly standingMayShip?: boolean;
};

/**
 What a case reads off a settlement and the run around it.
 */
type Outcome = {
  /**
   The settlement.
   */
  readonly settled: ConsolidationSettlement;

  /**
   Calls the client was asked.
   */
  readonly calls: number;

  /**
   Lines the stage logged.
   */
  readonly lines: readonly string[];
};

/**
 Settles an eligible standing the contest did not endorse, with the translate lane's text on the offer.

 @param shape - what the case varies
 @param shape.standing - wording in place
 @param shape.translate - translate lane's text
 @param shape.judgeCount - slate judges seated
 @param shape.ballots - ballots for the translate text, two when absent
 @param shape.slateReachable - slate judges reachable, absent where uncounted
 @param shape.lifeEnded - whether the entry records a death, true when absent
 @param shape.standingMayShip - whether the contest endorsed the standing, false when absent

 @returns Settlement, calls and log lines

 @example
 ```ts
 const outcome = await settleOver({ standing: FLAGGED_STANDING, translate: PAST_TEXT, judgeCount: 1, });
 ```
 */
async function settleOver(shape: Case,): Promise<Outcome> {
  const {
    standing,
    translate,
    judgeCount,
    ballots = 2,
    slateReachable,
    lifeEnded = true,
    standingMayShip = false,
  } = shape;
  const asked: Asked = { calls: 0, };
  const lines: string[] = [];
  /**
   The translate lane's text as the lane offer hands it over once it passed the rule.
   */
  const laneTexts: readonly LaneText[] = [
    {
      lane: 'translate',
      text: translate,
    },
  ];
  const settled = await settleConsolidation({
    client: rejectingClient({ asked, },),
    roster: ROSTER,
    judgeModelIds: ROSTER.slice(0, judgeCount,),
    subject: {
      sourceText: SOURCE,
      incumbentText: standing,
      ...(lifeEnded ? { lifeEnded: true, subjectNames: [...SUBJECT_NAMES,], } : {}),
    },
    // NO WRITER IS HEARD: the slate is the lane text alone, so no proposal is wrapped through the lint binary, whose
    // pinned version this tree's shared binary does not match until the typography round lands.
    voices: [],
    validity: [],
    producedFindings: [],
    standingText: standing,
    lineStructured: false,
    sliceIndex: 4,
    standingMayShip,
    standingEligible: true,
    laneTexts,
    translateWinnerBallots: ballots,
    ...((slateReachable === undefined) ? {} : { slateReachable, }),
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: tagged({
      tag: 'consolidate-tense-swap-test',
      l: capturingLogger({ messages: lines, },),
    },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    settled,
    calls: asked.calls,
    lines,
  };
}

/**
 What a case reads off a settlement.

 @param settled - settlement

 @returns Fields the owner's rule decides

 @example
 ```ts
 const read = readSettlement({ settled, });
 ```
 */
function readSettlement(
  { settled, }: { readonly settled: ConsolidationSettlement; },
): Readonly<Record<string, unknown>> {
  /**
   The settlement as a loose record, so a field a build lacks reads as absent.
   */
  const loose = settled as unknown as Readonly<Record<string, unknown>>;
  return {
    text: settled.text,
    archiveKept: settled.archiveKept,
    translateWinnerShipped: loose.translateWinnerShipped,
    translateWinnerReason: loose.translateWinnerReason,
    named: settled.findings.some(function namesIt(finding,): boolean {
      return finding.startsWith('translate-winner-ships-over-present-tense-standing',);
    },),
  };
}

/**
 What a settlement that kept the standing reads as.

 @param standing - wording in place

 @returns Fields of a settlement that shipped the standing and no translate text

 @example
 ```ts
 expect(readSettlement({ settled, },),).toEqual(keptStanding({ standing: FLAGGED_STANDING, },),);
 ```
 */
function keptStanding({ standing, }: { readonly standing: string; },): Readonly<Record<string, unknown>> {
  return {
    text: standing,
    archiveKept: undefined,
    translateWinnerShipped: undefined,
    translateWinnerReason: undefined,
    named: false,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: 'the translate text ships over an unendorsed standing the detector flags (owner, 2026-10-29)',
      children: [
        it({
          name: 'SHIPS THE TRANSLATE TEXT after a one-judge slate where the standing is flagged and the translate '
            + 'text is not, keeps nothing for a later run, and the log says why',
          fn: async () => {
            const {
              settled,
              lines,
            } = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 1,
            },);
            expect({
              ...readSettlement({ settled, },),
              terminal: settled.terminal,
              resumable: consolidationWorthResuming({ settlement: settled, standingMayShip: false, },),
              unendorsed: standingKeptUnendorsed({ settlement: settled, standingMayShip: false, },),
            },).toEqual({
              text: PAST_TEXT,
              archiveKept: undefined,
              translateWinnerShipped: true,
              translateWinnerReason: 'present-tense-standing',
              named: true,
              terminal: 'slate-declined-standing',
              resumable: false,
              unendorsed: false,
            },);
            // The line names the slice, what the detector made of each text, and the owner's answer.
            const said = lines.filter(function namesSwap(line,): boolean {
              return line.includes('slice 4',) && line.includes('present-tense',);
            },);
            expect(said.length,).toBe(1,);
            expect(said[0],).toContain('standing: 1 candidate',);
            expect(said[0],).toContain('translate text: 0 candidates',);
            expect(said[0],).toContain('Swap only where flagged',);
          },
        },),
        it({
          name: 'SHIPS THE TRANSLATE TEXT with no slate bought where fewer slate judges are reachable than a winner '
            + 'needs, and the log says why',
          fn: async () => {
            const {
              settled,
              calls,
              lines,
            } = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 3,
              slateReachable: 1,
            },);
            expect({
              calls,
              ...readSettlement({ settled, },),
              said: lines.some(function namesSwap(line,): boolean {
                return line.includes('slice 4',) && line.includes('Swap only where flagged',);
              },),
            },).toEqual({
              calls: 0,
              text: PAST_TEXT,
              archiveKept: undefined,
              translateWinnerShipped: true,
              translateWinnerReason: 'present-tense-standing',
              named: true,
              said: true,
            },);
          },
        },),
        it({
          name: 'BUYS NOTHING for a flagged unendorsed standing when fewer slate judges are reachable than a winner '
            + 'needs: no writer is paid for a slate whose answer is already known',
          fn: async () => {
            const asked: Asked = { calls: 0, };
            const settled = await buyConsolidationSlice({
              client: rejectingClient({ asked, },),
              roster: ROSTER,
              judgeModelIds: ROSTER,
              subject: {
                sourceText: SOURCE,
                incumbentText: FLAGGED_STANDING,
                repairText: FLAGGED_STANDING,
                translateText: PAST_TEXT,
                ballots: [],
                lineStructured: false,
                lifeEnded: true,
                subjectNames: [...SUBJECT_NAMES,],
              },
              standingText: FLAGGED_STANDING,
              lineStructured: false,
              sliceIndex: 7,
              standingMayShip: false,
              standingEligible: true,
              laneTexts: [{ lane: 'translate', text: PAST_TEXT, },],
              slateReachable: 1,
              translateWinnerBallots: 2,
              signal: AbortSignal.timeout(HANG_STOP_MS,),
              perCallTimeoutMs: HANG_STOP_MS,
              l: tagged({ tag: 'consolidate-tense-swap-test', },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect({
              calls: asked.calls,
              ...readSettlement({ settled, },),
            },).toEqual({
              calls: 0,
              text: PAST_TEXT,
              archiveKept: undefined,
              translateWinnerShipped: true,
              translateWinnerReason: 'present-tense-standing',
              named: true,
            },);
          },
        },),
        it({
          name: 'KEEPS THE STANDING where the translate text is flagged too, since the swap would not mend the tense',
          fn: async () => {
            const { settled, } = await settleOver({
              standing: FLAGGED_STANDING,
              translate: FLAGGED_TRANSLATE,
              judgeCount: 1,
            },);
            expect(readSettlement({ settled, },),).toEqual(keptStanding({ standing: FLAGGED_STANDING, },),);
          },
        },),
        it({
          name: 'KEEPS THE STANDING where the standing is not flagged, however the translate text reads',
          fn: async () => {
            const { settled, } = await settleOver({
              standing: PAST_TEXT,
              translate: FLAGGED_TRANSLATE,
              judgeCount: 1,
            },);
            expect(readSettlement({ settled, },),).toEqual(keptStanding({ standing: PAST_TEXT, },),);
            const pastToPast = await settleOver({
              standing: PAST_TEXT,
              translate: 'Mittens had been a tabby with a white chin.',
              judgeCount: 1,
            },);
            expect(readSettlement({ settled: pastToPast.settled, },),).toEqual(keptStanding({ standing: PAST_TEXT, },),);
          },
        },),
        it({
          name: 'LEAVES THE VOTE TO THE SLATE where two judges are seated, flagged standing or not',
          fn: async () => {
            const { settled, } = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 2,
            },);
            expect({
              ...readSettlement({ settled, },),
              terminal: settled.terminal,
            },).toEqual({
              ...keptStanding({ standing: FLAGGED_STANDING, },),
              terminal: 'slate-declined-standing',
            },);
            const reachable = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 2,
              slateReachable: 2,
            },);
            expect(readSettlement({ settled: reachable.settled, },),).toEqual(
              keptStanding({ standing: FLAGGED_STANDING, },),
            );
          },
        },),
        it({
          name: 'DOES NOT FIRE ON QUOTED WORDS: present tense inside quoted speech is not flagged, and the same '
            + 'words unquoted are (the control that the swap reads the detector)',
          fn: async () => {
            const quoted = await settleOver({
              standing: QUOTED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 1,
            },);
            expect(readSettlement({ settled: quoted.settled, },),).toEqual(keptStanding({ standing: QUOTED_STANDING, },),);
            const unquoted = await settleOver({
              standing: UNQUOTED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 1,
            },);
            expect(readSettlement({ settled: unquoted.settled, },),).toMatchObject({
              text: PAST_TEXT,
              translateWinnerShipped: true,
            },);
          },
        },),
        it({
          name: 'KEEPS THE STANDING where the translate text drew one ballot, where the entry records no death, and '
            + 'where the contest endorsed the standing',
          fn: async () => {
            const oneBallot = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 1,
              ballots: 1,
            },);
            const noDeath = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 1,
              lifeEnded: false,
            },);
            const endorsed = await settleOver({
              standing: FLAGGED_STANDING,
              translate: PAST_TEXT,
              judgeCount: 1,
              standingMayShip: true,
            },);
            expect({
              oneBallot: readSettlement({ settled: oneBallot.settled, },),
              noDeath: readSettlement({ settled: noDeath.settled, },),
              endorsed: readSettlement({ settled: endorsed.settled, },),
            },).toEqual({
              oneBallot: keptStanding({ standing: FLAGGED_STANDING, },),
              noDeath: keptStanding({ standing: FLAGGED_STANDING, },),
              endorsed: keptStanding({ standing: FLAGGED_STANDING, },),
            },);
          },
        },),
      ],
    },),
  ],
},);
