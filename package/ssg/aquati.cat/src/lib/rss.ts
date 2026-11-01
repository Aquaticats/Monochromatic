/**
 RSS feed generation per language.
 
 Uses feedsmith to produce RSS 2.0 XML for each language's post collection.
 */
import { generateRssFeed, } from 'feedsmith';

import {
  i18n,
  type Locale,
} from '../i18n/index.ts';

import type { Post, } from './content.ts';

/**
 Generates an RSS 2.0 XML feed for a specific language.
 
 @param lang - two-letter language code
 
 @param posts - posts filtered to the target language
 
 @param siteUrl - base URL of the site (e.g. `https://aquati.cat`)
 
 @returns RSS XML string
 
 @example
 ```ts
 const xml = generateLanguageRss({ lang: 'en', posts: enPosts, siteUrl: 'https://aquati.cat' });
 ```
 */
export function generateLanguageRss(
  {
    lang,
    posts,
    siteUrl,
  }: {
    readonly lang: Locale;
    readonly posts: readonly Post[];
    readonly siteUrl: string;
  },
): string {
  /**
   Newest git-derived update date across this locale's posts, used as the feed build date.
   */
  const lastBuildDate = posts
    .map(function toUpdatedDate(post,): Date {
      return post.data
        .updated;
    },)
    .toSorted(function newestFirst(
      a,
      b,
    ): number {
      return b.getTime()
        - a.getTime();
    },)
    .at(0,);

  return generateRssFeed({
    title: i18n.label(
      lang,
      'siteName',
    ),
    link: siteUrl,
    description: i18n.label(
      lang,
      'siteDescription',
    ),
    language: lang,
    ...(lastBuildDate === undefined
      ? {}
      : {
        pubDate: lastBuildDate,
        lastBuildDate,
      }),
    items: posts.map(function toRssItem(post,) {
      return {
        title: post.data
          .title,
        link: `${siteUrl}/${lang}/${post.name}`,
        description: post.data
          .description,
        pubDate: post.data
          .updated,
        categories: post.data
          .tags
          .map(function toCategory(tag,) {
            return { name: tag, };
          },),
      };
    },),
  },);
}
