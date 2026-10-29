/**
 Tests for the run-off of a carry choice that no candidate won by a strict majority (the lead's adoption of 2026-10-15 of the
 owner's rule of 2026-10-11 for this ask, open to the owner's veto): the bench is asked once more among the finalists (the
 candidates that tied for first, or a lone leader with the candidates next behind it), with the same quorum, window and label
 rules as the run-off among places, and a second split keeps the page as it stands.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { carryOffersOf, } from '@monochromatic-dev/translation-repair-consistency';
import {
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_HYPER_VISION,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import {
  choose,
  type ConsistencyCacheValue,
} from '../../dist/final/node/index.mjs';
import { promptUniqueClient, } from '@monochromatic-dev/translation-repair-stage/ts';
import {
  choosing,
  type SeatBehaviour,
  type SeatScript,
} from '@monochromatic-dev/translation-repair-run/ts/corpus-run/consistency-page.test-fixture.ts';
import { candidateTextsOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/consistency-sheet-reading.test-fixture.ts';
import {
  CARRY_REPEAT,
  carryScene,
  RECEIVING_ENGLISH,
  SPLICED_PARAGRAPH,
} from './consistency-carry.test-fixture.ts';

/**
 A judge that chooses the candidate its place on the bench points at, so a bench of three or more judges splits over three candidates.

 @param judges - bench, in roster order
 @param seat - judge asked
 @param sheet - sheet the judge was shown

 @returns Behaviour of a judge that picks by its place on the bench
 */
function splitAmong(
  {
    judges,
    seat,
    sheet,
  }: {
    readonly judges: readonly string[];
    readonly seat: string;
    readonly sheet: string;
  },
): SeatBehaviour {
  /**
   The candidates the sheet shows, in the order of the sheet.
   */
  const candidates = candidateTextsOf({ sheet, },);
  return choosing({ choice: candidates[judges.indexOf(seat,) % candidates.length]?.label ?? '', },);
}

/**
 A bench of six judges, whose quorum of three asks four seats, so that four choices can split two, one and one.
 */
const SIX_SEATS = [
  SEAT_HYPER_ONLY,
  SEAT_BEDROCK_ONLY_TEXT,
  SEAT_HYPER_VISION,
  SEAT_HYPER_TEXT_BEDROCK,
  SEAT_OPENROUTER_ONLY,
  SEAT_OPENROUTER_ONLY_CHECKER,
];

/**
 The candidate each of four asks names in a split of two, one and one: the first twice, the second, the third.
 */
const SPLIT_POSITIONS = [0, 0, 1, 2,];

/**
 What a writer wrote.
 */
const WRITTEN = 'Later the cat dozes in the sun.';

/**
 The three candidates: the current paragraph, the splice and one writer's paragraph.
 */
const OFFERS = carryOffersOf({
  current: RECEIVING_ENGLISH,
  splice: [SPLICED_PARAGRAPH,],
  written: [{ seat: 'hyper-cat', text: WRITTEN, },],
  lineStructured: false,
},);

/**
 Labels of the three candidates, in the order the sheet shows them.
 */
const LABELS = OFFERS.map(function labelOf({ label, },): string {
  return label;
},);

/**
 Label of the writer's paragraph.
 */
const WRITER_LABEL = OFFERS.find(function isWriter({ kind, },): boolean {
  return kind === 'writer';
},)
  ?.label ?? '';

/**
 The bench of the scene, in roster order.
 */
const BENCH_OF_THREE: readonly string[] = [SEAT_HYPER_ONLY, SEAT_BEDROCK_ONLY_TEXT, SEAT_HYPER_VISION,];

/**
 A bench that splits three ways in the first ask, one candidate for each seat, and agrees on the writer's paragraph after.

 @param before - how many calls came before this one
 @param seat - judge asked
 @param sheet - sheet the judge was shown

 @returns What the judge does

 @example
 ```ts
 const scene = carryScene({ behave: splitsThreeWaysThenAgrees, },);
 ```
 */
function splitsThreeWaysThenAgrees({ before, seat, sheet, }: Parameters<SeatScript>[0],): SeatBehaviour {
  const candidates = candidateTextsOf({ sheet, },);
  return (before < BENCH_OF_THREE.length)
    ? choosing({ choice: candidates[BENCH_OF_THREE.indexOf(seat,) % candidates.length]?.label ?? '', },)
    : choosing({ choice: WRITER_LABEL, },);
}

/**
 A bench whose third seat is lost, whose other two name the first and the third candidate in the first ask, and which names the
 last finalist after.

 @param seat - judge asked
 @param sheet - sheet the judge was shown

 @returns What the judge does

 @example
 ```ts
 const scene = carryScene({ behave: twoLeadersThenLastFinalist, },);
 ```
 */
function twoLeadersThenLastFinalist({ seat, sheet, }: Parameters<SeatScript>[0],): SeatBehaviour {
  if (seat === BENCH_OF_THREE[2])
    return { kind: 'throws', };
  const candidates = candidateTextsOf({ sheet, },);
  if (candidates.length === OFFERS.length)
    return choosing({ choice: (seat === BENCH_OF_THREE[0]) ? LABELS[0] ?? '' : LABELS[2] ?? '', },);
  return choosing({ choice: candidates.at(-1,)?.label ?? '', },);
}

/**
 What the carry choice comes to when it reads back the records an earlier run stored, each changed by a hand: the bench is
 asked once to store the two records (the first ask and the run-off), and the carry then reads them back instead of asking.

 @param behave - what the bench does when the records are first stored
 @param change - record to read back in place of the stored one, by the order the records were stored in
 @param judgeModelIds - bench, scene's by default

 @returns What the carry came to, the lines it logged, the keys the records were stored under and how often the bench was asked
 when they were read back

 @example
 ```ts
 const readBack = await readBackWith({ behave, change: ({ stored, }) => stored, },);
 ```
 */
async function readBackWith(
  {
    behave,
    change,
  }: {
    readonly behave: Parameters<typeof carryScene>[0]['behave'];
    readonly change: (
      parameters: { readonly stored: ConsistencyCacheValue; readonly position: number; },
    ) => ConsistencyCacheValue;
  },
): Promise<{
  readonly carry: Awaited<ReturnType<typeof choose>>;
  readonly lines: readonly string[];
  readonly keys: readonly string[];
  readonly askedAgain: number;
}> {
  /**
   The scene of the first run, whose bench stores the two records.
   */
  const first = carryScene({ behave, },);
  await choose({
    context: first.context,
    repeat: CARRY_REPEAT,
    chosen: first.chosen,
    unit: first.unit,
    offers: OFFERS,
    findings: [],
  },);
  /**
   The records read back, each as changed.
   */
  const resumed = new Map<string, ConsistencyCacheValue>(
    first.persisted.map(function readBack({ key, serialized, }, position,): readonly [string, ConsistencyCacheValue] {
      return [key, change({ stored: JSON.parse(serialized,) as ConsistencyCacheValue, position, },),];
    },),
  );
  /**
   The scene of the second run, whose bench is never to be asked.
   */
  const second = carryScene({
    behave: function neverAsked(): SeatBehaviour {
      return { kind: 'throws', };
    },
    resumed,
  },);
  /**
   What the carry came to.
   */
  const carry = await choose({
    context: second.context,
    repeat: CARRY_REPEAT,
    chosen: second.chosen,
    unit: second.unit,
    offers: OFFERS,
    findings: [],
  },);
  return {
    carry,
    lines: second.lines,
    keys: first.persisted.map(function keyOf({ key, },): string {
      return key;
    },),
    askedAgain: second.asked.length,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: choose.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RUNS OFF A THREE-WAY SPLIT among the three candidates, the bench asked once more, and carries the candidate the '
            + 'run-off chose, recording the run-off beside the first count',
          fn: async () => {
            const scene = carryScene({
              behave: function splitsThenAgrees({ before, seat, sheet, },): SeatBehaviour {
                const candidates = candidateTextsOf({ sheet, },);
                if (before < scene.context.judgeModelIds.length) {
                  return choosing({
                    choice: candidates[scene.context.judgeModelIds.indexOf(seat,) % candidates.length]?.label ?? '',
                  },);
                }
                return choosing({ choice: WRITER_LABEL, },);
              },
            },);
            const { carries, edits, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              first: [carries[0]?.usable, carries[0]?.quorum,],
              runOff: [carries[0]?.runOff?.finalists, carries[0]?.runOff?.usable, carries[0]?.runOff?.quorum,],
              runOffChoices: carries[0]?.runOff?.ballots.map(function choiceOf({ choice, },): string {
                return choice;
              },),
              edited: edits.length > 0,
            },).toEqual({
              decision: { kind: 'chosen', label: WRITER_LABEL, },
              first: [3, 2,],
              runOff: [LABELS, 3, 2,],
              runOffChoices: [WRITER_LABEL, WRITER_LABEL, WRITER_LABEL,],
              edited: true,
            },);
            expect(scene.asked.length,).toBe(6,);
          },
        },),
        it({
          name: 'SENDS THE RUN-OFF A SHEET OF ITS OWN WHEN EVERY CANDIDATE IS A FINALIST: a three-way split makes all three '
            + 'candidates finalists, and the run-off messages must differ from the first ask\'s so that a bench that replays '
            + 'an identical sheet is asked a new question',
          fn: async () => {
            const scene = carryScene({
              behave: function splitsByRosterPlace({ seat, sheet, },): SeatBehaviour {
                return splitAmong({ judges: scene.context.judgeModelIds, seat, sheet, },);
              },
            },);
            await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            const [firstSheet,] = scene.sheets;
            const [runOffSheet,] = scene.sheets.slice(3,);
            expect({
              asked: scene.sheets.length,
              firstAsksAreAlike: new Set(scene.sheets.slice(0, 3,),).size,
              runOffAsksAreAlike: new Set(scene.sheets.slice(3,),).size,
              runOffDiffersFromFirst: runOffSheet !== firstSheet,
            },).toEqual({
              asked: 6,
              firstAsksAreAlike: 1,
              runOffAsksAreAlike: 1,
              runOffDiffersFromFirst: true,
            },);
          },
        },),
        it({
          name: 'BREAKS THE TIE OF A THREE-WAY SPLIT WHEN THE BENCH ANSWERS A NEW SHEET ANEW: seats that replay their ballot on an '
            + 'identical sheet and choose the writer\'s paragraph on any other carry it, so the run-off decides',
          fn: async () => {
            /**
             The ballot each seat gave to each sheet it has been shown.
             */
            const given = new Map<string, SeatBehaviour>();
            const scene = carryScene({
              behave: function replaysSameSheet({ seat, sheet, },): SeatBehaviour {
                const key = `${seat}\n${sheet}`;
                const earlier = given.get(key,);
                if (earlier !== undefined)
                  return earlier;
                const candidates = candidateTextsOf({ sheet, },);
                const ballot = (given.size < scene.context.judgeModelIds.length)
                  ? choosing({
                    choice: candidates[scene.context.judgeModelIds.indexOf(seat,) % candidates.length]?.label ?? '',
                  },)
                  : choosing({ choice: WRITER_LABEL, },);
                given.set(
                  key,
                  ballot,
                );
                return ballot;
              },
            },);
            const { carries, edits, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              runOffChoices: carries[0]?.runOff?.ballots.map(function choiceOf({ choice, },): string {
                return choice;
              },),
              edited: edits.length > 0,
            },).toEqual({
              decision: { kind: 'chosen', label: WRITER_LABEL, },
              runOffChoices: [WRITER_LABEL, WRITER_LABEL, WRITER_LABEL,],
              edited: true,
            },);
          },
        },),
        it({
          name: 'DOES NOT ANSWER THE RUN-OFF FROM THE FIRST ROUND\'S PAYLOADS: behind the client that replays a repeated model and '
            + 'prompt, all three finalists of a three-way split are asked again and the provider is called for every seat twice',
          fn: async () => {
            const scene = carryScene({
              behave: function splitsThenAgrees({ before, seat, sheet, },): SeatBehaviour {
                const candidates = candidateTextsOf({ sheet, },);
                return (before < scene.context.judgeModelIds.length)
                  ? choosing({
                    choice: candidates[scene.context.judgeModelIds.indexOf(seat,) % candidates.length]?.label ?? '',
                  },)
                  : choosing({ choice: WRITER_LABEL, },);
              },
            },);
            const { client: scripted, } = scene.context;
            const replaying = promptUniqueClient({
              inner: {
                ...scripted,
                chatText: async function textOf(request,) {
                  const outcome = await scripted.chatJson({
                    ...request,
                    validate: function acceptsAny(_value,): _value is unknown {
                      return true;
                    },
                  },);
                  return { text: ('rawText' in outcome) ? outcome.rawText : '', };
                },
              },
            },);
            const { carries, } = await choose({
              context: { ...scene.context, client: replaying, },
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({ decision: carries[0]?.decision, providerCalls: scene.asked.length, },).toEqual({
              decision: { kind: 'chosen', label: WRITER_LABEL, },
              providerCalls: 6,
            },);
          },
        },),
        it({
          name: 'RUNS OFF A TIE OF TWO LEADERS WITH A SHEET OF THOSE TWO, keeping their original labels, and counts the run-off with its '
            + 'own quorum when one seat is lost both times',
          fn: async () => {
            const scene = carryScene({
              behave: function twoLeadersThenAgree({ seat, sheet, },): SeatBehaviour {
                if (seat === scene.context.judgeModelIds[2])
                  return { kind: 'throws', };
                const candidates = candidateTextsOf({ sheet, },);
                if (candidates.length === OFFERS.length) {
                  // The first ask: the two seats that answer name different candidates, the first and the third.
                  return choosing({ choice: (seat === scene.context.judgeModelIds[0]) ? LABELS[0] ?? '' : LABELS[2] ?? '', },);
                }
                return choosing({ choice: candidates.at(-1,)?.label ?? '', },);
              },
            },);
            const { carries, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              finalists: carries[0]?.runOff?.finalists,
              usable: carries[0]?.runOff?.usable,
            },).toEqual({
              decision: { kind: 'chosen', label: LABELS[2], },
              finalists: [LABELS[0], LABELS[2],],
              usable: 2,
            },);
            const lastSheet = scene.sheets.at(-1,) ?? '';
            expect(candidateTextsOf({ sheet: lastSheet, },).map(function labelOf({ label, },): string {
              return label;
            },),).toEqual([LABELS[0], LABELS[2],],);
          },
        },),
        it({
          name: 'RUNS OFF A LONE LEADER WITH THE CANDIDATES NEXT BEHIND IT: four choices splitting two, one and one leave no strict '
            + 'majority, so the bench is asked once more among the three candidates and carries the one the run-off chose',
          fn: async () => {
            const scene = carryScene({
              behave: function leaderThenAgree({ before, sheet, },): SeatBehaviour {
                const candidates = candidateTextsOf({ sheet, },);
                if (before < SPLIT_POSITIONS.length) {
                  // The first ask: the first two answers name the first candidate, the others the second and the third.
                  return choosing({ choice: LABELS[SPLIT_POSITIONS[before] ?? 0] ?? '', },);
                }
                return choosing({ choice: candidates.at(-1,)?.label ?? '', },);
              },
            },);
            const { carries, edits, } = await choose({
              context: { ...scene.context, judgeModelIds: SIX_SEATS, },
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              // SORTED, since the scene assigns a choice by the order the asks arrive, which the bench's rotation by the sheet text
              // sets, while the record lists the ballots in roster order; the split under test is the same four choices either way.
              first: carries[0]?.ballots.map(function choiceOf({ choice, },): string {
                return choice;
              },).toSorted(),
              runOffFinalists: carries[0]?.runOff?.finalists,
              edited: edits.length > 0,
            },).toEqual({
              decision: { kind: 'chosen', label: LABELS[2], },
              first: [LABELS[0], LABELS[0], LABELS[1], LABELS[2],],
              runOffFinalists: LABELS,
              edited: true,
            },);
          },
        },),
        it({
          name: 'KEEPS THE PAGE AS IT STANDS ON A SECOND SPLIT OF A LONE LEADER\'S RUN-OFF, recording the run-off and its decision',
          fn: async () => {
            const scene = carryScene({
              behave: function leaderThenSplitAgain({ before, sheet, },): SeatBehaviour {
                const candidates = candidateTextsOf({ sheet, },);
                return choosing({
                  choice: candidates[SPLIT_POSITIONS[before % SPLIT_POSITIONS.length] ?? 0]?.label ?? '',
                },);
              },
            },);
            const { carries, edits, } = await choose({
              context: { ...scene.context, judgeModelIds: SIX_SEATS, },
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              runOffFinalists: carries[0]?.runOff?.finalists,
              edits,
            },).toEqual({
              decision: { kind: 'none', reason: 'no-majority', },
              runOffFinalists: LABELS,
              edits: [],
            },);
          },
        },),
        it({
          name: 'KEEPS THE PAGE AS IT STANDS ON A SECOND SPLIT, recording the run-off and its decision',
          fn: async () => {
            const scene = carryScene({
              behave: function alwaysSplits({ seat, sheet, },): SeatBehaviour {
                return splitAmong({ judges: scene.context.judgeModelIds, seat, sheet, },);
              },
            },);
            const { carries, edits, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              runOffFinalists: carries[0]?.runOff?.finalists,
              edits,
            },).toEqual({
              decision: { kind: 'none', reason: 'no-majority', },
              runOffFinalists: LABELS,
              edits: [],
            },);
          },
        },),
        it({
          name: 'ASKS NO SECOND TIME WHEN A STRICT MAJORITY CHOSE, and records no run-off',
          fn: async () => {
            const scene = carryScene({ behave: function allChoose(): SeatBehaviour {
              return choosing({ choice: WRITER_LABEL, },);
            }, },);
            const { carries, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({ decision: carries[0]?.decision, runOff: carries[0]?.runOff, asked: scene.asked.length, },).toEqual({
              decision: { kind: 'chosen', label: WRITER_LABEL, },
              runOff: undefined,
              asked: 3,
            },);
          },
        },),
        it({
          name: 'COUNTS A CHOICE WRITTEN IN ANOTHER FORM OF A CANDIDATE LABEL, so the three seats of the last run that wrote '
            + '"2" and "Candidate 2" carry a candidate instead of losing their voices',
          fn: async () => {
            const scene = carryScene({
              behave: function writesLabelsInOtherForms({ seat, }): SeatBehaviour {
                const forms = [WRITER_LABEL.toLowerCase(), WRITER_LABEL.replace('CANDIDATE ', '',), ` ${WRITER_LABEL} `,];
                return choosing({ choice: forms[scene.context.judgeModelIds.indexOf(seat,)] ?? '', },);
              },
            },);
            const { carries, } = await choose({
              context: scene.context,
              repeat: CARRY_REPEAT,
              chosen: scene.chosen,
              unit: scene.unit,
              offers: OFFERS,
              findings: [],
            },);
            expect({
              decision: carries[0]?.decision,
              ballots: carries[0]?.ballots.map(function choiceOf({ choice, },): string {
                return choice;
              },),
            },).toEqual({
              decision: { kind: 'chosen', label: WRITER_LABEL, },
              ballots: [WRITER_LABEL, WRITER_LABEL, WRITER_LABEL,],
            },);
          },
        },),
        it({
          name: 'STORES THE RUN-OFF UNDER A KEY AND A STAGE OF ITS OWN, so a stored run-off is read back apart from the first ask, '
            + 'and LOGS THE SECOND ASK AND HOW THE RUN-OFF ENDED with the counts and the quorum it was counted with',
          fn: async () => {
            const readBack = await readBackWith({
              behave: splitsThreeWaysThenAgrees,
              change: function unchanged({ stored, },): ConsistencyCacheValue {
                return stored;
              },
            },);
            expect({
              keys: readBack.keys.length,
              distinct: new Set(readBack.keys,).size,
              askedAgain: readBack.askedAgain,
              decision: readBack.carry.carries[0]?.decision,
              lines: readBack.lines,
            },).toEqual({
              keys: 2,
              distinct: 2,
              askedAgain: 0,
              decision: { kind: 'chosen', label: WRITER_LABEL, },
              lines: [
                'info [choose] [gatherThroughCache] page-consistency-choice: resumed 3 voices',
                'info [choose] the choice among the candidates had no strict majority (3 usable); asking the bench once more among the '
                  + `finalists ${LABELS.join(' and ',)}`,
                'info [choose] [gatherThroughCache] page-consistency-choice-run-off: resumed 3 voices',
                `info [choose] the run-off among the candidates counted 3 usable of 3 heard (quorum 2): ${WRITER_LABEL} won`,
              ],
            },);
          },
        },),
        it({
          name: 'COUNTS THE RUN-OFF WITH ITS OWN QUORUM AND RECORDS IT, a run-off whose quorum is three counting two usable choices '
            + 'as no quorum where the first ask closed on two, and keeps the findings the run-off gathered',
          fn: async () => {
            const readBack = await readBackWith({
              behave: splitsThreeWaysThenAgrees,
              change: function runOffOfThree({ stored, position, },): ConsistencyCacheValue {
                return (position === 1)
                  ? {
                    kind: 'choice',
                    quorum: 3,
                    voices: [
                      { seat: SEAT_HYPER_ONLY, reason: 'The writer.', choice: WRITER_LABEL, },
                      { seat: SEAT_BEDROCK_ONLY_TEXT, reason: 'The writer.', choice: WRITER_LABEL, },
                    ],
                    findings: ['a seat of the run-off was lost'],
                  }
                  : stored;
              },
            },);
            const [carry,] = readBack.carry.carries;
            expect({
              decision: carry?.decision,
              first: [carry?.usable, carry?.quorum,],
              runOff: [carry?.runOff?.finalists, carry?.runOff?.usable, carry?.runOff?.quorum,],
              findings: readBack.carry.findings,
              edits: readBack.carry.edits,
              lines: readBack.lines,
            },).toEqual({
              decision: { kind: 'none', reason: 'no-quorum', },
              first: [3, 2,],
              runOff: [LABELS, 2, 3,],
              findings: ['a seat of the run-off was lost'],
              edits: [],
              lines: [
                'info [choose] [gatherThroughCache] page-consistency-choice: resumed 3 voices',
                'info [choose] the choice among the candidates had no strict majority (3 usable); asking the bench once more among the '
                  + `finalists ${LABELS.join(' and ',)}`,
                'info [choose] [gatherThroughCache] page-consistency-choice-run-off: resumed 2 voices',
                'info [choose] the run-off among the candidates counted 2 usable of 2 heard (quorum 3): no winner (no-quorum)',
              ],
            },);
          },
        },),
        it({
          name: 'COUNTS THE RUN-OFF AGAINST THE FINALISTS ONLY, so choices that name a candidate which was not a finalist are not '
            + 'counted and cannot win it, even when they are the most of the run-off\'s record',
          fn: async () => {
            const readBack = await readBackWith({
              behave: twoLeadersThenLastFinalist,
              change: function strayRunOff({ stored, position, },): ConsistencyCacheValue {
                return (position === 1)
                  ? {
                    kind: 'choice',
                    quorum: 2,
                    voices: [
                      { seat: SEAT_HYPER_ONLY, reason: 'The second.', choice: LABELS[1] ?? '', },
                      { seat: SEAT_BEDROCK_ONLY_TEXT, reason: 'The second.', choice: LABELS[1] ?? '', },
                      { seat: SEAT_HYPER_VISION, reason: 'The first.', choice: LABELS[0] ?? '', },
                    ],
                    findings: [],
                  }
                  : stored;
              },
            },);
            const [carry,] = readBack.carry.carries;
            expect({
              decision: carry?.decision,
              runOff: [carry?.runOff?.finalists, carry?.runOff?.usable, carry?.runOff?.quorum,],
              edits: readBack.carry.edits,
            },).toEqual({
              decision: { kind: 'none', reason: 'no-quorum', },
              runOff: [[LABELS[0], LABELS[2],], 1, 2,],
              edits: [],
            },);
          },
        },),
        it({
          name: 'RUNS OFF NOTHING WHEN THE FIRST CHOICES MISSED THEIR QUORUM, though the two they heard name different candidates: '
            + 'the bench is not asked again and no run-off is recorded',
          fn: async () => {
            const readBack = await readBackWith({
              behave: splitsThreeWaysThenAgrees,
              change: function missedQuorum({ stored, position, },): ConsistencyCacheValue {
                return (position === 0)
                  ? {
                    kind: 'choice',
                    quorum: 3,
                    voices: [
                      { seat: SEAT_HYPER_ONLY, reason: 'The first.', choice: LABELS[0] ?? '', },
                      { seat: SEAT_BEDROCK_ONLY_TEXT, reason: 'The second.', choice: LABELS[1] ?? '', },
                    ],
                    findings: [],
                  }
                  : stored;
              },
            },);
            const [carry,] = readBack.carry.carries;
            expect({
              decision: carry?.decision,
              first: [carry?.usable, carry?.quorum,],
              runOff: carry?.runOff,
              askedAgain: readBack.askedAgain,
              edits: readBack.carry.edits,
            },).toEqual({
              decision: { kind: 'none', reason: 'no-quorum', },
              first: [2, 3,],
              runOff: undefined,
              askedAgain: 0,
              edits: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
