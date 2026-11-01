/**
 Post language chooser page.
 
 When a post exists in multiple languages, this page lists all
 available language versions so the reader can pick one.
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import {
  isLocale,
  type Locale,
} from '../i18n/index.ts';

import type { Post, } from '../lib/content.ts';
import { pageLayout, } from '../template/layout.ts';

/**
 Generates the language chooser page for a specific post slug.
 
 @param name - post slug name
 
 @param posts - all language variants of this post
 
 @param canonicalUrl - full canonical URL for this page
 
 @returns complete HTML document listing available translations
 
 @example
 ```ts
 const html = namePage({
   name: 'hello-world',
   posts,
   canonicalUrl: 'https://aquati.cat/hello-world',
 });
 ```
 */
export function namePage(
  {
    name,
    posts,
    canonicalUrl,
  }: {
    readonly name: string;
    readonly posts: readonly Post[];
    readonly canonicalUrl: string;
  },
): string {
  /**
   Main element tree composed before the page layout wraps it with `<head>` and friends.
   */
  const content = h({
    tag: 'main',
    children: [
      h({
        tag: 'h1',
        text: name,
      },),
      h({
        tag: 'ul',
        children: posts.map(function langVariant(post,) {
          return h({
            tag: 'li',
            children: [
              h({
                tag: 'a',
                attrs: { href: `/${post.lang}/${post.name}`, },
                text: post.data
                  .title,
              },),
            ],
          },);
        },),
      },),
    ],
  },);

  /**
   Default to the first available translation's language, falling back to 'en'.
   */
  const [firstPost,] = posts;
  /**
   Resolved page locale used for head meta plus the lang switcher.
   */
  const lang: Locale = (firstPost !== undefined) && isLocale(firstPost.lang,)
    ? firstPost.lang
    : 'en';

  /**
   Use the first post's description when available, otherwise the slug name.
   */
  const description = firstPost !== undefined
    ? firstPost.data
      .description
    : name;

  /**
   Locale in which this slug actually has a translation.
   */
  const availableInLangs: readonly Locale[] = posts.map(function pickLang(p,) {
    return p.lang;
  },);

  return pageLayout({
    title: name,
    lang,
    content,
    description,
    canonicalUrl,
    currentName: name,
    availableInLangs,
  },);
}
