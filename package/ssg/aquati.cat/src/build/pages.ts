/**
 HTML page generation for the SSG build.
 
 Generates all HTML pages from posts and rendered content,
 writing each to the dist directory.
 */
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { Locale, } from '../i18n/index.ts';

import {
  groupByName,
  groupByTag,
} from '../lib/content-group.ts';
import type { Post, } from '../lib/content.ts';
import type { Logger, } from '../lib/types.ts';
import { indexPage, } from '../page/index.ts';
import { langPage, } from '../page/lang.ts';
import { namePage, } from '../page/name.ts';
import { postPage, } from '../page/post.ts';
import { tagPage, } from '../page/tag.ts';
import { writePage, } from './write-page.ts';

/**
 Generates all HTML pages from posts and rendered content.
 
 @param posts - all loaded posts
 
 @param renderedContent - map of `lang/name` to rendered HTML
 
 @param siteUrl - base URL for canonical link construction
 
 @param byLang - posts grouped by locale (pre-computed by build orchestrator)
 
 @param validLangs - locale codes present in the content
 
 @param l - parent logger for tagged output
 
 
 @example
 ```ts
 await generatePages({ posts, renderedContent, siteUrl: 'https://example.com', byLang, validLangs, l: rootLogger });
 ```
 */
export async function generatePages(
  {
    posts,
    renderedContent,
    siteUrl,
    byLang,
    validLangs,
    l,
  }: {
    readonly posts: readonly Post[];
    readonly renderedContent: ReadonlyMap<string, string>;
    readonly siteUrl: string;
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
    tag: generatePages.name,
    l: parentLogger,
  },);
  /**
   Posts grouped by slug name so the name page can iterate translation siblings.
   */
  const byName = groupByName(posts,);
  /**
   Distinct post slugs serving as the index for cross-locale write fan-out.
   */
  const names = Object.keys(byName,);

  /**
   Locale each post slug has a translation in.
   */
  const availableLangsByName: Record<string, readonly Locale[]> = Object
    .fromEntries(
    Object.entries(byName,)
      .map(function pickAvailable([name, namePosts,],) {
        return [
          name,
          namePosts.map(function langOf(p,) {
            return p.lang;
          },),
        ] as const;
      },),
    );

  /**
   Flat list of write promises gathered before the single Promise.all flush at the end.
   */
  const writes = [
    writePage({
      relativePath: 'index.html',
      content: indexPage({
        langs: validLangs,
        canonicalUrl: `${siteUrl}/`,
      },),
    },),
    ...validLangs.flatMap(function langWrites(lang,) {
      /**
       Posts narrowed to this locale; absent locales yield an empty list instead of an error.
       */
      const langPosts = byLang.get(lang,)
        ?? [];
      /**
       Per-locale tag bucketing computed once per language pass.
       */
      const langTags = groupByTag(langPosts,);
      return [
        writePage({
          relativePath: `${lang}/index.html`,
          content: langPage({
            lang,
            posts: langPosts,
            canonicalUrl: `${siteUrl}/${lang}/`,
          },),
        },),
        ...names.map(function postWrite(name,) {
          /**
           Locale-specific post for this slug or undefined when no translation exists.
           */
          const post = langPosts.find(function matchName(lp,) {
            return lp.name
              === name;
          },);
          /**
           Pre-rendered MDX body keyed by `lang/name`; absent for missing translations.
           */
          const html = renderedContent.get(`${lang}/${name}`,);
          return writePage({
            relativePath: `${lang}/${name}.html`,
            content: postPage({
              ...(post !== undefined ? { post, } : {}),
              lang,
              name,
              ...(html !== undefined ? { renderedHtml: html, } : {}),
              canonicalUrl: `${siteUrl}/${lang}/${name}`,
              availableInLangs: availableLangsByName[name]
                ?? [],
            },),
          },);
        },),
        ...Object.entries(langTags,)
          .map(function tagWrite([tag, tagPosts,],) {
            return writePage({
              relativePath: `${lang}/tag/${tag}.html`,
              content: tagPage({
                tag,
                lang,
                posts: tagPosts,
                canonicalUrl: `${siteUrl}/${lang}/tag/${tag}`,
              },),
            },);
          },),
      ];
    },),
    ...names.map(function nameWrite(name,) {
      /**
       Cross-language translations for this slug feeding the language picker on the name page.
       */
      const namePosts = byName[name]
        ?? [];
      return writePage({
        relativePath: `${name}.html`,
        content: namePage({
          name,
          posts: namePosts,
          canonicalUrl: `${siteUrl}/${name}`,
        },),
      },);
    },),
  ];

  await Promise.all(writes,);
  childLogger.info(`generated ${writes.length} pages`,);
}
