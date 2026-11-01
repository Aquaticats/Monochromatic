/**
 Static site build script.
 
 Orchestrates the full SSG pipeline:
 1. Load and validate MDX content
 2. Process changed files through the unified pipeline (with caching)
 3. Generate all HTML pages via h-html templates
 4. Generate CSS via h-css
 5. Generate RSS feeds per language
 6. Copy static assets from public/
 7. Post-process: zstd compression
 
 Run via `mise run build:site` or `node src/build.ts`.
 */
import type { ReadonlyDeep, } from 'type-fest';
import { relative, } from 'node:path';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { generateAssets, } from './build/assets.ts';
import { ensureFavicons, } from './build/favicon.ts';
import { generatePages, } from './build/pages.ts';
import {
  buildManifest,
  CACHE_MISS,
  type CacheEntry,
  computePipelineFingerprint,
  createCacheEntry,
  getCachedEntry,
  NO_CACHE,
  readCache,
  writeCache,
} from './lib/cache.ts';
import { groupByLang, } from './lib/content-group.ts';
import {
  attachDates,
  loadContent,
  type ResolvedDates,
} from './lib/content.ts';
import { warnOnAuthoredDateDivergence, } from './lib/date-divergence.ts';
import {
  getPostDates,
  resolveGitDatesContext,
} from './lib/git-dates.ts';
import { renderMdx, } from './lib/markdown.ts';

// File justification: 120 lines: linear pipeline script; splitting the
// orchestration across multiple files would scatter the build sequence.
 // module boundary marker

/**
 Tagged logger for the build pipeline.
 */
const l = tagged({
  tag: 'build',
  l: logger,
},);

/**
 Site base URL for RSS feed links.
 */
const SITE_URL = 'https://aquati.cat';

/**
 Content source directory.
 */
const CONTENT_DIR = 'src/content';

/**
 Glob matching all pipeline source files for cache invalidation.
 Changes to any matched file invalidate all cached content entries.
 */
const PIPELINE_GLOB = 'src/{lib,components,client,i18n}/**/*.ts';

//region Build orchestration: loads content, processes MDX, generates pages and assets

l.info('starting',);

/**
 Loaded posts, cached-manifest result, pipeline hash, and git-dates context fetched concurrently.
 */
const [loadedPosts, cacheResult, pipelineHash, gitCtx,] = await Promise.all([
  loadContent(CONTENT_DIR,),
  readCache({ l, },),
  computePipelineFingerprint(PIPELINE_GLOB,),
  resolveGitDatesContext(),
],);

l.info(`loaded ${loadedPosts.length} posts`,);

/**
 Manifest when one was read, or `undefined` (inferred local, never a written nullish union) when absent.
 */
const cache = cacheResult === NO_CACHE ? undefined : cacheResult;

/**
 Whether the processing pipeline source has changed since last build.
 */
const pipelineChanged = cache?.pipelineHash
  !== pipelineHash;
if (pipelineChanged)
  l.info('pipeline changed, invalidating all cache entries',);

/**
 Cache to use for lookups; `undefined` forces full reprocessing on pipeline change.
 */
const effectiveCache = pipelineChanged ? undefined : cache;

/**
 Whether cached git-derived dates can be reused without re-probing.
 Safe when HEAD is unchanged since the cache was written and the pipeline
 has not changed (pipeline change already invalidates all cache entries).
 */
const gitDatesReusable = (!pipelineChanged) && (cache?.headSha
  === gitCtx
    .headSha);

/**
 Map from absolute file path to resolved publication/update dates.
 */
const datesByFilePath = new Map<string, ResolvedDates>();

await Promise.all(loadedPosts.map(async function resolveDates(post,) {
  /**
   Repo-relative path used as the manifest key for this post.
   */
  const cacheKey = relative(
    '.',
    post.filePath,
  );
  if (gitDatesReusable) {
    /**
     Prior manifest entry reused only when both pipeline and git head are unchanged.
     */
    const cached = effectiveCache === undefined
      ? CACHE_MISS
      : getCachedEntry({
        manifest: effectiveCache,
        filePath: cacheKey,
        contentHash: post.contentHash,
      },);
    if (cached !== CACHE_MISS) {
      /**
       Cached dates reused when the git HEAD still matches the manifest.
       */
      const dates = {
        published: cached.frontmatter
          .published,
        updated: cached.frontmatter
          .updated,
      };
      warnOnAuthoredDateDivergence({
        authoredDates: post.authoredDates,
        resolvedDates: dates,
        filePath: post.filePath,
        l,
      },);
      datesByFilePath.set(
        post.filePath,
        dates,
      );
      return;
    }
  }

  /**
   Freshly resolved publication and update timestamps when no cache hit applies.
   */
  const dates = await getPostDates({
    filePath: post.filePath,
    isShallow: gitCtx.isShallow,
    githubSlug: gitCtx.githubSlug,
    l,
  },);
  warnOnAuthoredDateDivergence({
    authoredDates: post.authoredDates,
    resolvedDates: dates,
    filePath: post.filePath,
    l,
  },);
  datesByFilePath.set(
    post.filePath,
    dates,
  );
},),);

/**
 Fully-resolved posts with `published`/`updated` attached, sorted by updated desc.
 */
const posts = attachDates({
  loadedPosts,
  datesByFilePath,
},);

/**
 Results from processing each post: rendered HTML and cache entry.
 */
const processResults = await Promise.all(posts.map(async function processPost(
  post,
): Promise<ReadonlyDeep<{
  contentKey: string;
  cacheKey: string;
  entry: CacheEntry;
  fromCache: boolean;
}>> {
  /**
   Repo-relative path used as the manifest key for this post.
   */
  const cacheKey = relative(
    '.',
    post.filePath,
  );
  /**
   Prior manifest entry reused when the post content hash is unchanged.
   */
  const cached = effectiveCache === undefined
    ? CACHE_MISS
    : getCachedEntry({
      manifest: effectiveCache,
      filePath: cacheKey,
      contentHash: post.contentHash,
    },);

  if (cached !== CACHE_MISS) {
    /* Reuse rendered HTML; overlay freshly-resolved dates onto cached frontmatter
     * so downstream consumers see dates consistent with the current HEAD even
     * when `gitDatesReusable` was false. */
    /**
     Cached manifest entry with overlaid current-HEAD dates.
     */
    const entry = {
      contentHash: cached.contentHash,
      html: cached.html,
      frontmatter: {
        ...cached.frontmatter,
        published: post.data
          .published,
        updated: post.data
          .updated,
      },
    };
    return {
      contentKey: `${post.lang}/${post.name}`,
      cacheKey,
      entry,
      fromCache: true,
    };
  }

  /**
   Freshly rendered post body when no cache hit applies.
   */
  const html = await renderMdx(post.body,);
  /**
   New manifest entry persisted for future builds.
   */
  const entry = createCacheEntry({
    contentHash: post.contentHash,
    html,
    frontmatter: post.data,
  },);
  return {
    contentKey: `${post.lang}/${post.name}`,
    cacheKey,
    entry,
    fromCache: false,
  };
},),);

/**
 Map of `lang/name` content keys to rendered HTML strings.
 */
const renderedContent = new Map(
  processResults.map(function toContentEntry({
    contentKey,
    entry,
  },) {
    return [
      contentKey,
      entry.html,
    ] as const;
  },),
);

/**
 Cache entries keyed by relative file path for manifest persistence.
 */
const cacheEntries = Object.fromEntries(
  processResults.map(function toCacheEntry({
    cacheKey,
    entry,
  },) {
    return [
      cacheKey,
      entry,
    ];
  },),
);

/**
 Number of posts served from cache without reprocessing.
 */
const cacheHits = processResults
  .filter(function wasFromCache({ fromCache, },) {
    return fromCache;
  },)
  .length;

l.info(`processed ${posts.length
  - cacheHits} files, ${cacheHits} from cache`,);

/**
 Posts grouped by locale, computed once for both page and asset generation.
 */
const byLang = groupByLang(posts,);

/**
 Valid locale codes present in the loaded content; map keys are already `Locale`.
 */
const validLangs = [...byLang.keys(),];

await ensureFavicons({ l, },);
await Promise.all([
  generatePages({
    posts,
    renderedContent,
    siteUrl: SITE_URL,
    byLang,
    validLangs,
    l,
  },),
  generateAssets({
    siteUrl: SITE_URL,
    contentDir: CONTENT_DIR,
    byLang,
    validLangs,
    l,
  },),
],);
await writeCache(buildManifest({
  pipelineHash,
  headSha: gitCtx.headSha,
  entries: cacheEntries,
},),);
l.info('done',);

//endregion Build orchestration
