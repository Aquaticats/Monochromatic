/**
 Tests for the repeat agreement the read step prints for each arm and the warning it gives when the arms differ: the share
 of the trials an arm was asked in more than one pass whose passes all read the same way (a clean fix in every pass or in
 none), what counts as a repeat, and the exact test of the default effort's count against another arm's, at a level of 0.05
 divided by the number of other arms, above which the paired test's bound on a false pick is not assured. The warning
 prints the counts it tests, so a small number of trials asked again shows as one. The lines are read off the arm logs,
 and the built command prints them even for one log, since the runbook reads the high arm's agreement before the other
 arms exist.

 The logs are the arm-log fixture's, written by the probe's own writers. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArmLog,
  repeatLinesOf,
} from '../../dist/final/node/index.mjs';
import {
  inPass,
  POOL_TRIALS,
  poolRows,
  type RowSpec,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';
import { readAsBuilt as readAsBuiltIn, } from './editor-fidelity-arm-read-built.test-fixture.ts';
import {
  armOf,
  walkOf,
} from './editor-fidelity-arm-walk.test-fixture.ts';

/**
 What a warning ends with: the bound, when it holds, and what the simulation reached when the default effort repeats
 itself less than another arm.
 */
const WARNING_END = 'the bound of 0.05 on choosing another effort when none is better holds when the arms repeat themselves '
  + 'alike, and the simulation reached 0.107 at a true fix rate of 0.9 when the default effort repeats itself far less '
  + 'than the others, so a verdict of high or low is less sure than the 0.025 level says';

/**
 The rows of an arm that asks some trials in two passes: the first pass fixes every trial, and the second fixes all but
 the leading ones, so that the passes of those trials differ.

 @param trials - how many leading trials of the invented pool, two to an entry
 @param differ - how many leading trials the second pass does not fix
 @param secondStops - stop words of the trial at an index in the second pass, `ok` when omitted
 @param fixedAlike - whether the first pass fixes the trials, a trial not fixed in either pass agreeing with itself

 @returns Rows, first pass then the second

 @example
 ```ts
 const rows = twoPassRows({ trials: 44, differ: 4, },);
 ```
 */
function twoPassRows(
  {
    trials,
    differ,
    secondStops,
    fixedAlike = true,
  }: {
    readonly trials: number;
    readonly differ: number;
    readonly secondStops?: (index: number,) => string;
    readonly fixedAlike?: boolean;
  },
): readonly RowSpec[] {
  return [
    ...poolRows({
      count: trials,
      fixed: function firstPass(): boolean {
        return fixedAlike;
      },
    },),
    ...inPass({
      rows: poolRows({
        count: trials,
        fixed: function secondPass(index,): boolean {
          return fixedAlike && (index >= differ);
        },
        ...((secondStops === undefined) ? {} : { stops: secondStops, }),
      },),
      pass: 2,
    },),
  ];
}

/**
 A bench of three arms over trials asked twice, each with its own number of trials whose passes disagree.

 @param trials - how many trials
 @param high - trials whose passes disagree in the high arm
 @param medium - same in the medium arm
 @param low - same in the low arm

 @returns Three logs, high first

 @example
 ```ts
 const arms = benchOf({ trials: 44, high: 0, medium: 4, low: 0, },);
 ```
 */
function benchOf(
  {
    trials,
    high,
    medium,
    low,
  }: {
    readonly trials: number;
    readonly high: number;
    readonly medium: number;
    readonly low: number;
  },
): readonly ArmLog[] {
  return [
    armOf({
      arm: 'high',
      cardId: HIGH_CARD,
      rows: twoPassRows({ trials, differ: high, },),
      countedUsd: 2.5,
    },),
    armOf({
      arm: 'medium',
      cardId: MEDIUM_CARD,
      rows: twoPassRows({ trials, differ: medium, },),
      declaredUsd: 2.5,
      countedUsd: 1.5,
    },),
    armOf({
      arm: 'low',
      cardId: LOW_CARD,
      rows: twoPassRows({ trials, differ: low, },),
      declaredUsd: 4,
      countedUsd: 0.875,
    },),
  ];
}

/**
 The lines of the repeat agreement that start with a word.

 @param arms - arms
 @param word - what the lines start with

 @returns Lines

 @example
 ```ts
 const warnings = linesStarting({ arms, word: 'WARN', },);
 ```
 */
function linesStarting({ arms, word, }: { readonly arms: readonly ArmLog[]; readonly word: string; },): readonly string[] {
  return repeatLinesOf({ arms, },).filter(function starts(line,): boolean {
    return line.startsWith(`${word} `,);
  },);
}

/**
 Runs the built read step over the arms' logs.

 @param arms - logs, in the order the arms ran

 @returns Exit status and standard output

 @example
 ```ts
 const run = await readAsBuilt({ arms, },);
 ```
 */
function readAsBuilt({ arms, }: { readonly arms: readonly ArmLog[]; },): ReturnType<typeof readAsBuiltIn> {
  return readAsBuiltIn({
    arms,
    prefix: 'editor-fidelity-repeats-',
  },);
}

await describe({
  name: 'the repeat agreement of the read step',
  children: [
    describe({
      name: repeatLinesOf.name,
      children: [
        it({
          name: 'PRINTS EACH ARM\'S REPEAT AGREEMENT, the trials asked twice whose passes read alike over all the trials asked '
            + 'twice, and says nothing of 40 of 44 against 44 of 44, whose p-value of 0.0582 is over the level',
          fn: async () => {
            const arms = benchOf({ trials: 44, high: 0, medium: 4, low: 6, },);
            expect(linesStarting({ arms, word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: 44 of 44 trials gave the same reading in every pass (100 percent)',
              'REPEAT medium: 40 of 44 trials gave the same reading in every pass (91 percent)',
              'REPEAT low: 38 of 44 trials gave the same reading in every pass (86 percent)',
            ],);
            expect(linesStarting({ arms, word: 'WARN', },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'WARNS WHEN THE EXACT TEST OF THE DEFAULT EFFORT\'S COUNT AGAINST ANOTHER ARM\'S IS AT THE LEVEL, naming the '
            + 'default effort\'s counts, the arm it is under and its counts, the p-value, the level, and the bound the '
            + 'paired test holds only for arms that repeat themselves alike',
          fn: async () => {
            expect(linesStarting({ arms: benchOf({ trials: 44, high: 0, medium: 11, low: 0, },), word: 'WARN', },),)
              .toStrictEqual([
                `WARN repeat agreement: medium gave the same reading in every pass on 33 of 44 trials it was asked again (75 percent), against 44 of 44 (100 percent) for high; an exact one-sided test of the two counts gives p 0.0002, at or under 0.05 divided by 2, the number of other arms; ${WARNING_END}`,
              ],);
          },
        },),

        it({
          name: 'WARNS AT THE LEVEL AND NOT OVER IT, counting exactly: 15 of 20 trials against 20 of 20 (p 0.0236) is at 0.025 '
            + 'and 16 of 20 (p 0.0530) is over it, and 0 of 3 against 3 of 3 (p exactly one in twenty) is at the level of 0.05 '
            + 'when one other arm is read and over 0.05 divided by two when two are',
          fn: async () => {
            expect(linesStarting({ arms: benchOf({ trials: 20, high: 0, medium: 5, low: 0, },), word: 'WARN', },)[0],)
              .toContain('medium gave the same reading in every pass on 15 of 20 trials it was asked again (75 percent), against 20 of 20 (100 percent) for high; an exact one-sided test of the two counts gives p 0.0236, at or under 0.05 divided by 2, the number of other arms; ',);
            expect(linesStarting({ arms: benchOf({ trials: 20, high: 0, medium: 4, low: 0, },), word: 'WARN', },),)
              .toStrictEqual([],);
            /**
             Three trials asked twice, the medium arm disagreeing with itself on all of them.
             */
            const [high, medium, low,] = benchOf({ trials: 3, high: 0, medium: 3, low: 0, },);
            expect(linesStarting({ arms: [high as ArmLog, medium as ArmLog,], word: 'WARN', },)[0],)
              .toContain('medium gave the same reading in every pass on 0 of 3 trials it was asked again (0 percent), against 3 of 3 (100 percent) for high; an exact one-sided test of the two counts gives p 0.0500, at or under 0.05; ',);
            expect(linesStarting({ arms: [high as ArmLog, medium as ArmLog, low as ArmLog,], word: 'WARN', },),)
              .toStrictEqual([],);
          },
        },),

        it({
          name: 'NEEDS A LARGER DIFFERENCE WHEN FEW TRIALS WERE ASKED AGAIN, and has no floor on them: 12 trials asked twice, as a '
            + 'bench of 100 shared units holds, do not warn at 9 of 12 against 12 of 12 (p 0.1087) and warn at 7 of 12 (p 0.0186)',
          fn: async () => {
            expect(linesStarting({ arms: benchOf({ trials: 12, high: 0, medium: 3, low: 0, },), word: 'WARN', },),)
              .toStrictEqual([],);
            expect(linesStarting({ arms: benchOf({ trials: 12, high: 0, medium: 3, low: 0, },), word: 'REPEAT', },)[1],)
              .toBe('REPEAT medium: 9 of 12 trials gave the same reading in every pass (75 percent)',);
            expect(linesStarting({ arms: benchOf({ trials: 12, high: 0, medium: 5, low: 0, },), word: 'WARN', },)[0],)
              .toContain('on 7 of 12 trials it was asked again (58 percent), against 12 of 12 (100 percent) for high; an exact one-sided test of the two counts gives p 0.0186,',);
          },
        },),

        it({
          name: 'DOES NOT WARN WHEN THE DEFAULT EFFORT REPEATS ITSELF MORE THAN THE OTHERS (agreements 0, 0 and 1 for high, low '
            + 'and medium), the direction the simulation found no excess false pick in, and prints the figures all the same',
          fn: async () => {
            /**
             High and low disagree with themselves on every trial, medium on none.
             */
            const arms = benchOf({ trials: 44, high: 44, medium: 0, low: 44, },);
            expect(linesStarting({ arms, word: 'WARN', },),).toStrictEqual([],);
            expect(linesStarting({ arms, word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: 0 of 44 trials gave the same reading in every pass (0 percent)',
              'REPEAT medium: 44 of 44 trials gave the same reading in every pass (100 percent)',
              'REPEAT low: 0 of 44 trials gave the same reading in every pass (0 percent)',
            ],);
          },
        },),

        it({
          name: 'WARNS WHEN THE DEFAULT EFFORT IS THE ONE THAT REPEATS ITSELF LESS (agreements 1, 1 and 0), and not when the low '
            + 'or the high effort is the odd one out',
          fn: async () => {
            /**
             High and low repeat themselves on every trial, medium on none.
             */
            const mediumOdd = linesStarting({ arms: benchOf({ trials: 44, high: 0, medium: 44, low: 0, },), word: 'WARN', },);
            expect(mediumOdd.length,).toBe(1,);
            expect(mediumOdd[0],).toContain('medium gave the same reading in every pass on 0 of 44 trials it was asked again (0 percent), against 44 of 44 (100 percent) for high; an exact one-sided test of the two counts gives p under 0.0001, at or under 0.05 divided by 2, the number of other arms; ',);
            expect(linesStarting({ arms: benchOf({ trials: 44, high: 0, medium: 0, low: 44, },), word: 'WARN', },),)
              .toStrictEqual([],);
            expect(linesStarting({ arms: benchOf({ trials: 44, high: 44, medium: 0, low: 0, },), word: 'WARN', },),)
              .toStrictEqual([],);
          },
        },),

        it({
          name: 'MEASURES EVERY ARM OVER THE UNITS EVERY ARM HOLDS when several logs are read: the high arm walked 264 units, '
            + 'the others were stopped at 150, and all three read alike over those 150, so nothing is warned of',
          fn: async () => {
            /**
             The high arm over three whole passes: the leading 62 trials read differently in the second pass than in the
             first and the third, so its figure over all its rows is 26 of 88; the other trials read alike in every pass.
             */
            const high = armOf({
              arm: 'high',
              cardId: HIGH_CARD,
              rows: walkOf({ units: 264, fixedAt: ({ pass, index, },) => (pass !== 2) || (index >= 62), },),
              countedUsd: 2.5,
            },);
            /**
             The later arms over the leading 150 units: the leading 62 trials read differently in the second pass than in
             the first, exactly as the high arm's do over those units.
             */
            const rows = walkOf({ units: 150, fixedAt: ({ pass, },) => pass === 1, },);
            const arms = [
              high,
              armOf({ arm: 'medium', cardId: MEDIUM_CARD, rows, declaredUsd: 2.5, countedUsd: 1.5, },),
              armOf({ arm: 'low', cardId: LOW_CARD, rows, declaredUsd: 4, countedUsd: 0.875, },),
            ];
            expect(linesStarting({ arms, word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: 0 of 62 trials gave the same reading in every pass (0 percent)',
              'REPEAT medium: 0 of 62 trials gave the same reading in every pass (0 percent)',
              'REPEAT low: 0 of 62 trials gave the same reading in every pass (0 percent)',
            ],);
            expect(linesStarting({ arms, word: 'WARN', },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'KEEPS EACH ARM\'S OWN ROWS WHEN THE LOGS CANNOT FORM A BENCH, two logs without a low effort among them, since no '
            + 'unit is shared by a high, a medium and a low arm, and the checks refuse such a bench',
          fn: async () => {
            const [high, medium,] = benchOf({ trials: 44, high: 0, medium: 11, low: 0, },);
            const arms = [high as ArmLog, medium as ArmLog,];
            expect(linesStarting({ arms, word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: 44 of 44 trials gave the same reading in every pass (100 percent)',
              'REPEAT medium: 33 of 44 trials gave the same reading in every pass (75 percent)',
            ],);
            expect(linesStarting({ arms, word: 'WARN', },)[0],).toContain('medium gave the same reading in every pass on 33 of 44 trials it was asked again (75 percent), against 44 of 44 (100 percent) for high; an exact one-sided test of the two counts gives p 0.0002, at or under 0.05; ',);
          },
        },),

        it({
          name: 'COMPARES ARMS THAT WERE ASKED AGAIN IN DIFFERENT NUMBERS OF TRIALS over the units they share: the high arm walked '
            + '40 trials twice and the others 20, so all three are read over the 20 trials in both passes',
          fn: async () => {
            /**
             The arm asked 40 trials twice, which repeats every one.
             */
            const whole = armOf({ arm: 'high', cardId: HIGH_CARD, rows: twoPassRows({ trials: 40, differ: 0, },), },);
            /**
             The arms asked 20 trials twice, 3 of which disagree.
             */
            const [, medium, low,] = benchOf({ trials: 20, high: 0, medium: 3, low: 3, },);
            const arms = [whole, medium as ArmLog, low as ArmLog,];
            expect(linesStarting({ arms, word: 'WARN', },),).toStrictEqual([],);
            expect(linesStarting({ arms, word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: 20 of 20 trials gave the same reading in every pass (100 percent)',
              'REPEAT medium: 17 of 20 trials gave the same reading in every pass (85 percent)',
              'REPEAT low: 17 of 20 trials gave the same reading in every pass (85 percent)',
            ],);
          },
        },),

        it({
          name: 'SAYS WHEN NO TRIAL WAS ASKED IN MORE THAN ONE PASS, as an arm that walks the pool once never does, and does not '
            + 'warn, nor when only one arm has trials asked again',
          fn: async () => {
            /**
             One pass of 44 trials for each arm.
             */
            const once = [
              armOf({ arm: 'high', cardId: HIGH_CARD, rows: twoPassRows({ trials: 44, differ: 0, },).slice(0, 44,), },),
              armOf({ arm: 'medium', cardId: MEDIUM_CARD, rows: twoPassRows({ trials: 44, differ: 0, },).slice(0, 44,), },),
              armOf({ arm: 'low', cardId: LOW_CARD, rows: twoPassRows({ trials: 44, differ: 0, },).slice(0, 44,), },),
            ];
            expect(linesStarting({ arms: once, word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: no trial was asked in more than one pass',
              'REPEAT medium: no trial was asked in more than one pass',
              'REPEAT low: no trial was asked in more than one pass',
            ],);
            expect(linesStarting({ arms: once, word: 'WARN', },),).toStrictEqual([],);
            /**
             The first arm asked twice with 11 trials disagreeing, the others once.
             */
            const [first,] = benchOf({ trials: 44, high: 11, medium: 0, low: 0, },);
            expect(linesStarting({ arms: [first as ArmLog, ...once.slice(1,),], word: 'WARN', },),).toStrictEqual([],);
          },
        },),

        it({
          name: 'LEAVES OUT A ROW IN WHICH AN ASK THREW, as the shared trials do: the second pass of the first trial threw, so '
            + 'that trial was asked once and 43 trials are counted, all of them alike',
          fn: async () => {
            /**
             The high arm whose second-pass row of the first trial threw, and would have disagreed.
             */
            const high = armOf({
              arm: 'high',
              cardId: HIGH_CARD,
              rows: twoPassRows({
                trials: 44,
                differ: 1,
                secondStops: function threwFirst(index,): string {
                  return (index === 0) ? 'threw+ok' : 'ok';
                },
              },),
            },);
            expect(linesStarting({ arms: [high, ...benchOf({ trials: 44, high: 0, medium: 0, low: 0, },).slice(1,),], word: 'REPEAT', },)[0],)
              .toBe('REPEAT high: 43 of 43 trials gave the same reading in every pass (100 percent)',);
          },
        },),

        it({
          name: 'READS A ROW THAT ENDED AT THE OUTPUT CAP AS NOT A FIX, as the paired test does: a trial fixed in the first pass '
            + 'and cut off at the cap in the second disagrees with itself',
          fn: async () => {
            /**
             The high arm whose second-pass row of the first trial reads fixed but ended at the output cap.
             */
            const high = armOf({
              arm: 'high',
              cardId: HIGH_CARD,
              rows: twoPassRows({
                trials: 44,
                differ: 0,
                secondStops: function cappedFirst(index,): string {
                  return (index === 0) ? 'output-cap' : 'ok';
                },
              },),
            },);
            expect(linesStarting({ arms: [high, ...benchOf({ trials: 44, high: 0, medium: 0, low: 0, },).slice(1,),], word: 'REPEAT', },)[0],)
              .toBe('REPEAT high: 43 of 44 trials gave the same reading in every pass (98 percent)',);
          },
        },),

        it({
          name: 'COUNTS A TRIAL NOT FIXED IN ANY PASS AS A REPEAT THAT AGREES, as it does a trial fixed in every pass',
          fn: async () => {
            /**
             The high arm that fixes nothing in either pass.
             */
            const none = armOf({
              arm: 'high',
              cardId: HIGH_CARD,
              rows: twoPassRows({ trials: 44, differ: 0, fixedAlike: false, },),
            },);
            expect(linesStarting({ arms: [none, ...benchOf({ trials: 44, high: 0, medium: 0, low: 0, },).slice(1,),], word: 'REPEAT', },)[0],)
              .toBe('REPEAT high: 44 of 44 trials gave the same reading in every pass (100 percent)',);
          },
        },),

        it({
          name: 'COUNTS ONE TRIAL ASKED AGAIN IN THE SINGULAR: one of one trial gave the same reading',
          fn: async () => {
            /**
             The high arm that asked one trial in two passes, both fixed.
             */
            const one = armOf({ arm: 'high', cardId: HIGH_CARD, rows: twoPassRows({ trials: 1, differ: 0, },), },);
            expect(linesStarting({ arms: [one,], word: 'REPEAT', },),).toStrictEqual([
              'REPEAT high: 1 of 1 trial gave the same reading in every pass (100 percent)',
            ],);
          },
        },),

        it({
          name: 'PRINTS A LINE FOR EACH OF FOUR ARMS WHEN A SECOND MEDIUM IS READ, and warns of the second medium arm alone, '
            + 'which repeats itself less than the others, naming the first of the arms it is under',
          fn: async () => {
            const [high, medium, low,] = benchOf({ trials: 44, high: 0, medium: 0, low: 0, },);
            const arms = [
              high as ArmLog,
              medium as ArmLog,
              armOf({ arm: 'medium-b', cardId: MEDIUM_CARD, rows: twoPassRows({ trials: 44, differ: 22, },), },),
              low as ArmLog,
            ];
            expect(linesStarting({ arms, word: 'REPEAT', },).map(function armOfLine(line,): string {
              return line.slice(0, line.indexOf(':',),);
            },),).toStrictEqual(['REPEAT high', 'REPEAT medium', 'REPEAT medium-b', 'REPEAT low',],);
            expect(linesStarting({ arms, word: 'WARN', },),).toStrictEqual([
              `WARN repeat agreement: medium-b gave the same reading in every pass on 22 of 44 trials it was asked again (50 percent), against 44 of 44 (100 percent) for high; an exact one-sided test of the two counts gives p under 0.0001, at or under 0.05 divided by 3, the number of other arms; ${WARNING_END}`,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'the repeat agreement as the built read step prints it',
      children: [
        it({
          name: 'PRINTS THE REPEAT LINES AND THE WARNING AFTER THE CHECKS AND BEFORE THEIR COUNT, over the logs of two whole '
            + 'passes in which the medium arm repeats itself on 68 of 88 trials, and still ends on the verdict',
          fn: async () => {
            /**
             Rows of two whole passes of the invented pool, the leading `differ` trials disagreeing in the second.
             */
            const passes = (differ: number,): readonly RowSpec[] => twoPassRows({ trials: POOL_TRIALS, differ, },);
            const run = await readAsBuilt({
              arms: [
                armOf({ arm: 'high', cardId: HIGH_CARD, rows: passes(0,), countedUsd: 2.5, },),
                armOf({ arm: 'medium', cardId: MEDIUM_CARD, rows: passes(20,), declaredUsd: 2.5, countedUsd: 1.5, },),
                armOf({ arm: 'low', cardId: LOW_CARD, rows: passes(0,), declaredUsd: 4, countedUsd: 0.875, },),
              ],
            },);
            expect(run.code,).toBe(0,);
            const repeatAt = run.stdout.findIndex(function isRepeat(line,): boolean {
              return line.startsWith('REPEAT high: ',);
            },);
            expect(run.stdout[repeatAt],).toBe('REPEAT high: 88 of 88 trials gave the same reading in every pass (100 percent)',);
            expect(run.stdout[repeatAt + 1],).toBe('REPEAT medium: 68 of 88 trials gave the same reading in every pass (77 percent)',);
            expect(run.stdout[repeatAt + 3]?.startsWith('WARN repeat agreement: medium gave the same reading in every pass on 68 of 88 trials it was asked again (77 percent), against 88 of 88 (100 percent) for high; an exact one-sided test of the two counts gives p under 0.0001, at or under 0.05 divided by 2, the number of other arms;',),).toBe(true,);
            expect(run.stdout[repeatAt + 4]?.startsWith('CHECKS ',),).toBe(true,);
            expect(run.stdout.at(-1,)?.startsWith('VERDICT ',),).toBe(true,);
          },
        },),

        it({
          name: 'PRINTS NO REPEAT WARNING WHEN ONLY THE HIGH ARM REPEATS ITSELF LESS, the same logs with the high and medium '
            + 'arms exchanged, so that only the high fix rate warning (the medium arm fixes every unit) stands between the '
            + 'three repeat lines and the CHECKS line',
          fn: async () => {
            /**
             Rows of two whole passes of the invented pool, the leading `differ` trials disagreeing in the second.
             */
            const passes = (differ: number,): readonly RowSpec[] => twoPassRows({ trials: POOL_TRIALS, differ, },);
            const run = await readAsBuilt({
              arms: [
                armOf({ arm: 'high', cardId: HIGH_CARD, rows: passes(20,), countedUsd: 2.5, },),
                armOf({ arm: 'medium', cardId: MEDIUM_CARD, rows: passes(0,), declaredUsd: 2.5, countedUsd: 1.5, },),
                armOf({ arm: 'low', cardId: LOW_CARD, rows: passes(0,), declaredUsd: 4, countedUsd: 0.875, },),
              ],
            },);
            expect(run.code,).toBe(0,);
            const repeatAt = run.stdout.findIndex(function isRepeat(line,): boolean {
              return line.startsWith('REPEAT high: ',);
            },);
            expect(run.stdout[repeatAt],).toBe('REPEAT high: 68 of 88 trials gave the same reading in every pass (77 percent)',);
            expect(run.stdout[repeatAt + 3]?.startsWith('WARN high fix rate: medium cleanly fixed 176 of 176 shared units (100 percent)',),).toBe(true,);
            expect(run.stdout[repeatAt + 4]?.startsWith('CHECKS ',),).toBe(true,);
            expect(run.stdout.some(function isWarning(line,): boolean {
              return line.startsWith('WARN repeat agreement',);
            },),).toBe(false,);
          },
        },),

        it({
          name: 'PRINTS THE REPEAT LINE OF THE HIGH ARM READ ALONE, as the runbook reads it before the other arms exist: the '
            + 'bench check fails for want of three logs and the command exits with a refusal',
          fn: async () => {
            const run = await readAsBuilt({
              arms: [armOf({ arm: 'high', cardId: HIGH_CARD, rows: twoPassRows({ trials: POOL_TRIALS, differ: 0, },), },),],
            },);
            expect(run.code,).not.toBe(0,);
            expect(run.stdout.filter(function isRepeat(line,): boolean {
              return line.startsWith('REPEAT ',);
            },),).toStrictEqual(['REPEAT high: 88 of 88 trials gave the same reading in every pass (100 percent)',],);
          },
        },),
      ],
    },),
  ],
},);
