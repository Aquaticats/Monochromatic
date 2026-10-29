/**
 Tests that the selection ballot gives its reason before its vote (owner,
 2026-10-21, "Reason first everywhere", extending the ruling of 2026-09-27,
 "Reason before vote").

 THE REPLY IS READ BY PROPERTY NAME (`isCandidateBallotAsSent` destructures
 `best` and `assessment`), so the order changes only what a model writes first.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  expectReasonBeforeVoteInText,
  expectReasonFirstInSchema,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reason-order.test-fixture.ts';
import {
  buildCandidateSelectMessages,
  CANDIDATE_SELECT_RESPONSE_FORMAT,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'the selection ballot gives its reason before its vote',
  children: [
    it({
      name: 'DECLARES AND REQUIRES THE REASON FIRST in the schema, before the candidate number',
      fn: async () => {
        expectReasonFirstInSchema({ format: CANDIDATE_SELECT_RESPONSE_FORMAT, reason: 'assessment', vote: 'best', },);
      },
    },),
    it({
      name: 'SHOWS THE REPLY SHAPE WITH THE REASON FIRST in the instruction, before the candidate number',
      fn: async () => {
        /** The system half of one selection sheet, which carries the reply instruction. */
        const system = buildCandidateSelectMessages({
          task: 'Pick one.',
          criteria: ['Faithful.',],
          evidence: [{ label: 'ORIGINAL (Chinese)', text: '猫睡了。', },],
          rendered: ['The cat slept.', 'The cat napped.',],
        },).at(0,)?.content ?? '';
        expectReasonBeforeVoteInText({
          text: system,
          reason: '{"assessment": "..."',
          vote: '"best": 1}',
        },);
      },
    },),
  ],
},);
