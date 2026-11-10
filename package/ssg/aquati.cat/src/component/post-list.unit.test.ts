/**
 Tests for the post grid markup.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type Post,
  postListHtml,
} from '../../dist/final/node/index.mjs';

/**
 Git-derived publication date used by the post fixtures.
 */
const PUBLISHED_DATE = new Date('2026-04-16T10:01:43.000Z',);

/**
 Git-derived modification date used by the post fixtures.
 */
const UPDATED_DATE = new Date('2026-05-14T08:31:54.000Z',);

/**
 Builds a post fixture with one tag so the card renders its full tag list.
 
 @param name - slug used for the card link and the rendered title
 
 @returns post fixture ready for the grid
 
 @example
 ```ts
 const post = createPost('about');
 ```
 */
function createPost(name: string,): Post {
  return {
    lang: 'en',
    name,
    data: {
      title: `Post ${name}`,
      description: `Description for ${name}`,
      tags: ['accessibility',],
      published: PUBLISHED_DATE,
      updated: UPDATED_DATE,
    },
    body: '# Body',
    filePath: `src/content/en/${name}.mdx`,
    contentHash: `fixture-content-hash-${name}`,
  };
}

/**
 Two posts, enough to render a grid with more than one card.
 */
const FIXTURE_POSTS = [
  createPost('about',),
  createPost('contact',),
] as const;

/**
 Opening of a list item that holds one post card, as the grid must emit it.
 */
const LIST_ITEM_THEN_CARD = '<ul><li><post-card';

await describe({
  name: 'post grid markup',
  children: [
    it({
      name: 'puts the list item directly inside the list',
      fn: async function putsListItemInsideList(): Promise<void> {
        /**
         Rendered grid HTML.
         */
        const html = postListHtml(FIXTURE_POSTS,);

        expect(html,).toContain(LIST_ITEM_THEN_CARD,);
      },
    },),
    it({
      name: 'never places a custom element where a list item belongs',
      fn: async function neverPutsCustomElementInList(): Promise<void> {
        /**
         Rendered grid HTML.
         */
        const html = postListHtml(FIXTURE_POSTS,);

        expect(html,).not.toContain('<ul><post-card',);
        expect(html,).not.toContain('</post-card></ul>',);
      },
    },),
  ],
},);
