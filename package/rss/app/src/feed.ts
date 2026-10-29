// 105 lines: fetch, parse, and sort are a single pipeline; splitting obscures the data flow
import { mapIterableAsync, } from '@monochromatic-dev/async-iter-core/ts';
import {
  type AtomFeed,
  type Opml,
  parseAtomFeed,
  parseRssFeed,
  type RssFeed,
} from 'feedsmith';
import * as v from 'valibot';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import type { InnerOutlineWUrl, } from './outline.ts';
import type { DeepReadonly, } from './types.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the feed module.
 */
const l = tagged({
  tag: 'feed',
  l: parentLogger,
},);

/**
 Parsed feed data paired with its OPML outline metadata.
 Combines the feed content with source information for display.
 Deeply readonly because the sort and extract steps only read these values.
 */
export type FeedWOutline = DeepReadonly<{
  feed: RssFeed.Feed<string> | AtomFeed.Feed<string>;
  outline: Opml.Outline<string>;
}>;

//region Feed fetching and sorting: Retrieves feeds from URLs, parses them, and sorts by date

/**
 Fetches, parses, and date-sorts feeds from OPML outlines.
 Handles both RSS and Atom formats, discarding feeds that fail to fetch or parse.
 Delegates fetching and parsing to {@link fetchAndParseFeeds}, then sorts by
 the date {@link extractDate} reads from each feed.
 
 @param outlines - Outlines with validated xmlUrl properties
 
 @returns Feeds sorted by publication date (newest first)
 
 @mutates outlines - `fetchAndParseFeeds(outlines)` delegates to `mapIterableAsync`, which may invoke caller-owned iterator capabilities and passes reachable outline values to `fetchFeed`.
 
 @example
 ```ts
 const feeds = await getSortedFeeds(outlines);
 ```
 */
export async function getSortedFeeds(
  outlines: readonly DeepReadonly<InnerOutlineWUrl>[],
): Promise<FeedWOutline[]> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: getSortedFeeds.name,
    l,
  },);
  /**
   Fetched and parsed feeds held so the sort step works on a stable array.
   */
  const feeds = await fetchAndParseFeeds(outlines,);
  /**
   Date-sorted copy returned, preserving the input array's identity for callers.
   */
  const result = feeds.toSorted(function byDate(
    feedA,
    feedB,
  ) {
    return extractDate(feedB,)
      .getTime()
      - extractDate(feedA,)
        .getTime();
  },);
  innerL.debug(`sorted ${String(result.length,)} feeds`,);
  return result;
}

/**
 Fetches and parses feeds from outlines, discarding failures. Parses each
 feed text with {@link parseAtomFeed} or {@link parseRssFeed} depending on
 the outline's type.
 
 @param outlines - Outlines with xmlUrl properties
 
 @returns Successfully fetched and parsed feeds
 
 @mutates outlines - `mapIterableAsync` may invoke caller-owned iterator capabilities and passes reachable outline values to `fetchFeed`.
 */
async function fetchAndParseFeeds(
  outlines: readonly DeepReadonly<InnerOutlineWUrl>[],
): Promise<FeedWOutline[]> {
  /**
   Inner logger tagged with this function name for traceable log lines.
   */
  const innerL = tagged({
    tag: fetchAndParseFeeds.name,
    l,
  },);
  /**
   Unique sentinel returned for fetch/text failures so the filter step can drop them.
   */
  const DISCARD = Symbol('feed fetch or parse failed',);
  /**
   Fetched OPML text paired with its source outline for later parsing.
   */
  type TextWOutline = {
    readonly text: string;
    readonly outline: DeepReadonly<InnerOutlineWUrl>;
  };
  /**
   Fetched feed texts paired with outlines, filtered down to the successful subset.
   */
  const textsWOutline: TextWOutline[] = (await mapIterableAsync({
    fn: async function fetchFeed(
      outline: DeepReadonly<InnerOutlineWUrl>,
    ): Promise<TextWOutline | typeof DISCARD> {
      /**
       Single Response held so status check and text read share one network round trip.
       */
      const response = await fetch(outline.xmlUrl,);
      if (!response.ok) {
        innerL.warn(`${outline.xmlUrl} responded ${String(response.status,)}`,);
        return DISCARD;
      }
      try {
        return {
          text: await response.text(),
          outline,
        };
      }
      catch (error) {
        innerL.warn(
          `text conversion failed for ${outline.xmlUrl}: ${JSON.stringify(error,)}`,
        );
        return DISCARD;
      }
    },
    iterable: outlines,
  },))
    .filter(function notDiscard(value,): value is TextWOutline {
      return value !== DISCARD;
    },);
  innerL.debug(`fetched ${String(textsWOutline.length,)} feed texts`,);
  return textsWOutline.flatMap(function parse({
    text,
    outline,
  },) {
    /**
     Parser picked by outline type so each feed runs through the matching parser.
     */
    const parser = outline.type
      === 'atom' ? parseAtomFeed : parseRssFeed;
    try {
      return [{
        feed: parser(text,),
        outline,
      },];
    }
    catch (error) {
      innerL.warn(`parse failed for ${outline.xmlUrl}: ${JSON.stringify(error,)}`,);
      return [];
    }
  },);
}

/**
 Coerces string, number, or Date inputs into a Date instance.
 */
const coerceDateSchema = v.pipe(
  v.union([
    v.string(),
    v.number(),
    v.date(),
  ],),
  v.transform(function toDate(input,) {
    return new Date(input,);
  },),
  v.date(),
);

/**
 Extracts the publication date from a feed, falling back to epoch. The raw
 value runs through {@link coerceDateSchema} to normalize string, number,
 or `Date` inputs.
 
 @param feedWOutline - Feed with outline metadata
 
 @returns Parsed publication date
 */
function extractDate(feedWOutline: FeedWOutline,): Date {
  /**
   Destructured fields so the branch reads `outline.type` and `feed` directly.
   */
  const {
    feed,
    outline,
  } = feedWOutline;
  if (outline.type
    === 'atom') {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- outline.type discriminant narrows the feed type */
    /**
     Narrowed feed view used to read the Atom-specific `updated` field.
     */
    const atomFeed = feed as AtomFeed.Feed<string>;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    return v.parse(
      coerceDateSchema,
      atomFeed.updated
        ?? new Date(0,),
    );
  }
  /* oxlint-disable typescript/no-unsafe-type-assertion -- non-atom feeds are RSS */
  /**
   Narrowed feed view used to read the RSS-specific `pubDate` field.
   */
  const rssFeed = feed as RssFeed.Feed<string>;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  return v.parse(
    coerceDateSchema,
    rssFeed.pubDate
      ?? new Date(0,),
  );
}

//endregion Feed fetching and sorting
