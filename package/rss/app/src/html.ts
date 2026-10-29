/**
 HTML rendering for RSS feed pages, filtering out ignored entries.
 */

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { itemToFeed, } from './html-item.ts';
import {
  getIgnoreContent,
  parseIgnoredLinks,
} from './ignore.ts';
import type { ItemWDate, } from './item-type.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the html module.
 */
const l = tagged({
  tag: 'html',
  l: parentLogger,
},);

/**
 Maximum number of items rendered on a single page.
 */
const LIMIT = 100;

/**
 Closing HTML fragment appended after the rendered feed body.
 */
export const INDEX_HTML_END = '</body></html>';

/**
 Builds the HTML body from items, filtering out entries present in the ignore
 list ({@link getIgnoreContent} read through {@link parseIgnoredLinks}), then
 renders the surviving items via {@link itemToFeed}.
 
 @param options - Feed items with publication dates to render
 
 @returns Rendered HTML string for the feed list
 
 @example
 ```ts
 const body = await getIndexHtmlBody({ items });
 ```
 */
export async function getIndexHtmlBody(
  options: { readonly items: readonly ItemWDate[]; },
): Promise<string> {
  /**
   Destructured items so the loop body reads without `options.` prefix.
   */
  const { items, } = options;
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: getIndexHtmlBody.name,
    l,
  },);

  /**
   Raw JSONL content read once so parse runs over a stable snapshot.
   */
  const ignoreContent = await getIgnoreContent();
  /**
   Link set used as the membership predicate for filtering.
   */
  const ignoredLinks = parseIgnoredLinks(ignoreContent,);
  /**
   Items remaining after the ignore-link filter, before the page limit slice.
   */
  const filteredItems = items.filter(function notIgnored(item,) {
    if ((item.item
      .link
      === undefined) || (item.item
        .link
        === ''))
      return true;
    return !ignoredLinks.has(item.item
      .link,);
  },);

  innerL.debug(
    `${String(filteredItems.length,)} of ${String(items.length,)} items after filtering`,
  );

  return h({
    tag: 'ol',
    class: 'feeds',
    children: filteredItems
      .slice(
      0,
      LIMIT,
      )
      .map(function renderItem(
        item,
        index,
      ) {
        return itemToFeed({
          itemWDate: item,
          index,
        },);
      },),
  },);
}
