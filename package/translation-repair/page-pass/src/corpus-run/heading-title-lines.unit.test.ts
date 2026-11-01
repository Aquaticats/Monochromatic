/**
 Tests for reading heading titles from either shape the corpus writes them
 in: an ATX line, or an HTML heading element alone on its line (the shape
 the XingZ60 memorial uses). Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { headingTitles, } from '../../dist/final/node/index.mjs';

await describe({
  name: headingTitles.name,
  children: [
    it({
      name: 'READS ATX AND HTML HEADING TITLES in line order, trimmed, and passes over every other line',
      fn: async () => {
        expect(headingTitles({ text: '### 猫\n\n它睡了。\n\n<h3 align = "center"> 猫猫 </h3>\n\n<p>猫</p>', },),)
          .toEqual([
            '猫',
            '猫猫',
          ],);
      },
    },),
    it({
      name: 'PASSES OVER A LINE THAT OPENS LIKE AN HTML HEADING BUT IS NONE: a tag whose sixth letter is no '
        + 'heading level, an opening tag that never closes, and a heading whose closing tag is missing or names '
        + 'another level (T8, twentieth batch)',
      fn: async () => {
        expect([
          '<hr>',
          '<header>猫</header>',
          '<h7>猫</h7>',
          '<h3 class="猫"',
          '<h3>猫',
          '<h3>猫</h4>',
        ].flatMap(function titlesOf(line,): readonly string[] {
          return headingTitles({ text: line, },);
        },),).toEqual([],);
      },
    },),
  ],
},);
