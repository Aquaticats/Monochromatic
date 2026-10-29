/**
 Guards ledger H4, class one hundred eight's open half: the repair lane's own
 editor and checker are told an accepted addition is removed, not softened.

 The dispute note told every downstream sheet that a detail an accepted
 addition names is not page content "in the archive's wording or any softer
 one", but the repair lane's editor and checker never read it. So the editor
 repaired an accepted invented event by restating it more mildly, the checker
 called that fixed, and the softened detail reached the contest as the repair
 lane's candidate.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildResolutionMessages, } from '@monochromatic-dev/translation-repair-adjudication';
import { buildEditorMessages, } from '../dist/final/node/index.mjs';

/**
 Wording both sheets carry.
 */
const REMOVED_NOT_SOFTENED = 'restating that detail in vaguer or softer words keeps the addition';

await describe({
  name: 'an accepted addition is removed, not softened (ledger H4)',
  children: [
    it({
      name: 'TELLS the editor',
      fn: async () => {
        /**
         Editor system text.
         */
        const system = buildEditorMessages({
          sourceText: '猫睡了。',
          targetText: 'The cat slept after eating three fish.',
          envelopes: [],
          issues: [],
        },).messages.at(0,)?.content ?? '';
        expect(system,).toContain(REMOVED_NOT_SOFTENED,);
      },
    },),
    it({
      name: 'TELLS the checker, with its verdict',
      fn: async () => {
        /**
         Checker system text.
         */
        const system = buildResolutionMessages({
          sourceText: '猫睡了。',
          patchedText: 'The cat slept after a snack.',
          issues: [],
        },).messages.at(0,)?.content ?? '';
        expect(system,).toContain(REMOVED_NOT_SOFTENED,);
        expect(system,).toContain('such a restatement is not-fixed',);
      },
    },),
  ],
},);
