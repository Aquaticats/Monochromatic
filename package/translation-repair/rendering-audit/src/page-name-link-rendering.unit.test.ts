/**
 Tests which archive link renders which link of the original, where both
 documents link one href. An href names no one link: a page links one
 destination under a name in one sentence and under other words in another,
 and a table keyed by href gave every link of the original to it the last
 text the archive wrote for it. Cat-themed invention throughout; no corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { renderedLinks, } from '../dist/final/node/index.mjs';

/**
 Destination the fixtures link a cat's page under.
 */
const MIMI = 'https://example.invalid/mimi';

/**
 Destination the fixtures link a cat's diary under.
 */
const DIARY = 'https://example.invalid/diary';

await describe({
  name: renderedLinks.name,
  children: [
    it({
      name: 'GIVES EVERY LINK THE ONE TEXT THE ARCHIVE LINKS ITS HREF UNDER, however often either side links it, '
        + 'and LEAVES OUT a link whose href the archive never links',
      fn: async () => {
        expect(renderedLinks({
          sourceLinks: [
            { text: '咪咪', href: MIMI, },
            { text: '日记', href: DIARY, },
            { text: '她', href: MIMI, },
          ],
          archiveLinks: [
            { text: 'Mimi', href: MIMI, },
            { text: 'Mimi', href: MIMI, },
            { text: 'Mimi', href: MIMI, },
          ],
        },),).toEqual([
          { text: '咪咪', href: MIMI, rendering: 'Mimi', },
          { text: '她', href: MIMI, rendering: 'Mimi', },
        ],);
      },
    },),
    it({
      name: 'PAIRS BY PLACE AMONG THE LINKS TO ONE HREF where the archive links it under more than one text as '
        + 'often as the original does, and KEEPS PAGE ORDER across hrefs',
      fn: async () => {
        expect(renderedLinks({
          sourceLinks: [
            { text: '咪咪', href: MIMI, },
            { text: '日记', href: DIARY, },
            { text: '这里', href: MIMI, },
          ],
          archiveLinks: [
            { text: 'the diary', href: DIARY, },
            { text: 'Mimi', href: MIMI, },
            { text: 'here', href: MIMI, },
          ],
        },),).toEqual([
          { text: '咪咪', href: MIMI, rendering: 'Mimi', },
          { text: '日记', href: DIARY, rendering: 'the diary', },
          { text: '这里', href: MIMI, rendering: 'here', },
        ],);
      },
    },),
    it({
      name: 'LEAVES OUT EVERY LINK TO AN HREF THE ARCHIVE LINKS UNDER MORE THAN ONE TEXT a different number of '
        + 'times than the original, whichever side links it more, and READS the page\'s other hrefs',
      fn: async () => {
        expect(renderedLinks({
          sourceLinks: [
            { text: '咪咪', href: MIMI, },
            { text: '日记', href: DIARY, },
            { text: '日记本', href: DIARY, },
            { text: '那本日记', href: DIARY, },
            { text: '猫', href: 'https://example.invalid/cat', },
          ],
          archiveLinks: [
            { text: 'Mimi', href: MIMI, },
            { text: 'her page', href: MIMI, },
            { text: 'the diary', href: DIARY, },
            { text: 'that diary', href: DIARY, },
            { text: 'cat', href: 'https://example.invalid/cat', },
          ],
        },),).toEqual([
          { text: '猫', href: 'https://example.invalid/cat', rendering: 'cat', },
        ],);
      },
    },),
    it({
      name: 'READS NOTHING where either document carries no link',
      fn: async () => {
        expect([
          renderedLinks({
            sourceLinks: [],
            archiveLinks: [{ text: 'Mimi', href: MIMI, },],
          },),
          renderedLinks({
            sourceLinks: [{ text: '咪咪', href: MIMI, },],
            archiveLinks: [],
          },),
        ],).toEqual([
          [],
          [],
        ],);
      },
    },),
  ],
},);
