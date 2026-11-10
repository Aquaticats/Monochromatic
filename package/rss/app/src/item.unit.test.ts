/**
 Regression coverage for Atom entries reaching the page renderer as
 feedsmith's structured values instead of the strings the RSS-shaped item
 declares, which made the render throw `raw.replaceAll is not a function`.
 */

import { parseAtomFeed, } from 'feedsmith';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { itemToFeed, } from './html-item.ts';
import { getSortedItems, } from './item.ts';

/**
 Link the fixture entry points at, asserted on the rendered anchor.
 */
const ENTRY_LINK = 'https://example.com/1';

/**
 Atom document with one entry carrying a title, a link, and inline content,
 the three fields normalization used to copy through unwrapped.
 */
const ATOM_WITH_ONE_ENTRY = `<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example Feed</title>
  <updated>2025-01-01T00:00:00Z</updated>
  <entry>
    <title>Hello</title>
    <link href="${ENTRY_LINK}" rel="alternate"/>
    <id>urn:uuid:1</id>
    <updated>2025-01-01T00:00:00Z</updated>
    <content type="html">Body text</content>
  </entry>
</feed>`;

await describe({
  name: '',
  children: [
    describe({
      name: getSortedItems.name,
      children: [
        it({
          name: 'normalizes Atom entry title, link, and content into strings',
          fn: async () => {
            const items = getSortedItems([{
              feed: parseAtomFeed(ATOM_WITH_ONE_ENTRY,),
              outline: {
                type: 'atom',
                text: 'Example Feed',
              },
            },],);

            expect(items,).toHaveLength(1,);
            expect(items[0]?.item.title,).toBe('Hello',);
            expect(items[0]?.item.link,).toBe(ENTRY_LINK,);
            expect(items[0]?.item.description,).toBe('Body text',);
          },
        },),
      ],
    },),
    describe({
      name: itemToFeed.name,
      children: [
        it({
          name: 'renders an Atom entry title, link, and content without throwing',
          fn: async () => {
            const items = getSortedItems([{
              feed: parseAtomFeed(ATOM_WITH_ONE_ENTRY,),
              outline: {
                type: 'atom',
                text: 'Example Feed',
              },
            },],);
            const [itemWDate] = items;
            if (itemWDate === undefined)
              throw new Error('the fixture Atom feed yielded no item',);

            const html = itemToFeed({
              itemWDate,
              index: 0,
            },);

            expect(html,).toContain(`href="${ENTRY_LINK}"`,);
            expect(html,).toContain('>Hello<',);
            expect(html,).toContain(encodeURIComponent('Body text',),);
          },
        },),
      ],
    },),
  ],
},);
