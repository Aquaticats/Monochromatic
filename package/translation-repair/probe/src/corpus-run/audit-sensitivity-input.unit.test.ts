/**
 Tests for the invented inputs of the audit sensitivity runner: the planted
 defect must sit where the oracle says it does.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  AUDIT_ARMS,
  AUDIT_SOURCE_TEXT,
  ORACLE_CANDIDATE_SPAN,
  ORACLE_SOURCE_SPAN,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'AUDIT_ARMS',
  children: [
    it({
      name: 'RUNS the flipped rendering first and the clean one second, each with what a working instrument should conclude',
      fn: async () => {
        expect(AUDIT_ARMS.map(function brief(arm,) {
          return {
            arm: arm.arm,
            expectation: arm.expectation,
          };
        },),).toEqual([
          {
            arm: 'flipped',
            expectation: 'agreement at either tier on the oracle span',
          },
          {
            arm: 'clean',
            expectation: 'agreement at neither tier',
          },
        ],);
      },
    },),

    it({
      name: 'PLANTS the oracle span in the original and in the flipped rendering only, so the clean rendering holds no span the oracle scores',
      fn: async () => {
        /**
         Renderings of the two arms.
         */
        const flipped = nonNullishOrThrow(AUDIT_ARMS[0],).candidateText;
        const clean = nonNullishOrThrow(AUDIT_ARMS[1],).candidateText;

        expect([
          AUDIT_SOURCE_TEXT.includes(ORACLE_SOURCE_SPAN,),
          flipped.includes(ORACLE_CANDIDATE_SPAN,),
          clean.includes(ORACLE_CANDIDATE_SPAN,),
        ],).toEqual([
          true,
          true,
          false,
        ],);
      },
    },),
  ],
},);
