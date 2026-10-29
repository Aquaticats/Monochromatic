/**
 Tests for the strict reader of DeepSeek's `/user/balance` body. Fixtures are cat-themed invention; nothing reaches a
 provider.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DeepSeekBalanceShapeError,
  parseDeepSeekBalance,
} from '../dist/final/node/index.mjs';

/**
 A balance body as DeepSeek writes it.

 @param total - USD entry's `total_balance` text

 @param available - account's `is_available` flag

 @returns Body text

 @example
 ```ts
 const body = balanceBody({ total: '9.96', },);
 ```
 */
function balanceBody({ total, available = true, }: { readonly total: string; readonly available?: boolean; },): string {
  return JSON.stringify({
    is_available: available,
    balance_infos: [{ currency: 'USD', total_balance: total, granted_balance: '0.00', topped_up_balance: total, },],
  },);
}

/**
 Reads a body and returns what it threw, so a case can name the class.

 @param bodyText - body to parse

 @returns Thrown value

 @example
 ```ts
 expect(failureOf({ bodyText: 'not json', },),).toBeInstanceOf(DeepSeekBalanceShapeError,);
 ```
 */
function failureOf({ bodyText, }: { readonly bodyText: string; },): unknown {
  try {
    parseDeepSeekBalance({ bodyText, },);
  } catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: parseDeepSeekBalance.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'READS the USD total and availability',
      fn: async () => {
        expect(parseDeepSeekBalance({ bodyText: balanceBody({ total: '9.96', }), },),)
          .toEqual({ isAvailable: true, totalUsd: 9.96, },);
        expect(parseDeepSeekBalance({ bodyText: balanceBody({ total: '0.00', available: false, }), },),)
          .toEqual({ isAvailable: false, totalUsd: 0, },);
      },
    },),
    it({
      name: 'REFUSES every body that is not the documented shape, naming what failed',
      fn: async () => {
        const bad: readonly string[] = [
          'not json',
          '[]',
          '{"is_available":"yes","balance_infos":[]}',
          '{"is_available":true,"balance_infos":"none"}',
          '{"is_available":true,"balance_infos":[]}',
          JSON.stringify({
            is_available: true,
            balance_infos: [
              { currency: 'USD', total_balance: '1.00', },
              { currency: 'USD', total_balance: '2.00', },
            ],
          },),
          JSON.stringify({ is_available: true, balance_infos: [{ currency: 'USD', total_balance: 9.96, },], },),
          balanceBody({ total: '-1.00', },),
          balanceBody({ total: 'a lot', },),
          balanceBody({ total: '', },),
        ];
        for (const bodyText of bad)
          expect(failureOf({ bodyText, },),).toBeInstanceOf(DeepSeekBalanceShapeError,);
      },
    },),
    it({
      name: 'REFUSES a body that states its balance in yuan only, since the meter and its floor are in USD',
      fn: async () => {
        const bodyText = JSON.stringify({
          is_available: true,
          balance_infos: [{
            currency: 'CNY',
            total_balance: '70.00',
            granted_balance: '0.00',
            topped_up_balance: '70.00',
          },],
        },);
        expect(failureOf({ bodyText, },),).toBeInstanceOf(DeepSeekBalanceShapeError,);
      },
    },),
    it({
      name: 'COUNTS ONLY OBJECTS as balance entries: a bare string among them is not a USD entry, and beside a real '
        + 'one '
        + 'it is ignored',
      fn: async () => {
        const alone = JSON.stringify({ is_available: true, balance_infos: ['USD',], },);
        expect(failureOf({ bodyText: alone, },),).toBeInstanceOf(DeepSeekBalanceShapeError,);
        const beside = JSON.stringify({
          is_available: true,
          balance_infos: ['USD', { currency: 'USD', total_balance: '3.50', },],
        },);
        expect(parseDeepSeekBalance({ bodyText: beside, },),).toEqual({ isAvailable: true, totalUsd: 3.5, },);
      },
    },),
    it({
      name: 'REFUSES an amount written past the largest double, which would read as an infinite balance and never '
        + 'run dry',
      fn: async () => {
        expect(failureOf({ bodyText: balanceBody({ total: '9'.repeat(400,), }), },),)
          .toBeInstanceOf(DeepSeekBalanceShapeError,);
        expect(failureOf({ bodyText: balanceBody({ total: '1e999', }), },),).toBeInstanceOf(DeepSeekBalanceShapeError,);
      },
    },),
  ],
},);
