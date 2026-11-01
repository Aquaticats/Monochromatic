/**
 Tests for the resolution checker sheet and the verdict guard:
 issue ids bind by sheet number, claim lines render per issue, both
 documents fence verbatim, and only listed verdicts pass the guard.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildResolutionMessages,
  isResolutionReportWire,
  isResolutionVerdict,
  RESOLUTION_VERDICTS,
  usableResolutionReportFor,
} from '../dist/final/node/index.mjs';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Accepted issue whose fix the checkers must confirm.
 */
const NAP_ISSUE: AdjudicatedIssue = {
  issueId: 'issue/whisker',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'issue/whisker',
      claim: {
        category: 'accuracy/mistranslation',
        severity: 'major',
        summary: 'Napping is rendered as hunting.',
        spans: [
          {
            side: 'target',
            nodeId: 'block/1',
            nodeHash: hashContent({ content: 'The cat hunts at noon.', },),
            startOffset: 4,
            endOffset: 9,
            quotedText: 'hunts',
          },
        ],
      },
    },
  ],
  tallies: {},
};

/**
 Original carrying a row of five equals signs, the fence the builder once
 used, on a line of its own.
 */
const RULED_SOURCE = '第一行。\n=====\n第二行。';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: '',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        describe({
          name: buildResolutionMessages.name,
          children: [
            it({
              name: 'binds issue ids in sheet order and renders claim lines',
              fn: async () => {
                const plan = buildResolutionMessages({
                  sourceText: '猫猫在中午打盹。',
                  patchedText: 'The cat naps at noon.',
                  issues: [NAP_ISSUE,],
                },);
                expect(plan.issueIds,).toEqual(['issue/whisker',],);

                /**
                 User sheet carrying documents and issue blocks.
                 */
                const sheet = plan.messages[1]?.content ?? '';
                expect(sheet,).toContain('ISSUE 1',);
                expect(sheet,).toContain(
                  '- (accuracy/mistranslation, major): Napping is rendered as hunting.',
                );
              },
            },),
            it({
              name: 'fences the original beside the revised translation',
              fn: async () => {
                const plan = buildResolutionMessages({
                  sourceText: '猫猫在中午打盹。',
                  patchedText: 'The cat naps at noon.',
                  issues: [NAP_ISSUE,],
                },);

                /**
                 User sheet carrying the fenced pair.
                 */
                const sheet = plan.messages[1]?.content ?? '';
                expect(sheet,).toContain('ORIGINAL',);
                expect(sheet,).toContain('猫猫在中午打盹。',);
                expect(sheet,).toContain('REVISED TRANSLATION',);
                expect(sheet,).toContain('The cat naps at noon.',);
              },
            },),
          ],
        },),
        describe({
          name: isResolutionVerdict.name,
          children: [
            ...RESOLUTION_VERDICTS.map(function toCase(verdict,) {
              return it({
                name: `admits ${verdict}`,
                fn: async () => {
                  expect(isResolutionVerdict(verdict,),).toBe(true,);
                },
              },);
            },),
            it({
              name: 'rejects unlisted strings and non-strings',
              fn: async () => {
                expect(isResolutionVerdict('pounced',),).toBe(false,);
                expect(isResolutionVerdict(1,),).toBe(false,);
              },
            },),
          ],
        },),
        describe({
          name: isResolutionReportWire.name,
          children: [
            it({
              name: 'REFUSES a reply that is no object, a check that is no object, and a check whose issue is a '
                + 'numeric string the whole-number test alone would pass, since nothing there binds a verdict '
                + 'to an issue',
              fn: async () => {
                expect(isResolutionReportWire(5,),).toBe(false,);
                expect(isResolutionReportWire({ checks: [5,], },),).toBe(false,);
                // A string the remainder operator reads as the number one, so
                // only the number check refuses it: "one" would fail the
                // whole-number test as well.
                expect(isResolutionReportWire({
                  checks: [{ issue: '1', verdict: 'fixed', },],
                },),).toBe(false,);
              },
            },),
          ],
        },),
        describe({
          name: usableResolutionReportFor.name,
          children: [
            it({
              name: 'REFUSES a reply that is no wire report at all, so a checker whose reply carries no list of '
                + 'checks is read as unreadable and asked again',
              fn: async () => {
                /**
                 Guard for a sheet that showed two issues.
                 */
                const usable = usableResolutionReportFor({ issueCount: 2, },);

                expect([
                  usable(5,),
                  usable({ checks: 'none', },),
                  usable({ checks: [5,], },),
                ],).toEqual([false, false, false,],);
              },
            },),
            it({
              name: 'ACCEPTS a report with one known verdict on an issue the sheet showed, and REFUSES an empty '
                + 'report, one checking only issues the sheet never showed and one whose only verdict is unknown',
              fn: async () => {
                /**
                 Guard for a sheet that showed two issues.
                 */
                const usable = usableResolutionReportFor({ issueCount: 2, },);

                expect([
                  usable({ checks: [{ issue: 2, verdict: 'fixed', },], },),
                  usable({ checks: [], },),
                  usable({ checks: [{ issue: 3, verdict: 'fixed', },], },),
                  usable({ checks: [{ issue: 0, verdict: 'fixed', },], },),
                  usable({ checks: [{ issue: 1, verdict: 'pounced', },], },),
                ],).toEqual([true, false, false, false, false,],);
              },
            },),
          ],
        },),
      ],
    },),

    describe({
      name: 'fence choice',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FENCES the blocks with a delimiter the enclosed text cannot reproduce, so a passage holding a row '
            + 'of five equals signs cannot close its own block and turn what follows into instructions',
          fn: async () => {
            const content = userText({ messages: buildResolutionMessages({ sourceText: RULED_SOURCE, patchedText: 'Line one.', issues: [], },).messages, },);

            expect(content.includes('====== ORIGINAL ======',),).toBe(true,);
            expect(content.includes('\n===== ',),).toBe(false,);
            expect(content.includes(RULED_SOURCE,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
