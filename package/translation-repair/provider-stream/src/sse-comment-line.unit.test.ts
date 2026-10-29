/**
 Tests the reading of a streamed body's comment lines, which the data-line
 reader skips by design: NeuralWatt sends a request's measured energy and its
 cost as `: energy {...}` and `: cost {...}` comments before `data: [DONE]`
 (`https://docs.neuralwatt.com/energy-methodology.md`, fetched 2026-10-20).

 Fixtures are the documented sample lines and cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ssePayloadOf,
  sseCommentOf,
} from '../dist/final/node/index.mjs';

/**
 The energy comment exactly as the methodology page prints it.
 */
const ENERGY_LINE =
  ': energy {"energy_kwh": 0.000015, "energy_joules": 54.0, "attribution_method": "counter_exact_token_pool_weighted", "attribution_ratio": 0.25}';

/**
 The cost comment exactly as the methodology page prints it.
 */
const COST_LINE = ': cost {"request_cost_usd": 0.00001, "cache_savings_usd": 0.0, "allowance_remaining_usd": 32.68}';

await describe({
  name: sseCommentOf.name,
  children: [
    it({
      name: 'READS THE TEXT AFTER A NAMED COMMENT, the documented energy and cost lines among them',
      fn: async () => {
        expect(sseCommentOf({ line: ENERGY_LINE, name: 'energy', },),).toBe(
          '{"energy_kwh": 0.000015, "energy_joules": 54.0, "attribution_method": '
            + '"counter_exact_token_pool_weighted", "attribution_ratio": 0.25}',
        );
        expect(sseCommentOf({ line: COST_LINE, name: 'cost', },),).toBe(
          '{"request_cost_usd": 0.00001, "cache_savings_usd": 0.0, "allowance_remaining_usd": 32.68}',
        );
      },
    },),

    it({
      name: 'READS THE TIGHT FORM AND A CARRIAGE RETURN, since the format drops one optional space and a paired return',
      fn: async () => {
        expect(sseCommentOf({ line: ':energy {"energy_kwh":1}', name: 'energy', },),).toBe('{"energy_kwh":1}',);
        expect(sseCommentOf({ line: ': energy {"energy_kwh":1}\r', name: 'energy', },),).toBe('{"energy_kwh":1}',);
      },
    },),

    it({
      name: 'READS NOTHING FROM A COMMENT OF ANOTHER NAME, including one that only begins with the name',
      fn: async () => {
        expect(sseCommentOf({ line: COST_LINE, name: 'energy', },),).toBe('',);
        expect(sseCommentOf({ line: ': energyx {"energy_kwh":1}', name: 'energy', },),).toBe('',);
        expect(sseCommentOf({ line: ': energy', name: 'energy', },),).toBe('',);
        expect(sseCommentOf({ line: ': keep-alive', name: 'energy', },),).toBe('',);
      },
    },),

    it({
      name: 'READS NOTHING FROM A LINE THAT IS NOT A COMMENT, a data line quoting the comment included',
      fn: async () => {
        expect(sseCommentOf({
          line: String.raw`data: {"choices":[{"delta":{"content":": energy {\"energy_kwh\":9}"}}]}`,
          name: 'energy',
        },),).toBe('',);
        expect(sseCommentOf({ line: `  ${ENERGY_LINE}`, name: 'energy', },),).toBe('',);
        expect(sseCommentOf({ line: 'data: : energy {"energy_kwh":9}', name: 'energy', },),).toBe('',);
        expect(sseCommentOf({ line: '', name: 'energy', },),).toBe('',);
      },
    },),

    it({
      name: 'LEAVES THE DATA-LINE READER BLIND TO COMMENTS, so a comment never reaches the answer',
      fn: async () => {
        expect(ssePayloadOf({ line: ENERGY_LINE, },),).toBe('',);
        expect(ssePayloadOf({ line: COST_LINE, },),).toBe('',);
      },
    },),
  ],
},);
