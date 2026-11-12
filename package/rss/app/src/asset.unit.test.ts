/**
 The reader page's document head carries one non-empty `<title>`, so tabs,
 history entries, bookmarks, and assistive technology announce a page name
 instead of the URL.

 Regression cover for a head that emitted charset, viewport, stylesheet, and
 script while never naming the document.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { indexHtmlStart, } from '../dist/final/node/testing.mjs';

/**
 Title text every opening tag of the reader page announces.
 */
const EXPECTED_TITLE = 'RSS Reader';

/**
 Every `<title>` element the opening fragment emits, with its text.
 */
function titleElements(html: string,): RegExpMatchArray[] {
  return [...html.matchAll(/<title>([^<]*)<\/title>/g,)];
}

await describe({
  name: '',
  children: [
    describe({
      name: 'indexHtmlStart',
      children: [
        it({
          name: 'emits exactly one title element in the head',
          fn: async () => {
            /**
             Title elements found in the opening fragment.
             */
            const titles = titleElements(indexHtmlStart,);

            expect(titles,).toHaveLength(1,);
          },
        },),
        it({
          name: 'names the document in that title element',
          fn: async () => {
            /**
             Text of the single title element, or undefined when none exists.
             */
            const titleText = titleElements(indexHtmlStart,)[0]?.[1];

            expect(titleText,).toBe(EXPECTED_TITLE,);
          },
        },),
        it({
          name: 'keeps the title inside the head, before the body opens',
          fn: async () => {
            /**
             Offset of the title element in the fragment.
             */
            const titleOffset = indexHtmlStart.indexOf('<title>',);
            /**
             Offset of the closing head tag the title must precede.
             */
            const headEndOffset = indexHtmlStart.indexOf('</head>',);

            expect(titleOffset,).toBeGreaterThan(-1,);
            expect(titleOffset,).toBeLessThan(headEndOffset,);
          },
        },),
      ],
    },),
  ],
},);
