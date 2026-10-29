/**
 Tests for the answers the checks of an editor fidelity arm give, one at a time over facts read from small logs: the
 reasons they give for a FAIL are pinned in `editor-fidelity-arm-check.unit.test.ts` over whole logs, and these cases
 cover the branches that file reaches only in passing. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type ArmLog,
  exitAnswer,
  factsOf,
  rowAnswer,
  seatAnswer,
  spendAnswer,
} from '../../dist/final/node/index.mjs';

/**
 Prefix the logger puts on every line.
 */
const PREFIX = '[info] [2026-10-23T12:00:00.000Z] [probe] ';

/**
 An arm log of the lines given, with no exit recorded.

 @param lines - lines of the log

 @returns Arm

 @example
 ```ts
 const log = logOf({ lines: [], },);
 ```
 */
function logOf({ lines, }: { readonly lines: readonly string[]; }): ArmLog {
  return {
    arm: 'low',
    lines,
    exit: { recorded: false, },
  };
}

await describe({
  name: 'editor fidelity arm answers',
  children: [
    it({
      name: 'ANSWERS THE EXIT CHECK from the status recorded: zero holds, any other fails, none recorded fails',
      fn: async () => {
        expect(exitAnswer({ log: { ...logOf({ lines: [], },), exit: { recorded: true, code: 0, }, }, },).ok,).toBe(true,);
        expect(exitAnswer({ log: { ...logOf({ lines: [], },), exit: { recorded: true, code: 7, }, }, },).ok,).toBe(false,);
        expect(exitAnswer({ log: logOf({ lines: [], },), },).detail,).toBe('no exit status was recorded for this arm',);
      },
    },),

    it({
      name: 'ANSWERS THE ROW CHECK for a single trial in the singular, passing a row logged once',
      fn: async () => {
        const rows = [`${PREFIX}planned: 1 of 1 trial`, `${PREFIX}row 1 of 1: card on x: ok`, `${PREFIX}TRIALS completed 1 of 1`,];
        const answer = rowAnswer({ facts: factsOf({ log: logOf({ lines: rows, },), },), },);
        expect(answer.ok,).toBe(true,);
        expect(answer.detail,).toBe('1 row logged, each once',);
      },
    },),

    it({
      name: 'ANSWERS THE ROW CHECK for an arm capped by the guard: the rows logged are the trials completed, fewer than '
        + 'planned, and a CAPPED line says why; fewer completed than planned with no CAPPED line fails, and so does a log '
        + 'with no TRIALS line',
      fn: async () => {
        const capped = [
          `${PREFIX}planned: 3 of 88 trials`,
          `${PREFIX}row 1 of 3: card on x: ok`,
          `${PREFIX}row 2 of 3: card on y: ok`,
          `${PREFIX}CAPPED at unit 3 of 3: 2 units completed; the guard refused`,
          `${PREFIX}TRIALS completed 2 of 3`,
        ];
        expect(rowAnswer({ facts: factsOf({ log: logOf({ lines: capped, },), },), },).detail,).toBe(
          '2 rows logged, each once; the guard capped the arm before unit 3',
        );
        expect(rowAnswer({ facts: factsOf({ log: logOf({ lines: capped, },), },), },).ok,).toBe(true,);
        /**
         The same log without the line that says why it stopped short.
         */
        const silent = capped.filter(function notCapped(line,): boolean {
          return !line.includes('CAPPED',);
        },);
        expect(rowAnswer({ facts: factsOf({ log: logOf({ lines: silent, },), },), },),).toStrictEqual({
          ok: false,
          detail: '2 of 3 planned units were completed and no CAPPED line says why the arm stopped',
        },);
        expect(rowAnswer({
          facts: factsOf({
            log: logOf({ lines: capped.filter(function notTrials(line,): boolean {
              return !line.includes('TRIALS completed',);
            },), },),
          },),
        },),).toStrictEqual({ ok: false, detail: 'no TRIALS line was logged', },);
      },
    },),

    it({
      name: 'ANSWERS THE SEAT CHECK as failing when no card was seated, and the SPEND check as failing when the guard '
        + 'counted nothing',
      fn: async () => {
        expect(seatAnswer({ facts: factsOf({ log: logOf({ lines: [], },), },), },).ok,).toBe(false,);
        expect(spendAnswer({ log: logOf({ lines: [], },), facts: factsOf({ log: logOf({ lines: [], },), },), },).detail,).toBe(
          'no GUARD total line was logged',
        );
      },
    },),
  ],
},);
