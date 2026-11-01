/**
 Tests the display columns of text and the table they come from (ledger B1413): the code points the reviewer found counted
 as one column, the boundaries of every range of the table, the reading of the data file's format, and the version.

 Fixtures are cat-themed invention.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  columnsOfCodePoint,
  displayColumnsOf,
} from './display-columns.test-fixture.ts';
import {
  inRanges,
  mergeRanges,
  parseEastAsianWidth,
  rangesOfLines,
  textOfRange,
} from './east-asian-width-parse.test-fixture.ts';
import {
  generateTable,
  LINE_LIMIT,
  linesOfRanges,
  sourceAddressOf,
  TABLE_PATH,
  tableSourceOf,
} from './east-asian-width-generate.test-fixture.ts';
import {
  EAST_ASIAN_WIDTH_SHA256,
  EAST_ASIAN_WIDTH_VERSION,
  MARK_RANGE_LINES,
  WIDE_RANGE_LINES,
} from './east-asian-width-table.test-fixture.ts';

/**
 Ranges of two columns the generated table holds.
 */
const WIDE_RANGES = rangesOfLines({ lines: WIDE_RANGE_LINES, },);

/**
 Ranges of marks the generated table holds.
 */
const MARK_RANGES = rangesOfLines({ lines: MARK_RANGE_LINES, },);

/**
 A small data file in the format of `EastAsianWidth.txt`: its first line, a Wide range with a mark among it, a Fullwidth
 code point, a Narrow one, a Neutral mark and a comment.
 */
const PLANTED_FILE = [
  '# EastAsianWidth-99.0.0.txt',
  '# a comment line',
  '',
  '0020           ; Na # Zs         SPACE',
  '0300..0301     ; A  # Mn     [2] COMBINING GRAVE ACCENT..COMBINING ACUTE ACCENT',
  '2329..232A     ; W  # Ps     [2] LEFT-POINTING ANGLE BRACKET..RIGHT-POINTING ANGLE BRACKET',
  '3099           ; W  # Mn         COMBINING KATAKANA-HIRAGANA VOICED SOUND MARK',
  '309A           ; W  # Mn         COMBINING KATAKANA-HIRAGANA SEMI-VOICED SOUND MARK',
  'FF01           ; F  # Po         FULLWIDTH EXCLAMATION MARK',
  '',
].join('\n',);

await describe({
  name: 'display columns',
  concurrency: 1,
  children: [
    it({
      name: 'COUNTS TWO COLUMNS FOR THE CODE POINTS THE REVIEWER FOUND COUNTED AS ONE, and one for a letter, none for a mark',
      fn: async () => {
        // U+2329 is a Wide bracket, U+2FF0 an ideographic description character, U+3200 a parenthesized Hangul letter.
        expect(columnsOfCodePoint({ codePoint: 0x23_29, },),).toBe(2,);
        expect(columnsOfCodePoint({ codePoint: 0x2F_F0, },),).toBe(2,);
        expect(columnsOfCodePoint({ codePoint: 0x32_00, },),).toBe(2,);
        expect(columnsOfCodePoint({ codePoint: 'a'.codePointAt(0,) ?? 0, },),).toBe(1,);
        expect(columnsOfCodePoint({ codePoint: 0x03_01, },),).toBe(0,);
        expect(displayColumnsOf({ line: 'cat', },),).toBe(3,);
        expect(displayColumnsOf({ line: '猫猫', },),).toBe(4,);
        expect(displayColumnsOf({ line: 'á', },),).toBe(1,);
      },
    },),

    it({
      name: 'COUNTS A FLAG OF TWO REGIONAL INDICATORS AS TWO COLUMNS, as it renders, since the data file gives them Neutral',
      fn: async () => {
        expect(displayColumnsOf({ line: '\u{1F1E8}\u{1F1E6}', },),).toBe(2,);
      },
    },),

    it({
      name: 'GIVES THE FIRST AND LAST CODE POINT OF EVERY WIDE RANGE TWO COLUMNS, or none where it is a mark, and the code '
        + 'point either side of a range not two',
      fn: async () => {
        expect(WIDE_RANGES.length,).toBeGreaterThan(0,);
        for (const range of WIDE_RANGES) {
          for (const codePoint of [range.from, range.to,]) {
            expect(columnsOfCodePoint({ codePoint, },),).toBe(inRanges({ ranges: MARK_RANGES, codePoint, },) ? 0 : 2,);
          }
          // Ranges are merged, so the code point either side belongs to no wide range.
          for (const codePoint of [range.from - 1, range.to + 1,]) {
            expect(inRanges({ ranges: WIDE_RANGES, codePoint, },),).toBe(false,);
            expect(columnsOfCodePoint({ codePoint, },),).not.toBe(2,);
          }
        }
      },
    },),

    it({
      name: 'GIVES THE FIRST AND LAST CODE POINT OF EVERY MARK RANGE NO COLUMN, and keeps both tables in order without '
        + 'overlap or touching ranges',
      fn: async () => {
        for (const ranges of [WIDE_RANGES, MARK_RANGES,]) {
          expect(mergeRanges({ ranges, },),).toEqual(ranges,);
        }
        for (const range of MARK_RANGES) {
          expect(columnsOfCodePoint({ codePoint: range.from, },),).toBe(0,);
          expect(columnsOfCodePoint({ codePoint: range.to, },),).toBe(0,);
        }
      },
    },),

    it({
      name: 'NAMES THE UNICODE VERSION THE TREE\'S NODE REPORTS, so a new node asks for a new table, and a file hash',
      fn: async () => {
        expect(EAST_ASIAN_WIDTH_VERSION.startsWith(`${process.versions.unicode}.`,),).toBe(true,);
        expect(EAST_ASIAN_WIDTH_SHA256.length,).toBe(64,);
      },
    },),

    it({
      name: 'READS THE FORMAT OF THE DATA FILE: the version of its first line, Wide and Fullwidth ranges joined with the '
        + 'blocks that default to Wide, and marks from the category in the comment',
      fn: async () => {
        /**
         What the planted file gave.
         */
        const table = parseEastAsianWidth({ text: PLANTED_FILE, },);
        expect(table.version,).toBe('99.0.0',);
        expect(table.marks,).toEqual([{ from: 0x03_00, to: 0x03_01, }, { from: 0x30_99, to: 0x30_9A, },],);
        expect(inRanges({ ranges: table.wide, codePoint: 0x23_29, },),).toBe(true,);
        expect(inRanges({ ranges: table.wide, codePoint: 0x23_2A, },),).toBe(true,);
        expect(inRanges({ ranges: table.wide, codePoint: 0x23_2B, },),).toBe(false,);
        expect(inRanges({ ranges: table.wide, codePoint: 0xFF_01, },),).toBe(true,);
        expect(inRanges({ ranges: table.wide, codePoint: 0x20, },),).toBe(false,);
        // The Narrow and Ambiguous lines add nothing, and the default blocks stand though the file lists none of them.
        expect(inRanges({ ranges: table.wide, codePoint: 0x4E_00, },),).toBe(true,);
        expect(() => parseEastAsianWidth({ text: 'not the file\n', },),).toThrow('not EastAsianWidth.txt',);
      },
    },),

    it({
      name: 'WRITES RANGES THE READER READS BACK, in lines of at most the limit, and a module that names the version, the '
        + 'address and the hash',
      fn: async () => {
        /**
         What the planted file gave.
         */
        const table = parseEastAsianWidth({ text: PLANTED_FILE, },);
        /**
         Its wide ranges, written and read again.
         */
        const lines = linesOfRanges({ ranges: table.wide, },);
        expect(rangesOfLines({ lines, },),).toEqual(table.wide,);
        for (const line of lines)
          expect(line.length,).toBeLessThanOrEqual(LINE_LIMIT,);
        /**
         The module written for it.
         */
        const source = tableSourceOf({ table, sha256: 'cafe', },);
        expect(source,).toContain(sourceAddressOf({ version: '99.0.0', },),);
        expect(source,).toContain('\'cafe\'',);
        expect(source,).toContain(textOfRange({ range: table.marks[0] ?? { from: 0, to: 0, }, },),);
      },
    },),

    it({
      name: 'WRITES THE MODULE TO THE PATH IT IS GIVEN, from the file it is given, and the tree\'s own table sits at the '
        + 'path the task writes',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'east-asian-width-', },);
        /**
         Path the planted data file is written to.
         */
        const dataPath = join(scratch.path, 'EastAsianWidth.txt',);
        /**
         Path the generated module is written to.
         */
        const tablePath = join(scratch.path, 'table.test-fixture.ts',);
        await writeFile(dataPath, PLANTED_FILE,);
        expect(await generateTable({ dataPath, tablePath, },),).toBe('99.0.0',);
        /**
         What was written.
         */
        const written = await readFile(tablePath, 'utf8',);
        expect(written,).toContain('\'99.0.0\'',);
        expect(written,).toContain('WIDE_RANGE_LINES',);
        // The tree's own table names its version at the path the task writes.
        expect(await readFile(TABLE_PATH, 'utf8',),).toContain(`'${EAST_ASIAN_WIDTH_VERSION}'`,);
      },
    },),
  ],
},);
