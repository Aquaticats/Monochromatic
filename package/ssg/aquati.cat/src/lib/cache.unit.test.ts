/**
 Tests for the build cache's locale-aware lookup.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CACHE_MISS,
  type BuildManifest,
  type CacheEntry,
  createCacheEntry,
  getCachedEntry,
  type PostFrontmatter,
} from '../../dist/final/node/index.mjs';

/**
 Cache key the fixture manifest stores its single entry under.
 */
const FIXTURE_KEY = 'src/content/en/about.mdx';

/**
 Content hash shared by the cache fixtures.
 */
const FIXTURE_HASH = 'fixture-content-hash';

/**
 Frontmatter the cached entry carries.
 */
const FIXTURE_FRONTMATTER: PostFrontmatter = {
  title: 'About',
  description: 'About the author',
  tags: ['portfolio',],
  published: new Date('2026-04-16T10:01:43.000Z',),
  publishedOffsetMinutes: 0,
  updated: new Date('2026-05-14T08:31:54.000Z',),
  updatedOffsetMinutes: 0,
};

/**
 Manifest holding one entry rendered in English.
 */
const FIXTURE_MANIFEST: BuildManifest = {
  pipelineHash: 'fixture-pipeline-hash',
  headSha: 'fixture-head-sha',
  content: {
    [FIXTURE_KEY]: createCacheEntry({
      contentHash: FIXTURE_HASH,
      html: '<p>About</p>',
      frontmatter: FIXTURE_FRONTMATTER,
      lang: 'en',
    },),
  },
};

/**
 Looks the fixture entry up as a build for one locale would.
 
 @param lang - locale the current build would render in
 
 @param contentHash - hash to look up, defaulting to the stored one
 
 @returns the cached entry, or `CACHE_MISS`
 
 @example
 ```ts
 lookup({ lang: 'en' });
 ```
 */
function lookup({
  lang,
  contentHash = FIXTURE_HASH,
}: {
  readonly lang: string;
  readonly contentHash?: string;
},): CacheEntry | typeof CACHE_MISS {
  return getCachedEntry({
    manifest: FIXTURE_MANIFEST,
    filePath: FIXTURE_KEY,
    contentHash,
    lang,
  },);
}

await describe({
  name: 'cache locale key',
  children: [
    it({
      name: 'returns the entry for the locale it was rendered in',
      fn: async function returnsEntryForItsLocale(): Promise<void> {
        /**
         Lookup for the stored locale.
         */
        const entry = lookup({ lang: 'en', },);

        expect(entry,).not.toBe(CACHE_MISS,);
        if (entry === CACHE_MISS)
          throw new Error('Expected a cache hit for the stored locale.',);
        expect(entry.html,).toBe('<p>About</p>',);
      },
    },),
    it({
      name: 'misses when the requested locale differs from the cached render',
      fn: async function missesAcrossLocales(): Promise<void> {
        expect(lookup({ lang: 'ca', },),).toBe(CACHE_MISS,);
      },
    },),
    it({
      name: 'misses when the content hash differs',
      fn: async function missesOnStaleHash(): Promise<void> {
        expect(lookup({
          lang: 'en',
          contentHash: 'different-hash',
        },),).toBe(CACHE_MISS,);
      },
    },),
  ],
},);
