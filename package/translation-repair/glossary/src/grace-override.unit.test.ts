/**
 Tests for the straggler window an invocation's rounds run under.

 THE CASE THAT MATTERS IS THE WRONG OVERRIDE. The dial exists so two runs can
 differ in the window and in nothing else, and a value it cannot read that
 quietly became the built-in window would produce two matched runs and a
 recorded conclusion that a longer window buys nothing. That is not a failed
 measurement, it is a wrong one, and it would be believed.

 The empty-string case is the other one worth having, for the reason the cap
 override's suite gives: an exported-but-empty variable is a shell accident.

 The override text is injected rather than the environment mutated, so no case
 here can leak into another.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  adoptCalibrationGrace,
  type CalibrationGrace,
  CALIBRATION_STRAGGLER_GRACE_MS,
  graceOverrideNote,
  isTimerWindow,
  MAX_TIMER_DELAY_MS,
  readWindowDial,
  resolveStragglerGraceMs,
  STRAGGLER_GRACE_VAR,
  StatedRefusalError,
} from '../dist/final/node/index.mjs';

/**
 Built-in window these cases fall back to, standing in for the shipped one.
 */
const FALLBACK = 180_000;

/**
 Longer window a measurement run would ask for.
 */
const LONGER = 300_000;

/**
 Sets or clears the window variable for one case, restoring it after.

 @param says - value to set, or nothing to clear the variable

 @returns Disposable that puts the variable back

 @example
 ```ts
 using dial = windowSaying({},);
 ```
 */
function windowSaying({ says, }: { readonly says?: string; },): Disposable {
  /**
   Value before the case, restored on dispose.
   */
  const before = process.env[STRAGGLER_GRACE_VAR];

  if (says === undefined)
    delete process.env.TRANSLATION_REPAIR_STRAGGLER_GRACE_MS;
  else
    process.env[STRAGGLER_GRACE_VAR] = says;

  return {
    [Symbol.dispose]: () => {
      if (before === undefined)
        delete process.env.TRANSLATION_REPAIR_STRAGGLER_GRACE_MS;
      else
        process.env[STRAGGLER_GRACE_VAR] = before;
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: resolveStragglerGraceMs.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SPELLS the variable the way the documentation does, since an operator who exports a '
            + 'name nothing reads gets the built-in window and no complaint',
          fn: async () => {
            expect(STRAGGLER_GRACE_VAR,).toBe('TRANSLATION_REPAIR_STRAGGLER_GRACE_MS',);
          },
        },),

        it({
          name: 'USES the built-in window when nothing overrides it, which is every ordinary run',
          fn: async () => {
            expect(resolveStragglerGraceMs({
              fallback: FALLBACK,
              raw: '',
            },),).toBe(FALLBACK,);
          },
        },),

        it({
          name: 'HONORS a positive override, which is what makes a longer-window run one build apart '
            + 'from its matched control',
          fn: async () => {
            expect(resolveStragglerGraceMs({
              fallback: FALLBACK,
              raw: '300000',
            },),).toBe(LONGER,);
          },
        },),

        it({
          name: 'IGNORES an empty override and falls back, since an exported-but-empty variable is a '
            + 'shell accident rather than an intention',
          fn: async () => {
            expect(resolveStragglerGraceMs({
              fallback: FALLBACK,
              raw: '   ',
            },),).toBe(FALLBACK,);
          },
        },),

        it({
          name: 'REFUSES a value that is not a number rather than falling back, as a stated refusal '
            + 'naming the variable and the value, because a typo silently becoming the built-in window '
            + 'would compare two matched runs and conclude the window buys nothing',
          fn: async () => {
            /**
             What the reader threw on a value nothing could read.
             */
            const refusal = caught(function readProse(): number {
              return resolveStragglerGraceMs({
                fallback: FALLBACK,
                raw: 'five minutes',
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect((refusal as Error).message,).toContain(STRAGGLER_GRACE_VAR,);
            expect((refusal as Error).message,).toContain('five minutes',);
          },
        },),

        it({
          name: 'REFUSES a trailing-unit value such as `300s`, which `parseFloat` would have read as '
            + '300 and accepted: the number is right and the operator\'s belief about what they set '
            + 'is not',
          fn: async () => {
            expect(
              caught(function readUnit(): number {
                return resolveStragglerGraceMs({
                  fallback: FALLBACK,
                  raw: '300s',
                },);
              },),
            ).toBeInstanceOf(StatedRefusalError,);
          },
        },),

        it({
          name: 'REFUSES zero and negatives, which would abandon every straggler the instant quorum '
            + 'stood and lose the very voices the window exists to keep',
          fn: async () => {
            expect(
              caught(function readZero(): number {
                return resolveStragglerGraceMs({
                  fallback: FALLBACK,
                  raw: '0',
                },);
              },),
            ).toBeInstanceOf(StatedRefusalError,);
            expect(
              caught(function readNegative(): number {
                return resolveStragglerGraceMs({
                  fallback: FALLBACK,
                  raw: '-1000',
                },);
              },),
            ).toBeInstanceOf(StatedRefusalError,);
          },
        },),

        it({
          name: 'REFUSES a fraction and a value past the longest timer, because a timer rounds the one '
            + 'and clamps the other to a millisecond, so the window that ran would not be the window '
            + 'that was set',
          fn: async () => {
            expect(
              caught(function readFraction(): number {
                return resolveStragglerGraceMs({
                  fallback: FALLBACK,
                  raw: '0.5',
                },);
              },),
            ).toBeInstanceOf(StatedRefusalError,);
            expect(
              caught(function readOverflow(): number {
                return resolveStragglerGraceMs({
                  fallback: FALLBACK,
                  raw: String(MAX_TIMER_DELAY_MS + 1,),
                },);
              },),
            ).toBeInstanceOf(StatedRefusalError,);
            expect(resolveStragglerGraceMs({
              fallback: FALLBACK,
              raw: String(MAX_TIMER_DELAY_MS,),
            },),).toBe(MAX_TIMER_DELAY_MS,);
          },
        },),
      ],
    },),

    describe({
      name: isTimerWindow.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HOLDS exactly the whole numbers from one to the longest timer delay, the range a '
            + 'JavaScript timer runs as written',
          fn: async () => {
            expect(isTimerWindow({ ms: 1, },),).toBe(true,);
            expect(isTimerWindow({ ms: FALLBACK, },),).toBe(true,);
            expect(isTimerWindow({ ms: MAX_TIMER_DELAY_MS, },),).toBe(true,);
            expect(isTimerWindow({ ms: 0, },),).toBe(false,);
            expect(isTimerWindow({ ms: -1, },),).toBe(false,);
            expect(isTimerWindow({ ms: 1.5, },),).toBe(false,);
            expect(isTimerWindow({ ms: MAX_TIMER_DELAY_MS + 1, },),).toBe(false,);
            expect(isTimerWindow({ ms: Number.NaN, },),).toBe(false,);
            expect(isTimerWindow({ ms: Number.POSITIVE_INFINITY, },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: readWindowDial.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'THROWS on a fallback no timer holds rather than returning it on a blank dial, since '
            + 'a caller passing such a window is a defect and not an operator\'s input',
          fn: async () => {
            expect(
              caught(function readBadFallback(): number {
                return readWindowDial({
                  variable: STRAGGLER_GRACE_VAR,
                  fallback: 0,
                  raw: '',
                  unsetMeans: 'run under the built-in window',
                },);
              },),
            ).toBeInstanceOf(RangeError,);
          },
        },),

        it({
          name: 'NAMES the variable it was asked to read in the refusal, which is what lets two dials '
            + 'share one reader without the operator correcting the wrong one',
          fn: async () => {
            /**
             What the reader threw for a made-up variable.
             */
            const refusal = caught(function readOther(): number {
              return readWindowDial({
                variable: 'SOME_OTHER_WINDOW_MS',
                fallback: FALLBACK,
                raw: 'soon',
                unsetMeans: 'do nothing',
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect((refusal as Error).message,).toContain('SOME_OTHER_WINDOW_MS',);
            expect((refusal as Error).message,).toContain('do nothing',);
          },
        },),

        it({
          name: 'REFUSES a window not written in digits, which `Number` read as a window nobody typed: a '
            + 'hexadecimal, an exponent, a sign, a space either side and a point; a leading zero still reads as '
            + 'written (ledger B73)',
          fn: async () => {
            /**
             Spellings `Number` reads as one second that no operator writes as one.
             */
            const spellings = [
              '0x3E8',
              '1e3',
              '+1000',
              ' 1000',
              '1000 ',
              '1000.0',
            ];
            expect(spellings.map(function refusalOf(raw,): string {
              /**
               What the reader threw.
               */
              const refusal = caught(function readSpelling(): number {
                return readWindowDial({
                  variable: STRAGGLER_GRACE_VAR,
                  fallback: FALLBACK,
                  raw,
                  unsetMeans: 'run under the built-in window',
                },);
              },);
              expect(refusal,).toBeInstanceOf(StatedRefusalError,);
              return (refusal as Error).message;
            },),).toEqual(spellings.map(function expectedOf(raw,): string {
              return `${STRAGGLER_GRACE_VAR} must be a whole number of milliseconds from 1 to `
                + `${String(MAX_TIMER_DELAY_MS,)}, and ${JSON.stringify(raw,)} is not; leave it unset to run under `
                + 'the built-in window';
            },),);
            expect(readWindowDial({
              variable: STRAGGLER_GRACE_VAR,
              fallback: FALLBACK,
              raw: '01000',
              unsetMeans: 'run under the built-in window',
            },),).toBe(1_000,);
          },
        },),
      ],
    },),

    describe({
      name: graceOverrideNote.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SAYS NOTHING when the run is under the built-in window, so an ordinary run carries no '
            + 'note claiming an override it did not make',
          fn: async () => {
            expect(graceOverrideNote({
              effectiveMs: FALLBACK,
              builtInMs: FALLBACK,
            },),).toBe('',);
          },
        },),

        it({
          name: 'NAMES BOTH WINDOWS AND THE VARIABLE when they differ, so a reader of the log can tell '
            + 'which run this was without reading the shell that launched it',
          fn: async () => {
            /**
             Note for a run under the longer window.
             */
            const note = graceOverrideNote({
              effectiveMs: LONGER,
              builtInMs: FALLBACK,
            },);

            expect(note,).toContain(STRAGGLER_GRACE_VAR,);
            expect(note,).toContain('300000ms',);
            expect(note,).toContain('built-in 180000ms',);
          },
        },),
      ],
    },),

    describe({
      name: adoptCalibrationGrace.name,
      // ONE AT A TIME: every case points the process's own variable elsewhere,
      // to show the reader takes the environment it is handed instead.
      concurrency: 1,
      children: [
        it({
          name: 'ADOPTS the calibration window when the environment it is handed sets none, and writes no '
            + 'variable of the process, since the window now reaches the rounds as a value',
          fn: async () => {
            using dial = windowSaying({},);

            expect({
              grace: adoptCalibrationGrace({ env: {}, },),
              processVariable: process.env[STRAGGLER_GRACE_VAR],
            },).toStrictEqual({
              grace: {
                effectiveMs: CALIBRATION_STRAGGLER_GRACE_MS,
                source: 'calibration-default',
              },
              processVariable: undefined,
            },);
          },
        },),

        it({
          name: 'HONORS the override in the environment it is handed, not the value the process holds',
          fn: async () => {
            using dial = windowSaying({ says: 'five minutes', },);

            expect(adoptCalibrationGrace({ env: { [STRAGGLER_GRACE_VAR]: '180000', }, },),).toStrictEqual({
              effectiveMs: FALLBACK,
              source: 'override',
            },);
          },
        },),

        it({
          name: 'REFUSES an unreadable value in the environment it is handed the way the dial does, instead of '
            + 'adopting over it, while the process holds no value at all',
          fn: async () => {
            using dial = windowSaying({},);

            /**
             What the adoption threw on a value nothing could read.
             */
            const refusal = caught(function adoptsProse(): CalibrationGrace {
              return adoptCalibrationGrace({ env: { [STRAGGLER_GRACE_VAR]: 'five minutes', }, },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${STRAGGLER_GRACE_VAR} must be a whole number of milliseconds from 1 to `
                + `${String(MAX_TIMER_DELAY_MS,)}, and "five minutes" is not; leave it unset to run under the `
                + 'built-in window',
            );
          },
        },),
      ],
    },),
  ],
},);
