/**
 Tests that the critic and the panel judge obligatory differences between
 the languages by one rule (ledger L5).

 WHY. The critic carried a whole block on obligatory differences (a supplied
 subject or object, punctuation conventions, distinctions English cannot
 mark, connections made explicit) while the panel carried one line on
 conjunctions, connectives, pronouns and small words, so the panel upheld an
 addition claim against a possessor English had to state (TianqiChen66616
 slice 20). Rendered sheets showed the panel carried none of the critic's
 block.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import {
  buildAdjudicationMessages,
  buildCriticMessages,
} from '../dist/final/node/index.mjs';

/**
 Every message of one sheet, joined.

 @param messages - one sheet's messages

 @returns Whole sheet text

 @example
 ```ts
 const sheet = sheetOf({ messages, },);
 ```
 */
function sheetOf(
  { messages, }: { readonly messages: readonly Parameters<typeof messageText>[0]['message'][]; },
): string {
  return messages.map(function toText(message,) {
    return messageText({ message, },);
  },)
    .join('\n',);
}

/**
 Rendered critic and panel sheets for one invented pair.
 */
const SHEETS: Readonly<Record<string, string>> = {
  critic: sheetOf({ messages: buildCriticMessages({ sourceText: '猫看着这一面。', targetText: 'The cat watched this side of me.', },), },),
  panel: sheetOf({
    messages: buildAdjudicationMessages({ sourceText: '猫看着这一面。', targetText: 'The cat watched this side of me.', clusters: [], },)
      .messages,
  },),
};

/**
 Wording each rule of the shared block carries.
 */
const RULES = [
  'is REQUIRED, not added',
  'a possessor',
  'Punctuation and quotation conventions differ',
  'cannot be carried over',
  'making it explicit is legitimate',
] as const;

await describe({
  name: 'obligatory differences on the sheets',
  children: Object.entries(SHEETS,).map(function sheetCase([name, sheet,],) {
    return it({
      name: `${name.toUpperCase()} SHEET carries every obligatory-difference rule, a supplied possessor included`,
      fn: async () => {
        for (const rule of RULES)
          expect(sheet,).toContain(rule,);
      },
    },);
  },),
},);
