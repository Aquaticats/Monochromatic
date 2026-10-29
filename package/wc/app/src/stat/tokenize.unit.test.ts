/**
 Tests for text-tokenization primitives.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  computeMaxLength,
  countBytes,
  countGraphemes,
  isBlankLine,
  splitGraphemes,
  splitLines,
  splitParagraphs,
  splitSentences,
  splitWords,
} from './tokenize.ts';

await describe({
  name: countBytes.name,
  children: [
    it({
      name: 'counts ASCII text as one byte per character',
      fn: async function countsAscii(): Promise<void> {
        expect(countBytes('abc',),).toBe(3,);
      },
    },),
    it({
      name: 'counts multi-byte UTF-8 sequences',
      fn: async function countsMultiByte(): Promise<void> {
        expect(countBytes('café',),).toBe(5,);
      },
    },),
    it({
      name: 'returns 0 for empty text',
      fn: async function countsEmpty(): Promise<void> {
        expect(countBytes('',),).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: splitGraphemes.name,
  children: [
    it({
      name: 'collapses a combining accent onto its base letter into one grapheme',
      fn: async function collapsesCombiningAccent(): Promise<void> {
        /**
         Letter `a` followed by a combining acute accent (U+0301), two
         codepoints that render as one visible character.
         */
        const decomposed = 'á';

        expect(splitGraphemes(decomposed,),).toHaveLength(1,);
      },
    },),
    it({
      name: 'collapses a ZWJ emoji sequence into one grapheme',
      fn: async function collapsesEmojiSequence(): Promise<void> {
        expect(countGraphemes('👨‍👩‍👧',),).toBe(1,);
      },
    },),
    it({
      name: 'splits plain ASCII text into individual letters',
      fn: async function splitsPlainAscii(): Promise<void> {
        expect(splitGraphemes('abc',),).toEqual(['a', 'b', 'c',],);
      },
    },),
  ],
},);

await describe({
  name: splitWords.name,
  children: [
    it({
      name: 'drops punctuation-only segments from space-delimited text',
      fn: async function dropsPunctuation(): Promise<void> {
        expect(splitWords('Hello, world!',),).toEqual(['Hello', 'world',],);
      },
    },),
    it({
      name: 'segments CJK text with no spaces between words',
      fn: async function segmentsCjk(): Promise<void> {
        expect(splitWords('こんにちは、世界',),).toEqual(['こんにちは', '世界',],);
      },
    },),
    it({
      name: 'returns an empty list for empty text',
      fn: async function returnsEmptyForEmptyText(): Promise<void> {
        expect(splitWords('',),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: splitSentences.name,
  children: [
    it({
      name: 'splits on sentence-ending punctuation and trims surrounding whitespace',
      fn: async function splitsAndTrims(): Promise<void> {
        expect(
          splitSentences('She left. She returned.',),
        )
          .toEqual(['She left.', 'She returned.',],);
      },
    },),
    it({
      name: 'returns an empty list for empty text',
      fn: async function returnsEmptyForEmptyText(): Promise<void> {
        expect(splitSentences('',),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: splitLines.name,
  children: [
    it({
      name: 'returns an empty list for empty text',
      fn: async function returnsEmptyForEmptyText(): Promise<void> {
        expect(splitLines('',),).toEqual([],);
      },
    },),
    it({
      name: 'treats text with no newline as one line',
      fn: async function treatsNoNewlineAsOneLine(): Promise<void> {
        expect(splitLines('a',),).toEqual(['a',],);
      },
    },),
    it({
      name: 'does not add a phantom line for a single trailing newline',
      fn: async function dropsTrailingNewlinePhantom(): Promise<void> {
        expect(splitLines('a\n',),).toEqual(['a',],);
      },
    },),
    it({
      name: 'counts each newline-separated segment as its own line',
      fn: async function countsEachSegment(): Promise<void> {
        expect(splitLines('a\nb',),).toEqual(['a', 'b',],);
      },
    },),
    it({
      name: 'still counts a blank line that exists before end-of-text',
      fn: async function countsBlankLineBeforeEnd(): Promise<void> {
        expect(splitLines('a\n\n',),).toEqual(['a', '',],);
      },
    },),
  ],
},);

await describe({
  name: isBlankLine.name,
  children: [
    it({
      name: 'reports true for an empty string',
      fn: async function reportsTrueForEmpty(): Promise<void> {
        expect(isBlankLine('',),).toBe(true,);
      },
    },),
    it({
      name: 'reports true for whitespace-only content',
      fn: async function reportsTrueForWhitespace(): Promise<void> {
        expect(isBlankLine('   \t',),).toBe(true,);
      },
    },),
    it({
      name: 'reports false when the line has non-whitespace content',
      fn: async function reportsFalseForContent(): Promise<void> {
        expect(isBlankLine(' a ',),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: splitParagraphs.name,
  children: [
    it({
      name: 'groups consecutive non-blank lines and splits on a blank line',
      fn: async function groupsAndSplits(): Promise<void> {
        expect(
          splitParagraphs('one\ntwo\n\nthree',),
        )
          .toEqual(['one\ntwo', 'three',],);
      },
    },),
    it({
      name: 'returns an empty list for empty text',
      fn: async function returnsEmptyForEmptyText(): Promise<void> {
        expect(splitParagraphs('',),).toEqual([],);
      },
    },),
    it({
      name: 'returns an empty list for whitespace-only text',
      fn: async function returnsEmptyForWhitespaceOnlyText(): Promise<void> {
        expect(splitParagraphs('   \n\t\n',),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: computeMaxLength.name,
  children: [
    it({
      name: 'returns the largest length among items',
      fn: async function returnsLargestLength(): Promise<void> {
        expect(
          computeMaxLength({
            items: ['a', 'bb', 'ccc',],
            lengthOf: (item: string,) => item.length,
          },),
        )
          .toBe(3,);
      },
    },),
    it({
      name: 'returns 0 for an empty item list',
      fn: async function returnsZeroForEmptyItems(): Promise<void> {
        expect(
          computeMaxLength({
            items: [],
            lengthOf: (item: string,) => item.length,
          },),
        )
          .toBe(0,);
      },
    },),
  ],
},);
