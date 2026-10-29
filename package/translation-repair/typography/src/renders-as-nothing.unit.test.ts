/**
 Tests the one reading of "shows a reader nothing" that every check on a
 model's wording or reason shares (ledger B40): Unicode White_Space, the
 default-ignorable code points and the controls are nothing, whatever `trim()`
 makes of them, and any other character is something, wherever it stands.

 Every fixture is spelled as an escape: as characters these are invisible in a
 diff, and the intake fold's first test lost its literal fixtures to the tool
 that wrote it (`invisible-variants.ts`).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { rendersAsNothing, } from '../dist/final/node/index.mjs';

/**
 Texts that show nothing, each labelled by what it is made of.
 */
const NOTHING = [
  ['empty', '',],
  ['ASCII spaces, a tab and line ends', ' \t\n\r ',],
  ['ideographic space U+3000, which trim() removes', '\u{3000}',],
  ['no-break spaces U+00A0 and U+202F', '\u{00A0}\u{202F}',],
  ['line and paragraph separators U+2028 and U+2029', '\u{2028}\u{2029}',],
  ['next line U+0085', '\u{0085}',],
  ['zero-width space U+200B, which trim() keeps', '\u{200B}',],
  ['word joiner U+2060', '\u{2060}',],
  ['byte order mark U+FEFF', '\u{FEFF}',],
  ['soft hyphen U+00AD', '\u{00AD}',],
  ['Hangul filler U+3164, which the intake fold keeps', '\u{3164}',],
  ['Hangul choseong filler U+115F and halfwidth filler U+FFA0', '\u{115F}\u{FFA0}',],
  ['combining grapheme joiner U+034F', '\u{034F}',],
  ['variation selector-16 U+FE0F alone', '\u{FE0F}',],
  ['left-to-right mark U+200E and zero-width joiner U+200D', '\u{200E}\u{200D}',],
  ['language tag U+E0001 and tag letter U+E0041, past the first plane', '\u{E0001}\u{E0041}',],
  ['controls U+0007 and U+001B', '\u{0007}\u{001B}',],
  ['every kind mixed', ' \u{200B}\u{3000}\u{3164}\u{E0001}\n',],
] as const;

/**
 Texts that show something, each labelled by what it is made of.
 */
const SOMETHING = [
  ['an ASCII letter', 'a',],
  ['a Han character', '\u{732B}',],
  ['a hyphen', '-',],
  ['a non-breaking hyphen U+2011, which the fold turns into another visible hyphen', '\u{2011}',],
  ['a cat emoji past the first plane', '\u{1F408}',],
  ['a black cat, an emoji sequence joined by U+200D', '\u{1F408}\u{200D}\u{2B1B}',],
  ['a lone combining acute accent, which shows a mark', '\u{0301}',],
  ['a word with a zero-width space inside it', 'cat\u{200B}nap',],
  ['one letter among invisible characters', '\u{200B} a \u{3164}',],
] as const;

await describe({
  name: rendersAsNothing.name,
  children: [
    it({
      name: 'CALLS NOTHING every text made only of whitespace, default-ignorable code points and controls, '
        + 'including the ones trim() keeps and the ones the intake fold keeps',
      fn: async () => {
        expect(NOTHING.map(function readingOf({ 0: label, 1: text, },): string {
          return `${label}: ${String(rendersAsNothing({ text, },),)}`;
        },),).toEqual(NOTHING.map(function expectedOf({ 0: label, },): string {
          return `${label}: true`;
        },),);
      },
    },),
    it({
      name: 'CALLS SOMETHING every text holding one visible character, wherever it stands among invisible ones',
      fn: async () => {
        expect(SOMETHING.map(function readingOf({ 0: label, 1: text, },): string {
          return `${label}: ${String(rendersAsNothing({ text, },),)}`;
        },),).toEqual(SOMETHING.map(function expectedOf({ 0: label, },): string {
          return `${label}: false`;
        },),);
      },
    },),
    it({
      name: 'SAYS IN ITS HEADER THAT IT READS THE RUNTIME\'S OWN UNICODE TABLES, and names no Unicode version, since the property '
        + 'tests are answered by whichever tables the runtime ships and no version can be applied from the source',
      fn: async () => {
        /**
         The source of the file that defines the reading.
         */
        const source = await readFile(
          new URL(
            'renders-as-nothing.ts',
            import.meta.url,
          ),
          'utf8',
        );
        expect({
          readsRuntimeTables: source.includes('the runtime\'s own Unicode tables',),
          namesVersion: source.split('Unicode ',)
            .slice(1,)
            .some(function startsWithDigit(rest,): boolean {
              return Array.from('0123456789',)
                .includes(rest.charAt(0,),);
            },),
        },).toEqual({
          readsRuntimeTables: true,
          namesVersion: false,
        },);
      },
    },),
  ],
},);
