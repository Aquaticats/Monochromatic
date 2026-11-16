/**
 Which reviewer a second start of a review goes to, and whether a Pi run can reach it: the other of the pair the project reviews
 with, read from the model the first run's session names.

 Nothing here starts a run or asks a provider: each case is a pair of names.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  REVIEWING_MODEL,
  REVIEWING_PEER,
  REVIEWING_PROVIDER,
  otherReviewerOf,
  reviewerIsReachableByPi,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: otherReviewerOf.name,
      children: [
        it({
          name: 'sends a first review on this package reviewer to the peer, which is the other of the pair',
          fn: async () => {
            expect(otherReviewerOf({
              provider: REVIEWING_PROVIDER,
              model: REVIEWING_MODEL,
            },),).toEqual(REVIEWING_PEER,);
          },
        },),
        it({
          name: 'reads a session that names no model as one that ran on this package reviewer, and sends its second review to the peer',
          fn: async () => {
            expect(otherReviewerOf({
              provider: '',
              model: '',
            },),).toEqual(REVIEWING_PEER,);
          },
        },),
      ],
    },),
    describe({
      name: reviewerIsReachableByPi.name,
      children: [
        it({
          name: 'says a reviewer with no provider or model cannot be started by a Pi run here, and one with both can',
          fn: async () => {
            expect(reviewerIsReachableByPi(REVIEWING_PEER,),).toBe(false,);
            expect(reviewerIsReachableByPi({
              name: 'Claude Sonnet',
              provider: 'fixture-provider',
              model: 'fixture-model',
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
