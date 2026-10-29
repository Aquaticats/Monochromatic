/**
 Tests that the guillemet floor does not stand aside for a candidate the page grammar refuses (n159, X10). The floor reads
 prose, and a grammar that refuses a candidate tells no prose from code, so the floor reads every guillemet outside a comment as
 a possible quotation. The floors that run after the source-carry floors refuse an unparseable candidate only where the original
 parses (`translate-validate.ts`, the skeleton read after `sourceCarryFindings`), and the verdict is unknown where the original
 or the page is blind, so no later floor is relied on.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { guillemetFindings, } from '../dist/final/node/index.mjs';

/**
 The Chinese original of a cat said line, which carries no guillemet.
 */
const SOURCE = '猫说：「我饿了。」';

/**
 How many findings the floor gives for a candidate.

 @param candidateText - rendering under the floor

 @returns Number of findings

 @example
 ```ts
 const count = findingsFor({ candidateText: 'The cat said «I am hungry.»', },);
 ```
 */
function findingsFor({ candidateText, }: { readonly candidateText: string; },): number {
  return guillemetFindings({
    sourceText: SOURCE,
    candidateText,
  },).length;
}

await describe({
  name: guillemetFindings.name,
  children: [
    it({
      name: 'REFUSES A CANDIDATE THE GRAMMAR REFUSES WHEN IT CARRIES A GUILLEMET, since no parse tells its prose from code',
      fn: async () => {
        expect({
          brace: findingsFor({ candidateText: 'The cat said «I am hungry.» and {', },),
          tag: findingsFor({ candidateText: 'The cat said «I am hungry.» and <b>', },),
          single: findingsFor({ candidateText: 'The cat said ‹I am hungry.› and {', },),
        },).toEqual({
          brace: 1,
          tag: 1,
          single: 1,
        },);
      },
    },),
    it({
      name: 'STILL PASSES A CANDIDATE THE GRAMMAR REFUSES THAT CARRIES NO GUILLEMET, so another floor names its fault',
      fn: async () => {
        expect(findingsFor({ candidateText: 'The cat said it was hungry and {', },),).toBe(0,);
      },
    },),
    it({
      name: 'STILL READS A COMMENT AS NO PROSE in a candidate the grammar refuses',
      fn: async () => {
        expect(findingsFor({ candidateText: 'The cat <!-- «x» --> naps and {', },),).toBe(0,);
      },
    },),
    it({
      name: 'STILL PASSES A GUILLEMET IN CODE OR A TAG IN A CANDIDATE THE GRAMMAR READS, and refuses one in prose',
      fn: async () => {
        expect({
          code: findingsFor({ candidateText: 'The cat typed `«x»` today.', },),
          tag: findingsFor({ candidateText: 'The <b title="«x»">cat</b> naps.', },),
          prose: findingsFor({ candidateText: 'The cat said «I am hungry.»', },),
        },).toEqual({
          code: 0,
          tag: 0,
          prose: 1,
        },);
      },
    },),
  ],
},);
