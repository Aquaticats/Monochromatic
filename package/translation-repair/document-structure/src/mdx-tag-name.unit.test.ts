/**
 Tests the one reading of a tag's name against the package's own MDX parser,
 which uses the compiler's grammar: a name is read exactly where the parser
 reads one, with the offset it ends at and the spelling readers compare.
 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readTagName, } from '@monochromatic-dev/translation-repair-markup';
import {
  grammarAcceptsDocument,
  tagNameSpellings,
} from './tag-name-spellings.test-fixture.ts';

/**
 The document the strict grammar is asked about a name: the tag of a whole
 element.

 @param name - name as written between the brackets

 @returns Opener, a paragraph and closer

 @example
 ```ts
 const document = elementNamed({ name: 'Paw', },);
 ```
 */
function elementNamed({ name, }: { readonly name: string; }): string {
  return `<${name}>\n\nA cat naps.\n\n</${name}>\n`;
}

await describe({
  name: readTagName.name,
  children: [
    it({
      name: 'READS THE NAME, THE OFFSET IT ENDS AT AND WHETHER THE TAG CLOSES, an opener with attributes and a '
        + 'closer with whitespace before its bracket alike',
      fn: async () => {
        expect(readTagName({ text: '<details open>', at: 0, },),).toEqual([{
          closes: false,
          name: 'details',
          end: 8,
        },],);
        expect(readTagName({ text: 'A cat. </details >', at: 7, },),).toEqual([{
          closes: true,
          name: 'details',
          end: 16,
        },],);
        expect(readTagName({ text: '<猫-nap/>', at: 0, },),).toEqual([{
          closes: false,
          name: '猫-nap',
          end: 6,
        },],);
      },
    },),

    it({
      name: 'READS A MEMBER NAME AND A PREFIXED NAME WITHOUT THE WHITESPACE AROUND THEIR SEPARATORS, ending at '
        + 'the last character of the name',
      fn: async () => {
        expect(readTagName({ text: '<a . b c>', at: 0, },),).toEqual([{
          closes: false,
          name: 'a.b',
          end: 6,
        },],);
        expect(readTagName({ text: '</a.b.c>', at: 0, },),).toEqual([{
          closes: true,
          name: 'a.b.c',
          end: 7,
        },],);
        expect(readTagName({ text: '<a : b>', at: 0, },),).toEqual([{
          closes: false,
          name: 'a:b',
          end: 6,
        },],);
      },
    },),

    it({
      name: 'STEPS OVER THE WHITESPACE THE GRAMMAR STEPS OVER BEFORE A NAME, all of it before a closer\'s and all but '
        + 'markdown\'s space, tab and line endings before an opener\'s, which leave the bracket text',
      fn: async () => {
        expect(readTagName({ text: '<\u{00A0}b>', at: 0, },),).toEqual([{
          closes: false,
          name: 'b',
          end: 3,
        },],);
        expect(readTagName({ text: '</ b>', at: 0, },),).toEqual([{
          closes: true,
          name: 'b',
          end: 4,
        },],);
        for (const text of ['< b>', '<\tb>', '<\nb>', '<\rb>',])
          expect(readTagName({ text, at: 0, },),).toEqual([],);
      },
    },),

    it({
      name: 'READS NO NAME WHERE THE GRAMMAR READS NONE: a fragment, a name that starts with a digit, a name '
        + 'holding a character it refuses, one ending on a separator and one the text ends in',
      fn: async () => {
        for (
          const text of [
            '<>',
            '</>',
            '<3>',
            '<a@b>',
            '<a\u{200B}>',
            '<a\u{200E}b>',
            '<a.>',
            '<a.3>',
            '<a:b:c>',
            '<a.b:c>',
            '<a:b.c>',
            '<a',
            '<',
            '<a "x">',
          ]
        )
          expect(readTagName({ text, at: 0, },),).toEqual([],);
      },
    },),

    it({
      name: 'READS NO PREFIXED NAME FOLLOWED BY A CHARACTER THAT NEITHER ENDS THE TAG NOR STARTS AN ATTRIBUTE, '
        + 'where the grammar refuses the tag',
      fn: async () => {
        expect(grammarAcceptsDocument({ document: '<cat:nap @>\n\nA cat naps.\n\n</cat:nap>\n', },),).toBe(false,);
        expect(readTagName({ text: '<cat:nap @>', at: 0, },),).toEqual([],);
      },
    },),

    it({
      name: 'READS NO NAME HOLDING A LETTER PAST U+FFFF, which the compiler hands the grammar as two units '
        + 'neither of which is a letter, in each place a name may start or go on',
      fn: async () => {
        for (
          const name of [
            '\u{20000}b',
            'a\u{20000}',
            'a.\u{1D49C}b',
            'a:\u{1F431}',
          ]
        ) {
          expect(grammarAcceptsDocument({ document: elementNamed({ name, },), },),).toBe(false,);
          expect(readTagName({ text: `<${name}>`, at: 0, },),).toEqual([],);
        }
      },
    },),

    it({
      name: 'READS A NAME IN EACH SCRIPT IT STARTS WITH, and one going on with the joiners, the dash and the '
        + 'dollar sign, as the grammar reads it',
      fn: async () => {
        for (
          const name of [
            'Paw',
            '猫',
            'é',
            'Ω',
            '$paw',
            '_paw',
            'paw-pad',
            'paw3',
            'paw\u{200C}pad',
            'paw\u{200D}pad',
            'paw\u{00B7}pad',
          ]
        ) {
          expect(grammarAcceptsDocument({ document: elementNamed({ name, },), },),).toBe(true,);
          expect(readTagName({ text: `<${name}>`, at: 0, },).map(function nameOf(reading,): string {
            return reading.name;
          },),).toEqual([name,],);
        }
      },
    },),

    it({
      name: 'READS A NAME EXACTLY WHERE THE GRAMMAR DOES over every spelling of the differential set: each character '
        + 'class in each place a name is told apart, and the names written with whitespace around a separator',
      fn: async () => {
        /**
         Spellings the reading and the grammar disagree on.
         */
        const disagreeing = tagNameSpellings()
          .filter(function disagrees(name,): boolean {
            return grammarAcceptsDocument({ document: elementNamed({ name, },), },)
              !== (readTagName({ text: `<${name}>`, at: 0, },).length > 0);
          },);
        expect(disagreeing,).toEqual([],);
      },
    },),
  ],
},);
