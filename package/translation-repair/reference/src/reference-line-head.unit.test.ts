/**
 Tests the reference line head (ledger B25): the lookup writes it and the
 attestation reads the number back, so a quote is looked for in what the one
 line of the reference an item names says its page states (ledger B104).

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  numberedReferenceLines,
  referenceLineHead,
  ReferenceLineHeadError,
  referenceLineOf,
  referenceLinePageText,
  referencePageTexts,
} from '../dist/final/node/index.mjs';

/**
 What a block is refused with.

 @param referenceContext - block to read

 @returns Refusal text, or a sentence saying the block was read

 @example
 ```ts
 refusalOf({ referenceContext: 'Mittens naps.', },);
 ```
 */
function refusalOf(
  { referenceContext, }: { readonly referenceContext: string; },
): string {
  try {
    numberedReferenceLines({ referenceContext, },);
    return 'the block was read';
  } catch (error) {
    return caughtValueText(error,);
  }
}

await describe({
  name: 'reference line head (ledger B25)',
  children: [
    it({
      name: 'WRITES the head the lookup opens every line with, and reads each line\'s number back',
      fn: async () => {
        expect(referenceLineHead({
          index: 2,
          url: 'https://cats.example/b',
        },),).toBe('- reference 2 https://cats.example/b',);
        /**
         Lines as the lookup writes them, one fetched and one not.
         */
        const lines = [
          referenceLineOf({
            index: 1,
            record: {
              url: 'https://cats.example/a',
              fetchedAt: '2026-09-29T00:00:00.000Z',
              status: 'success',
              title: 'Naps',
              text: 'Mittens naps\non the sill.',
            },
          },),
          referenceLineOf({
            index: 12,
            record: {
              url: 'https://cats.example/l',
              fetchedAt: '2026-09-29T00:00:00.000Z',
              status: 'error',
              title: '',
              text: '',
              failure: 'NOT\nFOUND',
            },
          },),
        ];
        expect(lines,).toEqual([
          '- reference 1 https://cats.example/a ("Naps"): Mittens naps on the sill.',
          '- reference 12 https://cats.example/l: could not be fetched (NOT FOUND)',
        ],);
        expect(numberedReferenceLines({ referenceContext: lines.join('\n',), },),).toEqual([
          {
            reference: 1,
            line: lines[0],
          },
          {
            reference: 12,
            line: lines[1],
          },
        ],);
      },
    },),
    it({
      name: 'READS no lines from an empty block, the original that links nowhere',
      fn: async () => {
        expect(numberedReferenceLines({ referenceContext: '', },),).toEqual([],);
      },
    },),
    it({
      name: 'REFUSES a line without a numbered head, naming where it sits and not what it says',
      fn: async () => {
        /**
         Blocks with one line the lookup would never write.
         */
        const blocks = [
          '- reference 1 https://cats.example/a: Mittens naps.\nMittens naps.',
          '- reference 1 https://cats.example/a: Mittens naps.\n- reference two https://cats.example/b: Mittens purrs.',
          '- reference 1 https://cats.example/a: Mittens naps.\n- reference 2',
          '- reference 1 https://cats.example/a: Mittens naps.\n',
          // A list item as long as the mark, with a number and a space where
          // the head's number stands: only the mark tells it from a head.
          '- reference 1 https://cats.example/a: Mittens naps.\n* reference 3 https://cats.example/c: Mittens purrs.',
        ];
        for (const referenceContext of blocks) {
          expect(function read(): void {
            numberedReferenceLines({ referenceContext, },);
          },).toThrow(ReferenceLineHeadError,);
          expect(function read(): void {
            numberedReferenceLines({ referenceContext, },);
          },).toThrow('reference line 2 of 2',);
        }
        /**
         Message of the refusal for a line carrying page text.
         */
        const message = refusalOf({ referenceContext: 'Mittens naps.', },);
        expect(message,).toContain('reference line 1 of 1',);
        expect(message,).not
          .toContain('Mittens',);
      },
    },),
    it({
      name: 'REFUSES a reference number past the largest whole number a double holds exactly, which `Number` '
        + 'reads as a neighbouring reference nobody wrote (ledger B73)',
      fn: async () => {
        expect(refusalOf({
          referenceContext: `- reference ${String(BigInt(Number.MAX_SAFE_INTEGER,) + 2n,)} https://cats.example/a: `
            + 'Mittens naps.',
        },),).toBe('reference line 1 of 1 does not open with "- reference <number> ", so the reference it belongs '
          + 'to cannot be read; the block must be one referenceLineOf line per reference',);
      },
    },),
    it({
      name: 'READS what each page says without its head, and nothing from a failure note or an attested line '
        + '(ledger B28): the archive review anchors a retention only in a page\'s own words',
      fn: async () => {
        /**
         Lines as the lookup writes them: a page with a title, one without, one
         the lookup could not fetch and one with nothing readable; then an
         attested line, which quotes the archive.
         */
        const referenceContext = [
          referenceLineOf({
            index: 1,
            record: {
              url: 'https://cats.example/a',
              fetchedAt: '2026-09-29T00:00:00.000Z',
              status: 'success',
              title: 'Naps',
              text: 'Mittens naps on the sill.',
            },
          },),
          referenceLineOf({
            index: 2,
            record: {
              url: 'https://cats.example/b',
              fetchedAt: '2026-09-29T00:00:00.000Z',
              status: 'success',
              title: '',
              text: 'Mittens purrs at dawn.',
            },
          },),
          referenceLineOf({
            index: 3,
            record: {
              url: 'https://cats.example/c',
              fetchedAt: '2026-09-29T00:00:00.000Z',
              status: 'error',
              title: '',
              text: '',
              failure: 'NOT FOUND',
            },
          },),
          referenceLineOf({
            index: 4,
            record: {
              url: 'https://cats.example/d',
              fetchedAt: '2026-09-29T00:00:00.000Z',
              status: 'success',
              title: 'Blank',
              text: '',
            },
          },),
          '- attested: the ARCHIVE\'s "The cat naps on the sill." is stated by reference 1 ("Mittens naps"), '
            + '2 of 3 voices checked word for word',
        ].join('\n',);
        expect(referencePageTexts({ referenceContext, },),).toEqual([
          '("Naps"): Mittens naps on the sill.',
          'Mittens purrs at dawn.',
        ],);
        expect(referencePageTexts({ referenceContext: '', },),).toEqual([],);
        // A line opening with the mark and no number is still refused.
        expect(function read(): void {
          referencePageTexts({ referenceContext: '- reference two https://cats.example/b: Mittens purrs.', },);
        },).toThrow(ReferenceLineHeadError,);
      },
    },),
    it({
      name: 'READS one line\'s page text as the attestation does (ledger B104): the title and text after the '
        + 'address, and nothing for a failure note, an unreadable page or a head with nothing after it',
      fn: async () => {
        /**
         Page text of each line, read line by line.
         */
        const texts = numberedReferenceLines({
          referenceContext: [
            '- reference 1 https://cats.example/a ("Naps"): Mittens naps on the sill.',
            '- reference 2 https://cats.example/b: Mittens purrs at dawn.',
            '- reference 3 https://cats.example/c: could not be fetched (NOT FOUND)',
            '- reference 4 https://cats.example/d: nothing readable on the page',
            '- reference 5 https://cats.example/e',
          ].join('\n',),
        },).map(function pageTextOf(line,): string {
          return referenceLinePageText({ line, },);
        },);
        expect(texts,).toEqual([
          '("Naps"): Mittens naps on the sill.',
          'Mittens purrs at dawn.',
          '',
          '',
          '',
        ],);
      },
    },),
  ],
},);
