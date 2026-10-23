/**
 Tests that the contest ballot gives its reason before its vote (owner,
 2026-10-21, "Reason first everywhere", extending the ruling of 2026-09-27,
 "Reason before vote", which covered the issue panel's verdicts).

 THE SCHEMA IS SHARED by the lane contest, the consolidation gate and the
 polish gate, so one builder is pinned here and each gate's sheet is pinned in
 `consolidate`. The reply is read by property name, so the order changes only
 what a model writes first. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  expectReasonBeforeVoteInText,
  expectReasonFirstInSchema,
  propertyOrderOf,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reason-order.test-fixture.ts';
import {
  buildLaneContestMessages,
  contestResponseFormat,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'the contest ballot gives its reason before its vote',
  children: [
    it({
      name: 'DECLARES AND REQUIRES THE REASON FIRST in the schema, before the choice, whether or not the contest '
        + 'asks about the archive, and keeps the archive answer after the choice',
      fn: async () => {
        for (const asksArchive of [true, false,]) {
          /** Response format of one contest. */
          const format = asksArchive
            ? contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', 'neither',],
              asksArchive,
              archiveVerdicts: ['publishable', 'flawed',],
            },)
            : contestResponseFormat({
              schemaName: 'cat_contest',
              verdicts: ['tabby', 'neither',],
              asksArchive,
            },);
          expectReasonFirstInSchema({ format, reason: 'assessment', vote: 'verdict', },);
          /** Declared names. */
          const names = propertyOrderOf({ format, },);
          expect(names.at(-1,),).toBe(asksArchive ? 'verdictOnArchive' : 'verdict',);
          // The keys are declared in the order they sort (ledger B762), the archive answer among them, which the scan
          // of `source-scan` passes over because the contest spreads it into the literal.
          expect(names,).toEqual(
            asksArchive
              ? ['assessment', 'dropped', 'unsupported', 'verdict', 'verdictOnArchive',]
              : ['assessment', 'dropped', 'unsupported', 'verdict',],
          );
        }
      },
    },),
    it({
      name: 'ASKS FOR THE REASON FIRST in the lane contest\'s reply instruction, before the choice',
      fn: async () => {
        /** The user half of one contest sheet. */
        const user = buildLaneContestMessages({
          subject: {
            lineStructured: false,
            sourceText: '猫睡了。',
            incumbentText: 'The cat slept.',
            repairText: 'The cat slept.',
            translateText: 'The cat napped.',
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
