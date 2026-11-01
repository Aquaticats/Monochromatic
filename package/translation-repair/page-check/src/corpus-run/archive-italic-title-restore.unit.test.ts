/**
 Guards class one hundred seventy-three (TianqiChen6668, 2026-09-26): the
 archive sets an anime's title in italics, yet the page wrote the same words
 in curly quotes, broken across two lines after the colon, so the page named
 a work in a form the archive never used. The page-assembly pass reads the
 archive's italic spans and, where the page quotes the same words in prose,
 restores the italic form, moving a period or comma the quotes held outside.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreArchiveItalicTitles, } from '../../dist/final/node/index.mjs';
import { pairAt, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/archive-slice-pair.test-fixture.ts';

/**
 Text the page carries at slice 1 after the pass, where slice 0 is an
 archive paragraph no lane replaced and a lane wrote slice 1.

 @param archive - archive text of slice 0

 @param replacement - what a lane wrote for slice 1

 @returns Slice 1's text after the pass

 @example
 ```ts
 secondSlice({ archive: 'She loved *Long Nap*.', replacement: 'She loved “Long Nap”.', },);
 ```
 */
function secondSlice(
  {
    archive,
    replacement,
  }: {
    readonly archive: string;
    readonly replacement: string;
  },
): string {
  /**
   The page after the pass.
   */
  const page = restoreArchiveItalicTitles({
    slices: [
      pairAt({ sliceIndex: 0, target: archive, startOffset: 0, },),
      pairAt({ sliceIndex: 1, target: 'The cat napped.', startOffset: archive.length + 2, },),
    ],
    replacements: [{ sliceIndex: 1, replacementText: replacement, },],
    archiveOriginalSpans: [],
  },);
  return page.replacements
    .find(function isSecond(row,): boolean {
      return row.sliceIndex === 1;
    },)
    ?.replacementText ?? replacement;
}

await describe({
  name: 'restoreArchiveItalicTitles (class one hundred seventy-three)',
  children: [
    it({
      name: 'RESTORES a quoted title broken across lines to the archive italics, the period outside',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *Whiskers: The Long Nap* (a cartoon).',
          replacement: 'The kitten loved “Whiskers:\nThe Long Nap”.\nShe purred.',
        },),).toBe('The kitten loved *Whiskers: The Long Nap*.\nShe purred.',);
      },
    },),
    it({
      name: 'MOVES a period or comma the quotes held outside the italics',
      fn: async () => {
        expect([
          secondSlice({
            archive: 'The cat loved *Whiskers: The Long Nap*.',
            replacement: 'The kitten loved “Whiskers: The Long Nap.”',
          },),
          secondSlice({
            archive: 'The cat loved *Whiskers: The Long Nap*.',
            replacement: 'After “Whiskers: The Long Nap,” she slept.',
          },),
        ],).toEqual([
          'The kitten loved *Whiskers: The Long Nap*.',
          'After *Whiskers: The Long Nap*, she slept.',
        ],);
      },
    },),
    it({
      name: 'LEAVES a quotation that is no archive italic span, a bold span and code',
      fn: async () => {
        expect([
          secondSlice({
            archive: 'The cat loved *Whiskers: The Long Nap*.',
            replacement: 'She said “Please nap.”',
          },),
          secondSlice({
            archive: 'The cat loved **Nap Time** most.',
            replacement: 'She loved “Nap Time” most.',
          },),
          secondSlice({
            archive: 'The cat loved *Whiskers: The Long Nap*.',
            replacement: 'She typed `“Whiskers: The Long Nap”` twice.',
          },),
        ],).toEqual([
          'She said “Please nap.”',
          'She loved “Nap Time” most.',
          'She typed `“Whiskers: The Long Nap”` twice.',
        ],);
      },
    },),
    it({
      name: 'LEAVES a quote the page never closes, reading past it rather than running off the end of the text',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *Long Nap*.',
          replacement: 'She said “Long Nap and then trailed off.',
        },),).toBe('She said “Long Nap and then trailed off.',);
      },
    },),
    it({
      name: 'RESTORES a title that opens on an accented capital, which a test of A to Z alone never read as a title '
        + '(ledger B18)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *Été des chats*.',
          replacement: 'The kitten loved “Été des chats”.',
        },),).toBe('The kitten loved *Été des chats*.',);
      },
    },),
    it({
      name: 'WRITES the outer of two nested quoted titles and keeps the text after it, where the inner one\'s '
        + 'spacing shrinks when read as one line (ledger B70)',
      fn: async () => {
        // THE INNER TITLE GETS SHORTER in italics: its two spaces read as one.
        // Written one at a time, last first, the inner went in first and the
        // outer then cut at an end one unit past where its quote now closed,
        // taking the period with it.
        expect(secondSlice({
          archive: 'The cat loved *The "Long Nap" Chronicles* and *Long Nap*.',
          replacement: 'The kitten loved “The "Long  Nap" Chronicles”.',
        },),).toBe('The kitten loved *The "Long Nap" Chronicles*.',);
      },
    },),
    it({
      name: 'LEAVES a quoted title inside an emphasis span as it stands, since italics inside italics show the '
        + 'title no different from the words around it and the quotes were what set it apart (ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *Long Nap*.',
          replacement: '*The kitten loved “Long Nap” all day.*',
        },),).toBe('*The kitten loved “Long Nap” all day.*',);
      },
    },),
    it({
      name: 'READS NO TITLE out of an HTML comment, which the site never shows, so a page quoting the '
        + 'comment\'s words keeps its quotes (ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat napped. <!-- *Hidden Nap* -->',
          replacement: 'The kitten loved “Hidden Nap”.',
        },),).toBe('The kitten loved “Hidden Nap”.',);
      },
    },),
    it({
      name: 'READS NO TITLE out of a JSX comment beside an HTML comment: the HTML comment is blanked before '
        + 'the strict parse, which then reads the JSX comment as an expression, where plain Markdown would '
        + 'read its starred words as italics (ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat napped. <!-- a note --> {/*Hidden Nap*/}',
          replacement: 'The kitten loved “Hidden Nap”.',
        },),).toBe('The kitten loved “Hidden Nap”.',);
      },
    },),
    it({
      name: 'READS a linked title in italics as its words, which a split at the stars read as link markup and '
        + 'never as a title (ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *[Long Nap](https://example.com/nap)*.',
          replacement: 'The kitten loved “Long Nap”.',
        },),).toBe('The kitten loved *Long Nap*.',);
      },
    },),
    it({
      name: 'READS an italic span across two lines whole, never its first line\'s words alone as a title of '
        + 'their own (ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *Long\nNap* most.',
          replacement: 'She loved “Long” and “Long Nap”.',
        },),).toBe('She loved “Long” and *Long Nap*.',);
      },
    },),
    it({
      name: 'READS an italic title across a hard line break as its words with one space between, never run '
        + 'together (ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved *Long\\\nNap* most.',
          replacement: 'The kitten loved “Long Nap”.',
        },),).toBe('The kitten loved *Long Nap*.',);
      },
    },),
    it({
      name: 'READS a title the archive sets in italics with underscores, which a split at the stars never saw '
        + '(ledger B72)',
      fn: async () => {
        expect(secondSlice({
          archive: 'The cat loved _Long Nap_.',
          replacement: 'The kitten loved “Long Nap”.',
        },),).toBe('The kitten loved *Long Nap*.',);
      },
    },),
  ],
},);
