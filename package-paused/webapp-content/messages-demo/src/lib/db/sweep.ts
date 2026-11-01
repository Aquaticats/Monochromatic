/**
 * Event-driven, bounded sweeps. Called from write handlers, never on a
 * timer. Each sweep deletes at most `SWEEP_BATCH` rows so it adds at
 * most a few ms to its host op.
 *
 * Two sweep types only; there is **no** superseded-draft sweep:
 * ancestor drafts cannot be safely reaped while their head still chains
 * through them. The hard-delete chain walk handles ancestors when the
 * message itself is being removed.
 */

import {
  HOURS_PER_DAY,
  MS_PER_SECOND,
  SECONDS_PER_MINUTE,
} from '@monochromatic-dev/const-core/ts';

import db, {
  all,
  run,
} from '../db.ts';
import {
  CHAIN_END,
  stepToParent,
} from './chain.ts';

/**
 * Minutes in the orphan TTL span.
 */
const ORPHAN_TTL_MINUTES = 15;

/**
 * Days in the deleted-message hard-delete TTL.
 */
const DELETED_TTL_DAYS = 7;

/**
 * Maximum rows touched per sweep call.
 */
export const SWEEP_BATCH: number = 16;

/**
 * TTL for unfinalised drafts before they are reaped.
 */
export const ORPHAN_TTL_MS: number = ORPHAN_TTL_MINUTES
  * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 * TTL for soft-deleted messages before they are hard-deleted.
 */
export const DELETED_TTL_MS: number = DELETED_TTL_DAYS
  * HOURS_PER_DAY
  * SECONDS_PER_MINUTE
  * SECONDS_PER_MINUTE
  * MS_PER_SECOND;

/**
 * Reaps unfinalised drafts older than `ORPHAN_TTL_MS`.
 *
 * @param scope - optional `userId` to limit the sweep to one user. Use
 *                this on `POST /api/drafts` so the sweep cost is
 *                proportional to the user's recent activity.
 *
 * @example
 * ```ts
 * await sweepOrphans({ userId: 'user-a' });
 * await sweepOrphans({});   // safety net, called from finalize
 * ```
 */
export async function sweepOrphans(
  scope: { readonly userId?: string; },
): Promise<void> {
  /**
   * Threshold; drafts older than this and still unfinalised are reaped.
   */
  const cutoff = Date.now()
    - ORPHAN_TTL_MS;
  await run({
    sql: `DELETE FROM drafts WHERE id IN (
       SELECT id FROM drafts
         WHERE finalized = 0
           AND updated_at < ?
           AND (? IS NULL OR user_id = ?)
         LIMIT ?
     )`,
    params: [
      cutoff,
      scope.userId
        ?? null,
      scope.userId
        ?? null,
      SWEEP_BATCH,
    ],
  },);
}

/**
 * Hard-deletes soft-deleted messages whose `deleted_at` is older than
 * `DELETED_TTL_MS`. For each such message, walks the copy-on-write
 * chain and removes every ancestor draft, cascading to chunks via FK.
 *
 * Bounded to `SWEEP_BATCH` candidate messages per call.
 *
 * @example
 * ```ts
 * await sweepDeleted();
 * ```
 */
export async function sweepDeleted(): Promise<void> {
  /**
   * Threshold; messages soft-deleted before this are eligible for hard-delete.
   */
  const cutoff = Date.now()
    - DELETED_TTL_MS;
  /**
   * Bounded candidate batch; each row is hard-deleted in its own transaction below.
   */
  const candidates = await all<{
    id: number;
    draft_id: string;
  }>({
    sql: `SELECT id, draft_id FROM messages
       WHERE deleted_at IS NOT NULL AND deleted_at < ?
       LIMIT ?`,
    params: [
      cutoff,
      SWEEP_BATCH,
    ],
  },);
  // Each candidate runs in its own transaction; chain walk inside the
  // transaction is inherently sequential (each parent_id reads the
  // previous draft's row).
  /* oxlint-disable no-await-in-loop */
  for (const candidate of candidates) {
    await db.exec('BEGIN IMMEDIATE',);
    try {
      await run({
        sql: 'DELETE FROM messages WHERE id = ?',
        params: [candidate.id,],
      },);
      // Collect every draft in the chain, then DELETE them. Chunks
      // cascade via FK. We walk in JS because Turso does not implement
      // recursive CTEs. Chain depth is bounded by the 10-revision cap.
      /**
       * Collected draft chain ids; deleted in a single sweep at the end of the transaction.
       */
      const chainIds: string[] = [];
      /**
       * Walk cursor; advances to each draft's parent until the chain terminates.
       */
      let cursor: string | typeof CHAIN_END = candidate.draft_id;
      while (cursor !== CHAIN_END) {
        chainIds.push(cursor,);
        cursor = await stepToParent(cursor,);
      }
      for (const id of chainIds) {
        await run({
          sql: 'DELETE FROM drafts WHERE id = ?',
          params: [id,],
        },);
      }
      await db.exec('COMMIT',);
    }
    catch (error) {
      await db.exec('ROLLBACK',);
      throw error;
    }
  }
  /* oxlint-enable no-await-in-loop */
}
