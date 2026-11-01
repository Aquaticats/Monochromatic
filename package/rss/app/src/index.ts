import { memoizeAsync, } from '@monochromatic-dev/kv-store-memoize/ts';
import {
  defineHandler,
  getRouterParam,
  H3,
  HTTPError,
  serve,
} from 'h3';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { getSortedFeeds, } from './feed.ts';
import {
  ignore,
  serveIndex,
} from './handler.ts';
import { getIndexHtmlBody, } from './html.ts';
import { getIgnoreContent, } from './ignore.ts';
import { getFetchSalt, } from './interval.ts';
import { getSortedItems, } from './item.ts';
import { getOpmls, } from './opmls.ts';
import { getOutlinesFromOpmls, } from './outline.ts';
import { PORT, } from './port.ts';

/**
 Logger root for rss after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l: parentLogger, },);
 ```
 */
const parentLogger = tagged({ tag: 'rss', },);

/**
 Tagged logger for the server entry module.
 */
const l = tagged({
  tag: 'server',
  l: parentLogger,
},);

//region Memoized pipeline: Pull-based feed processing with content-derived cache invalidation

/**
 Memoized pipeline: OPML URLs ({@link getOpmls}) -\> outlines
 ({@link getOutlinesFromOpmls}) -\> feeds ({@link getSortedFeeds}) -\>
 sorted items ({@link getSortedItems}).
 Cache invalidates when the time bucket advances (controlled by RSS_FETCH_INTERVAL_MS).
 */
const memoizedGetSortedItems = await memoizeAsync({
  fn: async function fetchPipeline(): Promise<
    ReturnType<typeof getSortedItems>
    > {
    /**
     Inner logger tagged with this function name for traceable log lines.
     */
    const innerL = tagged({
      tag: fetchPipeline.name,
      l,
    },);
    innerL.debug('running feed pipeline',);
    /**
     Source URLs read first so the pipeline can fail fast on invalid env.
     */
    const opmls = getOpmls();
    /**
     Validated feed outlines fetched from those URLs.
     */
    const outlines = await getOutlinesFromOpmls(opmls,);
    /**
     Parsed and date-sorted feeds derived from the outlines.
     */
    const feeds = await getSortedFeeds(outlines,);
    /**
     Flattened, dated, sorted items returned to the memoize layer for caching.
     */
    const items = getSortedItems(feeds,);
    innerL.debug(`pipeline complete: ${String(items.length,)} items`,);
    return items;
  },
  keyFn: function emptyKey() {
    return '';
  },
},);

/**
 Memoized HTML renderer: sorted items -\> filtered + rendered HTML body via
 {@link getIndexHtmlBody}.
 Invalidates when either the fetch time bucket ({@link getFetchSalt}) or
 ignore file content changes.
 */
const memoizedGetHtmlBody = await memoizeAsync({
  fn: async function renderPipeline(): Promise<string> {
    /**
     Inner logger tagged with this function name for traceable log lines.
     */
    const innerL = tagged({
      tag: renderPipeline.name,
      l,
    },);
    innerL.debug('rendering HTML body',);
    /**
     Time-bucket salt so the upstream memoize cache invalidates on interval rollover.
     */
    const fetchSalt = getFetchSalt();
    /**
     Cached or freshly-pulled items used as the render input.
     */
    const items = await memoizedGetSortedItems({
      args: [],
      salt: fetchSalt,
    },);
    /**
     Rendered HTML body returned from the cache key handler.
     */
    const body = await getIndexHtmlBody({ items, },);
    innerL.debug(`rendered ${String(body.length,)} chars`,);
    return body;
  },
  keyFn: function emptyKey() {
    return '';
  },
},);

/**
 Computes the render salt and calls {@link memoizedGetHtmlBody}.
 Salt combines the fetch time bucket with {@link getIgnoreContent}'s
 ignore file content, so changes to either invalidate the render cache.
 
 @returns Rendered HTML body string
 */
async function getHtmlBody(): Promise<string> {
  /**
   Time-bucket salt component so interval rollover invalidates the render cache.
   */
  const fetchSalt = getFetchSalt();
  /**
   Ignore-file content salt component so user dismissals invalidate the render cache.
   */
  const ignoreContent = await getIgnoreContent();
  return memoizedGetHtmlBody({
    args: [],
    salt: fetchSalt + ignoreContent,
  },);
}

//endregion Memoized pipeline

//region h3 application: Maps HTTP method + path to handler functions

/**
 h3 application instance routing HTTP requests to handlers.
 */
const app = new H3();

/**
 Serves the rendered RSS feed index page.
 */
app.get(
  '/',
  defineHandler(function handleIndex() {
    return serveIndex({ getHtmlBody, },);
  },),
);

/**
 Adds an item to the ignore list.
 */
app.post(
  '/api/ignore/new',
  defineHandler(function handleIgnore(event,) {
    return ignore(event.req,);
  },),
);

//endregion h3 application

/**
 Running HTTP server instance listening on the configured port.
 */
const server = serve(
  app,
  { port: PORT, },
);

l.info(`listening on ${server.url}`,);
