/**
 * Unit tests for the dispatcher.
 *
 * Sets `DB_PATH=:memory:` before importing dependent modules so each
 * test runs against an isolated, ephemeral libSQL instance.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

Reflect.set(
  process.env,
  'DB_PATH',
  ':memory:',
);

const queriesMod = await import('../data/queries.ts');
const dispatcherMod = await import('./dispatcher.ts');
const fragmentKeysMod = await import('./fragment-keys.ts');
const memoryAdapterMod = await import('../storage/adapter-memory.ts');

const {
  createCommentWithEvent,
  createIssueWithEvent,
  insertLabel,
  insertRepo,
  insertUser,
  labelIssueWithEvent,
  upsertFragmentIndexIfNewer,
} = queriesMod;
const { processEvent, } = dispatcherMod;
const {
  ANY_LABEL,
  filterListKey,
  issueDetailKey,
} = fragmentKeysMod;
const { createMemoryStorage, } = memoryAdapterMod;

/** Module-scope counter container; held on a `const` object so the mutation stays out of module-root let. */
const counterState: { value: number; } = { value: 0, };

/** Generates a deterministic but unique id. */
function uniqueId(tag: string,): string {
  counterState.value += 1;
  return `t-${tag}-${String(counterState.value,)}`;
}

/**
 * Sets up a user, repo, and issue. Returns the ids for follow-up
 * writes plus the `(eventId, sequenceNumber)` produced by the issue
 * insert.
 */
async function setupIssue(): Promise<{
  userId: string;
  repoId: string;
  issueId: string;
  issueEventId: number;
  issueSequence: number;
}> {
  const userId = uniqueId('user',);
  const repoId = uniqueId('repo',);
  const issueId = uniqueId('issue',);
  const now = Date.now();
  await insertUser({
    id: userId,
    login: `login-${userId}`,
    createdAt: now,
  },);
  await insertRepo({
    id: repoId,
    ownerId: userId,
    name: `repo-${repoId}`,
    createdAt: now,
  },);
  const issueEventId = await createIssueWithEvent({
    id: issueId,
    repoId,
    number: 1,
    authorId: userId,
    title: 'Test',
    createdAt: now,
  },);
  return {
    userId,
    repoId,
    issueId,
    issueEventId,
    issueSequence: 1,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: processEvent.name,
      concurrency: 1,
      children: [
        it({
          name: 'rebuilds the issue detail fragment after a comment.created event',
          async fn() {
            const setup = await setupIssue();
            const storage = createMemoryStorage();
            // Process the issue.created event so the detail fragment exists.
            await processEvent({
              event: {
                kind: 'issue.created',
                resourceId: setup.issueId,
              },
              sequenceNumber: setup.issueSequence,
              eventId: setup.issueEventId,
              sink: storage,
            },);
            // Add a comment, advancing the sequence.
            const commentId = uniqueId('comment',);
            const commentEventId = await createCommentWithEvent({
              id: commentId,
              issueId: setup.issueId,
              authorId: setup.userId,
              body: 'A comment',
              createdAt: Date.now(),
            },);
            const result = await processEvent({
              event: {
                kind: 'comment.created',
                resourceId: setup.issueId,
              },
              sequenceNumber: 2,
              eventId: commentEventId,
              sink: storage,
            },);
            expect(result.fanout > 0,).toBe(true,);
            const detail = await storage.get(issueDetailKey({
              repoId: setup.repoId,
              issueId: setup.issueId,
            },),);
            expect(detail,).toBeDefined();
            const text = new TextDecoder().decode(detail,);
            expect(text.includes('A comment',),).toBe(true,);
          },
        },),
        it({
          name: 'discards a rebuild when the sequence guard races',
          async fn() {
            const setup = await setupIssue();
            const storage = createMemoryStorage();
            // Plant a future-dated index entry to simulate a winning race.
            const detailKey = issueDetailKey({
              repoId: setup.repoId,
              issueId: setup.issueId,
            },);
            await upsertFragmentIndexIfNewer({
              fragmentKey: detailKey,
              contentHash: 'planted-hash',
              lastBuiltAt: Date.now(),
              sourceEventId: 999,
              sourceEventSequence: 999,
            },);
            const result = await processEvent({
              event: {
                kind: 'issue.created',
                resourceId: setup.issueId,
              },
              sequenceNumber: setup.issueSequence,
              eventId: setup.issueEventId,
              sink: storage,
            },);
            expect(result.discarded > 0,).toBe(true,);
          },
        },),
        it({
          name: 'skips puts when the new content hash matches the previous one',
          async fn() {
            const setup = await setupIssue();
            const storage = createMemoryStorage();
            await processEvent({
              event: {
                kind: 'issue.created',
                resourceId: setup.issueId,
              },
              sequenceNumber: setup.issueSequence,
              eventId: setup.issueEventId,
              sink: storage,
            },);
            // Re-run the same event with a higher sequence number;
            // the rendered output is identical, so the hash matches
            // and every put is skipped.
            const second = await processEvent({
              event: {
                kind: 'issue.created',
                resourceId: setup.issueId,
              },
              sequenceNumber: setup.issueSequence + 5,
              eventId: setup.issueEventId + 5,
              sink: storage,
            },);
            expect(second.skipped,).toBe(second.fanout,);
            expect(second.written,).toBe(0,);
          },
        },),
        it({
          name: 'fans out filter-list invalidations on issue.labeled',
          async fn() {
            const setup = await setupIssue();
            const storage = createMemoryStorage();
            const labelA = uniqueId('lblA',);
            const labelB = uniqueId('lblB',);
            await insertLabel({
              id: labelA,
              repoId: setup.repoId,
              name: 'bug',
            },);
            await insertLabel({
              id: labelB,
              repoId: setup.repoId,
              name: 'feat',
            },);
            const eventId = await labelIssueWithEvent({
              issueId: setup.issueId,
              labelId: labelA,
              createdAt: Date.now(),
            },);
            const result = await processEvent({
              event: {
                kind: 'issue.labeled',
                resourceId: setup.issueId,
              },
              sequenceNumber: 2,
              eventId,
              sink: storage,
            },);
            // Detail + (any-label x 2 states) + (labelA x 2 states for "issue has labelA") + (labelA, labelB for "repo-wide on current state").
            expect(result.fanout > 0,).toBe(true,);
            const noFilterOpen = await storage.get(filterListKey({
              repoId: setup.repoId,
              labelId: ANY_LABEL,
              state: 'open',
            },),);
            expect(noFilterOpen,).toBeDefined();
            const labelBOpen = await storage.get(filterListKey({
              repoId: setup.repoId,
              labelId: labelB,
              state: 'open',
            },),);
            expect(labelBOpen,).toBeDefined();
          },
        },),
      ],
    },),
  ],
},);
