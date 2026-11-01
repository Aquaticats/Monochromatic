/**
 * Tests the orphan and hard-delete sweeps. Uses an in-memory database
 * via `DB_PATH=:memory:` set before the dynamic import.
 *
 * Concurrency 1: every sweep call walks the singleton database; tests
 * mutate `updated_at` and `deleted_at` directly with the `run` helper to
 * stage rows that look old enough to be reaped without sleeping.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

Reflect.set(
  process.env,
  'DB_PATH',
  ':memory:',
);

const dbMod = await import('../db.ts');
const draftsMod = await import('./drafts.ts');
const sweepMod = await import('./sweep.ts');
const messagesMod = await import('./messages.ts');

const {
  get,
  NO_ROW,
  run,
} = dbMod;
const {
  createDraft,
  putChunk,
  finalizeDraft,
  REJECTED,
} = draftsMod;
const {
  softDeleteMessage,
} = messagesMod;
const {
  sweepDeleted,
  sweepOrphans,
  SWEEP_BATCH,
  DELETED_TTL_MS,
  ORPHAN_TTL_MS,
} = sweepMod;

/**
 * Generates a deterministic but unique draft id.
 *
 * @param tag - short tag identifying the test, surfaced in the id
 *
 * @returns globally-unique draft id
 *
 * @example
 * ```ts
 * uniqueId('orphan'); // 't-orphan-1'
 * uniqueId('orphan'); // 't-orphan-2'
 * ```
 */
const uniqueId = (function makeUniqueId() {
  let counter = 0;
  return function nextId(tag: string,): string {
    counter += 1;
    return `t-${tag}-${String(counter,)}`;
  };
})();

/**
 * Reads back whether a draft row still exists.
 *
 * @param draftId - target id
 *
 * @returns `true` when the row was found
 */
async function draftExists(draftId: string,): Promise<boolean> {
  const row = await get<{ id: string; }>({
    sql: 'SELECT id FROM drafts WHERE id = ?',
    params: [draftId,],
  },);
  return row !== NO_ROW;
}

/**
 * Reads back whether a message row still exists.
 *
 * @param messageId - target id
 *
 * @returns `true` when the row was found
 */
async function messageRowExists(messageId: number,): Promise<boolean> {
  const row = await get<{ id: number; }>({
    sql: 'SELECT id FROM messages WHERE id = ?',
    params: [messageId,],
  },);
  return row !== NO_ROW;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: sweepOrphans.name,
      concurrency: 1,
      children: [
        it({
          name: 'reaps orphan drafts older than ORPHAN_TTL_MS',
          fn: async () => {
            const draftId = uniqueId('orph-old',);
            await createDraft({
              id: draftId,
              userId: 'user-a',            },);
            // Backdate the row so the sweep cutoff catches it.
            const stale = Date.now() - ORPHAN_TTL_MS - 1_000;
            await run({
              sql: 'UPDATE drafts SET updated_at = ? WHERE id = ?',
              params: [
                stale,
                draftId,
              ],
            },);
            await sweepOrphans({ userId: 'user-a', },);
            expect(await draftExists(draftId,),).toBe(false,);
          },
        },),

        it({
          name: 'leaves drafts within TTL alone',
          fn: async () => {
            const draftId = uniqueId('orph-fresh',);
            await createDraft({
              id: draftId,
              userId: 'user-a',            },);
            await sweepOrphans({ userId: 'user-a', },);
            expect(await draftExists(draftId,),).toBe(true,);
          },
        },),

        it({
          name: 'leaves finalised drafts alone even past TTL',
          fn: async () => {
            const draftId = uniqueId('orph-finalised',);
            await createDraft({
              id: draftId,
              userId: 'user-a',            },);
            await putChunk({
              draftId,
              seq: 0,
              chunk: {
                md: 'x',
                html: '<p>x</p>',
                charCount: 1,
              },
            },);
            const messageId = await finalizeDraft({
              draftId,
              userId: 'user-a',
              charCount: 1,
              chunkCount: 1,
              preview: 'x',
            },);
            expect(messageId,).not.toBe(REJECTED,);
            const stale = Date.now() - ORPHAN_TTL_MS - 1_000;
            await run({
              sql: 'UPDATE drafts SET updated_at = ? WHERE id = ?',
              params: [
                stale,
                draftId,
              ],
            },);
            await sweepOrphans({},);
            expect(await draftExists(draftId,),).toBe(true,);
          },
        },),

        it({
          name: 'scoped sweep ignores other users',
          fn: async () => {
            const aDraft = uniqueId('orph-a',);
            const bDraft = uniqueId('orph-b',);
            await createDraft({
              id: aDraft,
              userId: 'user-a',            },);
            await createDraft({
              id: bDraft,
              userId: 'user-b',            },);
            const stale = Date.now() - ORPHAN_TTL_MS - 1_000;
            await run({
              sql: 'UPDATE drafts SET updated_at = ? WHERE id IN (?, ?)',
              params: [
                stale,
                aDraft,
                bDraft,
              ],
            },);
            await sweepOrphans({ userId: 'user-a', },);
            expect(await draftExists(aDraft,),).toBe(false,);
            expect(await draftExists(bDraft,),).toBe(true,);
          },
        },),

        it({
          name: 'caps deletes per call at SWEEP_BATCH',
          fn: async () => {
            const ids = Array.from(
              {
                length: SWEEP_BATCH + 5,
              },
              function gen() {
                return uniqueId('orph-many',);
              },
            );
            for (const id of ids) {
              // oxlint-disable-next-line no-await-in-loop
              await createDraft({
                id,
                userId: 'user-a',              },);
            }
            const stale = Date.now() - ORPHAN_TTL_MS - 1_000;
            for (const id of ids) {
              // oxlint-disable-next-line no-await-in-loop
              await run({
                sql: 'UPDATE drafts SET updated_at = ? WHERE id = ?',
                params: [
                  stale,
                  id,
                ],
              },);
            }
            await sweepOrphans({ userId: 'user-a', },);
            let surviving = 0;
            for (const id of ids) {
              // oxlint-disable-next-line no-await-in-loop
              surviving += (await draftExists(id,)) ? 1 : 0;
            }
            // SWEEP_BATCH were deleted; the rest survived.
            expect(surviving,).toBe(ids.length - SWEEP_BATCH,);
          },
        },),
      ],
    },),

    describe({
      name: sweepDeleted.name,
      concurrency: 1,
      children: [
        it({
          name: 'hard-deletes a soft-deleted message past TTL and walks chain ancestors',
          fn: async () => {
            // Build 2-deep chain.
            const root = uniqueId('del-root',);
            await createDraft({
              id: root,
              userId: 'user-a',            },);
            await putChunk({
              draftId: root,
              seq: 0,
              chunk: {
                md: 'r1',
                html: '<p>r1</p>',
                charCount: 2,
              },
            },);
            const messageId = await finalizeDraft({
              draftId: root,
              userId: 'user-a',
              charCount: 2,
              chunkCount: 1,
              preview: 'p',
            },);
            expect(messageId,).not.toBe(REJECTED,);
            if ((typeof messageId) === 'symbol')
              throw new Error('finalizeDraft rejected the draft',);

            const child = uniqueId('del-child',);
            await createDraft({
              id: child,
              userId: 'user-a',
              parentId: root,
            },);
            await putChunk({
              draftId: child,
              seq: 0,
              chunk: {
                md: 'r2',
                html: '<p>r2</p>',
                charCount: 2,
              },
            },);
            await run({
              sql: 'UPDATE messages SET draft_id = ?, revision = 2 WHERE id = ?',
              params: [
                child,
                messageId,
              ],
            },);
            await run({
              sql: 'UPDATE drafts SET finalized = 1 WHERE id = ?',
              params: [child,],
            },);

            // Soft-delete and backdate.
            const out = await softDeleteMessage({
              messageId,
              userId: 'user-a',
            },);
            expect(out.kind,).toBe('ok',);
            const stale = Date.now() - DELETED_TTL_MS - 1_000;
            await run({
              sql: 'UPDATE messages SET deleted_at = ? WHERE id = ?',
              params: [
                stale,
                messageId,
              ],
            },);
            await sweepDeleted();

            expect(await messageRowExists(messageId,),).toBe(false,);
            expect(await draftExists(root,),).toBe(false,);
            expect(await draftExists(child,),).toBe(false,);
            // FK cascade reaps chunks too.
            const remaining = await get<{ count: number; }>({
              sql: 'SELECT COUNT(*) AS count FROM chunks WHERE draft_id IN (?, ?)',
              params: [
                root,
                child,
              ],
            },);
            if ((typeof remaining) === 'symbol')
              throw new Error('COUNT(*) returned no row',);
            expect(remaining.count,).toBe(0,);
          },
        },),

        it({
          name: 'leaves soft-deleted messages within TTL alone',
          fn: async () => {
            const draftId = uniqueId('del-fresh',);
            await createDraft({
              id: draftId,
              userId: 'user-a',            },);
            await putChunk({
              draftId,
              seq: 0,
              chunk: {
                md: 'x',
                html: '<p>x</p>',
                charCount: 1,
              },
            },);
            const messageId = await finalizeDraft({
              draftId,
              userId: 'user-a',
              charCount: 1,
              chunkCount: 1,
              preview: 'x',
            },);
            if ((typeof messageId) === 'symbol')
              throw new Error('finalizeDraft rejected the draft',);
            await softDeleteMessage({
              messageId,
              userId: 'user-a',
            },);
            await sweepDeleted();
            expect(await messageRowExists(messageId,),).toBe(true,);
            expect(await draftExists(draftId,),).toBe(true,);
          },
        },),

        it({
          name: 'leaves live (non-deleted) messages alone',
          fn: async () => {
            const draftId = uniqueId('del-alive',);
            await createDraft({
              id: draftId,
              userId: 'user-a',            },);
            await putChunk({
              draftId,
              seq: 0,
              chunk: {
                md: 'x',
                html: '<p>x</p>',
                charCount: 1,
              },
            },);
            const messageId = await finalizeDraft({
              draftId,
              userId: 'user-a',
              charCount: 1,
              chunkCount: 1,
              preview: 'x',
            },);
            if ((typeof messageId) === 'symbol')
              throw new Error('finalizeDraft rejected the draft',);
            await sweepDeleted();
            expect(await messageRowExists(messageId,),).toBe(true,);
            expect(await draftExists(draftId,),).toBe(true,);
          },
        },),

        it({
          name: 'caps candidates per call at SWEEP_BATCH',
          fn: async () => {
            const draftIds = Array.from(
              {
                length: SWEEP_BATCH + 3,
              },
              function gen() {
                return uniqueId('del-many',);
              },
            );
            const messageIds: number[] = [];
            for (const draftId of draftIds) {
              // oxlint-disable-next-line no-await-in-loop
              await createDraft({
                id: draftId,
                userId: 'user-a',              },);
              // oxlint-disable-next-line no-await-in-loop
              await putChunk({
                draftId,
                seq: 0,
                chunk: {
                  md: 'x',
                  html: '<p>x</p>',
                  charCount: 1,
                },
              },);
              // oxlint-disable-next-line no-await-in-loop
              const id = await finalizeDraft({
                draftId,
                userId: 'user-a',
                charCount: 1,
                chunkCount: 1,
                preview: 'x',
              },);
              if ((typeof id) === 'symbol')
                throw new Error('finalizeDraft rejected the draft',);
              messageIds.push(id,);
            }
            // Soft-delete + backdate every candidate.
            const stale = Date.now() - DELETED_TTL_MS - 1_000;
            for (const id of messageIds) {
              // oxlint-disable-next-line no-await-in-loop
              await softDeleteMessage({
                messageId: id,
                userId: 'user-a',
              },);
              // oxlint-disable-next-line no-await-in-loop
              await run({
                sql: 'UPDATE messages SET deleted_at = ? WHERE id = ?',
                params: [
                  stale,
                  id,
                ],
              },);
            }
            await sweepDeleted();
            let alive = 0;
            for (const id of messageIds) {
              // oxlint-disable-next-line no-await-in-loop
              alive += (await messageRowExists(id,)) ? 1 : 0;
            }
            expect(alive,).toBe(messageIds.length - SWEEP_BATCH,);
          },
        },),
      ],
    },),
  ],
},);
