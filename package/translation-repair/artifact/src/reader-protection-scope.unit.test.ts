/**
 Guards the owner's ruling of 2026-10-20 on how far reader protection reaches.

 XingZ60's page opens with her own words about doing dangerous things with a
 suicide pill on hand so her enemies could never take her. It is not how she
 died. The XingZ6015 run's lane contest chose a repair that made the pill "a
 risk to my life", two judges citing reader protection, because the rule
 named a method, a substance and a dose with no limit on where on the page
 they appear. The owner chose to scope the rule to how a death or an attempt
 happened, so a means named elsewhere (a person's own words about what they
 might do, a hypothetical, a quoted work, a story) is rendered as the
 ORIGINAL states it.

 Every sheet that restates the rule must carry the scope, or the sheet that
 restates it unscoped reads the rule as reaching the whole page.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { HOUSE_POLICY_BLOCK, } from '@monochromatic-dev/translation-repair-house-rule';
import {
  CITED_REFERENCE_AUDIT_RULE,
  CITED_REFERENCE_CANDIDATE_RULE,
  CITED_REFERENCE_RULE,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { TRANSLATE_ATTESTED_RULE, } from '@monochromatic-dev/translation-repair-translate-floor';
import { renderedSheets, } from './rendered-sheets.test-fixture.ts';

/**
 Words every restatement of the rule uses to name what the rule reaches.
 */
const SCOPE_PHRASE = 'the account of how a death or an attempt happened';

/**
 The sentence in the house rule itself that scopes it and lets a means named
 elsewhere stand.
 */
const SCOPE_SENTENCE = 'This rule reaches only the account of how a death or an attempt happened, whether the '
  + 'subject\'s or anyone\'s that the page tells of. A means named anywhere else (a person\'s own words about what '
  + 'they might do or carry, a hypothetical, a quoted work, a story) is outside this rule and is rendered as the '
  + 'ORIGINAL states it';

/**
 Text of each rendered sheet, by the stage it is written for.
 */
const SHEET_TEXT: ReadonlyMap<string, string> = new Map(renderedSheets()
  .map(function entryOf(sheet,): readonly [string, string] {
    return [sheet.name, sheet.text,];
  },),);

await describe({
  name: 'reader protection reaches the account of how a death or an attempt happened (owner, 2026-10-20)',
  children: [
    it({
      name: 'SAYS IN THE HOUSE RULE that a means named anywhere else is rendered as the ORIGINAL states it',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain(SCOPE_SENTENCE,);
      },
    },),
    it({
      name: 'KEEPS THE REPLICABILITY TEST, the medication allowance and the method words inside that scope',
      fn: async () => {
        expect(HOUSE_POLICY_BLOCK,).toContain('The test is replicability',);
        expect(HOUSE_POLICY_BLOCK,).toContain('Medication named with no substance, no dose and no source',);
        expect(HOUSE_POLICY_BLOCK,).toContain('Method words that tell a reader how (jumped, hanged, cut',);
        expect(HOUSE_POLICY_BLOCK,).toContain('Euthanasia or medically assisted dying',);
        expect(HOUSE_POLICY_BLOCK,).toContain('An immediate medical cause of death',);
      },
    },),
    it({
      name: 'SCOPES EVERY SHEET that says reader protection, so none reads it as reaching the whole page',
      fn: async () => {
        /**
         Sheets that say reader protection in any spelling.
         */
        const saying = renderedSheets()
          .filter(function mentions(sheet,): boolean {
            /**
             Sheet text with the hyphenated spelling folded to the spaced one.
             */
            const folded = sheet.text.toLowerCase()
              .replaceAll(
              'reader-protection',
              'reader protection',
              );
            return folded.includes('reader protection',);
          },);
        expect(saying.length,).toBeGreaterThan(10,);
        expect(saying.filter(function unscoped(sheet,): boolean {
          return !sheet.text.includes(SCOPE_PHRASE,);
        },).map(function nameOf(sheet,): string {
          return sheet.name;
        },),).toEqual([],);
      },
    },),
    it({
      name: 'NEVER LISTS a method, a substance and a dose as staying out without saying of what',
      fn: async () => {
        for (const rule of [
          TRANSLATE_ATTESTED_RULE,
          CITED_REFERENCE_RULE,
          CITED_REFERENCE_CANDIDATE_RULE,
          CITED_REFERENCE_AUDIT_RULE,
        ]) {
          expect(rule,).toContain(`in ${SCOPE_PHRASE}, a method, a substance, a dose, or a place that was the means`,);
          expect(rule,).toContain('stays out however plainly a reference states it',);
        }
      },
    },),
    it({
      name: 'SCOPES THE REFERENCE RULES to say a means named outside that account is not covered',
      fn: async () => {
        for (const rule of [
          CITED_REFERENCE_RULE,
          CITED_REFERENCE_CANDIDATE_RULE,
          CITED_REFERENCE_AUDIT_RULE,
        ])
          expect(rule,).toContain('a means named outside that account is not covered by this rule',);
        expect(TRANSLATE_ATTESTED_RULE,).toContain('a means named outside that account is not covered by this rule',);
      },
    },),
    it({
      name: 'SCOPES THE MEASURING TAIL, so text vaguer than the ORIGINAL about a means elsewhere is an omission',
      fn: async () => {
        /**
         Resolution sheet, which splices the measuring tail.
         */
        const resolution = SHEET_TEXT.get('resolution',) ?? '';
        expect(resolution,).toContain(
          `Reader protection keeps specific detail out of these pages only in ${SCOPE_PHRASE}`,
        );
        expect(resolution,).toContain(
          'about a means named outside that account (a person\'s own words about what they might do, a hypothetical, a '
            + 'quoted work, a story) has left out what the ORIGINAL states, and that is reported like any other omission',
        );
      },
    },),
    it({
      name: 'SCOPES THE JUDGES\' precedence sentence, so a candidate vaguer for a quoted means has left it out',
      fn: async () => {
        expect(SHEET_TEXT.get('lane contest',) ?? '',).toContain(
          `A candidate vaguer than the ORIGINAL because reader protection asks for it in ${SCOPE_PHRASE}`,
        );
      },
    },),
    it({
      name: 'SCOPES THE EDITOR\'S precedence sentence, so a region about a quoted means is restored as the ORIGINAL says',
      fn: async () => {
        expect(SHEET_TEXT.get('editor',) ?? '',).toContain(
          `because reader protection asks for it in ${SCOPE_PHRASE}, the region is not an omission to fill`,
        );
      },
    },),
    it({
      name: 'SCOPES THE RESOLUTION CHECKER\'S not-fixed answer to a detail kept out in that account',
      fn: async () => {
        expect(SHEET_TEXT.get('resolution',) ?? '',).toContain(
          `An issue asking for a detail reader protection keeps out in ${SCOPE_PHRASE} is answered not-fixed`,
        );
      },
    },),
  ],
},);
