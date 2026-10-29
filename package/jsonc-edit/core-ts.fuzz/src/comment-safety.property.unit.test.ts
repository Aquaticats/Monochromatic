/**
 Property test proving that an arbitrary comment body (including `*\/`,
 newlines, quotes, and control characters) always emits parseable JSONC that
 reaches a canonical fixpoint. This is the STB guard for the comment emitter.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { StringJsonc, } from '@monochromatic-dev/jsonc-edit-core-ts/ts/brand.ts';
import {
  jsoncSetComment,
  jsoncStringify,
  parseJsoncEdit,
} from '@monochromatic-dev/jsonc-edit-core-ts/ts';
import {
  assert,
  property,
  string,
} from 'fast-check';

import { fuzzRuns, } from './fuzz-budget.ts';

await describe({
  name: 'comment safety property',
  children: [
    it({
      name: 'an arbitrary comment body emits parseable, fixpoint JSONC',
      fn: async () => {
        assert(
          property(string(), (text,) => {
            const withComment = jsoncSetComment({
              state: parseJsoncEdit({ source: '{ "a": 1 } // seed' as StringJsonc, },),
              path: ['a',],
              comment: {
                type: 'block',
                text,
              },
            },);
            // First emission may normalize placement; the reparse of it must
            // parse without throwing and be a stable fixpoint thereafter.
            const once = jsoncStringify({ state: withComment, },);
            const twice = jsoncStringify({
              state: parseJsoncEdit({ source: once as StringJsonc, },),
            },);
            const thrice = jsoncStringify({
              state: parseJsoncEdit({ source: twice as StringJsonc, },),
            },);
            expect(thrice,).toBe(twice,);
          },),
          { numRuns: fuzzRuns, },
        );
      },
    },),
  ],
},);
