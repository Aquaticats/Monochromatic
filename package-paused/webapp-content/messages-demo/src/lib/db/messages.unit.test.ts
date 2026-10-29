/**
 * Tests the copy-on-write chain walk in `getChunk`. Uses an in-memory
 * SQLite database (set via `DB_PATH` before the dynamic import below)
 * so we never touch the disk fixture.
 *
 * Concurrency is 1: every test mutates the singleton database, so
 * isolating writes with unique ids is not enough; we serialise
 * to keep transactions linear.
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

const draftsMod = await import('./drafts.ts');
const messagesMod = await import('./messages.ts');

const {
  createDraft,
  putChunk,
  finalizeDraft,
  REJECTED,
} = draftsMod;
const {
  ABSENT,
  editMessage,
  getChunk,
  getSnapshot,
  listFeed,
  feedAggregates,
  messageExists,
  softDeleteMessage,
  MAX_REVISIONS,
} = messagesMod;

/**
 * Narrows a `getChunk` result to a present chunk, throwing when it is
 * the `ABSENT` sentinel. Keeps the positive-path assertions terse.
 *
 * @param chunk - result returned by `getChunk`
 *
 * @returns present chunk row
 *
 * @throws `Error` when `chunk` is `ABSENT`
 *
 * @example
 * ```ts
 * expect(requireChunk(await getChunk({ messageId, chunkIndex: 0 })).md).toBe('# hi');
 * ```
 */
function requireChunk(
  chunk: Awaited<ReturnType<typeof getChunk>>,
): {
  readonly md: string;
  readonly html: string;
} {
  // `typeof` narrows away the `ABSENT` symbol arm: dynamic `import()`
  // widens the exported `unique symbol` to `symbol`, so `=== ABSENT`
  // would not narrow the union here. The only symbol value is `ABSENT`.
  if ((typeof chunk) === 'symbol')
    throw new Error('expected a chunk, got ABSENT',);
  return chunk;
}

/**
 * Generates a deterministic but unique draft id for a single test. The
 * counter avoids collisions while keeping ids debuggable.
 *
 * @param tag - short tag identifying the test, surfaced in the id
 *
 * @returns globally-unique draft id
 *
 * @example
 * ```ts
 * uniqueId('finalize'); // 't-finalize-1'
 * uniqueId('finalize'); // 't-finalize-2'
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
 * Creates a draft with a single chunk and finalises it. Returns the
 * resulting message id.
 *
 * @param input - identity, draft id, and chunk content
 *
 * @returns inserted message id
 */
async function makeMessage(
  input: {
    readonly draftId: string;
    readonly userId: string;
    readonly md: string;
    readonly html: string;
  },
): Promise<number> {
  await createDraft({
    id: input.draftId,
    userId: input.userId,
  },);
  await putChunk({
    draftId: input.draftId,
    seq: 0,
    chunk: {
      md: input.md,
      html: input.html,
      charCount: input.md.length,
    },
  },);
  const id = await finalizeDraft({
    draftId: input.draftId,
    userId: input.userId,
    charCount: input.md.length,
    chunkCount: 1,
    preview: input.md.slice(
      0,
      50,
    ),
  },);
  // `typeof` narrows away the widened-symbol `REJECTED` arm; see `requireChunk`.
  if ((typeof id) === 'symbol')
    throw new Error('finalizeDraft rejected the draft',);
  return id;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: getChunk.name,
      concurrency: 1,
      children: [
        it({
          name: 'returns chunk content for a single-revision message',
          fn: async () => {
            const draftId = uniqueId('single',);
            const messageId = await makeMessage({
              draftId,
              userId: 'user-a',
              md: '# hello',
              html: '<h1>hello</h1>',
            },);
            const chunk = await getChunk({
              messageId,
              chunkIndex: 0,
            },);
            expect(requireChunk(chunk,).md,).toBe('# hello',);
            expect(requireChunk(chunk,).html,).toBe('<h1>hello</h1>',);
          },
        },),

        it({
          name: 'returns ABSENT for chunk index out of range',
          fn: async () => {
            const draftId = uniqueId('oor',);
            const messageId = await makeMessage({
              draftId,
              userId: 'user-a',
              md: 'a',
              html: '<p>a</p>',
            },);
            const chunk = await getChunk({
              messageId,
              chunkIndex: 99,
            },);
            expect(chunk,).toBe(ABSENT,);
          },
        },),

        it({
          name: 'returns ABSENT for non-existent message id',
          fn: async () => {
            const chunk = await getChunk({
              messageId: 9_999_999,
              chunkIndex: 0,
            },);
            expect(chunk,).toBe(ABSENT,);
          },
        },),

        it({
          name:
            '5-deep chain returns the latest revision of edited chunks and the original of unedited ones',
          fn: async () => {
            // Build initial 3-chunk message.
            const root = uniqueId('chain-root',);
            await createDraft({
              id: root,
              userId: 'user-a',
            },);
            for (const seq of [0, 1, 2,]) {
              // oxlint-disable-next-line no-await-in-loop
              await putChunk({
                draftId: root,
                seq,
                chunk: {
                  md: `r1-c${String(seq,)}`,
                  html: `<p>r1-c${String(seq,)}</p>`,
                  charCount: 5,
                },
              },);
            }
            const messageId = await finalizeDraft({
              draftId: root,
              userId: 'user-a',
              charCount: 15,
              chunkCount: 3,
              preview: 'p',
            },);
            if ((typeof messageId) === 'symbol')
              throw new Error('finalizeDraft rejected the draft',);

            // 4 edits, each editing one of the 3 chunks in rotation.
            let parent = root;
            const editOrder = [0, 1, 2, 0,];
            for (const [revisionOffset, editedSeq,] of editOrder.entries()) {
              const revision = revisionOffset + 2;
              const child = uniqueId('chain-edit',);
              // oxlint-disable-next-line no-await-in-loop
              await createDraft({
                id: child,
                userId: 'user-a',
                parentId: parent,
              },);
              // oxlint-disable-next-line no-await-in-loop
              await putChunk({
                draftId: child,
                seq: editedSeq,
                chunk: {
                  md: `r${String(revision,)}-c${String(editedSeq,)}`,
                  html: `<p>r${String(revision,)}-c${String(editedSeq,)}</p>`,
                  charCount: 5,
                },
              },);
              // oxlint-disable-next-line no-await-in-loop
              const outcome = await editMessage({
                messageId,
                userId: 'user-a',
                newDraftId: child,
                charCount: 15,
                chunkCount: 3,
                preview: 'p',
              },);
              expect(outcome.kind,).toBe('ok',);
              parent = child;
            }
            // Final revision is 5 (1 initial + 4 edits). Edits in
            // order: c0,c1,c2,c0. Last edit of each chunk: c0=r5,
            // c1=r3, c2=r4.
            const c0 = await getChunk({
              messageId,
              chunkIndex: 0,
            },);
            const c1 = await getChunk({
              messageId,
              chunkIndex: 1,
            },);
            const c2 = await getChunk({
              messageId,
              chunkIndex: 2,
            },);
            expect(requireChunk(c0,).md,).toBe('r5-c0',);
            expect(requireChunk(c1,).md,).toBe('r3-c1',);
            expect(requireChunk(c2,).md,).toBe('r4-c2',);
          },
        },),
      ],
    },),

    describe({
      name: editMessage.name,
      concurrency: 1,
      children: [
        it({
          name: 'returns not-found for unknown message id',
          fn: async () => {
            const newDraftId = uniqueId('e-nf',);
            await createDraft({
              id: newDraftId,
              userId: 'user-a',
            },);
            const outcome = await editMessage({
              messageId: 9_999_999,
              userId: 'user-a',
              newDraftId,
              charCount: 1,
              chunkCount: 1,
              preview: 'p',
            },);
            expect(outcome.kind,).toBe('not-found',);
          },
        },),

        it({
          name: 'returns forbidden when a different user attempts the edit',
          fn: async () => {
            const draftId = uniqueId('e-403',);
            const messageId = await makeMessage({
              draftId,
              userId: 'user-a',
              md: 'orig',
              html: '<p>orig</p>',
            },);
            const newDraftId = uniqueId('e-403-new',);
            await createDraft({
              id: newDraftId,
              userId: 'user-b',
              parentId: draftId,
            },);
            const outcome = await editMessage({
              messageId,
              userId: 'user-b',
              newDraftId,
              charCount: 1,
              chunkCount: 1,
              preview: 'p',
            },);
            expect(outcome.kind,).toBe('forbidden',);
          },
        },),

        it({
          name:
            'caps revisions at MAX_REVISIONS and returns "capped" on the next attempt',
          fn: async () => {
            const root = uniqueId('cap-root',);
            const messageId = await makeMessage({
              draftId: root,
              userId: 'user-a',
              md: 'orig',
              html: '<p>orig</p>',
            },);
            let parent = root;
            // We start at revision 1. Apply MAX_REVISIONS - 1 edits
            // to reach the cap (revision == MAX_REVISIONS).
            for (
              let edit = 0;
              edit < (MAX_REVISIONS - 1);
              edit += 1
            ) {
              const child = uniqueId('cap-edit',);
              // oxlint-disable-next-line no-await-in-loop
              await createDraft({
                id: child,
                userId: 'user-a',
                parentId: parent,
              },);
              // oxlint-disable-next-line no-await-in-loop
              await putChunk({
                draftId: child,
                seq: 0,
                chunk: {
                  md: 'edit',
                  html: '<p>edit</p>',
                  charCount: 4,
                },
              },);
              // oxlint-disable-next-line no-await-in-loop
              const okOutcome = await editMessage({
                messageId,
                userId: 'user-a',
                newDraftId: child,
                charCount: 4,
                chunkCount: 1,
                preview: 'p',
              },);
              expect(okOutcome.kind,).toBe('ok',);
              parent = child;
            }
            // Next attempt should return capped.
            const overCap = uniqueId('cap-over',);
            await createDraft({
              id: overCap,
              userId: 'user-a',
              parentId: parent,
            },);
            const cappedOutcome = await editMessage({
              messageId,
              userId: 'user-a',
              newDraftId: overCap,
              charCount: 1,
              chunkCount: 1,
              preview: 'p',
            },);
            expect(cappedOutcome.kind,).toBe('capped',);
          },
        },),
      ],
    },),

    describe({
      name: softDeleteMessage.name,
      concurrency: 1,
      children: [
        it({
          name: 'soft-deletes and excludes from feed; getSnapshot returns ABSENT',
          fn: async () => {
            const draftId = uniqueId('del',);
            const messageId = await makeMessage({
              draftId,
              userId: 'user-a',
              md: 'doomed',
              html: '<p>doomed</p>',
            },);
            const before = await getSnapshot(messageId,);
            expect(before,).not.toBe(ABSENT,);
            const outcome = await softDeleteMessage({
              messageId,
              userId: 'user-a',
            },);
            expect(outcome.kind,).toBe('ok',);
            const after = await getSnapshot(messageId,);
            expect(after,).toBe(ABSENT,);
            // messageExists is true even after soft delete.
            expect(await messageExists(messageId,),).toBe(true,);
            // Feed excludes deleted rows.
            const feed = await listFeed();
            const stillThere = feed.some(function findIt(row,) {
              return row.id === messageId;
            },);
            expect(stillThere,).toBe(false,);
          },
        },),

        it({
          name: 'returns forbidden when a different user attempts the delete',
          fn: async () => {
            const draftId = uniqueId('del-403',);
            const messageId = await makeMessage({
              draftId,
              userId: 'user-a',
              md: 'mine',
              html: '<p>mine</p>',
            },);
            const outcome = await softDeleteMessage({
              messageId,
              userId: 'user-b',
            },);
            expect(outcome.kind,).toBe('forbidden',);
          },
        },),

        it({
          name: 'returns not-found for unknown message id',
          fn: async () => {
            const outcome = await softDeleteMessage({
              messageId: 9_999_999,
              userId: 'user-a',
            },);
            expect(outcome.kind,).toBe('not-found',);
          },
        },),
      ],
    },),

    describe({
      name: feedAggregates.name,
      concurrency: 1,
      children: [
        it({
          name: 'returns positive ids and timestamps after at least one message exists',
          fn: async () => {
            const draftId = uniqueId('feed-agg',);
            await makeMessage({
              draftId,
              userId: 'user-a',
              md: 'agg',
              html: '<p>agg</p>',
            },);
            const aggregates = await feedAggregates();
            expect(aggregates.maxId,).toBeGreaterThan(0,);
            expect(aggregates.maxUpdatedAt,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
  ],
},);
