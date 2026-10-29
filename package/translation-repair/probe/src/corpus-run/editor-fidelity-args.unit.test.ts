/**
 Tests for the editor fidelity probe's command line: the entry filter, the trial cap, the defects to build and the
 amount it may spend, read from the probe's command line the way `reportingRefusals` reads it. Fixtures are cat-themed
 invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_EDITOR_MAX_USD,
  DEFAULT_EDITOR_TRIAL_CAP,
  EDITOR_BENCH_MAX_USD,
  EDITOR_DAMAGE_KINDS,
  readEditorFidelityArguments,
} from '../../dist/final/node/index.mjs';
import { refusalOf, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/cap-argument-refusal.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

/**
 Reads the probe's arguments off a command line carrying exactly what was typed.

 @param typed - what the operator wrote after the script path

 @returns What the reader settled on

 @throws StatedRefusalError when the reader refuses what was typed

 @example
 ```ts
 const asked = exactlyAskedFrom({ typed: ['--bench-spent-usd', '0',], },);
 ```
 */
function exactlyAskedFrom(
  { typed, }: { readonly typed: readonly string[]; },
): ReturnType<typeof readEditorFidelityArguments> {
  return readEditorFidelityArguments({
    line: lineOf({
      command: 'editor-fidelity-probe',
      typed,
    },),
  },);
}

/**
 Reads the probe's arguments off a command line carrying what was typed, with the spend of the earlier arms stated as
 zero in front when the line does not state it, since the probe refuses a line that leaves it out.

 @param typed - what the operator wrote after the script path

 @returns What the reader settled on

 @throws StatedRefusalError when the reader refuses what was typed

 @example
 ```ts
 const asked = askedFrom({ typed: ['--cap', '3',], },);
 ```
 */
function askedFrom(
  { typed, }: { readonly typed: readonly string[]; },
): ReturnType<typeof readEditorFidelityArguments> {
  return exactlyAskedFrom({
    typed: typed.includes('--bench-spent-usd',) ? typed : ['--bench-spent-usd', '0', ...typed,],
  },);
}

await describe({
  name: readEditorFidelityArguments.name,
  children: [
    it({
      name: 'REFUSES A LINE THAT DOES NOT STATE --bench-spent-usd, since an arm that is not told what the earlier arms '
        + 'spent cannot hold the bench to its total, and says that zero is what the first arm writes',
      fn: async () => {
        /**
         The refusal both lines draw.
         */
        const expected = '--bench-spent-usd is required: write what the arms already run measured (the sum of their '
          + 'GUARD total= lines, rounded up to the cent), or 0 for the first arm of a bench';
        expect(refusalOf({
          typed: [],
          askedFrom: exactlyAskedFrom,
        },),).toBe(expected,);
        expect(refusalOf({
          typed: ['--max-usd', '3', '--cap', '0',],
          askedFrom: exactlyAskedFrom,
        },),).toBe(expected,);
        expect(exactlyAskedFrom({ typed: ['--bench-spent-usd', '0',], },).benchSpentUsd,).toBe(0,);
      },
    },),
    it({
      name: 'TAKES every entry, the default cap, both defects and two dollars when no flag but the spend of the earlier '
        + 'arms is written',
      fn: async () => {
        expect(DEFAULT_EDITOR_MAX_USD,).toBe(2,);
        expect(askedFrom({ typed: [], },),).toEqual({
          onlyIds: [],
          cap: DEFAULT_EDITOR_TRIAL_CAP,
          damageKinds: EDITOR_DAMAGE_KINDS,
          maxUsd: DEFAULT_EDITOR_MAX_USD,
          benchSpentUsd: 0,
        },);
      },
    },),
    it({
      name: 'READS the entries, the cap, one defect and the amount as written',
      fn: async () => {
        expect(askedFrom({
          typed: [
            '--only',
            'tabby,,ginger',
            '--cap',
            '3',
            '--damage',
            'insertion',
            '--max-usd',
            '0.75',
            '--bench-spent-usd',
            '7.25',
          ],
        },),).toEqual({
          onlyIds: ['tabby', 'ginger',],
          cap: 3,
          damageKinds: ['insertion',],
          maxUsd: 0.75,
          benchSpentUsd: 7.25,
        },);
      },
    },),
    it({
      name: 'NAMES THE BENCH TOTAL at sixty dollars, the amount the owner ruled on 2026-10-24 for all the arms of the '
        + 'bench together, and starts an arm that reaches it exactly',
      fn: async () => {
        expect(EDITOR_BENCH_MAX_USD,).toBe(60,);
        expect(askedFrom({ typed: ['--bench-spent-usd', '58', '--max-usd', '2',], },).benchSpentUsd,).toBe(58,);
        expect(askedFrom({ typed: ['--bench-spent-usd', '42', '--max-usd', '18',], },).maxUsd,).toBe(18,);
      },
    },),
    it({
      name: 'REFUSES TO START an arm whose --max-usd, added to the spend the earlier arms of the bench measured, '
        + 'passes the bench total, naming the three figures and what --max-usd may still be',
      fn: async () => {
        expect(refusalOf({
          typed: ['--bench-spent-usd', '58.5', '--max-usd', '2',],
          askedFrom,
        },),).toBe(
          'the bench would spend up to 60.5 USD: --bench-spent-usd 58.5 plus --max-usd 2 passes the 60 USD the owner '
            + 'ruled for all the arms of the editor fidelity bench together; write a --max-usd of at most 1.5, or '
            + 'stop the bench here',
        );
        expect(refusalOf({
          typed: ['--bench-spent-usd', '60', '--max-usd', '0.01',],
          askedFrom,
        },),).toContain('write a --max-usd of at most 0',);
      },
    },),
    it({
      name: 'REFUSES A DELETION by saying why: its fix is a sentence the editor must write, which no deterministic '
        + 'test can score against the archive\'s own wording',
      fn: async () => {
        expect(refusalOf({
          typed: ['--damage', 'deletion',],
          askedFrom,
        },),).toBe(
          '--damage deletion is not built for the editor probe: its fix is a sentence the editor must write, '
            + 'which no deterministic test scores against the archive\'s wording; the probe takes insertion or alteration',
        );
      },
    },),
    it({
      name: 'REFUSES a defect this probe does not build, naming the ones it does, and a name that sits on '
        + 'Object.prototype',
      fn: async () => {
        expect(['scratches', 'constructor', '__proto__',].map(function refusedDefect(name,): string {
          return refusalOf({
            typed: ['--damage', name,],
            askedFrom,
          },);
        },),).toEqual([
          '--damage takes insertion or alteration, not "scratches"',
          '--damage takes insertion or alteration, not "constructor"',
          '--damage takes insertion or alteration, not "__proto__"',
        ],);
      },
    },),
    it({
      name: 'READS --trials as the number of leading trials an arm walks, leaving the member out when the flag is not '
        + 'written, and takes it beside a cap of zero, the preflight that prices those trials',
      fn: async () => {
        expect(askedFrom({ typed: ['--trials', '41',], },).trials,).toBe(41,);
        expect('trials' in askedFrom({ typed: [], },),).toBe(false,);
        expect(askedFrom({ typed: ['--trials', '41', '--cap', '0',], },).trials,).toBe(41,);
        expect(askedFrom({ typed: ['--trials', '41', '--cap', '0',], },).cap,).toBe(0,);
      },
    },),
    it({
      name: 'REFUSES --trials of zero, which would walk nothing, and --trials beside a cap above zero, since the two both '
        + 'name how many trials to run',
      fn: async () => {
        expect(refusalOf({
          typed: ['--trials', '0',],
          askedFrom,
        },),).toBe(
          '--trials needs at least 1, and 0 walks no trial; write --cap 0 to price an arm without running it',
        );
        expect(refusalOf({
          typed: ['--trials', '41', '--cap', '5',],
          askedFrom,
        },),).toBe(
          '--trials 41 and --cap 5 both name how many trials to run; write one of them, or --cap 0 beside --trials to '
            + 'price those trials without running them',
        );
      },
    },),
    it({
      name: 'READS --passes as the number of times an arm walks the pool, leaving the member out when the flag is not '
        + 'written, and takes it beside --trials, which then counts asks through the passes, and beside a cap',
      fn: async () => {
        expect(askedFrom({ typed: ['--passes', '2',], },).passes,).toBe(2,);
        expect('passes' in askedFrom({ typed: [], },),).toBe(false,);
        expect(askedFrom({ typed: ['--passes', '2', '--trials', '120', '--cap', '0',], },),).toMatchObject({
          passes: 2,
          trials: 120,
          cap: 0,
        },);
        expect(askedFrom({ typed: ['--passes', '3', '--cap', '150',], },),).toMatchObject({
          passes: 3,
          cap: 150,
        },);
      },
    },),
    it({
      name: 'REFUSES --passes of zero, which would walk the pool no time',
      fn: async () => {
        expect(refusalOf({
          typed: ['--passes', '0',],
          askedFrom,
        },),).toBe('--passes needs at least 1, and 0 walks the pool no time');
      },
    },),
    it({
      name: 'LEAVES THE DEFAULT CAP at a thousand asks, more than the pool walked through several passes',
      fn: async () => {
        expect(DEFAULT_EDITOR_TRIAL_CAP,).toBe(1_000,);
      },
    },),
    it({
      name: 'REFUSES a cap or an amount mistyped, and a flag written last or naming nobody',
      fn: async () => {
        /**
         Usage line every refusal of the probe's command line ends with.
         */
        const usage = 'Usage: editor-fidelity-probe [--only <entry ids>] [--cap <count>] '
          + '[--damage <insertion|alteration>] [--candidates <seatable ids>] [--max-usd <dollars>] '
          + '[--bench-spent-usd <dollars>] [--trials <count>] [--passes <count>] [--candidates-alone]';
        /**
         Command lines an operator mistyped, each with the refusal it must draw.
         */
        const mistyped: readonly (readonly [readonly string[], string])[] = [
          [['--cap', 'fourty',], '--cap needs a whole number written in digits, at most 9007199254740991, and "fourty" is not one',],
          [
            ['--cap', '-3',],
            `--cap cannot be below zero, and -3 is; leave it off to run the default of ${
              String(DEFAULT_EDITOR_TRIAL_CAP,)
            } trials`,
          ],
          [
            ['--max-usd', 'two',],
            '--max-usd needs a plain amount of dollars written in digits with at most one decimal point, '
              + 'and "two" is not one',
          ],
          [['--max-usd',], `--max-usd needs a value written after it. ${usage}`,],
          [
            ['--bench-spent-usd', '-1',],
            '--bench-spent-usd needs a plain amount of dollars written in digits with at most one decimal point, '
              + 'and "-1" is not one',
          ],
          [['--only', ',',], '--only needs at least one entry id, and "," names none',],
          [['--damage',], `--damage needs a value written after it. ${usage}`,],
        ];
        expect(mistyped.map(function refusalFor([typed,],): string {
          return refusalOf({
            typed,
            askedFrom,
          },);
        },),).toEqual(mistyped.map(function expectedOf([, said,],): string {
          return said;
        },),);
      },
    },),
  ],
},);
