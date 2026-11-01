/**
 Tests for the fidelity probe's command line: the entry filter, the trial
 cap, the defects to build and the context switch, read from the probe's
 command line the way `reportingRefusals` reads it (ledger B75). Fixtures
 are cat-themed invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DAMAGE_KINDS,
  DEFAULT_TRIAL_CAP,
  readFidelityArguments,
} from '../../dist/final/node/index.mjs';
import { capNotDigits, refusalOf, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/cap-argument-refusal.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';

/**
 Usage line every refusal of the probe's command line ends with.
 */
const USAGE = 'Usage: judge-fidelity-probe [--only <entry ids>] [--cap <count>] '
  + '[--damage <deletion|insertion|alteration>] [--candidates <seatable ids>] [--context] [--candidates-alone]';

/**
 Reads the probe's arguments off a command line carrying what was typed.

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
): ReturnType<typeof readFidelityArguments> {
  return readFidelityArguments({
    line: lineOf({
      command: 'judge-fidelity-probe',
      typed,
    },),
  },);
}

await describe({
  name: readFidelityArguments.name,
  children: [
    it({
      name: 'TAKES every entry, the default cap, every defect and no context when no flag is written',
      fn: async () => {
        expect(askedFrom({ typed: [], },),).toEqual({
          onlyIds: [],
          cap: DEFAULT_TRIAL_CAP,
          damageKinds: DAMAGE_KINDS,
          withContext: false,
        },);
      },
    },),
    it({
      name: 'READS the entries, the cap, one defect and the context switch as written',
      fn: async () => {
        expect(askedFrom({
          typed: ['--only', 'tabby,,ginger', '--cap', '3', '--damage', 'insertion', '--context',],
        },),).toEqual({
          onlyIds: ['tabby', 'ginger',],
          cap: 3,
          damageKinds: ['insertion',],
          withContext: true,
        },);
      },
    },),
    it({
      name: 'REFUSES a defect this probe does not build, naming the ones it does, and a name that sits on '
        + 'Object.prototype, which a plain-object table once answered with a function (ledger B75)',
      fn: async () => {
        expect(['scratches', 'constructor', '__proto__', 'toString',].map(function refusedDefect(name,): string {
          return refusalOf({ typed: ['--damage', name,], askedFrom, },);
        },),).toEqual([
          '--damage takes deletion, insertion or alteration, not "scratches"',
          '--damage takes deletion, insertion or alteration, not "constructor"',
          '--damage takes deletion, insertion or alteration, not "__proto__"',
          '--damage takes deletion, insertion or alteration, not "toString"',
        ],);
      },
    },),
    it({
      name: 'REFUSES a cap that is no whole number written in digits, or below zero, and a flag written last or '
        + 'naming nobody, where each fell back to a default or to every entry in silence and spent trials nobody '
        + 'asked for (ledger B73)',
      fn: async () => {
        /**
         Command lines an operator mistyped, each with the refusal it must draw.
         */
        const mistyped: readonly (readonly [readonly string[], string])[] = [
          [['--cap', 'fourty',], capNotDigits({ cap: 'fourty', },),],
          [['--cap', '4.9',], capNotDigits({ cap: '4.9', },),],
          [
            ['--cap', '-3',],
            '--cap cannot be below zero, and -3 is; leave it off to run the default of '
              + `${String(DEFAULT_TRIAL_CAP,)} trials`,
          ],
          [['--cap',], `--cap needs a value written after it. ${USAGE}`,],
          [['--only',], `--only needs a value written after it. ${USAGE}`,],
          [['--only', ',',], '--only needs at least one entry id, and "," names none',],
          [['--damage',], `--damage needs a value written after it. ${USAGE}`,],
          [['--context=yes',], `--context takes no value, and --context=yes gives it one. ${USAGE}`,],
        ];
        expect(mistyped.map(function refusalFor([typed,],): string {
          return refusalOf({ typed, askedFrom, },);
        },),).toEqual(mistyped.map(function expectedOf([, said,],): string {
          return said;
        },),);
      },
    },),
  ],
},);
