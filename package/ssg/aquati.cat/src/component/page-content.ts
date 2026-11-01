/**
 Content area between header and footer.
 
 Wraps page-specific content in a `<page-content>` custom element
 with max-width, centering, and padding. Optionally marks the
 content for Pagefind indexing via `data-pagefind-body`.
 */
import {
  cssRem,
  hCss as $,
  hHtml as h,
} from '@monochromatic-dev/hyperscript-core/ts';

import {
  GAP,
  MAX_WIDTH,
} from '../style/constants.ts';

//region CSS

/**
 Layout styles for the `<page-content>` custom element.
 
 @returns CSS string for the content area
 
 @example
 ```ts
 const styles = css();
 ```
 */
export function css(): string {
  return $({
    rule: 'page-content',
    children: [
      $({
        rule: '> main',
        decls: {
          'max-inline-size': cssRem(MAX_WIDTH,),
          'margin-inline': 'auto',
          'padding-inline': cssRem(GAP,),
          'padding-block': cssRem(GAP,),
        },
      },),
    ],
  },);
}

//endregion CSS

//region HTML

/**
 Renders the content wrapper as a `<page-content>` custom element.
 
 @param content - inner HTML to place inside the element
 
 @param searchable - whether Pagefind should index this page's content
 
 @returns HTML string for the `<page-content>` element
 
 @example
 ```ts
 const markup = html({ content: '<p>Hello</p>', searchable: true });
 ```
 */
export function html(
  {
    content,
    searchable = false,
  }: {
    readonly content: string;
    readonly searchable?: boolean;
  },
): string {
  return h({
    tag: 'page-content',
    attrs: searchable
      ? {
        'data-is': '',
        'data-pagefind-body': '',
      }
      : { 'data-is': '', },
    html: content,
  },);
}

//endregion HTML
