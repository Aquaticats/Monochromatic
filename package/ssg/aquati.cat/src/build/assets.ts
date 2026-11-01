/**
 Asset generation for the SSG build.
 
 Generates CSS, RSS feeds, and copies static assets to dist.
 */
import {
  copyFile,
  mkdir,
} from 'node:fs/promises';
import {
  dirname,
  join,
  relative,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import readdir from 'tiny-readdir-glob';

import type { Locale, } from '../i18n/index.ts';

import type { Post, } from '../lib/content.ts';
import { generateLanguageRss, } from '../lib/rss.ts';
import type { Logger, } from '../lib/types.ts';
import { generateSiteCss, } from '../style/base.ts';
import {
  DIST,
  writePage,
} from './write-page.ts';

/**
 Copies all files from a source directory into dist, preserving relative structure.
 Creates target directories before copying.
 
 @param sourceDir - base directory to copy from
 
 @param files - absolute file paths to copy
 */
async function copyTreeToDist(
  {
    sourceDir,
    files,
  }: {
    readonly sourceDir: string;
    readonly files: readonly string[];
  },
): Promise<void> {
  /**
   Deduplicated parent directories pre-created before the per-file copy fan-out.
   */
  const targetDirs = [...new Set(
    files.map(function targetDir(filePath,) {
      return join(
        DIST,
        dirname(relative(
          sourceDir,
          filePath,
        ),),
      );
    },),
  ),];

  await Promise.all(
    targetDirs.map(function ensureDir(dir,) {
      return mkdir(
        dir,
        { recursive: true, },
      );
    },),
  );

  await Promise.all(
    files.map(function copyOneFile(filePath,) {
      return copyFile(
        filePath,
        join(
          DIST,
          relative(
            sourceDir,
            filePath,
          ),
        ),
      );
    },),
  );
}

/**
 Generates CSS, RSS feeds, and copies static assets to dist.
 
 @param siteUrl - base URL for RSS feed links
 
 @param contentDir - content source directory path
 
 @param byLang - posts grouped by locale (pre-computed by build orchestrator)
 
 @param validLangs - locale codes present in the content
 
 @param l - parent logger for tagged output
 
 
 @example
 ```ts
 await generateAssets({ siteUrl: 'https://example.com', contentDir: 'src/content', byLang, validLangs, l: rootLogger });
 ```
 */
export async function generateAssets(
  {
    siteUrl,
    contentDir,
    byLang,
    validLangs,
    l,
  }: {
    readonly siteUrl: string;
    readonly contentDir: string;
    readonly byLang: ReadonlyMap<Locale, readonly Post[]>;
    readonly validLangs: readonly Locale[];
    readonly l: Logger;
  },
): Promise<void> {
  /**
   Parent logger retained by tagged wrapper.
   */
  const parentLogger = l;
  /**
   Function-scoped logger tagged with the caller name for traceable log lines.
   */
  const childLogger = tagged({
    tag: generateAssets.name,
    l: parentLogger,
  },);

  /**
   Per-language RSS write promises kicked off concurrently.
   */
  const rssWrites = validLangs.map(function writeRss(lang,) {
    /**
     Posts narrowed to this locale; absent locales yield an empty feed instead of an error.
     */
    const langPosts = byLang.get(lang,)
      ?? [];
    /**
     Pre-rendered XML body written to `{lang}/rss.xml`.
     */
    const rssXml = generateLanguageRss({
      lang,
      posts: langPosts,
      siteUrl,
    },);
    return writePage({
      relativePath: `${lang}/rss.xml`,
      content: rssXml,
    },);
  },);

  // Copies all content files (including MDX source) to dist intentionally,
  // so readers can inspect the original source of any post.
  /**
   Directory listings for content and public trees fetched concurrently.
   */
  const [contentResult, publicResult,] = await Promise.all([
    readdir(`${contentDir}/**/*`,),
    readdir('public/**/*',),
  ],);

  /**
   robots.txt allowing all crawlers with sitemap references.
   */
  const robotsTxt = [
    'User-agent: *',
    'Allow: /',
    '',
    ...validLangs.map(function sitemapEntry(lang,) {
      return `Sitemap: ${siteUrl}/${lang}/rss.xml`;
    },),
    '',
  ]
    .join('\n',);

  await Promise.all([
    writePage({
      relativePath: 'styles.css',
      content: generateSiteCss(),
    },),
    writePage({
      relativePath: 'robots.txt',
      content: robotsTxt,
    },),
    ...rssWrites,
    copyTreeToDist({
      sourceDir: contentDir,
      files: contentResult.files,
    },),
    copyTreeToDist({
      sourceDir: 'public',
      files: publicResult.files,
    },),
  ],);

  childLogger.info('generated CSS, RSS, robots.txt, and static assets',);
}
