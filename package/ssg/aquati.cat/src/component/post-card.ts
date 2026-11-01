/**
 Post card component.
 
 Renders a single blog post as a card with title, description,
 tags, and date range inside a `<post-card>` custom element.
 Styles and template are colocated so internal class names
 (`.overlay`, `.description`, `.tags`, `.date`, `.tag-link`)
 are scoped to this file.
 */
import {
  cssInt,
  cssRem,
  cssVar,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import type { Post, } from '../lib/content.ts';
import {
  FONT_SIZE_H2,
  FONT_SIZE_SMALL,
  GAP_SMALL,
} from '../style/constants.ts';
import { prettyDate, } from '../template/pretty-date.ts';

//region CSS

/**
 Card layout and internal element styles.
 
 @returns CSS string for the `<post-card>` element
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'post-card li',
      decls: {
        position: 'relative',
      },
      children: [
        $({
          rule: '& .overlay',
          decls: {
            position: 'absolute',
            inset: cssInt(0,),
            'font-size': cssInt(0,),
            'text-decoration-line': 'none',
          },
        },),
        $({
          rule: '& h2',
          decls: {
            'margin-block-start': 0,
            'margin-block-end': cssRem(GAP_SMALL,),
            'font-size': cssRem(FONT_SIZE_H2,),
          },
        },),
        $({
          rule: '& .description',
          decls: {
            'margin-block': 0,
            color: cssVar('color-muted',),
          },
        },),
        $({
          rule: '& .tags',
          decls: {
            position: 'relative',
            'z-index': 1,
            display: 'flex',
            gap: cssRem(GAP_SMALL,),
            'list-style-type': 'none',
            'padding-inline-start': 0,
            'flex-wrap': 'wrap',
          },
        },),
        $({
          rule: '& .date',
          decls: {
            'font-size': cssRem(FONT_SIZE_SMALL,),
            color: cssVar('color-subtle',),
          },
        },),
      ],
    },),
    $({
      rule: 'post-card .tag-link',
      decls: {
        'font-size': cssRem(FONT_SIZE_SMALL,),
        color: cssVar('color-subtle',),
      },
    },),
  ]
    .join('\n',);
}

//endregion CSS

//region HTML

/**
 Renders a post card inside a `<post-card>` custom element.
 
 @param post - post data to render
 
 @returns HTML string for the post card
 
 @example
 ```ts
 const markup = html(post);
 ```
 */
export function html(post: Post,): string {
  /**
   Pre-rendered tag list injected into the card body so the JSX structure stays flat.
   */
  const tagItems = post.data
    .tags
    .map(function renderTag(tag,) {
      return h({
        tag: 'li',
        class: 'tag-link',
        children: [
          h({
            tag: 'a',
            attrs: { href: `/${post.lang}/tag/${tag}`, },
            text: tag,
          },),
        ],
      },);
    },);

  return h({
    tag: 'post-card',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'li',
        children: [
          h({
            tag: 'a',
            class: 'overlay',
            attrs: { href: `/${post.lang}/${post.name}`, },
            text: post.data
              .title,
          },),
          h({
            tag: 'div',
            children: [
              h({
                tag: 'h2',
                text: post.data
                  .title,
              },),
              h({
                tag: 'p',
                class: 'description',
                text: post.data
                  .description,
              },),
              h({
                tag: 'aside',
                children: [
                  h({
                    tag: 'ul',
                    class: 'tags',
                    children: tagItems,
                  },),
                  h({
                    tag: 'span',
                    class: 'date',
                    children: [
                      prettyDate({
                        date: post.data
                          .published,
                        lang: post.lang,
                      },),
                      ' - ',
                      prettyDate({
                        date: post.data
                          .updated,
                        lang: post.lang,
                      },),
                    ],
                  },),
                ],
              },),
            ],
          },),
        ],
      },),
    ],
  },);
}

//endregion HTML
