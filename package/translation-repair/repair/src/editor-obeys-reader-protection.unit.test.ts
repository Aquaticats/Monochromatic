/**
 Tests that the stage which REWRITES memorial text is told what this corpus is
 written under.

 THE FAILURE THIS GUARDS is the one `house-policy.ts` was written for, stated
 in its own header: a page that keeps a suicide method vague is obeying the
 corpus's rule, a critic ignorant of that rule files it as an omission, and
 the editor RESTORES the detail the rule exists to remove. The critic was
 given the block. The editor was not, while five rules in its own sheet told
 it that every detail of the original must survive and that an omission must
 be translated in full sentences with nothing dropped.

 BOTH PATHS ARE COVERED. A calibration addendum builds a different system
 prompt, and a policy that reached only one of the two would be absent from
 every call in a run that uses an addendum.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildEditorMessages, } from '../dist/final/node/index.mjs';

/**
 One region the editor is asked to fix, standing in for a real envelope.
 */
const ENVELOPE = {
  envelopeId: 'cat-region-1',
  startOffset: 8,
  endOffset: 12,
  baseText: 'naps',
  baseHash: 'not-checked-by-the-prompt-builder',
  issueIds: [],
};

/**
 Builds the system half for one editor call.
 */
function systemFor({ addendum, }: { readonly addendum?: string; },): string {
  return buildEditorMessages({
    sourceText: '猫在窗台上睡觉。',
    targetText: 'The cat naps on the sill.',
    envelopes: [ENVELOPE,],
    issues: [],
    ...((addendum === undefined) ? {} : { editorRuleAddendum: addendum, }),
  },)
    .messages
    .filter(function isSystem(message,): boolean {
      return message.role === 'system';
    },)
    .map(function toContent(message,): string {
      return message.content;
    },)
    .join('\n',);
}

await describe({
  name: 'editor obeys reader protection',
  children: [
    it({
      name: 'TELLS the editor that a protective omission is not an omission',
      fn: async () => {
        expect(systemFor({},).includes('Reader protection outranks completeness',),).toBe(true,);
      },
    },),
    it({
      name: 'RANKS the house rules above its own restore-in-full rules',
      fn: async () => {
        // The sheet asks five separate times for every detail to survive. A
        // block sitting beside those rules without precedence loses to them.
        expect(systemFor({},).includes('OUTRANK EVERY RULE IN THIS LIST',),).toBe(true,);
        expect(systemFor({},).includes('omit that region entirely rather than restoring the detail',),)
          .toBe(true,);
      },
    },),
    it({
      name: 'KEEPS the rules on the calibration-addendum path too',
      fn: async () => {
        const withAddendum = systemFor({ addendum: 'Keep verse lines intact.', },);
        expect(withAddendum.includes('Reader protection outranks completeness',),).toBe(true,);
        expect(withAddendum.includes('Keep verse lines intact.',),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS the reply shape last on both paths',
      fn: async () => {
        // Wire instructions that end up above content rules are the ones models
        // drop first, so the reply block closes the sheet.
        for (const system of [systemFor({},), systemFor({ addendum: 'Keep verse lines intact.', },),]) {
          const reply = system.indexOf('Reply with ONLY a JSON object',);
          const policy = system.indexOf('Reader protection outranks completeness',);
          // BOTH BOUNDS, because an absent policy is index -1 and any reply
          // index beats it: the ordering assertion would pass on a sheet that
          // carries no policy at all.
          expect(policy,).toBeGreaterThan(-1,);
          expect(reply,).toBeGreaterThan(policy,);
        }
      },
    },),
  ],
},);
