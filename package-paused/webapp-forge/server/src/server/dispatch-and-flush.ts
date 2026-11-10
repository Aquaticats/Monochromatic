/**
 * Glue: dispatch all events newer than a given baseline, draining the
 * write buffer at the end.
 *
 * Used by route handlers and the seed CLI to make sure every fragment
 * affected by the last write is durable in storage before returning to
 * the caller.
 */

import {
  type EventKind,
  listEventsAfter,
} from '../data/queries.ts';
import type { Storage, } from '../storage/adapter.ts';
import type { WriteBuffer, } from '../storage/write-buffer.ts';
import { processEvent, } from '../worker/dispatcher.ts';

/**
 * Fragment-event kinds the dispatcher rebuilds. Phase 1 covers issue +
 * comment + label; Phase 2 adds PR lifecycle events that share their
 * resource id with the underlying issue.
 */
const DISPATCHABLE_KINDS: ReadonlySet<EventKind> = new Set<EventKind>([
  'comment.created',
  'issue.created',
  'issue.labeled',
  'pr.opened',
  'pr.merged',
  'pr.closed',
  'review.submitted',
  'push',
],);

/**
 * Default batch size for the event-drain loop.
 */
const DEFAULT_BATCH_SIZE = 256;

/**
 * Drains every event newer than `afterEventId`, dispatching each one in
 * id order, then waits for the write buffer to flush.
 *
 * @param row - dispatch inputs
 *
 * @returns the highest `events.id` processed (`afterEventId` when no
 *          events were drained)
 *
 * @throws Error naming the storage keys whose write failed; the buffer keeps
 *          those items queued, so a later flush retries them
 *
 * @example
 * ```ts
 * const cursor = await dispatchAndFlush({
 *   afterEventId: 0,
 *   storage,
 *   writeBuffer,
 * });
 * ```
 */
export async function dispatchAndFlush(row: {
  readonly afterEventId: number;
  readonly storage: Storage;
  readonly writeBuffer: WriteBuffer;
  readonly batchSize?: number;
},): Promise<number> {
  /**
   * Bounded page size protects the loop from unbounded backlog.
   */
  const batchSize = row.batchSize
    ?? DEFAULT_BATCH_SIZE;
  /**
   * Advances through `events.id` order; returned to caller as the new high-water mark.
   */
  let cursor = row.afterEventId;
  // Process up to `batchSize` events per loop turn so an unbounded
  // backlog cannot starve the caller. Phase 1 callers usually only
  // expect 1-2 events; the loop terminates once `listEventsAfter`
  // returns an empty page.
  while (true) {
    /* oxlint-disable no-await-in-loop -- sequential by design */
    /**
     * Next page of unprocessed events ordered by id.
     */
    const events = await listEventsAfter({
      afterId: cursor,
      limit: batchSize,
    },);
    /* oxlint-enable no-await-in-loop */
    if (events.length
      === 0)
      break;
    for (const eventRow of events) {
      cursor = eventRow.id;
      /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- event-log column is a string subtype */
      /**
       * Narrowed event kind; the column type is a free string at the SQL boundary.
       */
      const kind = eventRow.kind as EventKind;
      /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
      if (!DISPATCHABLE_KINDS.has(kind,))
        continue;
      /**
       * Comment id pulled from payload for the comment-created branch only.
       */
      const commentId = kind === 'comment.created'
        ? extractCommentId(eventRow.payload,)
        : undefined;
      /**
       * Synthetic event passed to the dispatcher; `commentId` is omitted when absent.
       */
      const event = commentId === undefined
        ? {
          kind,
          resourceId: eventRow.resource_id,
        }
        : {
          kind,
          resourceId: eventRow.resource_id,
          commentId,
        };
      // oxlint-disable-next-line no-await-in-loop -- per-resource ordering relies on sequential dispatch
      await processEvent({
        event,
        sequenceNumber: eventRow.sequence_number,
        eventId: eventRow.id,
        sink: row.writeBuffer,
      },);
    }
    if (events.length
      < batchSize)
      break;
  }
  await row.writeBuffer
    .flush();
  return cursor;
}

/**
 * Extracts the `commentId` field from a `comment.created` event's
 * payload JSON. Returns `undefined` when the payload is malformed or the
 * field is missing.
 *
 * @param payload - the event row's `payload` column (JSON string)
 *
 * @returns the comment id, or `undefined` when it is absent
 *
 * @example
 * ```ts
 * extractCommentId('{"commentId":"c1","authorId":"u1"}'); // 'c1'
 * ```
 */
/**
 * Wraps `JSON.parse` so failures resolve to `undefined` instead of
 * throwing, isolating the catch so the caller can stay declarative.
 *
 * @param payload - JSON-serialised event payload
 *
 * @returns parsed value, or `undefined` when the input is not valid JSON
 *
 * @example
 * ```ts
 * tryParseJson('{"commentId":"c1"}'); // { commentId: 'c1' }
 * tryParseJson('not json'); // undefined
 * ```
 */
function tryParseJson(payload: string,): unknown {
  try {
    return JSON.parse(payload,);
  }
  catch {
    return undefined;
  }
}

/**
 * Reads the `commentId` field from an event row's JSON payload.
 *
 * Returns `undefined` when the payload is malformed JSON or when the
 * `commentId` field is missing or not a string.
 *
 * @param payload - the event row's `payload` column (JSON string)
 *
 * @returns the comment id, or `undefined` when it is absent
 *
 * @example
 * ```ts
 * extractCommentId('{"commentId":"c1","authorId":"u1"}'); // 'c1'
 * extractCommentId('{}'); // undefined
 * ```
 */
function extractCommentId(payload: string,): string | undefined {
  /**
   * Parsed JSON payload; `undefined` when the input is not valid JSON.
   */
  const parsed = tryParseJson(payload,);
  if ((parsed === null) || ((typeof parsed) !== 'object'))
    return undefined;
  if (!('commentId' in parsed))
    return undefined;
  /**
   * Destructured commentId narrowed to string below.
   */
  const { commentId, } = parsed;
  return ((typeof commentId) === 'string') ? commentId : undefined;
}
