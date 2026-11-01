/**
 Tests for full post page date rendering and metadata.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Post, } from '../lib/content.ts';

import { postPage, } from './post.ts';

/**
 Git-derived publication date used by the post fixture.
 */
const PUBLISHED_DATE = new Date('2026-04-16T10:01:43.000Z',);

/**
 Git-derived modification date used by the post fixture.
 */
const UPDATED_DATE = new Date('2026-05-14T08:31:54.000Z',);

/**
 Minimal post fixture covering full post rendering.
 */
const FIXTURE_POST: Post = {
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
  contentHash: 'fixture-content-hash',
};

await describe({
  name: postPage.name,
  children: [
    it({
      name: 'renders git-derived dates on full post pages',
      fn: async function rendersGitDerivedDates(): Promise<void> {
        /**
         Complete post page rendered from the fixture post.
         */
        const html = postPage({
          post: FIXTURE_POST,
          lang: 'en',
          name: 'about',
          renderedHtml: '<p>Rendered post body.</p>',
          canonicalUrl: 'https://aquati.cat/en/about',
          availableInLangs: ['en',],
        },);

        expect(html,).toContain('Published: <time datetime="2026-04-16T10:01:43.000Z"',);
        expect(html,).toContain('Updated: <time datetime="2026-05-14T08:31:54.000Z"',);
      },
    },),
    it({
      name: 'emits Open Graph article dates from git-derived dates',
      fn: async function emitsOpenGraphArticleDates(): Promise<void> {
        /**
         Complete post page rendered from the fixture post.
         */
        const html = postPage({
          post: FIXTURE_POST,
          lang: 'en',
          name: 'about',
          renderedHtml: '<p>Rendered post body.</p>',
          canonicalUrl: 'https://aquati.cat/en/about',
          availableInLangs: ['en',],
        },);

        expect(html,).toContain('property="og:type" content="article"',);
        expect(html,).toContain(
          'property="article:published_time" content="2026-04-16T10:01:43.000Z"',
        );
        expect(html,).toContain(
          'property="article:modified_time" content="2026-05-14T08:31:54.000Z"',
        );
      },
    },),
  ],
},);
