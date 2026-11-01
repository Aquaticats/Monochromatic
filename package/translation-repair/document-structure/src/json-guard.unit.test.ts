/**
 Tests for JSON narrowing guards shared by protocol parsing and
 model-content validation.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isJsonArray,
  isJsonRecord,
  requireJsonSyntaxRefusal,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: isJsonRecord.name,
      children: [
        it({
          name: 'admits plain records',
          fn: async () => {
            expect(isJsonRecord({ cat: '喵', },),).toBe(true,);
          },
        },),
        it({
          name: 'rejects arrays, since every call site that read one as a record either said so in '
            + 'its own refusal message or forwarded it on to a crash or a spurious index-keyed entry '
            + '(ledger B92)',
          fn: async () => {
            expect(isJsonRecord(['喵',],),).toBe(false,);
          },
        },),
        it({
          name: 'rejects null despite its object typeof',
          fn: async () => {
            expect(isJsonRecord(null,),).toBe(false,);
          },
        },),
        it({
          name: 'rejects primitives',
          fn: async () => {
            expect(isJsonRecord('喵',),).toBe(false,);
            expect(isJsonRecord(1,),).toBe(false,);
            expect(isJsonRecord(undefined,),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: isJsonArray.name,
      children: [
        it({
          name: 'admits arrays',
          fn: async () => {
            expect(isJsonArray(['喵',],),).toBe(true,);
          },
        },),
        it({
          name: 'rejects records, null, and primitives',
          fn: async () => {
            expect(isJsonArray({ cat: '喵', },),).toBe(false,);
            expect(isJsonArray(null,),).toBe(false,);
            expect(isJsonArray('喵',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: requireJsonSyntaxRefusal.name,
      children: [
        it({
          name: 'RETURNS the syntax refusal a catch around JSON.parse holds, the same object',
          fn: async () => {
            /**
             Refusal the parser raised over text that is not JSON.
             */
            const refusal = caught(function parseHairball(): void {
              JSON.parse('the cat sat on {',);
            },);

            expect(refusal,).toBeInstanceOf(SyntaxError,);
            expect(requireJsonSyntaxRefusal({ error: refusal, },),).toBe(refusal,);
          },
        },),
        it({
          name: 'RETHROWS anything else unchanged, an error or not, since an unexpected state must keep propagating',
          fn: async () => {
            /**
             A failure that is not a syntax refusal.
             */
            const stray = new TypeError('the cat knocked the parser off the table',);

            expect(caught(function narrowStray(): void {
              requireJsonSyntaxRefusal({ error: stray, },);
            },),).toBe(stray,);
            expect(caught(function narrowString(): void {
              requireJsonSyntaxRefusal({ error: 'hairball', },);
            },),).toBe('hairball',);
          },
        },),
      ],
    },),
  ],
},);
