/**
 Full page layout template.
 
 Composes `<head>`, `<site-header>`, `<page-content>`, and `<site-footer>`
 into a complete HTML document. All markup generated via h-html and
 colocated component render functions; no raw HTML strings or template files.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { html as pageContentHtml, } from '../component/page-content.ts';
import { html as siteFooterHtml, } from '../component/site-footer.ts';
import { html as siteHeaderHtml, } from '../component/site-header.ts';
import type { Locale, } from '../i18n/index.ts';

import {
  headFragment,
  type ArticleDates,
} from './head.ts';

/**
 Renders a complete HTML document with the standard site shell.
 
 @param title - page title (prepended to site name in `<title>`)
 
 @param lang - language code for `<html lang>` and localized elements
 
 @param content - inner HTML to place between header and footer
 
 @param description - page-specific meta description
 
 @param canonicalUrl - full canonical URL for this page
 
 @param searchable - whether pagefind should index this page's content
 
 @param currentName - current post slug; forwarded to the header so the
 language switcher can link items to the same post in each locale
 
 @param availableInLangs - locales in which the current post exists;
 forwarded to the header so locales without a translation fall back
 to the locale landing
 
 @param articleDates - optional git-derived post dates emitted as Open Graph metadata
 
 @returns complete HTML document string including `<!DOCTYPE html>`
 
 @example
 ```ts
 const html = pageLayout({
   title: 'Home',
   lang: 'en',
   content: '<p>Hello</p>',
   description: 'Welcome',
   canonicalUrl: 'https://aquati.cat/en/',
 });
 ```
 */
export function pageLayout(
  {
    title,
    lang,
    content,
    description,
    canonicalUrl,
    searchable = false,
    currentName,
    availableInLangs,
    articleDates,
  }: {
    readonly title: string;
    readonly lang: Locale;
    readonly content: string;
    readonly description: string;
    readonly canonicalUrl: string;
    readonly searchable?: boolean;
    readonly currentName?: string;
    readonly availableInLangs?: readonly Locale[];
    readonly articleDates?: ArticleDates;
  },
): string {
  return `<!DOCTYPE html>\n${
    h({
      tag: 'html',
      attrs: { lang, },
      children: [
        headFragment({
          title,
          lang,
          description,
          canonicalUrl,
          ...(articleDates !== undefined ? { articleDates, } : {}),
        },),
        h({
          tag: 'body',
          children: [
            siteHeaderHtml({
              lang,
              ...(currentName !== undefined ? { currentName, } : {}),
              ...(availableInLangs !== undefined ? { availableInLangs, } : {}),
            },),
            pageContentHtml({
              content,
              searchable,
            },),
            siteFooterHtml(),
          ],
        },),
      ],
    },)
  }`;
}
