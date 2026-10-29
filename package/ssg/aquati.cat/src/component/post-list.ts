/**
 Post list grid component.
 
 Renders a grid of post cards as a `<post-list>` custom element
 wrapping a semantic `<ul>`.
 */
import {
  cssCompounded,
  cssRem,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import type { Post, } from '../lib/content.ts';
import {
  GAP,
  POST_GRID_MIN,
} from '../style/constants.ts';
import { html as postCardHtml, } from './post-card.ts';

//region CSS

/**
 Grid layout styles for the `<post-list>` element.
 
 @returns CSS string for the post grid
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return [
    $({
      rule: 'post-list ul',
      decls: {
        display: 'grid',
        'grid-template-columns': cssCompounded([
          `repeat(auto-fit, minmax(${cssRem(POST_GRID_MIN,)}, 1fr))`,
        ],),
        gap: cssRem(2,),
        'list-style-type': 'none',
        'padding-inline-start': 0,
      },
    },),
  ]
    .join('\n',);
}

//endregion CSS

//region HTML

/**
 Renders a grid of post cards inside a `<post-list>` custom element.
 
 @param posts - posts to display
 
 @returns HTML string for the post list
 
 @example
 ```ts
 const markup = html(posts);
 ```
 */
export function html(posts: readonly Post[],): string {
  return h({
    tag: 'post-list',
    attrs: { 'data-is': '', },
    children: [
      h({
        tag: 'ul',
        children: posts.map(function renderCard(post,) {
          return postCardHtml(post,);
        },),
      },),
    ],
  },);
}

//endregion HTML
