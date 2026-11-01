/**
 Tests for RSS feed date output.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Post, } from './content.ts';
import { generateLanguageRss, } from './rss.ts';

/**
 Older publication date that must not become item pubDate.
 */
const PUBLISHED_DATE = new Date('2026-04-16T10:01:43.000Z',);

/**
 Git-derived update date expected in the RSS item pubDate.
 */
const UPDATED_DATE = new Date('2026-05-14T08:31:54.000Z',);

/**
 Older post update date, included so lastBuildDate must choose the newest update.
 */
const OLDER_UPDATED_DATE = new Date('2026-05-10T06:04:54.000Z',);

/**
 Post fixture whose publication and update dates intentionally differ.
 */
const UPDATED_POST: Post = {
  lang: 'en',
  name: 'about',
  data: {
    title: 'About Aquaticat',
    description: 'UI/UX designer and developer bridging design and engineering.',
    tags: ['portfolio',],
    published: PUBLISHED_DATE,
    updated: UPDATED_DATE,
  },
  body: '# About',
  filePath: 'src/content/en/about.mdx',
  contentHash: 'fixture-content-hash-about',
};

/**
 Additional post fixture with an older update date.
 */
const OLDER_POST: Post = {
  lang: 'en',
  name: 'older',
  data: {
    title: 'Older post',
    description: 'Older post description.',
    tags: ['archive',],
    published: new Date('2026-04-01T00:00:00.000Z',),
    updated: OLDER_UPDATED_DATE,
  },
  body: '# Older',
  filePath: 'src/content/en/older.mdx',
  contentHash: 'fixture-content-hash-older',
};

await describe({
  name: generateLanguageRss.name,
  children: [
    it({
      name: 'uses git-derived updated dates for items and channel build date',
      fn: async function usesUpdatedDates(): Promise<void> {
        /**
         RSS XML generated from posts whose publication and update dates differ.
         */
        const xml = generateLanguageRss({
          lang: 'en',
          posts: [OLDER_POST, UPDATED_POST,],
          siteUrl: 'https://aquati.cat',
        },);

        expect(xml,).toContain('<lastBuildDate>Thu, 14 May 2026 08:31:54 GMT</lastBuildDate>',);
        expect(xml,).toContain('<pubDate>Thu, 14 May 2026 08:31:54 GMT</pubDate>',);
        expect(xml,).not.toContain('Thu, 16 Apr 2026 10:01:43 GMT',);
      },
    },),
  ],
},);
