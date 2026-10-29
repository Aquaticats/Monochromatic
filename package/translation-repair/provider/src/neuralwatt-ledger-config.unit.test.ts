/**
 Tests where the NeuralWatt energy ledger lives and how much a day it holds
 (`https://docs.neuralwatt.com/billing/daily-free-energy`, fetched 2026-10-20:
 500 Wh on the Max plan and 50 Wh for the product-news email), and the two
 errors reading it can raise.

 The environment and the home directory are handed in as values, so no case
 reads the real ones. Fixtures are cat-themed invention.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  defaultNeuralWattLedgerPath,
  NEURALWATT_DAILY_WH,
  NEURALWATT_DAILY_WH_VAR,
  NEURALWATT_LEDGER_PATH_VAR,
  NeuralWattAllowanceOverrideError,
  neuralWattDailyWhFrom,
  NeuralWattLedgerShapeError,
  neuralWattLedgerPathFrom,
} from '../dist/final/node/index.mjs';

/**
 Home directory every case hangs the default off; no case reads the real one.
 */
const HOME = '/home/whiskers';

/**
 Where the ledger lives under that home when nothing says otherwise.
 */
const DEFAULT_PATH = '/home/whiskers/.local/state/translation-repair/neuralwatt-energy.jsonl';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: defaultNeuralWattLedgerPath.name,
      children: [
        it({
          name: 'LIVES UNDER THE STATE DIRECTORY OF THE HOME IT IS GIVEN, never a spelled-out username',
          fn: async () => {
            expect(defaultNeuralWattLedgerPath({ home: HOME, },),).toBe(DEFAULT_PATH,);
            expect(defaultNeuralWattLedgerPath({ home: '/home/tabby', },),).toBe(
              '/home/tabby/.local/state/translation-repair/neuralwatt-energy.jsonl',
            );
          },
        },),
      ],
    },),

    describe({
      name: neuralWattDailyWhFrom.name,
      children: [
        it({
          name: 'STANDS AT 550 WATT-HOURS A DAY when the environment does not name an amount, absent, empty and blank '
            + 'all counting as not naming one',
          fn: async () => {
            expect(NEURALWATT_DAILY_WH,).toBe(550,);
            for (const env of [{}, { [NEURALWATT_DAILY_WH_VAR]: '', }, { [NEURALWATT_DAILY_WH_VAR]: ' \t ', },])
              expect(neuralWattDailyWhFrom({ env, },),).toBe(NEURALWATT_DAILY_WH,);
          },
        },),

        it({
          name: 'TAKES THE AMOUNT THE ENVIRONMENT NAMES, a whole number, a fraction, and ZERO for a day with no promo',
          fn: async () => {
            expect(neuralWattDailyWhFrom({ env: { [NEURALWATT_DAILY_WH_VAR]: '120.5', }, },),).toBe(120.5,);
            expect(neuralWattDailyWhFrom({ env: { [NEURALWATT_DAILY_WH_VAR]: '800', }, },),).toBe(800,);
            expect(neuralWattDailyWhFrom({ env: { [NEURALWATT_DAILY_WH_VAR]: '0', }, },),).toBe(0,);
          },
        },),

        it({
          name: 'REFUSES AN AMOUNT THAT IS NOT A PLAIN NON-NEGATIVE DECIMAL, naming the variable and quoting what it '
            + 'held, for a word, a sign, an exponent, a prefix, a non-number, a stray unit and a figure too large to be finite',
          fn: async () => {
            for (const value of ['plenty', '-5', '+5', '1e3', '0x10', 'NaN', 'Infinity', '5 Wh', ' 5', '1'.repeat(400,),]) {
              /**
               What reading that value throws.
               */
              const refusal = caught(function read(): void {
                neuralWattDailyWhFrom({ env: { [NEURALWATT_DAILY_WH_VAR]: value, }, },);
              },);
              expect(refusal,).toBeInstanceOf(NeuralWattAllowanceOverrideError,);
              expect(String(refusal,),).toContain(NEURALWATT_DAILY_WH_VAR,);
              expect(String(refusal,),).toContain(JSON.stringify(value,),);
            }
          },
        },),
      ],
    },),

    describe({
      name: neuralWattLedgerPathFrom.name,
      children: [
        it({
          name: 'FALLS BACK TO THE DEFAULT UNDER THE HOME when the environment does not name a path, absent, empty '
            + 'and blank all counting as not naming one',
          fn: async () => {
            for (const env of [{}, { [NEURALWATT_LEDGER_PATH_VAR]: '', }, { [NEURALWATT_LEDGER_PATH_VAR]: '   ', },])
              expect(neuralWattLedgerPathFrom({ env, home: HOME, },),).toBe(DEFAULT_PATH,);
          },
        },),

        it({
          name: 'TAKES THE PATH THE ENVIRONMENT NAMES as it stands, whatever the home is',
          fn: async () => {
            expect(neuralWattLedgerPathFrom({
              env: { [NEURALWATT_LEDGER_PATH_VAR]: '/var/tmp/nap.jsonl', },
              home: HOME,
            },),).toBe('/var/tmp/nap.jsonl',);
          },
        },),
      ],
    },),

    describe({
      name: NeuralWattAllowanceOverrideError.name,
      children: [
        it({
          name: 'NAMES THE VARIABLE AND QUOTES THE VALUE, declares its message safe to forward and names itself',
          fn: async () => {
            /**
             Error for a value that cannot be an allowance.
             */
            const refusal = new NeuralWattAllowanceOverrideError({ value: '-5', },);
            expect(refusal.message,).toBe(
              'TRANSLATION_REPAIR_NEURALWATT_DAILY_WH must be a plain non-negative number of watt-hours; it holds "-5"',
            );
            expect(refusal.name,).toBe('NeuralWattAllowanceOverrideError',);
            expect(refusal.messageNamesOnly,).toBe(true,);
            expect(refusal,).toBeInstanceOf(Error,);
          },
        },),

        it({
          name: 'QUOTES A VALUE WITH A QUOTE OR A NEWLINE IN IT ESCAPED, so the message stays one line',
          fn: async () => {
            expect(new NeuralWattAllowanceOverrideError({ value: 'a"b\nc', },).message,).toContain(String.raw`"a\"b\nc"`,);
          },
        },),
      ],
    },),

    describe({
      name: NeuralWattLedgerShapeError.name,
      children: [
        it({
          name: 'NAMES THE FILE, THE LINE AND WHAT WAS WRONG, declares its message safe to forward and names itself',
          fn: async () => {
            /**
             Error for the third line of a ledger file.
             */
            const refusal = new NeuralWattLedgerShapeError({
              path: '/var/tmp/nap.jsonl',
              line: 3,
              detail: 'wh is not a non-negative number',
            },);
            expect(refusal.message,).toBe(
              'NeuralWatt energy ledger /var/tmp/nap.jsonl line 3 violated expectations: wh is not a non-negative number',
            );
            expect(refusal.name,).toBe('NeuralWattLedgerShapeError',);
            expect(refusal.messageNamesOnly,).toBe(true,);
            expect(refusal,).toBeInstanceOf(Error,);
          },
        },),
      ],
    },),
  ],
},);
