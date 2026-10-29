/**
 Tests that the archive judges give their reason before their verdict (owner,
 2026-10-21, "Reason first everywhere", extending the ruling of 2026-09-27,
 "Reason before vote"): the absolute naturalness review, the coverage judge
 and the archive block review.

 EACH REPLY IS READ BY PROPERTY NAME, so the order changes only what a model
 writes first. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import {
  expectReasonBeforeVoteInText,
  expectReasonFirstInSchema,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/reason-order.test-fixture.ts';
import {
  ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
  ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,
  buildAbsoluteNaturalnessReviewMessages,
  buildArchiveBlockReviewMessages,
  buildCoverageMessages,
  COVERAGE_RESPONSE_FORMAT,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'the archive judges give their reason before their verdict',
  children: [
    it({
      name: 'ASKS THE NATURALNESS REVIEW for the reason first, in the schema and the instruction, before whether '
        + 'the candidate is acceptable',
      fn: async () => {
        expectReasonFirstInSchema({
          format: ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
          reason: 'assessment',
          vote: 'isAcceptable',
        },);
        /** The user half of one review sheet. */
        const user = messageText({
          message: buildAbsoluteNaturalnessReviewMessages({
            subject: {
              sourceText: '猫睡了。',
              candidateText: 'The cat slept.',
              paragraphs: ['The cat slept.',],
              lineStructured: false,
            },
          },).at(1,) ?? { role: 'user', content: '', },
        },);
        expectReasonBeforeVoteInText({
          text: user.slice(user.indexOf('Return only JSON',),),
          reason: 'assessment one sentence',
          vote: 'isAcceptable boolean',
        },);
      },
    },),
    it({
      name: 'ASKS THE COVERAGE JUDGE for the reason first, in the schema and the instruction, before the coverage '
        + 'verdict',
      fn: async () => {
        expectReasonFirstInSchema({ format: COVERAGE_RESPONSE_FORMAT, reason: 'assessment', vote: 'coverage', },);
        /** Every message of one coverage question, joined. */
        const sheet = buildCoverageMessages({
          sourcePassage: '猫睡了。',
          translationText: 'The cat slept.',
        },)
          .messages
          .map(function content(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',);
        expectReasonBeforeVoteInText({
          text: sheet.slice(sheet.indexOf('Reply with JSON only',),),
          reason: '"assessment":',
          vote: '"coverage":',
        },);
      },
    },),
    it({
      name: 'ASKS THE ARCHIVE BLOCK REVIEW for its assessment, the reason it gives, first in the schema and the '
        + 'instruction, before the disposition',
      fn: async () => {
        expectReasonFirstInSchema({
          format: ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,
          reason: 'assessment',
          vote: 'disposition',
        },);
        /** Every message of one review, joined. */
        const sheet = buildArchiveBlockReviewMessages({
          sourceText: '猫在窗台上打盹。',
          targetText: '**Come back later!**\n\nThe cat naps on the sill.',
          blockText: '**Come back later!**',
          priorFindings: [],
        },)
          .map(function content(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',);
        expectReasonBeforeVoteInText({
          text: sheet.slice(sheet.indexOf('Reply with JSON only',),),
          reason: '"assessment":',
          vote: '"disposition":',
        },);
      },
    },),
  ],
},);
