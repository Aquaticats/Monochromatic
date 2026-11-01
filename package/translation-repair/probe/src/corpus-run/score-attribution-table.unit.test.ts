/**
 Tests for the critic table `score-attribution` prints.

 The table reads rates over the chunks a critic HEARD, and refuses a row for a critic
 heard on none, since the report lists none.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  attributionCriticLine,
  attributionHeaderLine,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'score-attribution-table',
  children: [
    it({
      name: 'RENDERS the header with every column right-aligned over its figures',
      fn: async () => {
        expect(attributionHeaderLine(),).toBe(
          'CRITIC                                      heard  raised  emitted   hits  raised/ch   hits/ch',
        );
      },
    },),

    it({
      name: 'RENDERS a critic heard on several chunks with both rates over its chunks heard',
      fn: async () => {
        expect(attributionCriticLine({
          critic: {
            modelId: 'hf:cat/Tabby-1',
            chunksHeard: 4,
            claimsRaised: 6,
            emissions: 7,
            acceptedHits: 2,
          },
        },),).toBe(
          'hf:cat/Tabby-1                                  4       6        7      2       1.50      0.50',
        );
      },
    },),

    it({
      name: 'RENDERS a critic that raised and hit nothing with both rates at zero',
      fn: async () => {
        expect(attributionCriticLine({
          critic: {
            modelId: 'hf:cat/Mouser-1',
            chunksHeard: 3,
            claimsRaised: 0,
            emissions: 0,
            acceptedHits: 0,
          },
        },),).toBe(
          'hf:cat/Mouser-1                                 3       0        0      0       0.00      0.00',
        );
      },
    },),

    it({
      name: 'REFUSES a critic heard on no chunk, since no row of the report is one',
      fn: async () => {
        /**
         What rendering the row raised.
         */
        const refusal = caught(function rendersUnheardCritic(): unknown {
          return attributionCriticLine({
            critic: {
              modelId: 'hf:cat/Biscuit-1',
              chunksHeard: 0,
              claimsRaised: 0,
              emissions: 0,
              acceptedHits: 0,
            },
          },);
        },);

        expect(refusal,).toBeInstanceOf(Error,);
        expect(String(refusal,),).toBe(
          'Error: unreachable: hf:cat/Biscuit-1 has no chunk heard, yet a row exists only for a heard critic',
        );
      },
    },),
  ],
},);
