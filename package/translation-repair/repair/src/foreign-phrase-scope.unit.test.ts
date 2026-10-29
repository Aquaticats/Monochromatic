/**
 Guards ledger S20: the critic and editor foreign-phrase rules take names
 and titles out of their scope, so a Japanese title rendered by its English
 title alone is not reported for dropping the original wording. Cat-themed
 invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { buildCriticMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import { buildEditorMessages, } from '../dist/final/node/index.mjs';

/**
 Sentence both sheets must carry, verbatim.
 */
const SCOPE = 'A name or the title of a work is no such phrase: it follows the house name and title rules, where its '
  + 'English form stands and the original wording may follow it in parentheses.';

await describe({
  name: 'foreign-phrase rule scope (ledger S20)',
  children: [
    it({
      name: 'TAKES NAMES AND TITLES OUT of the critic rule, beside the rule it limits',
      fn: async () => {
        /**
         Critic system sheet.
         */
        const system = messageText({
          message: buildCriticMessages({
            sourceText: '猫在读《蜘蛛の糸》。',
            targetText: 'The cat reads *The Spider\'s Thread*.',
          },)[0] ?? {
            role: 'system',
            content: '',
          },
        },);
        expect(system,).toContain(SCOPE,);
        expect(system.indexOf(SCOPE,),).toBeGreaterThan(system.indexOf('policy/foreign-phrase-gloss',),);
      },
    },),
    it({
      name: 'TAKES NAMES AND TITLES OUT of the editor rule, beside the rule it limits',
      fn: async () => {
        /**
         Every editor message, joined.
         */
        const sheet = buildEditorMessages({
          sourceText: '猫在读《蜘蛛の糸》。',
          targetText: 'The cat reads *The Spider\'s Thread*.',
          envelopes: [],
          issues: [],
        },).messages
          .map(function text(message,): string {
            return messageText({ message, },);
          },)
          .join('\n',);
        expect(sheet,).toContain(`Never replace such a phrase with its meaning alone. ${SCOPE}`,);
      },
    },),
  ],
},);
