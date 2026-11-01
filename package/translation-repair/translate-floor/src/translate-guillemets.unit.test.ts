/**
 Guards the guillemet floor (ledger B24): English prose on these pages sets
 quotations in quotation marks, and guillemets are never its marks. The
 TianqiChen66611 run shipped one translated slice with «» around a
 quotation although one judge's reason named them, so a candidate whose
 prose carries «, », ‹ or › is refused before any judge reads it, unless the
 original or the page it would replace carries one.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  guillemetFindings,
  validateTranslatedSlice,
} from '../dist/final/node/index.mjs';

/**
 Original with a quotation in corner brackets.
 */
const SOURCE = '猫说：「我饿了。」';

/**
 Findings a validation refuses a candidate with, empty when it passes.

 @param candidateText - rendering under the floors

 @param sourceText - original slice

 @param pageText - page slice the rendering would replace

 @returns Findings joined, or empty

 @example
 ```ts
 findingsOf({ candidateText: 'The cat said, «I am hungry.»', },);
 ```
 */
function findingsOf(
  {
    candidateText,
    sourceText = SOURCE,
    pageText = '',
  }: {
    readonly candidateText: string;
    readonly sourceText?: string;
    readonly pageText?: string;
  },
): string {
  /**
   Verdict of the whole chain.
   */
  const result = validateTranslatedSlice({
    sourceText,
    candidateText,
    pageText,
  },);
  return (result.kind === 'invalid') ? result.findings.join(' ',) : '';
}

await describe({
  name: 'guillemets in an English rendering (ledger B24)',
  children: [
    it({
      name: 'REFUSES a quotation set in guillemets, double or single, and says to write quotation marks',
      fn: async () => {
        expect(findingsOf({ candidateText: 'The cat said, «I am hungry.»', },),).toContain('guillemets',);
        expect(findingsOf({ candidateText: 'The cat said, ‹I am hungry.›', },),).toContain('guillemets',);
      },
    },),
    it({
      name: 'ACCEPTS the quotation in quotation marks',
      fn: async () => {
        expect(findingsOf({ candidateText: 'The cat said, “I am hungry.”', },),).toBe('',);
      },
    },),
    it({
      name: 'ACCEPTS guillemets the original or the page carries',
      fn: async () => {
        expect(findingsOf({
          sourceText: '猫说：«我饿了。»',
          candidateText: 'The cat said, «I am hungry.»',
        },),).toBe('',);
        expect(findingsOf({
          candidateText: 'The cat said, «I am hungry.»',
          pageText: 'The cat said, «I’m hungry.»',
        },),).toBe('',);
      },
    },),
    it({
      name: 'READS PROSE ONLY: a guillemet inside a code span, a tag or a comment is content, and names each '
        + 'guillemet of the prose once',
      fn: async () => {
        for (const candidateText of [
          'Type `«meow»` to feed the cat.',
          'The cat <Paw label="«left»" /> waved.',
          'The cat waved. <!-- «left» -->',
        ])
          expect(guillemetFindings({ sourceText: SOURCE, candidateText, },),).toStrictEqual([],);
        /**
         Findings for prose with two quotations and a code span.
         */
        const findings = guillemetFindings({
          sourceText: SOURCE,
          candidateText: 'The cat said, «I am hungry.» Then, «Feed me.» Type `‹x›`.',
        },);
        expect(findings,).toHaveLength(1,);
        expect(findings[0],).toContain('guillemets « »,',);
      },
    },),
  ],
},);
