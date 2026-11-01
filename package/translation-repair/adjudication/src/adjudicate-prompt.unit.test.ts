/**
 Tests for the panel prompt sheet builder and its index maps.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type { ClaimCluster, } from '@monochromatic-dev/translation-repair-shared-contract';
import { buildAdjudicationMessages, } from '../dist/final/node/index.mjs';
import { userText, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Two clusters: a two-member group then a solo group,
 numbering claims one through three.
 */
const CLUSTERS: readonly ClaimCluster[] = [
  {
    clusterId: 'cluster/nap',
    position: 5,
    members: [
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
              startOffset: 5,
              endOffset: 20,
              quotedText: 'hunts at noon',
            },
          ],
        },
      },
      {
        claimId: 'issue/paw',
        claim: {
          category: 'accuracy/omission',
          severity: 'minor',
          summary: 'The noon detail is dropped.',
          spans: [
            {
              side: 'target',
              nodeId: 'block/1',
              nodeHash: hashContent({ content: 'The cat hunts at noon.', },),
              startOffset: 12,
              endOffset: 12,
              quotedText: '',
            },
          ],
        },
      },
    ],
  },
  {
    clusterId: 'cluster/chase',
    position: 90,
    members: [
      {
        claimId: 'issue/tail',
        claim: {
          category: 'fluency/grammar',
          severity: 'minor',
          summary: 'Verb agreement slips in the chase sentence.',
          spans: [
            {
              side: 'source',
              nodeId: 'block/2',
              nodeHash: hashContent({ content: '猫猫追蝴蝶。', },),
              startOffset: 90,
              endOffset: 96,
              quotedText: '猫猫追蝴蝶。',
            },
          ],
        },
      },
    ],
  },
];

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
      name: buildAdjudicationMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'requires a claim be checked against its own quoted evidence',
          fn: async () => {
            /** Panelist system instructions. */
            const system = buildAdjudicationMessages({
              sourceText: '猫猫在中午打盹。',
              targetText: 'The cat naps at noon.',
              clusters: CLUSTERS,
            },)
              .messages[0]
              ?.content ?? '';
            // A claim alleging an omission its own target quote contains survived
            // adjudication in the graded sample.
            expect(system,).toContain('check the claim against its OWN quoted evidence',);
            expect(system,).toContain('already carries it',);
            expect(system,).toContain('however confidently it is worded',);
          },
        },),
        it({
          name: 'ASKS FOR THE REASON BEFORE THE VOTE, in the instruction and in the reply shape it shows '
            + '(owner, 2026-09-27, "Reason before vote"): a panelist writes out what decides the claim, '
            + 'then votes, and the reason is stored with the ballot',
          fn: async () => {
            /** Panelist system instructions. */
            const system = buildAdjudicationMessages({
              sourceText: '猫猫在中午打盹。',
              targetText: 'The cat naps at noon.',
              clusters: CLUSTERS,
            },)
              .messages[0]
              ?.content ?? '';
            expect(system,).toContain('write its reason first, then cast exactly one vote',);
            /**
             Where the reply shape the sheet shows names each field.
             */
            const shape = system.slice(system.indexOf('{"groups"',),);
            expect(shape.indexOf('"reason"',),).toBeGreaterThan(0,);
            expect(shape.indexOf('"reason"',),).toBeLessThan(shape.indexOf('"vote"',),);
          },
        },),
        it({
          name: 'numbers claims globally and maps ids in prompt order',
          fn: async () => {
            /** Plan for the two-cluster sheet. */
            const plan = buildAdjudicationMessages({
              sourceText: '猫猫在中午打盹，然后追蝴蝶。',
              targetText: 'The cat hunts at noon, then chases butterflies.',
              clusters: CLUSTERS,
            },);
            expect(plan.claimIds,).toEqual([
              'issue/whisker',
              'issue/paw',
              'issue/tail',
            ],);
            expect(plan.clusterIds,).toEqual(['cluster/nap', 'cluster/chase',],);
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            expect(sheet,).toContain('CLAIM 1',);
            expect(sheet,).toContain('CLAIM 2',);
            expect(sheet,).toContain('CLAIM 3',);
            expect(sheet,).toContain('GROUP 1',);
            expect(sheet,).toContain('GROUP 2',);
          },
        },),

        it({
          name: 'asks the same-defect question only for multi-member groups',
          fn: async () => {
            /** Plan for the two-cluster sheet. */
            const plan = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: CLUSTERS,
            },);
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            expect(sheet,).toContain('GROUP 1 (claims below may describe one defect; answer sameDefect)',);
            expect(sheet.includes('GROUP 2 (claims below',),).toBe(false,);
          },
        },),

        it({
          name: 'presents quoted evidence and insertion points distinctly',
          fn: async () => {
            /** Plan for the two-cluster sheet. */
            const plan = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: CLUSTERS,
            },);
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            expect(sheet,).toContain('- evidence (TRANSLATION): hunts at noon',);
            expect(sheet,).toContain('- evidence (TRANSLATION): insertion point, content claimed missing here',);
            expect(sheet,).toContain('- evidence (ORIGINAL): 猫猫追蝴蝶。',);
          },
        },),

        it({
          name: 'CARRIES the cited references after the claims with their rule (class thirty-five), and no block at all when the original links nowhere',
          fn: async () => {
            /** Plan built with one reference line. */
            const plan = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: CLUSTERS,
              referenceContext: '- reference 1 https://cats.example/post: the kitten has an older sister.',
            },);
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            expect(sheet.indexOf('CITED REFERENCES, EVIDENCE ONLY',),).toBeGreaterThan(sheet.indexOf('CLAIMS',),);
            expect(sheet,).toContain('- reference 1 https://cats.example/post: the kitten has an older sister.',);
            expect(sheet,).toContain('never report or support it as accuracy/addition',);
            /** Sheet built without references. */
            const bare = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: CLUSTERS,
            },).messages[1]?.content ?? '';
            expect(bare,).not
              .toContain('CITED REFERENCES',);
          },
        },),

        it({
          name: 'CARRIES THE DECLARED NAMES before the documents and the rule reading them (ledger S4), and no block '
            + 'when the page declares nothing',
          fn: async () => {
            /** Sheet built with one declared name. */
            const plan = buildAdjudicationMessages({
              sourceText: '咪咪在睡觉。',
              targetText: 'Mittens is asleep.',
              clusters: CLUSTERS,
              identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
            },);
            /** System half, where standing rules live. */
            const system = plan.messages[0]?.content ?? '';
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            expect(sheet,).toContain('- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',);
            expect(sheet.indexOf('DECLARED NAMES',),).toBeGreaterThan(-1,);
            expect(sheet.indexOf('DECLARED NAMES',),).toBeLessThan(sheet.indexOf('咪咪在睡觉。',),);
            expect(system,).toContain('Declared identity, when a DECLARED NAMES block precedes the documents:',);
            expect(system,).toContain('Vote unsupported on a claim whose whole case is a rendering the block makes correct.',);
            /** Sheet built without declarations. */
            const bare = buildAdjudicationMessages({
              sourceText: '咪咪在睡觉。',
              targetText: 'Mittens is asleep.',
              clusters: CLUSTERS,
            },).messages[1]?.content ?? '';
            expect(bare,).not
              .toContain('DECLARED NAMES',);
          },
        },),

        it({
          name: 'leaves the nearby original line blank when only the nearby translation is supplied',
          fn: async () => {
            /** Plan with incumbent context but no source-side neighbour. */
            const plan = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: [],
              neighbouringIncumbentText: 'Mittens dozed on the mat.',
            },);
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            /** Nearby-context block, cut right after the incumbent line it ends on. */
            const nearbySection = sheet.slice(
              sheet.indexOf('===== NEARBY ORIGINAL',),
              sheet.indexOf('Mittens dozed on the mat.',) + 'Mittens dozed on the mat.'.length,
            );
            expect(nearbySection,).toBe(
              '===== NEARBY ORIGINAL, CONTEXT ONLY =====\n\n'
                + '===== NEARBY EXISTING TRANSLATION, CONTEXT ONLY =====\nMittens dozed on the mat.',
            );
          },
        },),

        it({
          name: 'leaves the nearby translation line blank when only the nearby original is supplied',
          fn: async () => {
            /** Plan with source-side neighbour context but no incumbent neighbour. */
            const plan = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: [],
              neighbouringSourceText: '猫在窗台上打盹。',
            },);
            /** Sheet text shown to the panelist. */
            const sheet = plan.messages[1]?.content ?? '';
            /** Header the blank incumbent line sits directly after. */
            const incumbentHeader = '===== NEARBY EXISTING TRANSLATION, CONTEXT ONLY =====';
            /** Nearby-context block, cut right after the blank incumbent line and its line break. */
            const nearbySection = sheet.slice(
              sheet.indexOf('===== NEARBY ORIGINAL',),
              sheet.indexOf(incumbentHeader,) + incumbentHeader.length
                + 2,
            );
            expect(nearbySection,).toBe(
              '===== NEARBY ORIGINAL, CONTEXT ONLY =====\n猫在窗台上打盹。\n'
                + '===== NEARBY EXISTING TRANSLATION, CONTEXT ONLY =====\n\n',
            );
          },
        },),

        it({
          name: 'keeps proposer identity out of the sheet',
          fn: async () => {
            /** Plan for the two-cluster sheet. */
            const plan = buildAdjudicationMessages({
              sourceText: '原文',
              targetText: 'translation',
              clusters: CLUSTERS,
            },);
            /** Whole prompt joined for scanning. */
            const wholePrompt = plan
              .messages
              .map(function toContent(message,) {
                return messageText({ message, },);
              },)
              .join('\n',);
            expect(wholePrompt.includes('GLM',),).toBe(false,);
            expect(wholePrompt.includes('issue/',),).toBe(false,);
            expect(wholePrompt.includes('cluster/',),).toBe(false,);
          },
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
            const content = userText({ messages: buildAdjudicationMessages({ sourceText: RULED_SOURCE, targetText: 'Line one.', clusters: [], },).messages, },);

            expect(content.includes('====== ORIGINAL ======',),).toBe(true,);
            expect(content.includes('\n===== ',),).toBe(false,);
            expect(content.includes(RULED_SOURCE,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
