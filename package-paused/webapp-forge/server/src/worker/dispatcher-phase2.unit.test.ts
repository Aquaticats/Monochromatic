/**
 * Phase 2 dispatcher tests: end-to-end coverage that PR events fan out
 * the way the dependency graph promises and the renderers actually run.
 *
 * Sets `DB_PATH=:memory:` before importing dependent modules so each
 * test file runs against an isolated, ephemeral libSQL instance.
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
  createPullRequestWithEvent,
  insertRepo,
  insertUser,
  submitReviewWithEvent,
} = queriesMod;
const { processEvent, } = dispatcherMod;
const {
  commentKey,
  mergeStatusKey,
  prDetailKey,
  reviewThreadKey,
} = fragmentKeysMod;
const { createMemoryStorage, } = memoryAdapterMod;

/** Module-scope counter container; held on a `const` object so the mutation stays out of module-root let. */
const counterState: { value: number; } = { value: 0, };

/** Generates a deterministic but unique id within this test file. */
function uniqueId(tag: string,): string {
  counterState.value += 1;
  return `dpx-${tag}-${String(counterState.value,)}`;
}

/** Sets up a user and repo, returns their ids. */
async function setupUserRepo(): Promise<{
  userId: string;
  repoId: string;
}> {
  const userId = uniqueId('user',);
  const repoId = uniqueId('repo',);
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
  return {
    userId,
    repoId,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'processEvent (Phase 2)',
      concurrency: 1,
      children: [
        it({
          name: 'pr.opened rebuilds prDetail, mergeStatus, and reviewThread',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('pr',);
            const eventId = await createPullRequestWithEvent({
              issueId,
              repoId,
              number: 1,
              authorId: userId,
              title: 'feat',
              baseRef: 'refs/heads/main',
              headRef: 'refs/heads/feat-x',
              headSha: '0123456789abcdef0123456789abcdef01234567',
              createdAt: Date.now(),
            },);
            const storage = createMemoryStorage();
            const result = await processEvent({
              event: {
                kind: 'pr.opened',
                resourceId: issueId,
              },
              sequenceNumber: 1,
              eventId,
              sink: storage,
            },);
            expect(result.fanout > 0,).toBe(true,);
            const detail = await storage.get(prDetailKey({
              repoId,
              issueId,
            },),);
            expect(detail,).toBeDefined();
            const merge = await storage.get(mergeStatusKey({
              repoId,
              issueId,
            },),);
            expect(merge,).toBeDefined();
            const reviews = await storage.get(reviewThreadKey({
              repoId,
              issueId,
            },),);
            expect(reviews,).toBeDefined();
          },
        },),
        it({
          name:
            'review.submitted rebuilds prDetail and reviewThread but not the reviewless mergeStatus body',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('pr',);
            const reviewerId = uniqueId('reviewer',);
            await insertUser({
              id: reviewerId,
              login: `login-${reviewerId}`,
              createdAt: Date.now(),
            },);
            const openedEventId = await createPullRequestWithEvent({
              issueId,
              repoId,
              number: 2,
              authorId: userId,
              title: 'feat',
              baseRef: 'refs/heads/main',
              headRef: 'refs/heads/feat-y',
              headSha: 'aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111',
              createdAt: Date.now(),
            },);
            const storage = createMemoryStorage();
            await processEvent({
              event: {
                kind: 'pr.opened',
                resourceId: issueId,
              },
              sequenceNumber: 1,
              eventId: openedEventId,
              sink: storage,
            },);
            const reviewEventId = await submitReviewWithEvent({
              id: uniqueId('rv',),
              prIssueId: issueId,
              reviewerId,
              state: 'approved',
              body: 'LGTM',
              createdAt: Date.now(),
            },);
            const result = await processEvent({
              event: {
                kind: 'review.submitted',
                resourceId: issueId,
              },
              sequenceNumber: 2,
              eventId: reviewEventId,
              sink: storage,
            },);
            expect(result.fanout > 0,).toBe(true,);
            const detail = await storage.get(prDetailKey({
              repoId,
              issueId,
            },),);
            const detailText = new TextDecoder().decode(detail,);
            expect(detailText.includes('1 approved',),).toBe(true,);
            const reviews = await storage.get(reviewThreadKey({
              repoId,
              issueId,
            },),);
            const reviewsText = new TextDecoder().decode(reviews,);
            expect(reviewsText.includes('LGTM',),).toBe(true,);
          },
        },),
        it({
          name: 'comment.created with commentId rebuilds the standalone comment fragment',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('icmt',);
            const issueEventId = await createIssueWithEvent({
              id: issueId,
              repoId,
              number: 1,
              authorId: userId,
              title: 'Issue',
              createdAt: Date.now(),
            },);
            const storage = createMemoryStorage();
            await processEvent({
              event: {
                kind: 'issue.created',
                resourceId: issueId,
              },
              sequenceNumber: 1,
              eventId: issueEventId,
              sink: storage,
            },);
            const commentId = uniqueId('cmt',);
            const commentEventId = await createCommentWithEvent({
              id: commentId,
              issueId,
              authorId: userId,
              body: 'Hello there',
              createdAt: Date.now(),
            },);
            await processEvent({
              event: {
                kind: 'comment.created',
                resourceId: issueId,
                commentId,
              },
              sequenceNumber: 2,
              eventId: commentEventId,
              sink: storage,
            },);
            const standalone = await storage.get(commentKey(commentId,),);
            expect(standalone,).toBeDefined();
            const text = new TextDecoder().decode(standalone,);
            expect(text.includes('Hello there',),).toBe(true,);
            expect(text.includes(`id="comment-${commentId}"`,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
