/**
 * Unit tests for Phase 2 query modules that write event-log rows: PRs,
 * reviews, and the mention reverse index.
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

const queriesMod = await import('./queries.ts');

const {
  addMention,
  createPullRequestWithEvent,
  getPullRequest,
  insertRepo,
  insertReview,
  insertUser,
  listEventsAfter,
  listFragmentsMentioningUser,
  listPullRequestsByHeadSha,
  listReviewsForPr,
  listUsersMentionedByFragment,
  pushPullRequestHead,
  removeMention,
  replaceMentionsForFragment,
  submitReviewWithEvent,
} = queriesMod;

/** Module-scope counter container; held on a `const` object so the mutation stays out of module-root let. */
const counterState: { value: number; } = { value: 0, };

/** Generates a deterministic but unique id within this test file. */
function uniqueId(tag: string,): string {
  counterState.value += 1;
  return `q2e-${tag}-${String(counterState.value,)}`;
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
    name: `name-${repoId}`,
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
      name: 'pull requests',
      concurrency: 1,
      children: [
        it({
          name: 'createPullRequestWithEvent inserts issue + pr + emits pr.opened',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('pr',);
            const headSha = '0123456789abcdef0123456789abcdef01234567';
            const eventId = await createPullRequestWithEvent({
              issueId,
              repoId,
              number: 1,
              authorId: userId,
              title: 'feat',
              baseRef: 'refs/heads/main',
              headRef: 'refs/heads/feat-x',
              headSha,
              createdAt: 100,
            },);
            const pr = await getPullRequest(issueId,);
            expect(pr?.head_sha,).toBe(headSha,);
            expect(pr?.mergeable,).toBe('unknown',);

            const matched = await listPullRequestsByHeadSha(headSha,);
            expect(matched.length,).toBe(1,);
            expect(matched[0]?.issue_id,).toBe(issueId,);

            const newer = await listEventsAfter({
              afterId: eventId - 1,
              limit: 10,
            },);
            const last = newer.at(-1,);
            expect(last?.kind,).toBe('pr.opened',);
            expect(last?.resource_type,).toBe('pr',);
            expect(last?.resource_id,).toBe(issueId,);
          },
        },),
        it({
          name: 'pushPullRequestHead updates head_sha + emits push event',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('prpush',);
            const oldSha = 'aaaa1111aaaa1111aaaa1111aaaa1111aaaa1111';
            const newSha = 'bbbb2222bbbb2222bbbb2222bbbb2222bbbb2222';
            await createPullRequestWithEvent({
              issueId,
              repoId,
              number: 2,
              authorId: userId,
              title: 'feat',
              baseRef: 'refs/heads/main',
              headRef: 'refs/heads/feat-y',
              headSha: oldSha,
              createdAt: 100,
            },);
            const pushEventId = await pushPullRequestHead({
              issueId,
              headSha: newSha,
              mergeable: 'clean',
              createdAt: 200,
            },);
            const pr = await getPullRequest(issueId,);
            expect(pr?.head_sha,).toBe(newSha,);
            expect(pr?.mergeable,).toBe('clean',);

            const events = await listEventsAfter({
              afterId: pushEventId - 1,
              limit: 10,
            },);
            const last = events.at(-1,);
            expect(last?.kind,).toBe('push',);
          },
        },),
      ],
    },),
    describe({
      name: 'reviews',
      concurrency: 1,
      children: [
        it({
          name: 'submitReviewWithEvent inserts review + emits review.submitted',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('rvpr',);
            await createPullRequestWithEvent({
              issueId,
              repoId,
              number: 3,
              authorId: userId,
              title: 'feat',
              baseRef: 'refs/heads/main',
              headRef: 'refs/heads/feat-z',
              headSha: 'cccc3333cccc3333cccc3333cccc3333cccc3333',
              createdAt: 100,
            },);
            const reviewerId = uniqueId('reviewer',);
            await insertUser({
              id: reviewerId,
              login: `l-${reviewerId}`,
              createdAt: 1,
            },);
            const eventId = await submitReviewWithEvent({
              id: uniqueId('rv',),
              prIssueId: issueId,
              reviewerId,
              state: 'approved',
              body: 'LGTM',
              createdAt: 200,
            },);
            const reviews = await listReviewsForPr(issueId,);
            expect(reviews.length,).toBe(1,);
            expect(reviews[0]?.state,).toBe('approved',);

            const newer = await listEventsAfter({
              afterId: eventId - 1,
              limit: 10,
            },);
            const last = newer.at(-1,);
            expect(last?.kind,).toBe('review.submitted',);
          },
        },),
        it({
          name: 'insertReview is idempotent on duplicate id (no event)',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('rvdup',);
            await createPullRequestWithEvent({
              issueId,
              repoId,
              number: 4,
              authorId: userId,
              title: 'feat',
              baseRef: 'refs/heads/main',
              headRef: 'refs/heads/feat-w',
              headSha: 'dddd4444dddd4444dddd4444dddd4444dddd4444',
              createdAt: 100,
            },);
            const reviewerId = uniqueId('reviewer',);
            await insertUser({
              id: reviewerId,
              login: `l-${reviewerId}`,
              createdAt: 1,
            },);
            const reviewId = uniqueId('rv',);
            await insertReview({
              id: reviewId,
              prIssueId: issueId,
              reviewerId,
              state: 'commented',
              body: 'first',
              createdAt: 200,
            },);
            await insertReview({
              id: reviewId,
              prIssueId: issueId,
              reviewerId,
              state: 'changes_requested',
              body: 'second',
              createdAt: 300,
            },);
            const reviews = await listReviewsForPr(issueId,);
            expect(reviews.length,).toBe(1,);
            expect(reviews[0]?.body,).toBe('first',);
          },
        },),
      ],
    },),
    describe({
      name: 'mention index',
      concurrency: 1,
      children: [
        it({
          name: 'add + list + remove',
          async fn() {
            const userIdA = uniqueId('m',);
            const userIdB = uniqueId('m',);
            await insertUser({
              id: userIdA,
              login: `l-${userIdA}`,
              createdAt: 1,
            },);
            await insertUser({
              id: userIdB,
              login: `l-${userIdB}`,
              createdAt: 1,
            },);
            const fragmentKey = `frags/${uniqueId('f',)}`;
            await addMention({
              userId: userIdA,
              fragmentKey,
            },);
            await addMention({
              userId: userIdB,
              fragmentKey,
            },);
            const users = await listUsersMentionedByFragment(fragmentKey,);
            expect(users.length,).toBe(2,);

            const fragmentsForA = await listFragmentsMentioningUser(userIdA,);
            expect(fragmentsForA,).toEqual([fragmentKey,],);

            await removeMention({
              userId: userIdA,
              fragmentKey,
            },);
            const fragmentsAfter = await listFragmentsMentioningUser(userIdA,);
            expect(fragmentsAfter.length,).toBe(0,);
          },
        },),
        it({
          name: 'replaceMentionsForFragment swaps the set atomically',
          async fn() {
            const userIdA = uniqueId('rm',);
            const userIdB = uniqueId('rm',);
            const userIdC = uniqueId('rm',);
            await insertUser({
              id: userIdA,
              login: `l-${userIdA}`,
              createdAt: 1,
            },);
            await insertUser({
              id: userIdB,
              login: `l-${userIdB}`,
              createdAt: 1,
            },);
            await insertUser({
              id: userIdC,
              login: `l-${userIdC}`,
              createdAt: 1,
            },);
            const fragmentKey = `frags/${uniqueId('rf',)}`;
            await addMention({
              userId: userIdA,
              fragmentKey,
            },);
            await addMention({
              userId: userIdB,
              fragmentKey,
            },);
            await replaceMentionsForFragment({
              fragmentKey,
              userIds: [userIdC,],
            },);
            const after = await listUsersMentionedByFragment(fragmentKey,);
            expect(after,).toEqual([userIdC,],);
          },
        },),
      ],
    },),
  ],
},);
