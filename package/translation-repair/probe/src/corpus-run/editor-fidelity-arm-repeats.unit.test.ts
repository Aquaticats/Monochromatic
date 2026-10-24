/**
 Tests for the repeat agreement the read step prints for each arm and the warning it gives when the arms differ: the share
 of the trials an arm was asked in more than one pass whose passes all read the same way (a clean fix in every pass or in
 none), what counts as a repeat, and the difference between the arms, in points, above which the exact test's bound on a
 false pick is not assured. The lines are read off the arm logs, and the built command prints them even for one log, since
 the runbook reads the high arm's agreement before the other arms exist.

 The logs are the arm-log fixture's, written by the probe's own writers. Fixtures are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  type ArmLog,
  repeatLinesOf,
} from '../../dist/final/node/index.mjs';
import {
  armLogOf,
  inPass,
  POOL_TRIALS,
  poolRows,
  type RowSpec,
  tallyFromRows,
} from '../editor-fidelity-arm-log.test-fixture.ts';
import {
  HIGH_CARD,
  LOW_CARD,
  MEDIUM_CARD,
} from '../editor-fidelity.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';

/**
 What a warning ends with: the bound, when it holds, and what the simulation reached when the arms differ.
 */
const WARNING_END = 'the bound of 0.05 on choosing another effort when none is better holds when the arms repeat themselves '
  + 'alike, and the simulation reached 0.107 when the default effort repeats itself far less than the others, so a verdict '
  + 'of high or low is less sure than the 0.025 level says';

/**
 The rows of an arm that asks some trials in two passes: the first pass fixes every trial, and the second fixes all but
 the leading ones, so that the passes of those trials differ.

 @param trials - how many leading trials of the invented pool, two to an entry
 @param differ - how many leading trials the second pass does not fix
 @param secondStops - the stop words of the trial at an index in the second pass, `ok` when omitted
 @param fixedAlike - whether the first pass fixes the trials, a trial not fixed in either pass agreeing with itself

 @returns The rows, the first pass then the second

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
 One arm's log over the given rows.

 @param arm - the arm's name
 @param cardId - the card it seated
 @param rows - the rows it walked
 @param declaredUsd - what its guard line says the earlier arms spent
 @param countedUsd - what its guard total says it counted

 @returns The arm's log

 @example
 ```ts
 const log = armOf({ arm: 'high', cardId: HIGH_CARD, rows, },);
 ```
 */
function armOf(
  {
    arm,
    cardId,
    rows,
    declaredUsd = 0,
    countedUsd = 0.5,
  }: {
    readonly arm: string;
    readonly cardId: RosterModelId;
    readonly rows: readonly RowSpec[];
    readonly declaredUsd?: number;
    readonly countedUsd?: number;
  },
): ArmLog {
  return armLogOf({
    arm,
    tally: tallyFromRows({ cardId, rows, },),
    stops: [],
    rows,
    seat: `SEAT ${cardId} asked=${String(rows.length,)} usable=${String(rows.length,)} unusable=0 threw=0 silent=0 abandoned=0 unreachable=0`,
    benchSpentUsd: declaredUsd,
    guardUsd: countedUsd,
  },);
}

/**
 A bench of three arms over trials asked twice, each with its own number of trials whose passes disagree.

 @param trials - how many trials
 @param high - trials whose passes disagree in the high arm
 @param medium - the same in the medium arm
 @param low - the same in the low arm

 @returns The three logs, high first

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

 @param arms - the arms
 @param word - what the lines start with

 @returns The lines

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
 What a run of the built read step left behind.
 */
type BuiltRun = {
  /**
   Exit status, zero when every check held.
   */
  readonly code: number;

  /**
   Standard output, one entry per line.
   */
  readonly stdout: readonly string[];
};

/**
 Runs the built read step over logs written into an owned directory, each beside its exit file.

 @param arms - the logs, in the order the arms ran

 @returns Exit status and standard output

 @example
 ```ts
 const run = await readAsBuilt({ arms, },);
 ```
 */
async function readAsBuilt({ arms, }: { readonly arms: readonly ArmLog[]; },): Promise<BuiltRun> {
  await using owned = await scratchDir({ prefix: 'editor-fidelity-repeats-', },);
  await Promise.all(arms.flatMap(function filesOf({ arm, lines, },): readonly Promise<void>[] {
    return [
      writeFile(
        join(
          owned.path,
          `${arm}.log`,
        ),
        `${lines.join('\n',)}\n`,
      ),
      writeFile(
        join(
          owned.path,
          `${arm}.exit`,
        ),
        '0\n',
      ),
    ];
  },),);
  /**
   What the built command did, whatever its exit code.
   */
  const run = await runBuiltCommand({
    command: 'editor-fidelity-read',
    packageEntry: PROBE_ENTRY,
    args: arms.map(function pathOf({ arm, },): string {
      return join(
        owned.path,
        `${arm}.log`,
      );
    },),
    cwd: owned.path,
    env: {},
  },);
  return {
    code: run.code,
    stdout: run.stdout.split('\n',)
      .filter(function present(line,): boolean {
        return line !== '';
      },),
  };
}

await describe({
  name: repeatLinesOf.name,
  children: [
    it({
      name: 'PRINTS EACH ARM\'S REPEAT AGREEMENT, the trials asked twice whose passes read alike over all the trials asked '
        + 'twice, and says nothing of a difference of 14 points',
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
      name: 'WARNS WHEN THE ARMS DIFFER BY MORE THAN 15 POINTS, naming each arm\'s figure, the difference and the bound the '
        + 'exact test holds only for arms that repeat themselves alike',
      fn: async () => {
        expect(linesStarting({ arms: benchOf({ trials: 44, high: 0, medium: 11, low: 0, },), word: 'WARN', },),)
          .toStrictEqual([
            `WARN repeat agreement: high 100 percent, medium 75 percent and low 100 percent differ by 25 points, over 15; ${WARNING_END}`,
          ],);
      },
    },),

    it({
      name: 'WARNS ABOVE 15 POINTS AND NOT AT 15, counting exactly: 17 of 20 trials against 20 of 20 is 15 points, which a '
        + 'float difference of 0.15000000000000002 would pass over, and 16 of 20 is 20 points',
      fn: async () => {
        expect(linesStarting({ arms: benchOf({ trials: 20, high: 0, medium: 3, low: 0, },), word: 'WARN', },),)
          .toStrictEqual([],);
        expect(linesStarting({ arms: benchOf({ trials: 20, high: 0, medium: 4, low: 0, },), word: 'WARN', },)[0],)
          .toContain('high 100 percent, medium 80 percent and low 100 percent differ by 20 points, over 15',);
      },
    },),

    it({
      name: 'COMPARES ARMS THAT WERE ASKED AGAIN IN DIFFERENT NUMBERS OF TRIALS by their shares: 17 of 20 against 40 of 40 is '
        + '15 points and says nothing',
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
        expect(linesStarting({ arms, word: 'REPEAT', },)[0],)
          .toBe('REPEAT high: 40 of 40 trials gave the same reading in every pass (100 percent)',);
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
      name: 'PRINTS A LINE FOR EACH OF FOUR ARMS WHEN A SECOND MEDIUM IS READ, and names all four in the warning',
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
        expect(linesStarting({ arms, word: 'WARN', },)[0],).toContain(
          'high 100 percent, medium 100 percent, medium-b 50 percent and low 100 percent differ by 50 points, over 15',
        );
      },
    },),
  ],
},);

await describe({
  name: 'the repeat agreement as the built read step prints it',
  children: [
    it({
      name: 'PRINTS THE REPEAT LINES AND THE WARNING AFTER THE CHECKS AND BEFORE THEIR COUNT, over the logs of two whole '
        + 'passes in which the high arm repeats itself on 68 of 88 trials, and still ends on the verdict',
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
        expect(run.stdout[repeatAt + 1],).toBe('REPEAT medium: 88 of 88 trials gave the same reading in every pass (100 percent)',);
        expect(run.stdout[repeatAt + 3]?.startsWith('WARN repeat agreement: high 77 percent, medium 100 percent and low 100 percent differ by 23 points, over 15;',),).toBe(true,);
        expect(run.stdout[repeatAt + 4]?.startsWith('CHECKS ',),).toBe(true,);
        expect(run.stdout.at(-1,)?.startsWith('VERDICT ',),).toBe(true,);
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
},);
