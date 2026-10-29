/**
 Guards ledger H8 and the probe half of S18: the introduced-defect prober
 reads the declared names, the community renderings and the page apparatus
 rule.

 The refinement probe's verdict can roll a whole slice back, and it read no
 declared names, so a refinement using a declared name the ORIGINAL spells
 otherwise could read as an introduced substitution. It read no community
 renderings, so an edit losing the community's word for a term read as a
 neutral rewording. And its rule "Dropping wording the ORIGINAL never had is a
 correct repair" called deleting the page's own apparatus a repair, against
 the clause every writer and judge reads.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildIntroducedDefectMessages, } from '../dist/final/node/index.mjs';

/**
 One edit that loses the community word for 自切.
 */
const regions = [{
  envelopeId: 'paragraph/0',
  issueIds: [],
  before: 'Mittens told of her self-surgery.',
  editorAfter: 'Mittens told of her operation.',
},];

/**
 Both halves of a probe sheet.

 @param identityContext - declared names, absent for a page with none

 @returns System text and user text
 */
function sheetFor({ identityContext, }: { readonly identityContext?: string; },): {
  readonly system: string;
  readonly user: string;
} {
  /**
   Messages one prober is sent.
   */
  const { messages, } = buildIntroducedDefectMessages({
    sourceText: '猫猫讲起自切的经历。',
    baselineText: 'Mittens told of her self-surgery.',
    regions,
    issues: [],
    ...((identityContext === undefined) ? {} : { identityContext, }),
  },);
  return {
    system: messages.at(0,)?.content ?? '',
    user: messages.at(1,)?.content ?? '',
  };
}

await describe({
  name: 'the introduced-defect probe reads the page rules (ledger H8, S18)',
  children: [
    it({
      name: 'SHOWS the declared names with their rules, and only where the page declares any',
      fn: async () => {
        /**
         Sheet for a page declaring Mittens.
         */
        const declared = sheetFor({ identityContext: '- name: ORIGINAL declares "猫猫", TRANSLATION declares "Mittens"', },);
        expect(declared.user,).toContain('TRANSLATION declares "Mittens"',);
        expect(declared.system,).toContain('Declared identity, when a DECLARED NAMES block precedes the documents',);
        expect(sheetFor({},).system.includes('Declared identity, when a DECLARED NAMES block',),).toBe(false,);
      },
    },),
    it({
      name: 'NAMES the AFTER text that lost the community word the BEFORE text carried',
      fn: async () => {
        const { user, } = sheetFor({},);
        expect(user,).toContain('COMMUNITY RENDERINGS, evidence to weigh, not a verdict:',);
        expect(user,).toContain('- AFTER 1 carries none of the community\'s renderings of 自切',);
        expect(user.includes('BEFORE 1 carries none',),).toBe(false,);
      },
    },),
    it({
      name: 'NEVER CALLS dropping the page\'s own apparatus a correct repair',
      fn: async () => {
        const { system, } = sheetFor({},);
        expect(system,).toContain('unless it is page apparatus',);
        expect(system,).toContain('a gloss of a name or a term',);
      },
    },),
  ],
},);
