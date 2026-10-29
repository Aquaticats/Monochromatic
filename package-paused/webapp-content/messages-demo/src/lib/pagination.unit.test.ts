import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  decodeCursor,
  encodeCursor,
  FEED_PAGE_SIZE,
} from './pagination.ts';

await describe({
  name: '',
  children: [
    describe({
      name: encodeCursor.name,
      children: [
        it({
          name: 'produces a URL-safe token (no +, /, =)',
          fn: async () => {
            const token = encodeCursor({
              createdAt: 1_714_080_000_000,
              id: 1_042,
            },);
            // oxlint-disable-next-line no-restricted-syntax/no-regex -- assertion verifies absence of base64url-forbidden chars; bounded literal class, no backtracking
            expect(token,).not.toMatch(/[+/=]/u,);
          },
        },),

        it({
          name: 'is total: handles zero coordinates',
          fn: async () => {
            const token = encodeCursor({
              createdAt: 0,
              id: 0,
            },);
            expect(typeof token,).toBe('string',);
            expect(token.length,).toBeGreaterThan(0,);
          },
        },),

        it({
          name: 'is total: handles MAX_SAFE_INTEGER coordinates',
          fn: async () => {
            const token = encodeCursor({
              createdAt: Number.MAX_SAFE_INTEGER,
              id: Number.MAX_SAFE_INTEGER,
            },);
            expect(typeof token,).toBe('string',);
          },
        },),

        it({
          name: 'is stable: same input -> same token',
          fn: async () => {
            const cursor = {
              createdAt: 1_714_080_000_000,
              id: 1_042,
            };
            expect(encodeCursor(cursor,),).toBe(encodeCursor(cursor,),);
          },
        },),
      ],
    },),

    describe({
      name: decodeCursor.name,
      children: [
        it({
          name: 'round-trips a typical cursor',
          fn: async () => {
            const original = {
              createdAt: 1_714_080_000_000,
              id: 1_042,
            };
            expect(
              decodeCursor(encodeCursor(original,),),
            ).toEqual(original,);
          },
        },),

        it({
          name: 'round-trips zero coordinates',
          fn: async () => {
            const original = {
              createdAt: 0,
              id: 0,
            };
            expect(
              decodeCursor(encodeCursor(original,),),
            ).toEqual(original,);
          },
        },),

        it({
          name: 'round-trips MAX_SAFE_INTEGER coordinates',
          fn: async () => {
            const original = {
              createdAt: Number.MAX_SAFE_INTEGER,
              id: Number.MAX_SAFE_INTEGER,
            };
            expect(
              decodeCursor(encodeCursor(original,),),
            ).toEqual(original,);
          },
        },),

        it({
          name: 'throws on token with no colon separator',
          fn: async () => {
            /* oxlint-disable no-restricted-syntax/no-regex -- builds a malformed-but-base64url-shaped fixture by stripping `=` padding (anchored, no backtracking) and asserts the decoder throws the documented `malformed cursor` message */
            const token = globalThis
              .btoa('abcdef',)
              .replaceAll(
                '+',
                '-',
              )
              .replaceAll(
                '/',
                '_',
              )
              .replace(
                /=+$/u,
                '',
              );
            expect(function decodeMalformed() {
              decodeCursor(token,);
            },)
              .toThrow(/malformed cursor/u,);
            /* oxlint-enable no-restricted-syntax/no-regex */
          },
        },),

        it({
          name: 'throws on token with non-numeric coordinates',
          fn: async () => {
            /* oxlint-disable no-restricted-syntax/no-regex -- builds a base64url-shaped fixture with a non-numeric `:` body (anchored `=+$` strip, no backtracking) and asserts the documented decoder error */
            const token = globalThis
              .btoa('abc:def',)
              .replaceAll(
                '+',
                '-',
              )
              .replaceAll(
                '/',
                '_',
              )
              .replace(
                /=+$/u,
                '',
              );
            expect(function decodeMalformed() {
              decodeCursor(token,);
            },)
              .toThrow(/malformed cursor/u,);
            /* oxlint-enable no-restricted-syntax/no-regex */
          },
        },),

        it({
          name: 'round-trips many cursors deterministically',
          fn: async () => {
            const cases = Array.from(
              {
                length: 100,
              },
              function gen(_, index,) {
                return {
                  createdAt: 1_700_000_000_000 + (index * 1_000),
                  id: index,
                };
              },
            );
            for (const original of cases)
              expect(
                decodeCursor(encodeCursor(original,),),
              ).toEqual(original,);
          },
        },),
      ],
    },),

    describe({
      name: 'FEED_PAGE_SIZE',
      children: [
        it({
          name: 'is a positive finite integer',
          fn: async () => {
            expect(Number.isInteger(FEED_PAGE_SIZE,),).toBe(true,);
            expect(FEED_PAGE_SIZE,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
  ],
},);
