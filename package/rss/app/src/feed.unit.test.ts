/**
 Regression coverage for feeds that must be discarded rather than abort the
 pipeline: a publication date no calendar can read, and a fetch that rejects
 before any response exists.
 */

import { logger, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getSortedFeeds, } from '../dist/final/node/testing.mjs';

/**
 OPML `xmlUrl` of the feed carrying a readable publication date.
 */
const READABLE_URL = 'https://readable.example/feed.xml';

/**
 OPML `xmlUrl` of the feed whose publication date no calendar can read.
 */
const UNPARSEABLE_URL = 'https://unparseable.example/feed.xml';

/**
 Publication date the RFC 822 parser resolves, later than the epoch fallback.
 */
const READABLE_PUB_DATE = 'Mon, 06 Jan 2025 10:00:00 GMT';

/**
 Publication date text that names no instant, the wild-data case the sort
 must survive.
 */
const UNPARSEABLE_PUB_DATE = 'Tue, 32 Jan 2025 10:00:00 GMT';

/**
 OPML `xmlUrl` whose host cannot be reached, so `fetch` rejects without a
 response.
 */
const UNREACHABLE_URL = 'https://unreachable.example/feed.xml';

/**
 Message a stubbed rejection carries, so a passing assertion proves the caught
 text reached the log instead of an empty object.
 */
const FETCH_FAILURE_MESSAGE = 'fetch failed: getaddrinfo ENOTFOUND unreachable.example';

/**
 Builds a minimal RSS document whose channel carries one publication date.
 
 @param pubDate - channel `pubDate` text placed in the document
 
 @returns RSS document text
 */
function rssWithPubDate(pubDate: string,): string {
  return '<?xml version="1.0"?><rss version="2.0"><channel>'
    + `<title>fixture</title><pubDate>${pubDate}</pubDate>`
    + '<item><title>One</title><link>https://example.com/1</link></item>'
    + '</channel></rss>';
}

await describe({
  name: getSortedFeeds.name,
  concurrency: 1,
  children: [
    it({
      name: 'sorts a feed with an unparseable publication date after a readable one',
      fn: async ctx => {
        /**
         Document per stubbed URL so each outline fetches its own feed body.
         */
        const bodies: Readonly<Record<string, string>> = {
          [READABLE_URL]: rssWithPubDate(READABLE_PUB_DATE,),
          [UNPARSEABLE_URL]: rssWithPubDate(UNPARSEABLE_PUB_DATE,),
        };
        ctx.sinon
          .stub(globalThis, 'fetch',)
          .value(async function stubFeedFetch(url: string,) {
            return new Response(
              bodies[url] ?? '',
              { status: 200, },
            );
          } as typeof fetch,);

        const feeds = await getSortedFeeds([
          {
            type: 'rss',
            text: 'unparseable',
            xmlUrl: UNPARSEABLE_URL,
          },
          {
            type: 'rss',
            text: 'readable',
            xmlUrl: READABLE_URL,
          },
        ],);

        expect(feeds,).toHaveLength(2,);
        expect(feeds.map(function outlineText(feed,) {
          return feed.outline.text;
        },),).toEqual(['readable', 'unparseable',],);
      },
    },),
    it({
      name: 'discards a feed whose fetch rejects instead of rejecting the pipeline',
      fn: async ctx => {
        /**
         Rendered warn lines the console sink emits while the stub is active.
         */
        const lines: string[] = [];
        ctx.sinon
          .stub(console, 'warn',)
          .callsFake((...args: unknown[]) => {
            lines.push(args.map(String,).join(' ',),);
          },);
        ctx.sinon
          .stub(globalThis, 'fetch',)
          .value(async function stubUnreachableFetch(url: string,) {
            if (url === UNREACHABLE_URL)
              throw new TypeError(FETCH_FAILURE_MESSAGE,);
            return new Response(
              rssWithPubDate(READABLE_PUB_DATE,),
              { status: 200, },
            );
          } as typeof fetch,);

        const feeds = await getSortedFeeds([
          {
            type: 'rss',
            text: 'readable',
            xmlUrl: READABLE_URL,
          },
          {
            type: 'rss',
            text: 'unreachable',
            xmlUrl: UNREACHABLE_URL,
          },
        ],);
        await logger.flush();

        expect(feeds,).toHaveLength(1,);
        expect(feeds[0]?.outline.text,).toBe('readable',);
        expect(lines.join('\n',),).toContain(FETCH_FAILURE_MESSAGE,);
      },
    },),
  ],
},);
