/**
 Guards ledger S5: every sheet that files, votes on or keeps page apparatus
 reads one list of its kinds.

 Before the shared list, the critic named citations, credits, identifying
 glosses and translator notes; the panel named the same four; the contest named
 a name, a spelled-out referent, a gloss and a translator note; the writers'
 clause named glosses, notes, asides, referents, credits and a citation's
 translator; the archive block review named a credit and a citation. A kind
 missing from one list read to that sheet as an addition, so an explanatory
 aside the writers kept was a defect to the critic, and a spelled-out referent
 the contest kept was an addition to the panel.

 The critic and panel sheets also licensed accurate added detail without the
 narrative bound the contest and the writers carried, so an invented event
 could pass there as "accurate detail a translator added". The archive block
 review and the introduced-defect probe still listed the kinds without it
 (ledger B28).

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CONTEST_POLICY, } from '@monochromatic-dev/translation-repair-lane-contest';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  buildArchiveBlockReviewMessages,
} from '@monochromatic-dev/translation-repair-archive';
import {
  APPARATUS_KINDS,
  NARRATIVE_DETAIL_IS_NOT_APPARATUS,
  PAGE_APPARATUS_IS_KEPT,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { renderedSheets, } from './rendered-sheets.test-fixture.ts';
import { systemOf, } from '@monochromatic-dev/translation-repair-provider/ts/chat-message-reading.test-fixture.ts';

/**
 Whether a sheet carries a rule as written, or as a JSON state escapes it,
 since the typed decision sends its policy inside a JSON document.

 @param text - sheet text

 @param rule - rule text to find

 @returns Whether the sheet carries the rule
 */
function carries({ text, rule, }: { readonly text: string; readonly rule: string; },): boolean {
  return text.includes(rule,) || text.includes(JSON.stringify(rule,).slice(1, -1,),);
}

/**
 Critic system instructions for a cat-themed pair.
 */
const critic = systemOf({
  messages: buildCriticMessages({
    sourceText: '猫睡了。',
    targetText: 'The cat slept.',
  },),
},);

/**
 Panelist system instructions for a cat-themed pair with no claims.
 */
const panel = systemOf({
  messages: buildAdjudicationMessages({
    sourceText: '猫睡了。',
    targetText: 'The cat slept.',
    clusters: [],
  },).messages,
},);

/**
 Archive block review system instructions for a cat-themed credit block.
 */
const blockReview = systemOf({
  messages: buildArchiveBlockReviewMessages({
    sourceText: '猫睡了。',
    targetText: 'The cat slept.\n\nTranslated by a tabby.',
    blockText: 'Translated by a tabby.',
    priorFindings: [],
  },),
},);

await describe({
  name: 'apparatus kinds are one list (ledger S5)',
  children: [
    it({
      name: 'LISTS the union of every kind any sheet named, so no sheet reads a kept kind as an addition',
      fn: async () => {
        expect(APPARATUS_KINDS,).toContain('a gloss of a name or a term',);
        expect(APPARATUS_KINDS,).toContain('a gloss identifying a person, place or work',);
        expect(APPARATUS_KINDS,).toContain('a translator\'s note explaining a pun',);
        expect(APPARATUS_KINDS,).toContain('an explanatory aside',);
        expect(APPARATUS_KINDS,).toContain('a spelled-out referent',);
        expect(APPARATUS_KINDS,).toContain('a contributor credit',);
        expect(APPARATUS_KINDS,).toContain('ISBN',);
      },
    },),
    it({
      name: 'GIVES the critic the whole list, where an explanatory aside the writers keep was not named',
      fn: async () => {
        expect(critic,).toContain(APPARATUS_KINDS,);
        expect(critic,).toContain('an explanatory aside',);
      },
    },),
    it({
      name: 'GIVES the panel the whole list, where a spelled-out referent the contest keeps was not named',
      fn: async () => {
        expect(panel,).toContain(APPARATUS_KINDS,);
        expect(panel,).toContain('a spelled-out referent',);
      },
    },),
    it({
      name: 'GIVES the contest the whole list, where a citation\'s edition or ISBN was not named',
      fn: async () => {
        expect(CONTEST_POLICY,).toContain(APPARATUS_KINDS,);
        expect(CONTEST_POLICY,).toContain('ISBN',);
      },
    },),
    it({
      name: 'GIVES the writers\' clause the whole list, where an identifying gloss was not named',
      fn: async () => {
        expect(PAGE_APPARATUS_IS_KEPT,).toContain(APPARATUS_KINDS,);
        expect(PAGE_APPARATUS_IS_KEPT,).toContain('a gloss identifying a person',);
      },
    },),
    it({
      name: 'GIVES the archive block review the whole list, where a gloss of a term was not editorial context',
      fn: async () => {
        expect(blockReview,).toContain(APPARATUS_KINDS,);
        expect(blockReview,).toContain('a gloss of a name or a term',);
      },
    },),
    it({
      name: 'GIVES the archive block selector the whole list and the narrative bound, where its first criterion '
        + 'said to remove every claim the original does not state',
      fn: async () => {
        expect(ARCHIVE_BLOCK_SELECTION_CRITERIA[0],).toContain(APPARATUS_KINDS,);
        expect(ARCHIVE_BLOCK_SELECTION_CRITERIA[0],).toContain(NARRATIVE_DETAIL_IS_NOT_APPARATUS,);
      },
    },),
    it({
      name: 'BOUNDS the critic and the panel with the narrative rule, so accurate-looking added events are not licensed',
      fn: async () => {
        expect(critic,).toContain(NARRATIVE_DETAIL_IS_NOT_APPARATUS,);
        expect(panel,).toContain(NARRATIVE_DETAIL_IS_NOT_APPARATUS,);
        expect(critic,).toContain('WHAT HAPPENED IS NEVER APPARATUS',);
        expect(panel,).toContain('WHAT HAPPENED IS NEVER APPARATUS',);
      },
    },),
    it({
      name: 'PAIRS the kinds with the narrative bound on every rendered sheet (ledger B28): a sheet that excuses '
        + 'wording as apparatus is told what apparatus never is, where the archive block review and the '
        + 'introduced-defect probe listed the kinds alone',
      fn: async () => {
        /** Every rendered sheet that lists the apparatus kinds. */
        const withKinds = renderedSheets().filter(function listsKinds(sheet,): boolean {
          return carries({ text: sheet.text, rule: APPARATUS_KINDS, },);
        },);
        /** Names of those sheets. */
        const names = withKinds.map(function nameOf(sheet,): string {
          return sheet.name;
        },);
        // The scan reaches the sheets this case is about before it judges them.
        expect(names,).toContain('critic',);
        expect(names,).toContain('archive block review',);
        expect(names,).toContain('introduced defect',);
        expect(withKinds
          .filter(function lacksBound(sheet,): boolean {
            return !carries({ text: sheet.text, rule: NARRATIVE_DETAIL_IS_NOT_APPARATUS, },);
          },)
          .map(function nameOf(sheet,): string {
            return sheet.name;
          },),).toEqual([],);
      },
    },),
  ],
},);
