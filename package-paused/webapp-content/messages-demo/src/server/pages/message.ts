/**
 * GET /m/:id and GET /m/:id/c/:idx; single-message read pages.
 *
 * Returns one chunk per page with prev/next/back navigation. Honours
 * If-None-Match against a strong ETag derived from `messages.revision`
 * + chunk index, so a browser cache hit serves verbatim while the
 * never-stale guarantee is maintained via `Cache-Control: no-cache`.
 *
 * Wraps the read in a `BEGIN DEFERRED` transaction so the snapshot
 * (revision, draft chain) and the chunk SELECT see a consistent point-
 * in-time view; an edit that commits between those two statements
 * would otherwise risk a chunk-from-newer-revision result.
 */

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import db from '../../lib/db.ts';
import {
  ABSENT,
  getChunk,
  getSnapshot,
  messageExists,
  type MessageSnapshot,
} from '../../lib/db/messages.ts';
import {
  etagForChunk,
  matches,
} from '../../lib/etag.ts';
import {
  HTTP_GONE,
  HTTP_INTERNAL_SERVER_ERROR,
  HTTP_NOT_FOUND,
  HTTP_NOT_MODIFIED,
  HTTP_OK,
} from '../../lib/http.ts';
import { CHUNK_TARGET_BYTES, } from '../../lib/markdown-stream.ts';
import {
  renderPage,
  renderSimplePage,
} from './_layout.ts';

import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core/ts';

/**
 * Tier-2 boundary in kibibytes.
 */
const TIER_2_THRESHOLD_KIB = 8;

/**
 * Tier-3 boundary in kibibytes.
 */
const TIER_3_THRESHOLD_KIB = 1_024;

/**
 * Tier-2 size threshold in characters.
 */
const TIER_2_THRESHOLD = TIER_2_THRESHOLD_KIB * BYTES_PER_KIB;

/**
 * Tier-3 size threshold in characters.
 */
const TIER_3_THRESHOLD = TIER_3_THRESHOLD_KIB * BYTES_PER_KIB;

/**
 * Tier hint computed from message size; passed to the composer in edit
 * mode so it can pre-select chunk-paginated UI for huge messages.
 *
 * @param charCount - total source characters in the message
 *
 * @returns 1, 2, or 3 depending on size buckets
 *
 * @example
 * ```ts
 * tierFor(50_000); // 2
 * ```
 */
/* oxlint-disable eslint/no-magic-numbers -- tier discriminant return values */
function tierFor(charCount: number,): 1 | 2 | 3 {
  if (charCount < TIER_2_THRESHOLD)
    return 1;
  if (charCount < TIER_3_THRESHOLD)
    return 2;
  return 3;
}
/* oxlint-enable eslint/no-magic-numbers */

/**
 * Renders the page for one chunk of a message.
 *
 * @param input - message id, chunk index from the URL, and the request
 *                `If-None-Match` header for ETag negotiation
 *
 * @returns HTTP `Response`: 200 with HTML, 304 Not Modified, 404 Not
 *          Found, or 410 Gone
 *
 * @example
 * ```ts
 * const response = await renderMessageChunk({ messageId: 5, chunkIndex: 0 });
 * ```
 */
export async function renderMessageChunk(
  input: {
    readonly messageId: number;
    readonly chunkIndex: number;
    readonly ifNoneMatch?: string;
  },
): Promise<Response> {
  await db.exec('BEGIN DEFERRED',);
  try {
    /**
     * Snapshot reused across the chunk count check, ETag, and chunk fetch.
     */
    const snapshot = await getSnapshot(input.messageId,);
    if (snapshot === ABSENT) {
      // Distinguish gone from not-found so the client can decide
      // whether to clear cached state or follow a redirect.
      /**
       * Tells 410 Gone (existed once) from 404 Not Found (never existed).
       */
      const exists = await messageExists(input.messageId,);
      await db.exec('COMMIT',);
      return new Response(
        renderSimplePage({
          title: exists ? 'Gone' : 'Not found',
          body: h({
            tag: 'div',
            attrs: { class: 'empty-state', },
            text: exists
              ? 'This message has been deleted.'
              : 'No message with that id.',
          },),
        },),
        {
          status: exists ? HTTP_GONE : HTTP_NOT_FOUND,
          headers: {
            'Content-Type': 'text/html; charset=utf-8',
            'Cache-Control': 'no-store',
          },
        },
      );
    }

    /**
     * Computed ETag; sent on both 200 and 304 responses.
     */
    const etag = etagForChunk({
      revision: snapshot.revision,
      chunkIndex: input.chunkIndex,
    },);
    if ((input.ifNoneMatch
      !== undefined) && matches({
      ifNoneMatch: input.ifNoneMatch,
      etag,
    },)) {
      await db.exec('COMMIT',);
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

    if ((input.chunkIndex
      < 0) || (input.chunkIndex
        >= snapshot
        .chunkCount)) {
      await db.exec('COMMIT',);
      return new Response(
        renderSimplePage({
          title: 'Chunk out of range',
          body: h({
            tag: 'div',
            attrs: { class: 'empty-state', },
            text: 'No such chunk in this message.',
          },),
        },),
        {
          status: HTTP_NOT_FOUND,
          headers: {
            'Content-Type': 'text/html; charset=utf-8',
            'Cache-Control': 'no-store',
          },
        },
      );
    }

    /**
     * Chunk row resolved via the copy-on-write draft chain walk.
     */
    const chunk = await getChunk({
      messageId: input.messageId,
      chunkIndex: input.chunkIndex,
    },);
    await db.exec('COMMIT',);

    if (chunk === ABSENT) {
      return new Response(
        'chunk not found',
        {
          status: HTTP_INTERNAL_SERVER_ERROR,
          headers: { 'Cache-Control': 'no-store', },
        },
      );
    }

    /**
     * Rendered chunk body HTML; embedded in the layout's main slot.
     */
    const body = renderChunkBody({
      snapshot,
      chunkIndex: input.chunkIndex,
      chunkHtml: chunk.html,
    },);
    /**
     * Complete HTML document returned to the client.
     */
    const html = renderPage({
      title: `Message ${String(snapshot.id,)} (chunk ${
        String(input.chunkIndex
          + 1,)
      } of ${String(snapshot.chunkCount,)})`,
      body,
      editMessageId: snapshot.id,
      initialTier: tierFor(snapshot.chunkCount
        * CHUNK_TARGET_BYTES,),
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
  catch (error) {
    try {
      await db.exec('ROLLBACK',);
    }
    catch { /* nothing to roll back if BEGIN failed */ }
    throw error;
  }
}

/**
 * Builds the per-page body: header card with metadata + chunk HTML +
 * prev/next navigation. Chunk HTML is pre-rendered and safe.
 *
 * @param input - snapshot, chunk index, pre-rendered chunk HTML
 *
 * @returns HTML body string
 */
function renderChunkBody(
  input: {
    readonly snapshot: MessageSnapshot;
    readonly chunkIndex: number;
    readonly chunkHtml: string;
  },
): string {
  /**
   * Destructured early so the chunk-nav branches read the fields directly.
   */
  const {
    snapshot,
    chunkIndex,
    chunkHtml,
  } = input;
  /**
   * True when the prev link is at chunk 0; renders as a disabled span instead.
   */
  const prevDisabled = chunkIndex === 0;
  /**
   * True when the next link is at the last chunk; renders as a disabled span instead.
   */
  const nextDisabled = chunkIndex >= (snapshot.chunkCount
    - 1);

  /**
   * Three pre-rendered nav items in the order prev, position, next.
   */
  const navItems: string[] = [
    prevDisabled
      ? h({
        tag: 'span',
        attrs: {
          class: 'chunk-nav-prev disabled',
          'aria-disabled': 'true',
        },
        text: '« prev',
      },)
      : h({
        tag: 'a',
        attrs: {
          class: 'chunk-nav-prev',
          href: `/m/${String(snapshot.id,)}/c/${String(chunkIndex - 1,)}`,
          rel: 'prev',
        },
        text: '« prev',
      },),
    h({
      tag: 'span',
      attrs: { class: 'chunk-nav-position', },
      text: `chunk ${String(chunkIndex + 1,)} of ${String(snapshot.chunkCount,)}`,
    },),
    nextDisabled
      ? h({
        tag: 'span',
        attrs: {
          class: 'chunk-nav-next disabled',
          'aria-disabled': 'true',
        },
        text: 'next »',
      },)
      : h({
        tag: 'a',
        attrs: {
          class: 'chunk-nav-next',
          href: `/m/${String(snapshot.id,)}/c/${String(chunkIndex + 1,)}`,
          rel: 'next',
        },
        text: 'next »',
      },),
  ];

  /**
   * Message-meta header HTML (back link, author, revision).
   */
  const meta = h({
    tag: 'header',
    attrs: { class: 'message-meta', },
    children: [
      h({
        tag: 'a',
        attrs: {
          href: '/',
          class: 'back-to-feed',
        },
        text: '← feed',
      },),
      h({
        tag: 'span',
        attrs: { class: 'message-author', },
        text: snapshot.userName,
      },),
      h({
        tag: 'span',
        attrs: { class: 'message-revision', },
        text: `revision ${String(snapshot.revision,)}`,
      },),
    ],
  },);

  /**
   * Chunk-nav HTML (prev, position, next) for both top and bottom of the chunk body.
   */
  const nav = h({
    tag: 'nav',
    attrs: {
      class: 'chunk-nav',
      'aria-label': 'chunk navigation',
    },
    html: navItems.join('',),
  },);

  return h({
    tag: 'article',
    attrs: {
      class: 'message-article',
      'data-message-id': String(snapshot.id,),
    },
    html: meta + nav
      + h({
      tag: 'div',
      attrs: { class: 'message-body', },
      html: chunkHtml,
    },)
      + nav,
  },);
}
