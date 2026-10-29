/**
 Tests for the table the plain reading starts on the label line of a footnote definition, where the page reads none.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  bodyStartsOnLabelLine,
  withoutLabelLineTable,
} from '../../dist/final/node/index.mjs';

await describe({
  name: 'the table on the label line of a footnote definition',
  children: [
    describe({
      name: withoutLabelLineTable.name,
      children: [
        it({
          name: 'WRITES THE DELIMITER ROW WITH LETTERS WHERE THE PARSE WOULD START A TABLE ON THE FIRST LINE',
          fn: async () => {
            expect(withoutLabelLineTable({ body: '`{||\n-|-\n`', },),).toBe('`{||\nx|x\n`',);
            expect(withoutLabelLineTable({ body: '|a|b|\n|:-|-:|\n', },),).toBe('|a|b|\n|xx|xx|\n',);
          },
        },),
        it({
          name: 'LEAVES A BODY WHOSE FIRST LINES START NO TABLE AS IT STANDS',
          fn: async () => {
            expect(withoutLabelLineTable({ body: 'x\n-|-\n', },),).toBe('x\n-|-\n',);
            expect(withoutLabelLineTable({ body: 'a b\nc', },),).toBe('a b\nc',);
            expect(withoutLabelLineTable({ body: 'cat\n\na||\n-|-\n', },),).toBe('cat\n\na||\n-|-\n',);
          },
        },),
      ],
    },),
    describe({
      name: bodyStartsOnLabelLine.name,
      children: [
        it({
          name: 'SAYS WHETHER THE LABEL AND THE COLON STAND BEFORE THE BODY ON ITS LINE',
          fn: async () => {
            expect(bodyStartsOnLabelLine({ source: '> [^1]: x', start: 8, },),).toBe(true,);
            expect(bodyStartsOnLabelLine({ source: '[^1]:\n    x', start: 10, },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
