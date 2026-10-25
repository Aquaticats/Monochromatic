/**
 Tests for the readers of flag values the probes and the rendering audit
 share (ledger B73): what a whole number, an id list and a file flag each
 read as, and which reachable ids an id list keeps, given what the
 command-line reader handed on (ledger B75), whose
 own refusals of a flag written with nothing after it, written twice or
 written in a form nobody reads are tested in `command-line.unit.test.ts`.
 Fixtures are cat-themed invention only.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  askedAmong,
  type FlagValue,
  idListFlag,
  usdFlag,
  wholeNumberFlag,
  writtenOr,
} from '../../dist/final/node/index.mjs';
import { statedRefusalMessage, } from '@monochromatic-dev/translation-repair-glossary/ts/stated-refusal-message.test-fixture.ts';

/**
 Largest whole number a double holds exactly, as digits.
 */
const LARGEST = String(Number.MAX_SAFE_INTEGER,);

/**
 Naps a probe takes when nobody names a number.
 */
const DEFAULT_NAPS = 12;

/**
 A flag nobody wrote.
 */
const UNWRITTEN: FlagValue = {
  kind: 'unwritten',
  flag: '--naps',
};

/**
 What a flag carried when the person wrote a value after it.

 @param value - text written after the flag

 @returns The flag as written with that value

 @example
 ```ts
 const asked = written({ value: '3', },);
 ```
 */
function written({ value, }: { readonly value: string; },): FlagValue {
  return {
    kind: 'written',
    flag: '--naps',
    value,
  };
}

/**
 Reads a nap cap the way a probe reads its cap.

 @param asked - what the flag carried

 @returns Naps asked for, or the default

 @throws StatedRefusalError when the reader refuses what was typed

 @example
 ```ts
 const naps = napCap({ asked: written({ value: '3', },), },);
 ```
 */
function napCap({ asked, }: { readonly asked: FlagValue; },): number {
  return wholeNumberFlag({
    asked,
    unwritten: DEFAULT_NAPS,
    leaveOffTo: `take the default of ${String(DEFAULT_NAPS,)} naps`,
  },);
}

/**
 Entries a runner can reach, in the order it walks them.
 */
const LITTER: readonly string[] = [
  'Tabby_01',
  'Ginger42',
  'Mittens',
];

/**
 What the litter is, completing "which ... does not hold".
 */
const LITTER_WITHIN = 'the litter';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wholeNumberFlag.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE CALLER\'S NUMBER when the flag is not written, a sentinel the caller reads as no limit '
            + 'among them',
          fn: async () => {
            expect(napCap({ asked: UNWRITTEN, },),).toBe(DEFAULT_NAPS,);
            expect(wholeNumberFlag({
              asked: UNWRITTEN,
              unwritten: -1,
              leaveOffTo: 'nap without limit',
            },),).toBe(-1,);
          },
        },),
        it({
          name: 'READS zero, a leading zero and the largest whole number a double holds exactly as the numbers written',
          fn: async () => {
            expect(['0', '04', LARGEST,].map(function readsDigits(value,): number {
              return napCap({ asked: written({ value, },), },);
            },),).toEqual([0, 4, Number.MAX_SAFE_INTEGER,],);
          },
        },),
        it({
          name: 'REFUSES a minus sign before digits as a number below zero, saying what leaving the flag off does',
          fn: async () => {
            expect(statedRefusalMessage({
              read: function readsNegative(): void {
                napCap({ asked: written({ value: '-3', },), },);
              },
            },),).toBe('--naps cannot be below zero, and -3 is; leave it off to take the default of 12 naps',);
          },
        },),
        it({
          name: 'REFUSES what is no whole number written in digits, by the package\'s one rule: a word, a point, an '
            + 'exponent, a radix, a sign, a space, minus zero, which names no number below zero, and one past the '
            + 'exact range',
          fn: async () => {
            /**
             Numbers as an operator mistypes them.
             */
            const mistyped = [
              'fourty',
              '4.9',
              '1e1',
              '0x4',
              '+4',
              ' 4',
              '-0',
              String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,),
            ];
            expect(mistyped.map(function readsMistyped(value,): string {
              return statedRefusalMessage({
                read: function read(): void {
                  napCap({ asked: written({ value, },), },);
                },
              },);
            },),).toEqual(mistyped.map(function expectedOf(value,): string {
              return `--naps needs a whole number written in digits, at most ${LARGEST}, and ${JSON.stringify(value,)} is not one`;
            },),);
          },
        },),
      ],
    },),

    describe({
      name: usdFlag.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'TAKES THE CALLER\'S AMOUNT when the flag is not written',
          fn: async () => {
            expect(usdFlag({
              asked: UNWRITTEN,
              unwritten: 2,
            },),).toBe(2,);
          },
        },),
        it({
          name: 'READS a plain amount of dollars, a whole number, a decimal and zero among them',
          fn: async () => {
            expect(['0', '2', '0.5', '1.25', '007.50',].map(function readsAmount(value,): number {
              return usdFlag({
                asked: written({ value, },),
                unwritten: 2,
              },);
            },),).toEqual([0, 2, 0.5, 1.25, 7.5,],);
          },
        },),
        it({
          name: 'REFUSES what is no plain amount: a sign, a point missing a digit, an exponent, a radix, a word, a '
            + 'space, a comma and a digit run past the largest double',
          fn: async () => {
            /**
             Amounts as an operator mistypes them.
             */
            const mistyped = [
              '-1',
              '+1',
              '.5',
              '5.',
              '1e1',
              '0x4',
              'two',
              ' 2',
              '1,5',
              `1${'0'.repeat(400,)}`,
            ];
            expect(mistyped.map(function readsMistyped(value,): string {
              return statedRefusalMessage({
                read: function read(): void {
                  usdFlag({
                    asked: written({ value, },),
                    unwritten: 2,
                  },);
                },
              },);
            },),).toEqual(mistyped.map(function expectedOf(value,): string {
              return `--naps needs a plain amount of dollars written in digits with at most one decimal point, and ${JSON.stringify(value,)} is not one`;
            },),);
          },
        },),
      ],
    },),

    describe({
      name: idListFlag.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ANSWERS with no ids when the flag is not written, which callers read as no restriction',
          fn: async () => {
            expect(idListFlag({
              asked: UNWRITTEN,
              naming: 'nap id',
            },),).toEqual([],);
          },
        },),
        it({
          name: 'READS the ids named, dropping the space around each and the gaps a stray comma leaves',
          fn: async () => {
            expect(idListFlag({
              asked: written({ value: ',tabby, ginger ,,\tcalico,', },),
              naming: 'nap id',
            },),).toEqual(['tabby', 'ginger', 'calico',],);
          },
        },),
        it({
          name: 'REFUSES separators and space that name no id, which would read as no restriction one line later',
          fn: async () => {
            expect([',', ', ,', ' ',].map(function readsNobody(value,): string {
              return statedRefusalMessage({
                read: function read(): void {
                  idListFlag({
                    asked: written({ value, },),
                    naming: 'nap id',
                  },);
                },
              },);
            },),).toEqual([
              '--naps needs at least one nap id, and "," names none',
              '--naps needs at least one nap id, and ", ," names none',
              '--naps needs at least one nap id, and " " names none',
            ],);
          },
        },),
      ],
    },),

    describe({
      name: writtenOr.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the text written, and takes the default only when the flag was not written',
          fn: async () => {
            expect(writtenOr({
              asked: written({ value: 'basket.md', },),
              unwritten: 'cushion.md',
            },),).toBe('basket.md',);
            expect(writtenOr({
              asked: UNWRITTEN,
              unwritten: 'cushion.md',
            },),).toBe('cushion.md',);
          },
        },),
      ],
    },),

    describe({
      name: askedAmong.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS every reachable id when none was asked, in the order the runner walks them',
          fn: async () => {
            expect(askedAmong({
              asked: [],
              known: LITTER,
              source: '--only',
              within: LITTER_WITHIN,
            },),).toEqual(LITTER,);
          },
        },),
        it({
          name: 'KEEPS the reachable ids asked for, in the order the runner walks them rather than as written, '
            + 'each once',
          fn: async () => {
            expect(askedAmong({
              asked: [
                'Mittens',
                'Tabby_01',
                'Mittens',
              ],
              known: LITTER,
              source: '--only',
              within: LITTER_WITHIN,
            },),).toEqual([
              'Tabby_01',
              'Mittens',
            ],);
          },
        },),
        it({
          name: 'REFUSES ids the runner cannot reach, alone or among ids it can, naming each once and as typed, '
            + 'where a runner once walked what was left, or nothing, without a word (ledger B76)',
          fn: async () => {
            expect(function asksForNone(): void {
              askedAmong({
                asked: ['Tabby_0l',],
                known: LITTER,
                source: '--only',
                within: LITTER_WITHIN,
              },);
            },).toThrow('--only asks for "Tabby_0l", which the litter does not hold',);
            expect(function asksForSome(): void {
              askedAmong({
                asked: [
                  'Gingr42',
                  'Tabby_01',
                  'Mit tens',
                  'Gingr42',
                ],
                known: LITTER,
                source: 'sentinel-probe',
                within: LITTER_WITHIN,
              },);
            },).toThrow('sentinel-probe asks for "Gingr42", "Mit tens", which the litter does not hold',);
            expect(caught(function asksForNoneAgain(): unknown {
              return askedAmong({
                asked: ['Tabby_0l',],
                known: LITTER,
                source: '--only',
                within: LITTER_WITHIN,
              },);
            },),).toBeInstanceOf(StatedRefusalError,);
          },
        },),
      ],
    },),
  ],
},);
