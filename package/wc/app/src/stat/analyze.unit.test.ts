/**
 Tests for aggregate text-statistics computation.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { analyzeText, } from './analyze.ts';

await describe({
  name: analyzeText.name,
  children: [
    it({
      name: 'reports every field as 0 for empty text',
      fn: async function reportsZerosForEmptyText(): Promise<void> {
        expect(
          analyzeText('',),
        )
          .toEqual({
            bytes: 0,
            chars: 0,
            maxCharLength: 0,
            lines: 0,
            maxLineLength: 0,
            words: 0,
            maxWordLength: 0,
            sentences: 0,
            maxSentenceLength: 0,
            paragraphs: 0,
            maxParagraphLength: 0,
          },);
      },
    },),
    it({
      name: 'computes every field for a two-line, two-sentence sample',
      fn: async function computesSample(): Promise<void> {
        expect(
          analyzeText('Hi there.\nBye.',),
        )
          .toEqual({
            bytes: 14,
            chars: 14,
            maxCharLength: 1,
            lines: 2,
            maxLineLength: 9,
            words: 3,
            maxWordLength: 5,
            sentences: 2,
            maxSentenceLength: 2,
            paragraphs: 1,
            maxParagraphLength: 2,
          },);
      },
    },),
    it({
      name: 'reports maxParagraphLength as the sentence count of the longest paragraph',
      fn: async function reportsMaxParagraphLength(): Promise<void> {
        /**
         First paragraph has 3 sentences, second paragraph has 1.
         */
        const stats = analyzeText('One. Two. Three.\n\nJust one.',);

        expect(stats.paragraphs,).toBe(2,);
        expect(stats.maxParagraphLength,).toBe(3,);
      },
    },),
    it({
      name: 'reports maxCharLength as the byte length of the widest grapheme cluster',
      fn: async function reportsMaxCharLength(): Promise<void> {
        /**
         The ZWJ family emoji encodes as 18 UTF-8 bytes, wider than every
         other grapheme in the sample (the accented "é" is 2 bytes).
         */
        const stats = analyzeText('a café 👨‍👩‍👧',);

        expect(stats.maxCharLength,).toBe(18,);
      },
    },),
    it({
      name: 'excludes blank lines from the line count and treats them as separators only',
      fn: async function excludesBlankLines(): Promise<void> {
        /**
         Three non-blank lines split across two paragraphs by one blank
         line; the blank line must not itself be counted as a line.
         */
        const stats = analyzeText('one\ntwo\n\nthree',);

        expect(stats.lines,).toBe(3,);
        expect(stats.paragraphs,).toBe(2,);
      },
    },),
  ],
},);
