/**
 Tests the reading of where the strict grammar reads a table or a footnote
 definition in a text. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  footnoteDefinitionRangesOf,
  pageFootnoteDefinitionRangesOf,
  tableRangesOf,
} from '../dist/final/node/index.mjs';

/**
 A table of one body row.
 */
const TABLE = '| a | b |\n| - | - |\n| c | d |\n';

await describe({
  name: 'gfm block ranges',
  children: [
    describe({
      name: tableRangesOf.name,
      children: [
        it({
          name: 'READS THE RANGE OF A TABLE, from its header row to the end of its last row',
          fn: async () => {
            expect(tableRangesOf({ body: `她说\n\n${TABLE}`, },),).toEqual([
              {
                start: 4,
                end: (4 + TABLE.length) - 1,
              },
            ],);
          },
        },),

        it({
          name: 'READS EACH TABLE OF A TEXT IN TEXT ORDER, in a block quote and a list item too',
          fn: async () => {
            /**
             Three tables, the second in a block quote and the third in a list item.
             */
            const body = `${TABLE}\n> | e | f |\n> | - | - |\n\n- | g | h |\n  | - | - |\n`;
            expect(tableRangesOf({ body, },).length,).toBe(3,);
            expect(tableRangesOf({ body, },).map(function startOf({ start, },): number {
              return start;
            },),).toEqual([0, body.indexOf('> |',) + 2, body.indexOf('| g',),],);
          },
        },),

        it({
          name: 'READS NO TABLE IN A ROW WITHOUT A DELIMITER ROW, in a text without a pipe and in a text nested past '
            + 'the bound',
          fn: async () => {
            expect(tableRangesOf({ body: '| a | b |\n| c | d |\n', },),).toEqual([],);
            expect(tableRangesOf({ body: '猫\n', },),).toEqual([],);
            expect(tableRangesOf({ body: `${'> '.repeat(300,)}${TABLE}`, },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: footnoteDefinitionRangesOf.name,
      children: [
        it({
          name: 'ENDS A FOOTNOTE AT THE LINE AFTER AN UNCLOSED FENCE INDENTED UNDER IT, where the plain reading goes '
            + 'on',
          fn: async () => {
            expect(footnoteDefinitionRangesOf({ body: '[^1]: x\n    ~~~\n<>\n', },),).toEqual([
              {
                start: 0,
                end: 15,
              },
            ],);
          },
        },),

        it({
          name: 'KEEPS A LAZY LINE OF THE FOOTNOTE\'S PARAGRAPH IN THE FOOTNOTE, and ends it at a blank line and an '
            + 'unindented block',
          fn: async () => {
            expect(footnoteDefinitionRangesOf({ body: '[^1]: x\nlazy\n    more\nafter\n', },),).toEqual([
              {
                start: 0,
                end: 27,
              },
            ],);
            expect(footnoteDefinitionRangesOf({ body: '[^1]: x\n\n    para\n\nafter\n', },),).toEqual([
              {
                start: 0,
                end: 17,
              },
            ],);
          },
        },),

        it({
          name: 'READS NO FOOTNOTE AT A LINE INDENTED FOUR SPACES AFTER A PARAGRAPH, which is a lazy line of the paragraph',
          fn: async () => {
            expect(footnoteDefinitionRangesOf({ body: 'a `x\n    [^1]: {c}` {d}\n', },),).toEqual([],);
          },
        },),

        it({
          name: 'ENDS A FOOTNOTE AT A LINE OF RAW HTML, as a reader of the source in GFM does',
          fn: async () => {
            // A tag alone on a line starts an HTML block, which ends the footnote's paragraph there.
            expect(footnoteDefinitionRangesOf({ body: '[^1]: x\n<q>\n`y`\n', },),).toEqual([
              {
                start: 0,
                end: 11,
              },
            ],);
          },
        },),

        it({
          name: 'READS NO FOOTNOTE IN A TEXT WITHOUT A LABEL OPENER OR NESTED PAST THE BOUND',
          fn: async () => {
            expect(footnoteDefinitionRangesOf({ body: '猫\n', },),).toEqual([],);
            expect(footnoteDefinitionRangesOf({ body: `${'> '.repeat(300,)}[^1]: x\n`, },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: pageFootnoteDefinitionRangesOf.name,
      children: [
        it({
          name: 'KEEPS A LINE OF RAW HTML IN THE FOOTNOTE\'S PARAGRAPH, since the page writes it as text and the paragraph '
            + 'goes on',
          fn: async () => {
            // A tag alone on a line is a line of the paragraph, so the code span on the line after it is the footnote's.
            expect(pageFootnoteDefinitionRangesOf({ body: '[^1]: x\n<q>\n`y`\n', },),).toEqual([
              {
                start: 0,
                end: 15,
              },
            ],);
            // A tag with an attribute, and a definition on the line after it that a paragraph takes for a lazy line.
            expect(pageFootnoteDefinitionRangesOf({ body: '[^1]: x\n<a href="x">\n[r]: u\n', },),).toEqual([
              {
                start: 0,
                end: 27,
              },
            ],);
          },
        },),

        it({
          name: 'STARTS A FOOTNOTE AT A LINE INDENTED FOUR SPACES OR MORE AFTER A PARAGRAPH, since the page has no indented code',
          fn: async () => {
            expect(pageFootnoteDefinitionRangesOf({ body: 'a `x\n    [^1]: {c}` {d}\n', },),).toEqual([
              {
                start: 9,
                end: 23,
              },
            ],);
            expect(pageFootnoteDefinitionRangesOf({ body: 'a\n\t[^1]: x\n', },),).toEqual([
              {
                start: 3,
                end: 10,
              },
            ],);
          },
        },),

        it({
          name: 'READS NO FOOTNOTE IN A TEXT WITHOUT A LABEL OPENER OR NESTED PAST THE BOUND',
          fn: async () => {
            expect(pageFootnoteDefinitionRangesOf({ body: '猫\n', },),).toEqual([],);
            expect(pageFootnoteDefinitionRangesOf({ body: `${'> '.repeat(300,)}[^1]: x\n`, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
