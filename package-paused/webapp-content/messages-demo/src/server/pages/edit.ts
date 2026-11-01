/**
 * GET /m/:id/edit; the composer mounted in edit mode for an existing
 * message.
 *
 * Server renders the standard layout with the composer footer pre-tagged
 * with `data-edit-message-id`; the client script picks this up and
 * fetches the existing chunks via `/m/:id/c/:idx/md`, then mounts the
 * editor in the appropriate tier (inline for small messages, chunk-
 * paginated for messages over 1 MB).
 */

import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import {
  ABSENT,
  getSnapshot,
  messageExists,
} from '../../lib/db/messages.ts';
import {
  HTTP_GONE,
  HTTP_NOT_FOUND,
  HTTP_OK,
} from '../../lib/http.ts';
import { CHUNK_TARGET_BYTES, } from '../../lib/markdown-stream.ts';
import {
  renderPage,
  renderSimplePage,
} from './_layout.ts';

import { BYTES_PER_MIB, } from '@monochromatic-dev/const-core/ts';

/**
 * Tier-3 character threshold: one mebibyte.
 */
const TIER_3_THRESHOLD_BYTES: number = BYTES_PER_MIB;

/**
 * Maximum revisions allowed per message; mirrors `messages.MAX_REVISIONS`.
 */
const MAX_REVISIONS_DISPLAY = 10;

/**
 * Translates the message's chunk count into a starting tier hint.
 *
 * @param chunkCount - number of chunks in the message
 *
 * @returns 1, 2, or 3 depending on approximate size
 *
 * @example
 * ```ts
 * startingTier(1); // 1
 * ```
 */
/* oxlint-disable eslint/no-magic-numbers -- tier discriminant return values */
function startingTier(chunkCount: number,): 1 | 2 | 3 {
  if (chunkCount === 1)
    return 1;
  if ((chunkCount * CHUNK_TARGET_BYTES) < TIER_3_THRESHOLD_BYTES)
    return 2;
  return 3;
}
/* oxlint-enable eslint/no-magic-numbers */

/**
 * Renders the edit page for a message. The composer mounts in edit
 * mode via the layout's `editMessageId` parameter; the client lazy-
 * loads the chunks.
 *
 * @param messageId - target message id
 *
 * @returns HTTP `Response`: 200 with the edit shell, 404, or 410
 *
 * @example
 * ```ts
 * return await renderEditPage(42);
 * ```
 */
export async function renderEditPage(messageId: number,): Promise<Response> {
  /**
   * Snapshot of the message; `ABSENT` branches to 404 / 410 below.
   */
  const snapshot = await getSnapshot(messageId,);
  if (snapshot === ABSENT) {
    /**
     * Disambiguates 410 Gone (was a message, now deleted) from 404 Not Found.
     */
    const exists = await messageExists(messageId,);
    return new Response(
      renderSimplePage({
        title: exists ? 'Gone' : 'Not found',
        body: h({
          tag: 'div',
          attrs: { class: 'empty-state', },
          text: exists
            ? 'This message has been deleted and cannot be edited.'
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
   * Edit shell body HTML (back link, heading, revision label).
   */
  const body = h({
    tag: 'section',
    attrs: { class: 'edit-shell', },
    children: [
      h({
        tag: 'a',
        attrs: {
          href: `/m/${String(messageId,)}/c/0`,
          class: 'back-to-message',
        },
        text: '← back to message',
      },),
      h({
        tag: 'h1',
        attrs: { class: 'edit-heading', },
        text: `Edit message ${String(messageId,)}`,
      },),
      h({
        tag: 'p',
        attrs: { class: 'edit-meta', },
        text: `Revision ${String(snapshot.revision,)} of ${
          String(MAX_REVISIONS_DISPLAY,)
        }. Identity below must match the message author.`,
      },),
    ],
  },);

  /**
   * Complete HTML document for the edit page.
   */
  const html = renderPage({
    title: `Edit message ${String(messageId,)}`,
    body,
    editMessageId: messageId,
    initialTier: startingTier(snapshot.chunkCount,),
  },);

  return new Response(
    html,
    {
      status: HTTP_OK,
      headers: {
        'Content-Type': 'text/html; charset=utf-8',
        'Cache-Control': 'no-store',
      },
    },
  );
}
