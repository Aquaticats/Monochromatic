/**
 Tests the reading of where the strict grammar reads a table or a footnote
 definition in a text. Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  footnoteDefinitionRangesOf,
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
          name: 'READS NO FOOTNOTE IN A TEXT WITHOUT A LABEL OPENER OR NESTED PAST THE BOUND',
          fn: async () => {
            expect(footnoteDefinitionRangesOf({ body: '猫\n', },),).toEqual([],);
            expect(footnoteDefinitionRangesOf({ body: `${'> '.repeat(300,)}[^1]: x\n`, },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
