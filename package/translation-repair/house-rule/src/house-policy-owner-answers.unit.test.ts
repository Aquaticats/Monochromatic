/**
 Guards the owner's answers of 2026-09-27 to the whole-package audit on two
 house rules every sheet reads.

 MEDICATION: the rule called "took medication" a method kept vague while
 class one hundred twenty-two let the page say that she had taken
 medication. The owner answered that "took medication" is fine because it
 is not replicable, and replicability is the test
 (`doc/decision/translation-repair-reader-protection-cause-of-death.md`).

 TITLES: the rule set a title in quotation marks while most archives set a
 work's title in italics and a page pass restores them. The owner chose
 italics for works (`doc/decision/translation-repair-title-style.md`).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { HOUSE_POLICY_BLOCK, } from '../dist/final/node/index.mjs';

/**
 Clause letting medication with nothing to copy stay, read verbatim so a
 rewording that drops the allowance fails here.
 */
const MEDICATION_MAY_STAY = 'Medication named with no substance, no dose and no source ("took medication", '
  + '"had taken medication", "overdosed") tells a reader nothing to copy and may stay as the ORIGINAL states it.';

/**
 Clause setting a work's title in italics and a short piece in quotation
 marks.
 */
const TITLE_STYLE = 'In English prose the title of a work (a book, a game, a film, a show, an album) stands in '
  + 'italics (*To Live*), and the title of a song, a poem, an episode or a chapter in quotation marks '
  + '("Zero Hour"); never in the Chinese title marks 《》.';

await describe({
  name: 'house rules as the owner answered on 2026-09-27',
  children: [
    it({
      name: 'LETS MEDICATION WITH NOTHING TO COPY STAY, and no longer lists "took medication" among the methods',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(MEDICATION_MAY_STAY,);
        expect(HOUSE_POLICY_BLOCK,).not.toContain('(took medication, breathed in gas',);
        expect(HOUSE_POLICY_BLOCK,).toContain('Method words that tell a reader how',);
      },
    },),
    it({
      name: 'SETS A WORK\'S TITLE IN ITALICS and a short piece in quotation marks',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(TITLE_STYLE,);
        expect(HOUSE_POLICY_BLOCK,).not.toContain('In English prose a title stands in quotation marks',);
      },
    },),
  ],
},);
