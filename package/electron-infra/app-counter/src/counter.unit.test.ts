/**
 Unit tests for shared counter domain functions.
 
 Tests import from built `dist/app` so they verify the artifact the Electron
 app and tools consume, not sibling source.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  APP_TITLE_PREFIX,
  formatCount,
  formatDocumentTitle,
  incrementCount,
  INITIAL_COUNT,
  parseDocumentTitle,
  UNPARSEABLE_DOCUMENT_TITLE,
} from '../dist/app/counter.js';

/** Count value used for visible-label formatting. */
const formattedLabelCount = 7;

/** Count value used for document-title formatting. */
const formattedTitleCount = 3;

/** Count value used for document-title parsing. */
const parsedTitleCount = 9;

await describe({
  name: '',
  children: [
    describe({
      name: incrementCount.name,
      children: [
        it({
          name: 'increments initial count by one',
          fn: async () => {
            expect(incrementCount({ current: INITIAL_COUNT, },),).toBe(1,);
          },
        },),
      ],
    },),
    describe({
      name: formatCount.name,
      children: [
        it({
          name: 'formats visible count text',
          fn: async () => {
            expect(formatCount({ count: formattedLabelCount, },),).toBe('Count: 7',);
          },
        },),
      ],
    },),
    describe({
      name: formatDocumentTitle.name,
      children: [
        it({
          name: 'formats parseable title with app prefix',
          fn: async () => {
            expect(formatDocumentTitle({ count: formattedTitleCount, },),).toBe(
              `${APP_TITLE_PREFIX} :: count=3`,
            );
          },
        },),
      ],
    },),
    describe({
      name: parseDocumentTitle.name,
      children: [
        it({
          name: 'parses title produced by formatter',
          fn: async () => {
            expect(parseDocumentTitle({
              title: formatDocumentTitle({ count: parsedTitleCount, },),
            },),).toBe(parsedTitleCount,);
          },
        },),
        it({
          name: 'ignores unrelated title prefixes',
          fn: async () => {
            expect(parseDocumentTitle({ title: 'Other :: count=1', },),).toBe(
              UNPARSEABLE_DOCUMENT_TITLE,
            );
          },
        },),
        it({
          name: 'ignores missing count suffix',
          fn: async () => {
            expect(parseDocumentTitle({ title: `${APP_TITLE_PREFIX} :: count=`, },),)
              .toBe(UNPARSEABLE_DOCUMENT_TITLE,);
          },
        },),
        it({
          name: 'ignores nonnumeric count suffix',
          fn: async () => {
            expect(parseDocumentTitle({ title: `${APP_TITLE_PREFIX} :: count=nan`, },),)
              .toBe(UNPARSEABLE_DOCUMENT_TITLE,);
          },
        },),
      ],
    },),
  ],
},);
