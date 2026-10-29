/**
 Tests for the ranges a rewrite must leave alone.

 A CODE SPAN IS WHAT THE PARSE READS as one (ledger B68), not a backtick
 run matched to the next one before a blank line: a heading written on the
 line after a paragraph ends it with no blank line, and a scan bounded by
 blank lines let a stray backtick close on a span in a later block.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  protectedRanges,
} from '../../dist/final/node/index.mjs';

await describe({
  name: protectedRanges.name,
  children: [
    it({
      name: 'NEVER MATCHES A CODE SPAN ACROSS A HEADING: a stray backtick before one cat word '
        + 'protects nothing, and the code span past the heading is the one range (ledger B68)',
      fn: async () => {
        // The backtick at offset 11 has no partner in its own paragraph, which
        // the heading ends with no blank line; the one code span is the one
        // the paragraph past the heading writes, at 36 to 46.
        expect(protectedRanges({ text: 'A cat says `meow\n# Loud heading\nand `not meow` again.\n', },),)
          .toStrictEqual([{
            start: 36,
            end: 46,
          },],);
      },
    },),
    it({
      name: 'READS NO CODE SPAN where neither grammar reads the text, rather than throwing, and the same '
        + 'span one quote deep is protected, so the absence is the refusal\'s (ledger B100)',
      fn: async () => {
        /**
         A line carrying one code span.
         */
        const line = 'The cat says `meow`.';
        expect(protectedRanges({ text: `> ${line}`, },),).toStrictEqual([{
          start: '> '.length + line.indexOf('`',),
          end: '> '.length + line.lastIndexOf('`',) + 1,
        },],);
        // Plain markdown descends once per quotation marker, and this many
        // exhaust its stack.
        expect(protectedRanges({ text: `${'>'.repeat(16_000,)} ${line}`, },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'PROTECTS a line comment in an expression and READS ITS END at the line end, so a closing '
        + 'brace written inside the comment closes nothing, or at the text end where the line never ends',
      fn: async () => {
        // The brace at offset 10 is comment text; the one that closes the
        // expression opens the line after, at 12.
        expect(protectedRanges({ text: '{ // meow }\n} purr', },),).toStrictEqual([{
          start: 0,
          end: 13,
        },],);
        expect(protectedRanges({ text: '{ // meow', },),).toStrictEqual([{
          start: 0,
          end: 9,
        },],);
      },
    },),

    it({
      name: 'PROTECTS an expression quoting with a backtick to the brace past the template literal, '
        + 'since a closing brace inside the literal is text of the quote',
      fn: async () => {
        expect(protectedRanges({ text: '{ `meow } meow` } purr', },),).toStrictEqual([{
          start: 0,
          end: 17,
        },],);
      },
    },),

    it({
      name: 'PROTECTS A CODE FENCE through its closing fence, and a code span past the fence apart from it',
      fn: async () => {
        expect(protectedRanges({ text: 'A cat.\n```\ncode `x` more\n```\nA cat `y`.\n', },),).toStrictEqual([
          {
            start: 7,
            end: 28,
          },
          {
            start: 35,
            end: 38,
          },
        ],);
      },
    },),

    it({
      name: 'PROTECTS a code span whose opening backtick follows an escaped backtick, as the parse reads '
        + 'it, and READS NO span in a bare double backtick (ledger B130)',
      fn: async () => {
        // The backtick at offset 7 is escaped and is text; the span opens at 8.
        expect(protectedRanges({ text: 'A cat \\``meow` naps.', },),).toStrictEqual([{
          start: 8,
          end: 14,
        },],);
        expect(protectedRanges({ text: '``', },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'READS A BLOCK COMMENT THAT NEVER CLOSES as running to the text end, a closing brace inside it '
        + 'closing nothing',
      fn: async () => {
        expect(protectedRanges({ text: '{ /* meow } purr', },),).toStrictEqual([{
          start: 0,
          end: 16,
        },],);
      },
    },),

    it({
      name: 'PROTECTS THE FRONT MATTER OF A PAGE OPENING WITH A BYTE ORDER MARK, the mark inside the range, '
        + 'and reads the code span past it at the offsets of the text as written',
      fn: async () => {
        /**
         Page with front matter and one code span, as the same page reads without a mark.
         */
        const page = '---\nname: Mittens\n---\nA cat `x`.\n';

        expect(protectedRanges({ text: page, },),).toStrictEqual([
          {
            start: 0,
            end: 22,
          },
          {
            start: 28,
            end: 31,
          },
        ],);
        expect(protectedRanges({ text: `\uFEFF${page}`, },),).toStrictEqual([
          {
            start: 0,
            end: 23,
          },
          {
            start: 29,
            end: 32,
          },
        ],);
      },
    },),

    it({
      name: 'PROTECTS a bare web address in prose up to the space that ends it',
      fn: async () => {
        expect(protectedRanges({ text: 'See https://cat.example/a cat today.', },),).toStrictEqual([{
          start: 4,
          end: 25,
        },],);
      },
    },),
  ],
},);
