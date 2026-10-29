/**
 Verifies measured quoted-form scope reaches writers, critics and selectors.
 Live evidence chooses the remedy; these checks only lock down its integration.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import {
  buildTranslateMessages,
  translateSelectionCriteria,
} from '@monochromatic-dev/translation-repair-translate-floor';
import { buildLaneContestMessages, } from '@monochromatic-dev/translation-repair-lane-contest';
import { buildEditorMessages, } from '@monochromatic-dev/translation-repair-repair';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '@monochromatic-dev/translation-repair-adjudication';
import { buildConsolidateMessages, } from '../dist/final/node/index.mjs';

/** Invented character/name contrast, not a quotation to copy untranslated. */
const SOURCE = '> 喵？不是，我叫 Miu。';
/** Archive keeps the spelling being discussed rather than normalizing it. */
const ARCHIVE = '> “喵”? No, I go by Miu.';
/** Declared identity applies to references, not the denied character form. */
const IDENTITY = '- name: ORIGINAL declares "Miu", TRANSLATION declares "Miu"';
/** Shared rule's specific scope, rather than a general fidelity promise. */
const SCOPE = 'retain the form under discussion';

/**
 Reads system instructions without confusing them with source examples.

 @param messages - production builder output

 @returns Instruction text only
 */
function systemText(messages: ReturnType<typeof buildCriticMessages>,): string {
  return messages.filter(function system(message,) {
    return message.role === 'system';
  },).map(function text(message,) {
    return messageText({ message, },);
  },).join('\n',);
}

/** Real sheets for independently owned writing and judging paths. */
const sheets = [
  {
    name: 'initial translator',
    text: systemText(buildTranslateMessages({ sourceText: SOURCE, existingText: ARCHIVE, identityContext: IDENTITY, },).messages,),
  },
  {
    name: 'repair editor',
    text: systemText(buildEditorMessages({ sourceText: SOURCE, targetText: ARCHIVE, envelopes: [], issues: [], },).messages,),
  },
  {
    name: 'consolidation writer',
    text: systemText(buildConsolidateMessages({ subject: { sourceText: SOURCE, incumbentText: ARCHIVE, repairText: ARCHIVE, translateText: ARCHIVE, ballots: [], lineStructured: false, identityContext: IDENTITY, }, },),),
  },
  {
    name: 'critic',
    text: systemText(buildCriticMessages({ sourceText: SOURCE, targetText: ARCHIVE, identityContext: IDENTITY, },),),
  },
  {
    name: 'lane contest',
    text: systemText(buildLaneContestMessages({ subject: { lineStructured: false, sourceText: SOURCE, incumbentText: ARCHIVE, repairText: ARCHIVE, translateText: ARCHIVE, identityContext: IDENTITY, }, },),),
  },
];

await describe({
  name: 'name reference and mentioned-form policy',
  children: [
    it({
      name: 'KEEPS established names outside the first-use generic-term gloss requirement',
      fn: async () => {
        const adjudication = systemText(buildAdjudicationMessages({
          sourceText: '猫猫加入了猫友会。',
          targetText: 'Cat joined Whisker Friends.',
          clusters: [],
        },).messages,);
        expect(adjudication,).toContain('an established translation-side name is not a generic community term',);
        expect(adjudication,).toContain('This does not exempt ordinary terminology or an explicit source definition',);
        expect(adjudication,).toContain('A generic term with no English equivalent',);
      },
    },),
    ...sheets.map(function scopeReached(sheet,) {
      return it({
        name: `SCOPES name normalization in ${sheet.name}`,
        fn: async () => {
          expect(sheet.text,).toContain(SCOPE,);
          expect(sheet.text,).toContain('does not license leaving an entire source-language quotation untranslated',);
        },
      },);
    },),
    it({
      name: 'QUALIFIES the actual faithfulness and name criteria rather than appending a competing rule',
      fn: async () => {
        const criteria = translateSelectionCriteria({ lineStructured: false, },);
        expect(criteria[1],).toContain('TO REFER TO a person',);
        expect(criteria[1],).toContain('forms being questioned, contrasted, or denied',);
        expect(criteria[2],).toContain(SCOPE,);
        expect(criteria[2],).toContain('A declared identity is not extra content',);
      },
    },),
    it({
      name: 'LEAVES dedicated metadata criteria outside the prose-name exception',
      fn: async () => {
        const criteria = translateSelectionCriteria({ lineStructured: false, syntax: 'front-matter', },);
        expect(criteria.join('\n',),).not.toContain(SCOPE,);
        expect(criteria.join('\n',),).toContain('Source authority',);
      },
    },),
  ],
},);
