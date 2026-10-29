/**
 Guards class one hundred forty-three (hulicaijia26, 2026-09-26). The
 archive's footnote 7 carried the translator's note explaining the wordplay
 between 晚安 (goodnight) and 金刚烷胺 (amantadine), which the English cannot
 carry by itself. deepseek-v4.1-flash filed it as accuracy/addition, the panel
 accepted it (minimax-m3 and gemma-4-26b-a4b-it for, Mercury 2.5 against), the
 acceptance disputed the archive under class one hundred seven, and four of
 five contest judges called the note "an addition the ORIGINAL never states".
 The critic and the panel sheets named citations, credits and identifying
 glosses as accurate translator detail, but no sheet named a note explaining
 wordplay, and the page-apparatus clause the writers and the contest read was
 on neither sheet that files and accepts the claim.

 The same clause said "only what the ORIGINAL contradicts is an addition",
 which is the wording a class one hundred eight gate ballot used to keep an
 invented overdose detail ("which the Chinese does not contradict"). The
 apparatus is what explains the text; what happened is never apparatus.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { PAGE_APPARATUS_IS_KEPT, } from '@monochromatic-dev/translation-repair-shared-contract';
import { CONTEST_POLICY, } from '@monochromatic-dev/translation-repair-lane-contest';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import { archiveDisputeNote, } from '../dist/final/node/index.mjs';

/**
 Wording naming a translator's note that explains wordplay as apparatus.
 */
const WORDPLAY_NOTE = 'a translator\'s note explaining a pun, wordplay, an allusion or a term';

/**
 Wording saying narrative detail is an addition however uncontradicted.
 */
const NARRATIVE_DETAIL = 'WHAT HAPPENED IS NEVER APPARATUS';

/**
 Dispute note for a slice whose one accepted claim names a translator's note
 on a pun, as hulicaijia27 chunk 69's did.
 */
const noteDispute = archiveDisputeNote({
  dispute: {
    sliceIndex: 9,
    standIn: '[^3]: That is, the fish mentioned earlier.',
    standInEligible: true,
    acceptedClaims: [
      'accuracy/addition major: The translation of footnote 3 adds the pun explanation from the ARCHIVE note, '
        + 'whereas the ORIGINAL footnote only says that it refers to the fish mentioned earlier.',
    ],
  },
},);

/**
 Critic system instructions for a cat-themed pair.
 */
const critic = buildCriticMessages({
  sourceText: '猫说晚安。',
  targetText: 'The cat said goodnight.',
},)
  .filter(function isSystem(message,): boolean {
    return message.role === 'system';
  },)
  .map(function toContent(message,): string {
    return message.content;
  },)
  .join('\n',);

/**
 Panelist system instructions for a cat-themed pair with no claims.
 */
const panel = buildAdjudicationMessages({
  sourceText: '猫说晚安。',
  targetText: 'The cat said goodnight.',
  clusters: [],
},)
  .messages[0]
  ?.content ?? '';

await describe({
  name: 'translator note is apparatus (class one hundred forty-three)',
  children: [
    it({
      name: 'NAMES a translator note explaining wordplay as apparatus on the critic sheet, which filed it as an addition',
      fn: async () => {
        expect(critic,).toContain(WORDPLAY_NOTE,);
      },
    },),
    it({
      name: 'NAMES it on the adjudication panel sheet, which accepted the claim 2 to 1',
      fn: async () => {
        expect(panel,).toContain(WORDPLAY_NOTE,);
      },
    },),
    it({
      name: 'NAMES it in the contest policy, where four of five ballots called the note an addition',
      fn: async () => {
        expect(CONTEST_POLICY,).toContain(WORDPLAY_NOTE,);
      },
    },),
    it({
      name: 'NAMES it in the page-apparatus clause every writer and slate judge reads',
      fn: async () => {
        expect(PAGE_APPARATUS_IS_KEPT,).toContain(WORDPLAY_NOTE,);
      },
    },),
    it({
      name: 'SAYS what happened is never apparatus wherever an uncontradicted detail was exempt (class one hundred '
        + 'eight\'s gate kept an invented overdose detail "which the Chinese does not contradict")',
      fn: async () => {
        expect(PAGE_APPARATUS_IS_KEPT,).toContain(NARRATIVE_DETAIL,);
        expect(CONTEST_POLICY,).toContain(NARRATIVE_DETAIL,);
        expect(PAGE_APPARATUS_IS_KEPT.includes('Only what the ORIGINAL contradicts is an addition.',),).toBe(false,);
      },
    },),
    it({
      name: 'LEAVES a claimed translator note to the apparatus rule on the dispute note every downstream sheet reads '
        + '(hulicaijia27: the panel accepted the note as an addition 3 to 2, and the dispute note told the translate '
        + 'slate that any detail the claim names is not the page\'s apparatus, so all four judges voted it out)',
      fn: async () => {
        expect(noteDispute,).toContain(WORDPLAY_NOTE,);
        expect(noteDispute,).toContain(NARRATIVE_DETAIL,);
        expect(noteDispute.includes('A detail those claims name is not page content',),).toBe(false,);
      },
    },),
  ],
},);
