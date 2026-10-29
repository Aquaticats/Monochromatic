/**
 Tests for folding a footnote label to the spelling mdast keys its nodes by.

 The claim under test is an agreement with another library, so the cases here
 are the ones where a hand-rolled fold and the parser's would part company.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  normalizeFootnoteIdentifier,
  relabelsFootnote,
} from '@monochromatic-dev/translation-repair-document-structure';
import { parseDocument, } from '../dist/final/node/index.mjs';

/**
 Folds a label the way this module does.

 @param identifier - label as written

 @returns Folded label

 @example
 ```ts
 const folded = fold('Note',);
 ```
 */
function fold(identifier: string,): string {
  return normalizeFootnoteIdentifier({ identifier, },);
}

/**
 Identifier mdast gives a document whose reference carries this label.

 Goes through a real parse rather than a restated rule, since the whole point
 of this module is to agree with the parser rather than with a description of
 it.

 @param identifier - label to write into both halves of a footnote pair

 @returns Identifier mdast keyed the definition by

 @example
 ```ts
 const parsed = parsedIdentifier('Note',);
 ```
 */
function parsedIdentifier(identifier: string,): string {
  /**
   Document carrying one footnote written with this exact label.
   */
  const parsed = parseDocument({
    text: `The cat naps[^${identifier}] here.\n\n[^${identifier}]: Its spot.\n`,
  },);

  /**
   Definition hit the parse produced, which is the node whose identifier is
   being compared.
   */
  const [definition,] = parsed.footnoteGraph
    .definitions;
  if (definition === undefined)
    throw new Error(`no definition parsed for ${identifier}`,);
  return definition.identifier;
}

await describe({
  name: normalizeFootnoteIdentifier.name,
  children: [
    it({
      name: 'folds case, which is the whole reason this exists: markdown reads '
        + '`[^Note]` and `[^note]` as one footnote and mdast hands back one '
        + 'spelling, while every scan in this package sees the source spelling',
      fn: async () => {
        expect(fold('Note',),).toBe('note',);
        expect(fold('NOTE',),).toBe('note',);
        expect(fold('note',),).toBe('note',);
      },
    },),

    it({
      name: 'leaves a numeric label exactly as written, which is what makes '
        + 'this change invisible on the corpus at pin a41fc60: all 209 of its '
        + 'GFM markers are digits',
      fn: async () => {
        expect(fold('1',),).toBe('1',);
        expect(fold('10',),).toBe('10',);
      },
    },),

    it({
      name: 'collapses whitespace runs to one space and trims the ends, since '
        + 'a label may carry them and the parser compares the collapsed form',
      fn: async () => {
        expect(fold('long  note',),).toBe('long note',);
        expect(fold('long\tnote',),).toBe('long note',);
        expect(fold(' note ',),).toBe('note',);
        expect(fold('\n note \n',),).toBe('note',);
      },
    },),

    it({
      name: 'folds DOWN, UP, then down again, which is not the same answer as '
        + 'one lowercase pass. The sharp s lowercases to itself and uppercases '
        + 'to two letters, so a single pass keeps two labels apart that the '
        + 'parser reads as one',
      fn: async () => {
        expect(fold('Straße',),).toBe('strasse',);
        expect(fold('STRASSE',),).toBe('strasse',);
        // The `Straße` claim is the interesting half: a plain lowercase would
        // leave this one different from the other two.
        expect('Straße'.toLowerCase(),).not.toBe(fold('Straße',),);
      },
    },),

    it({
      name: 'AGREES WITH THE PARSER on every one of those, which is the claim '
        + 'that matters: a fold of our own that mdast does not share would '
        + 'trade one lookup miss for another',
      fn: async () => {
        for (const identifier of [
          'note',
          'Note',
          'NOTE',
          '1',
          'Straße',
          'Kitten-Note',
        ]) {
          expect(parsedIdentifier(identifier,),).toBe(fold(identifier,),);
        }
      },
    },),
    it({
      name: 'A RELABEL MOVES A NOTE only when the two labels fold to different footnotes',
      fn: async () => {
        expect([
          { from: 'Whisker', to: 'whisker', },
          { from: 'Nap  Time', to: 'nap time', },
          { from: '1', to: '2', },
          { from: 'Paw', to: 'Claw', },
        ].map(function moves(relation,): boolean {
          return relabelsFootnote({ relation, },);
        },),).toEqual([false, false, true, true,],);
      },
    },),
  ],
},);
