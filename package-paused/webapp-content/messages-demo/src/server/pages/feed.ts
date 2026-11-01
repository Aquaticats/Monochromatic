/**
 * GET / and GET /p/:cursor; the message feed.
 *
 * Newest-first keyset pagination over `messages`. Honours
 * `If-None-Match` against an ETag derived from `MAX(id, updated_at)`,
 * so feeds revalidate cheaply while never serving stale content.
 */

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';
import {
  MS_PER_SECOND,
  SECONDS_PER_DAY,
  SECONDS_PER_HOUR,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

import {
  feedAggregates,
  type FeedMessage,
  listFeed,
} from '../../lib/db/messages.ts';
import {
  etagForFeed,
  matches,
} from '../../lib/etag.ts';
import {
  HTTP_NOT_MODIFIED,
  HTTP_OK,
} from '../../lib/http.ts';
import {
  decodeCursor,
  encodeCursor,
  FEED_PAGE_SIZE,
} from '../../lib/pagination.ts';
import { renderPage, } from './_layout.ts';

/**
 * Renders one page of the feed.
 *
 * @param input - opaque cursor from the URL (omitted for first page) and
 *                the request `If-None-Match` header for ETag negotiation
 *
 * @returns HTTP `Response`
 *
 * @example
 * ```ts
 * await renderFeed({});                                       // first page
 * await renderFeed({ cursorToken: 'MTcxNDA4MDAwMDA6MTA0Mg' }); // older page
 * ```
 */
export async function renderFeed(
  input: {
    readonly cursorToken?: string;
    readonly ifNoneMatch?: string;
  },
): Promise<Response> {
  /**
   * Aggregate signature fed into the ETag; matches against `If-None-Match` for 304s.
   */
  const aggregates = await feedAggregates();
  /**
   * Computed ETag; sent on both 200 and 304 responses.
   */
  const etag = etagForFeed(aggregates,);
  if ((input.ifNoneMatch
    !== undefined) && matches({
    ifNoneMatch: input.ifNoneMatch,
    etag,
  },)) {
    return new Response(
      null,
      {
        status: HTTP_NOT_MODIFIED,
        headers: {
          ETag: etag,
          'Cache-Control': 'no-cache, must-revalidate',
        },
      },
    );
  }

  /**
   * Decoded keyset cursor; absent on the first page or when the token is absent.
   */
  const cursor = input.cursorToken
    === undefined ? undefined : decodeCursor(input.cursorToken,);
  /**
   * Materialised page of feed entries used to render the cards and the pagination link.
   */
  const messages = await listFeed(cursor,);
  /**
   * Rendered feed body HTML; embedded in the layout's main slot.
   */
  const body = renderFeedBody(messages,);

  /**
   * Complete HTML document returned to the client.
   */
  const html = renderPage({
    title: cursor === undefined ? 'messages-demo' : `messages-demo (older)`,
    body,
  },);

  return new Response(
    html,
    {
      status: HTTP_OK,
      headers: {
        'Content-Type': 'text/html; charset=utf-8',
        'Cache-Control': 'no-cache, must-revalidate',
        ETag: etag,
      },
    },
  );
}

/**
 * Renders the body of one feed page: a list of message cards plus the
 * "older" link.
 *
 * @param messages - one page of feed entries
 *
 * @returns HTML string
 */
function renderFeedBody(messages: readonly FeedMessage[],): string {
  if (messages.length
    === 0) {
    return h({
      tag: 'div',
      attrs: { class: 'empty-state', },
      text: 'No messages yet. Use the composer below to send the first one.',
    },);
  }

  /**
   * Mapped HTML string per card; joined below into the feed section.
   */
  const cards = messages
    .map(function toCard(message,) {
      /**
       * Card header HTML (author, date, optional revision badge).
       */
      const cardHeader = h({
        tag: 'header',
        attrs: { class: 'card-meta', },
        children: [
          h({
            tag: 'span',
            attrs: { class: 'card-author', },
            text: message.userName,
          },),
          h({
            tag: 'time',
            attrs: {
              class: 'card-date',
              datetime: new Date(message.createdAt,).toISOString(),
            },
            text: formatRelative(message.createdAt,),
          },),
          message.revision
            > 1
            ? h({
              tag: 'span',
              attrs: { class: 'card-revision', },
              text: `edited (rev ${String(message.revision,)})`,
            },)
            : '',
        ],
      },);

      /**
       * Card preview body HTML (truncated message text).
       */
      const cardBody = h({
        tag: 'p',
        attrs: { class: 'card-preview', },
        text: message.preview,
      },);

      /**
       * Card footer HTML (open link, chunk count when paginated).
       */
      const cardFooter = h({
        tag: 'footer',
        attrs: { class: 'card-actions', },
        children: [
          h({
            tag: 'a',
            attrs: {
              class: 'card-open',
              href: `/m/${String(message.id,)}/c/0`,
            },
            text: message.chunkCount
              === 1
              ? 'open'
              : `open (${String(message.chunkCount,)} chunks)`,
          },),
        ],
      },);

      return h({
        tag: 'article',
        attrs: {
          class: 'feed-card',
          'data-message-id': String(message.id,),
        },
        html: cardHeader + cardBody
          + cardFooter,
      },);
    },)
    .join('',);

  /**
   * Last entry on the page; cursor seed for the "older" link.
   */
  const last = messages.at(-1,);
  /**
   * Pagination HTML; empty when the page is incomplete.
   */
  const pagination = ((messages.length
    === FEED_PAGE_SIZE) && (last !== undefined))
    ? h({
      tag: 'nav',
      attrs: {
        class: 'feed-pagination',
        'aria-label': 'pagination',
      },
      children: [
        h({
          tag: 'a',
          attrs: {
            class: 'pagination-older',
            href: `/p/${
              encodeCursor({
                createdAt: last.createdAt,
                id: last.id,
              },)
            }`,
          },
          text: 'older messages »',
        },),
      ],
    },)
    : '';

  return h({
    tag: 'section',
    attrs: {
      class: 'feed',
      'aria-label': 'message feed',
    },
    html: cards,
  },)
    + pagination;
}

/**
 * Day count after which the formatter switches to an ISO date.
 */
const ISO_DATE_THRESHOLD_DAYS = 30;

/**
 * Index where ISO 8601 `YYYY-MM-DD` ends.
 */
const ISO_DATE_PREFIX_LENGTH = 10;

/**
 * Coarse "X minutes/hours/days ago" formatter for the feed cards. We
 * deliberately avoid pulling in a date library; the demo only needs
 * approximate human time.
 *
 * @param timestamp - ms since epoch
 *
 * @returns relative-time string
 *
 * @example
 * ```ts
 * formatRelative(Date.now() - 5 * 60 * 1_000); // "5m ago"
 * ```
 */
function formatRelative(timestamp: number,): string {
  /**
   * Current ms-since-epoch used as the relative anchor.
   */
  const now = Date.now();
  /**
   * Seconds elapsed between `timestamp` and `now`; non-negative for past entries.
   */
  const deltaSeconds = Math.floor((now - timestamp) / MS_PER_SECOND,);
  if (deltaSeconds < SECONDS_PER_MINUTE)
    return 'just now';
  if (deltaSeconds < SECONDS_PER_HOUR)
    return `${String(Math.floor(deltaSeconds / SECONDS_PER_MINUTE,),)}m ago`;
  if (deltaSeconds < SECONDS_PER_DAY)
    return `${String(Math.floor(deltaSeconds / SECONDS_PER_HOUR,),)}h ago`;
  if (deltaSeconds < (SECONDS_PER_DAY * ISO_DATE_THRESHOLD_DAYS))
    return `${String(Math.floor(deltaSeconds / SECONDS_PER_DAY,),)}d ago`;
  return new Date(timestamp,).toISOString()
    .slice(
    0,
    ISO_DATE_PREFIX_LENGTH,
  );
}
