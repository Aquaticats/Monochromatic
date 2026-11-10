/**
 Regression coverage for a feed whose publication date no calendar can read:
 the sort used to throw a ValiError out of the date coercion and take the
 whole page with it instead of falling back to the documented epoch.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { getSortedFeeds, } from './feed.ts';

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
  ],
},);
