/**
 Tests that the consolidation gate and the polish gate ask for their reason
 before their vote (owner, 2026-10-21, "Reason first everywhere").

 Both gates answer on the contest ballot's schema, pinned in `lane-contest`;
 what is pinned here is each gate's own reply instruction. Fixtures are
 cat-themed invention.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { expectReasonBeforeVoteInText, } from '@monochromatic-dev/translation-repair-test-fixture/ts/reason-order.test-fixture.ts';
import {
  buildConsolidateGateMessages,
  buildConsolidationPolishGateMessages,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'the gates ask for their reason before their vote',
  children: [
    it({
      name: 'ASKS THE CONSOLIDATION GATE for the reason first, before the verdict',
      fn: async () => {
        /** The user half of one gate sheet. */
        const user = buildConsolidateGateMessages({
          subject: {
            lineStructured: false,
            sourceText: '猫在窗台上睡觉。',
            incumbentText: 'The cat sleeps on the sill, purring.',
            consolidatedText: 'The cat sleeps on the window sill.',
            standingText: 'The cat is napping on the ledge.',
          },
        },).at(1,)?.content ?? '';
        expectReasonBeforeVoteInText({
          text: user.slice(user.indexOf('Return JSON',),),
          reason: 'assessment one sentence',
          vote: 'verdict one of',
        },);
      },
    },),
    it({
      name: 'ASKS THE POLISH GATE for the reason first, before the verdict',
      fn: async () => {
        /** The user half of one gate sheet. */
        const user = buildConsolidationPolishGateMessages({
          subject: {
            sourceText: '猫在窗边睡了一下午。',
            archiveText: 'The cat slept by the window all afternoon.',
            baseText: 'The cat slept by the window the whole afternoon.',
            polishedText: 'The cat napped by the window all afternoon.',
            mode: { kind: 'comparative', },
            lineStructured: false,
          },
        },).at(1,)?.content ?? '';
        expectReasonBeforeVoteInText({
          text: user.slice(user.indexOf('Return JSON',),),
          reason: 'assessment one sentence',
          vote: 'verdict one of',
        },);
      },
    },),
  ],
},);
