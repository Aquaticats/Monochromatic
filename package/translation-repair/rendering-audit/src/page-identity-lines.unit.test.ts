/**
 Guards ledger B28: the page's declared identity is assembled in one place,
 which preparation and the page title lexicon both read, so each part the
 sheets rely on must be in it, in the order every sheet reads it.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  collectIdentityLines,
  sourcePronounLines,
} from '@monochromatic-dev/translation-repair-house-rule';
import {
  entryNoteLines,
  pageIdentityLines,
} from '../dist/final/node/index.mjs';

/**
 Original: a declared name, a subject written as 她 throughout, a community
 word, a linked name and a footnote.
 */
const SOURCE = [
  '---',
  'name: 咪咪',
  '---',
  '',
  '咪咪是学霸。她喜欢[小鱼](https://cats.example/fish)。她每天睡觉。她很可爱。[^1]',
  '',
  '[^1]: 小鱼干是猫的零食',
  '',
].join('\n',);

/**
 Archive: its declared name, the linked name under the same address, and a
 contributor line.
 */
const ARCHIVE = [
  '---',
  'name: Mittens',
  '---',
  '',
  'Mittens was a top student. She liked [Little Fish](https://cats.example/fish). She slept every day.',
  '',
  'Contributors for this entry: Mika',
  '',
].join('\n',);

/**
 Evidence lines bought outside preparation.
 */
const CONTEXT_LINES: readonly string[] = [
  'web lookup 《猫之歌》: "Song of the Cats"',
  '- 小猫 (entry OtherCat): "Little Cat"',
];

/**
 Both documents, parsed.
 */
const sourceDocument = parseDocument({ text: SOURCE, },);

/**
 Archive, parsed.
 */
const targetDocument = parseDocument({ text: ARCHIVE, },);

/**
 The assembled identity.
 */
const lines = pageIdentityLines({
  sourceDocument,
  targetDocument,
  sourceText: SOURCE,
  targetText: ARCHIVE,
  contextLines: CONTEXT_LINES,
},);

/**
 Position of the first line a test accepts, or -1 when none does.

 @param accepts - which line counts

 @returns Index into the assembled lines

 @example
 ```ts
 const at = firstAt({ accepts: (line) => line.includes('学霸'), },);
 ```
 */
function firstAt({ accepts, }: { readonly accepts: (line: string) => boolean; },): number {
  return lines.findIndex(accepts,);
}

await describe({
  name: `${pageIdentityLines.name} (ledger B28)`,
  children: [
    it({
      name: 'CARRIES every part: declared names, pronoun, contributors, notes, community words, page names, context',
      fn: async () => {
        /** Front matter lines, both sides. */
        const declared = collectIdentityLines({
          sourceData: sourceDocument.frontMatter?.data,
          targetData: targetDocument.frontMatter?.data,
        },);
        /** Pronoun lines. */
        const pronoun = sourcePronounLines({ text: SOURCE, },);
        /** Note lines. */
        const notes = entryNoteLines({ sourceDocument, targetDocument, },);
        // Each part is present in the fixture, so any absence this assertion finds is the helper's.
        expect({
          declared: declared.length > 0,
          pronoun: pronoun.length > 0,
          notes: notes.length > 0,
        },).toEqual({
          declared: true,
          pronoun: true,
          notes: true,
        },);
        expect({
          declared: declared.every(function carried(line,): boolean {
            return lines.includes(line,);
          },),
          pronoun: pronoun.every(function carried(line,): boolean {
            return lines.includes(line,);
          },),
          contributor: lines.includes('target contributor: Mika',),
          notes: notes.every(function carried(line,): boolean {
            return lines.includes(line,);
          },),
          community: lines.some(function namesWord(line,): boolean {
            return line.includes('学霸',);
          },),
          pageName: lines.some(function namesLink(line,): boolean {
            return line.includes('Little Fish',);
          },),
          context: lines.slice(-CONTEXT_LINES.length,),
        },).toEqual({
          declared: true,
          pronoun: true,
          contributor: true,
          notes: true,
          community: true,
          pageName: true,
          context: [...CONTEXT_LINES,],
        },);
      },
    },),
    it({
      name: 'ORDERS the parts as every sheet reads them, the caller\'s context last',
      fn: async () => {
        /** Where each part first stands. */
        const positions = [
          firstAt({ accepts: (line) => line.startsWith('- name: ',), },),
          firstAt({ accepts: (line) => sourcePronounLines({ text: SOURCE, },).includes(line,), },),
          firstAt({ accepts: (line) => line.startsWith('target contributor: ',), },),
          firstAt({ accepts: (line) => line.includes(' note: ',), },),
          firstAt({ accepts: (line) => line.includes('学霸',), },),
          firstAt({ accepts: (line) => line.includes('Little Fish',), },),
          firstAt({ accepts: (line) => line === CONTEXT_LINES[0], },),
        ];
        expect(positions,).toEqual(positions.toSorted(function ascending(left, right,): number {
          return left - right;
        },),);
        expect(positions.includes(-1,),).toBe(false,);
      },
    },),
  ],
},);
