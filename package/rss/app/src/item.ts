import type { ReadonlyDeep, } from 'type-fest';
import type {
  AtomFeed,
  RssFeed,
} from 'feedsmith';
import * as v from 'valibot';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { FeedWOutline, } from './feed.ts';
import type {
  AtomItem,
  Item,
  ItemWDate,
  NormalizedItem,
} from './item-type.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

export type { ItemWDate, } from './item-type.ts';

/**
 Tagged logger for the item module.
 */
const l = tagged({
  tag: 'item',
  l: parentLogger,
},);

//region Item extraction and normalization: Converts feed entries to a uniform dated format

/**
 Extracts, normalizes, dates, and sorts all items from sorted feeds.
 Items come from {@link extractItems} and pass through {@link getNormalizedItem}
 before dates are parsed and sorted.
 
 @param feeds - Date-sorted feeds with outline metadata
 
 @returns Items sorted by publication date (newest first)
 
 @example
 ```ts
 const items = getSortedItems(await getSortedFeeds(outlines));
 ```
 */
export function getSortedItems(feeds: readonly FeedWOutline[],): ItemWDate[] {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: getSortedItems.name,
    l,
  },);
  /**
   Flat per-feed items before format normalization.
   */
  const items = extractItems(feeds,);
  /**
   Format-normalized items so downstream code works on one shape.
   */
  const normalized = items.map(function normalize(feedItem,) {
    return getNormalizedItem(feedItem,);
  },);
  /**
   Items decorated with parsed Date instances so sorting compares Dates not strings.
   */
  const dated = normalized.map(function addDate(item,) {
    return {
      ...item,
      pubDateDate: v.parse(
        v.pipe(
          v.union([
            v.string(),
            v.number(),
            v.date(),
          ],),
          v.transform(function toDate(input,) {
            return new Date(input,);
          },),
          v.date(),
        ),
        item.item
          .pubDate
          ?? new Date(0,),
      ),
    };
  },);
  /**
   Date-sorted copy returned, preserving the input array's identity for callers.
   */
  const result = dated.toSorted(function byDate(
    itemA: ReadonlyDeep<ItemWDate>,
    itemB: ReadonlyDeep<ItemWDate>,
  ) {
    return itemB.pubDateDate
      .getTime()
      - itemA
        .pubDateDate
        .getTime();
  },);
  innerL.debug(`${String(result.length,)} sorted items`,);
  return result;
}

/**
 Extracts individual items from feeds, handling RSS items and Atom entries.
 
 @param feeds - Feeds with outline metadata
 
 @returns Flat array of items with parent feed metadata
 */
function extractItems(feeds: readonly FeedWOutline[],): Item[] {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: extractItems.name,
    l,
  },);
  /**
   Flat-mapped items from every feed so the caller gets one homogeneous array.
   */
  const result: Item[] = feeds.flatMap(
    function extractFeedItems({
      feed,
      outline,
    }: FeedWOutline,): Item[] {
      if (outline.type
        === 'atom') {
        /* oxlint-disable typescript/no-unsafe-type-assertion -- outline.type discriminant narrows the feed type */
        /**
         Narrowed Atom feed so the entries split below reads typed fields.
         */
        const atomFeed = feed as AtomFeed.Feed<string>;
        /* oxlint-enable typescript/no-unsafe-type-assertion */
        /**
         Destructured to separate entries from feed metadata reused per item.
         */
        const {
          entries,
          ...feedWithoutEntries
        } = atomFeed;
        if ((entries === undefined) || (entries.length
          === 0)) {
          innerL.warn(`atom feed ${outline.text
            ?? 'unnamed'} has no entries`,);
          return [];
        }
        return entries.map(function wrapEntry(entry,) {
          return {
            feed: feedWithoutEntries,
            outline,
            item: entry,
          } as Item;
        },);
      }
      /* oxlint-disable typescript/no-unsafe-type-assertion -- non-atom feeds are RSS */
      /**
       Narrowed RSS feed so the items split below reads typed fields.
       */
      const rssFeed = feed as RssFeed.Feed<string>;
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      /**
       Destructured to separate items from feed metadata reused per item.
       */
      const {
        items,
        ...feedWithoutItems
      } = rssFeed;
      if ((items === undefined) || (items.length
        === 0)) {
        innerL.warn(`rss feed ${outline.text
          ?? 'unnamed'} has no items`,);
        return [];
      }
      return items.map(function wrapItem(rssItem,) {
        return {
          feed: feedWithoutItems,
          outline,
          item: rssItem,
        } as Item;
      },);
    },
  );
  innerL.debug(`extracted ${String(result.length,)} items`,);
  return result;
}

/**
 Normalizes an item to RSS-like structure.
 Atom entries are converted so downstream rendering handles one shape.
 
 @param item - Raw feed item (RSS or Atom)
 
 @returns Normalized item in RSS-compatible format
 */
function getNormalizedItem(item: Item,): NormalizedItem {
  if (item.outline
    .type
    !== 'atom') {
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- non-atom items already match NormalizedItem shape
    return item as NormalizedItem;
  }

  /* oxlint-disable typescript/no-unsafe-type-assertion -- outline.type === 'atom' narrows the item */
  /**
   Narrowed atom item so the per-field copy below reads typed fields.
   */
  const atomItem = item as AtomItem;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  /**
   Destructured feed metadata so each optional field can be copied conditionally.
   */
  const {
    title,
    subtitle,
  } = atomItem.feed;
  /**
   Output feed object built up incrementally to preserve only defined fields.
   */
  const newFeed: Record<string, string> = {};
  if (title !== undefined)
    newFeed.title = title.value;
  if (subtitle !== undefined)
    newFeed.subtitle = subtitle.value;

  /**
   Source atom entry held as the read base for the RSS-shaped output.
   */
  const atomEntry = atomItem.item;
  /**
   First link if present so the output `link` field stays a single value, not an array.
   */
  const link = atomEntry.links
    ?.at(0,);
  /**
   Output item object built up incrementally to preserve only defined fields.
   */
  const newItem: Record<
    string,
    string | AtomFeed.Link<string> | AtomFeed.Category[]
    > = {};
  if (atomEntry.title
    !== undefined)
    newItem.title = atomEntry.title;
  if (link !== undefined)
    newItem.link = link;
  if (atomEntry.content
    !== undefined)
    newItem.description = atomEntry.content;
  if (atomEntry.categories
    !== undefined)
    newItem.categories = atomEntry.categories;
  /**
   Preferred timestamp falling back to `published` so feeds without `updated` still sort.
   */
  const pubDate = atomEntry.updated
    ?? atomEntry
      .published;
  if (pubDate !== undefined)
    newItem.pubDate = pubDate;
  if (atomEntry.id
    !== undefined)
    newItem.guid = atomEntry.id;

  return {
    ...item,
    feed: newFeed,
    item: newItem,
    originalItem: item.item,
    originalFeed: item.feed,
  };
}

//endregion Item extraction and normalization
