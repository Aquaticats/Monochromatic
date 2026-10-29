/**
 Tests for the two steps of the refusal run that one character cannot say: a backslash that escapes the character after it (or
 ends a line), and a line break that is a single break inside a paragraph.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { isSingleBreakAt, startsEscape, } from '../dist/final/node/index.mjs';

/**
 The suite of the backslash steps.
 */
const escapeSuite = describe({
  name: startsEscape.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'READS A BACKSLASH BEFORE AN ASCII PUNCTUATION CHARACTER OR A LINE BREAK AS THE START OF AN ESCAPE',
      fn: async () => {
        expect([
          startsEscape({ text: String.raw`kittens\*’`, index: 7, },),
          startsEscape({ text: String.raw`kittens\_’`, index: 7, },),
          startsEscape({ text: String.raw`kittens\\’`, index: 7, },),
          startsEscape({ text: String.raw`kittens\'`, index: 7, },),
          startsEscape({ text: 'kittens\\\n’', index: 7, },),
          startsEscape({ text: 'kittens\\\r\n’', index: 7, },),
        ],).toEqual([true, true, true, true, true, true,],);
      },
    },),
    it({
      name: 'DOES NOT READ A BACKSLASH BEFORE ANY OTHER CHARACTER, A BACKSLASH AT THE END, OR A CHARACTER THAT IS NO BACKSLASH',
      fn: async () => {
        expect([
          startsEscape({ text: String.raw`kittens\a’`, index: 7, },),
          startsEscape({ text: String.raw`kittens\’`, index: 7, },),
          startsEscape({ text: String.raw`kittens\ ’`, index: 7, },),
          startsEscape({ text: 'kittens\\', index: 7, },),
          startsEscape({ text: 'kittens*’', index: 7, },),
          startsEscape({ text: 'kittens', index: 20, },),
        ],).toEqual([false, false, false, false, false, false,],);
      },
    },),
  ],
},);

/**
 The suite of the line break steps.
 */
const breakSuite = describe({
  name: isSingleBreakAt.name,
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'READS A LINE BREAK INSIDE A PARAGRAPH AS A SINGLE BREAK, a soft break, two spaces and a break, a backslash and a '
        + 'break and a carriage return with a line feed, at either half of the pair',
      fn: async () => {
        expect([
          isSingleBreakAt({ text: 'kittens\n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens  \n’ bowl.', index: 9, },),
          isSingleBreakAt({ text: 'kittens\\\n’ bowl.', index: 8, },),
          isSingleBreakAt({ text: 'kittens\r\n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\r\n’ bowl.', index: 8, },),
          isSingleBreakAt({ text: 'kittens\n  ’ bowl.', index: 7, },),
        ],).toEqual([true, true, true, true, true, true,],);
      },
    },),
    it({
      name: 'READS THE NEXT-LINE CONTROL AND THE LINE AND PARAGRAPH SEPARATORS AS A SINGLE LINE BREAK, alone, after white space '
        + 'and before it, and two of them or one beside a line feed too, since the page grammar keeps one paragraph there',
      fn: async () => {
        expect([
          isSingleBreakAt({ text: 'kittens\u{0085}’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\u{2028}’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\u{2029}’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens  \u{2028}’ bowl.', index: 9, },),
          isSingleBreakAt({ text: 'kittens\u{2028} ’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\u{2028}\u{2028}’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\u{2028}\u{2029}’ bowl.', index: 8, },),
          isSingleBreakAt({ text: 'kittens\u{0085} \n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\u{2029}’ bowl.', index: 7, },),
        ],).toEqual([true, true, true, true, true, true, true, true, true,],);
      },
    },),
    it({
      name: 'READS A LINE THAT THE PARSE DOES NOT CALL BLANK AS PART OF ONE PARAGRAPH, whichever character it holds: an en space, a '
        + 'line separator, a no-break space between a line feed and a carriage return, a byte order mark between carriage returns, '
        + 'a next-line control',
      fn: async () => {
        expect([
          isSingleBreakAt({ text: 'kittens\n\u{2002}\n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\u{2002}\n’ bowl.', index: 9, },),
          isSingleBreakAt({ text: 'kittens\n\u{2028}\n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\u{00A0}\r’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\u{00A0}\r’ bowl.', index: 9, },),
          isSingleBreakAt({ text: 'kittens\r\u{FEFF}\r’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\u{0085}\n’ bowl.', index: 9, },),
        ],).toEqual([true, true, true, true, true, true, true,],);
      },
    },),
    it({
      name: 'READS A LINE OF ONLY SPACES AND TABS AS A BLANK LINE: a space or a tab between two line feeds ends the paragraph',
      fn: async () => {
        expect([
          isSingleBreakAt({ text: 'kittens\n \n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\t\n’ bowl.', index: 9, },),
        ],).toEqual([false, false,],);
      },
    },),
    it({
      name: 'READS A LINE OF ONLY ONE INVISIBLE OR NON-ASCII SPACE CHARACTER AS INSIDE THE PARAGRAPH (n156, W9), as CommonMark does: an '
        + 'ideographic space, a zero-width space, a byte order mark, a no-break space, a word joiner, a Mongolian vowel separator, a '
        + 'narrow no-break space, a figure space and a zero-width non-joiner between two line feeds are no blank line',
      fn: async () => {
        /**
         The characters, built from code points so no literal separator stands in this file.
         */
        const characters = [0x30_00, 0x20_0B, 0xFE_FF, 0xA0, 0x20_60, 0x18_0E, 0x20_2F, 0x20_07, 0x20_0C,].map(
          function characterOf(code,): string {
            return String.fromCodePoint(code,);
          },
        );
        expect(characters.flatMap(function breaksOf(character,): readonly boolean[] {
          /**
           The text with the character alone on a line.
           */
          const text = `kittens\n${character}\n’ bowl.`;
          return [
            isSingleBreakAt({ text, index: 7, },),
            isSingleBreakAt({ text, index: 9, },),
          ];
        },),).toEqual(characters.flatMap(function bothBreaks(): readonly boolean[] {
          return [true, true,];
        },),);
      },
    },),
    it({
      name: 'DOES NOT READ A BLANK LINE, A BREAK AT THE EDGE OF THE TEXT, A BREAK BEFORE ANOTHER BLOCK OR A UNIT THAT IS NO BREAK',
      fn: async () => {
        expect([
          isSingleBreakAt({ text: 'kittens\n\n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n\n’ bowl.', index: 8, },),
          isSingleBreakAt({ text: 'kittens\n  \n’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens\r\n\r\n’ bowl.', index: 8, },),
          isSingleBreakAt({ text: '\n’ bowl.', index: 0, },),
          isSingleBreakAt({ text: 'kittens\n', index: 7, },),
          isSingleBreakAt({ text: 'kittens\n- ’ bowl.', index: 7, },),
          isSingleBreakAt({ text: 'kittens ’ bowl.', index: 7, },),
        ],).toEqual([false, false, false, false, false, false, false, false,],);
      },
    },),
  ],
},);

/**
 The suite of the header's account of the controls the run steps over (n156, W6).
 */
const headerSuite = describe({
  name: 'the header of consistency-run-steps.ts',
  concurrency: DEFAULT_CONCURRENCY,
  children: [
    it({
      name: 'SAYS THAT A CONTROL SHOWS A BLANK GAP OR NOTHING, not that it renders nothing, names the C1 range, and says why stepping '
        + 'over the controls is the safe direction',
      fn: async () => {
        /**
         The source of the file that defines the steps.
         */
        const source = (await readFile(
          new URL(
            'consistency-run-steps.ts',
            import.meta.url,
          ),
          'utf8',
        ))
          .split('\n',)
          .map(function textOf(line,): string {
            return line.startsWith('// ',) ? line.slice(3,) : line;
          },)
          .join(' ',);
        expect({
          blankGap: source.includes('a blank gap',),
          c0AndC1: source.includes('U+0080 to U+0084, U+0086 to U+009F',),
          safeDirection: source.includes('stepping over them is the safe direction',),
          rendersNothingForC0: source.includes('a control that shows nothing (U+0001',),
        },).toEqual({
          blankGap: true,
          c0AndC1: true,
          safeDirection: true,
          rendersNothingForC0: false,
        },);
      },
    },),
  ],
},);

await describe({
  name: '',
  concurrency: 1,
  children: [
    escapeSuite,
    breakSuite,
    headerSuite,
  ],
},);
