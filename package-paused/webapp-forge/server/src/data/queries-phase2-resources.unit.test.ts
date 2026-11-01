/**
 * Unit tests for Phase 2 resource query modules: orgs, repo membership,
 * issue assignees, and milestones.
 *
 * Sets `DB_PATH=:memory:` before importing dependent modules so each test
 * file runs against an isolated, ephemeral libSQL instance.
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
  assignUserToIssue,
  clearIssueMilestone,
  createIssueWithEvent,
  getIssueMilestoneId,
  getMilestone,
  getOrg,
  getOrgByName,
  getRepoMember,
  insertLabel,
  insertMilestone,
  insertOrg,
  insertRepo,
  insertUser,
  listIssueAssignees,
  listRepoMembers,
  listRepoMilestones,
  removeRepoMember,
  setIssueMilestone,
  unassignUserFromIssue,
  upsertRepoMember,
} = queriesMod;

/** Module-scope counter container; held on a `const` object so the mutation stays out of module-root let. */
const counterState: { value: number; } = { value: 0, };

/** Generates a deterministic but unique id within this test file. */
function uniqueId(tag: string,): string {
  counterState.value += 1;
  return `q2r-${tag}-${String(counterState.value,)}`;
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
      name: 'orgs',
      concurrency: 1,
      children: [
        it({
          name: 'insertOrg + getOrg + getOrgByName round-trip',
          async fn() {
            const orgId = uniqueId('org',);
            const orgName = `org-${orgId}`;
            await insertOrg({
              id: orgId,
              name: orgName,
              createdAt: 100,
            },);
            const byId = await getOrg(orgId,);
            const byName = await getOrgByName(orgName,);
            expect(byId?.id,).toBe(orgId,);
            expect(byName?.name,).toBe(orgName,);
            expect(byId?.created_at,).toBe(100,);
          },
        },),
        it({
          name: 'insertOrg is idempotent on duplicate id',
          async fn() {
            const orgId = uniqueId('org-dup',);
            await insertOrg({
              id: orgId,
              name: `n-${orgId}`,
              createdAt: 1,
            },);
            await insertOrg({
              id: orgId,
              name: `n-${orgId}-rename`,
              createdAt: 2,
            },);
            const row = await getOrg(orgId,);
            expect(row?.name,).toBe(`n-${orgId}`,);
          },
        },),
      ],
    },),
    describe({
      name: 'repo membership',
      concurrency: 1,
      children: [
        it({
          name: 'upsert + role update + remove',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            await upsertRepoMember({
              repoId,
              userId,
              role: 'reader',
            },);
            const initial = await getRepoMember({
              repoId,
              userId,
            },);
            expect(initial?.role,).toBe('reader',);
            await upsertRepoMember({
              repoId,
              userId,
              role: 'owner',
            },);
            const promoted = await getRepoMember({
              repoId,
              userId,
            },);
            expect(promoted?.role,).toBe('owner',);
            await removeRepoMember({
              repoId,
              userId,
            },);
            const removed = await getRepoMember({
              repoId,
              userId,
            },);
            expect(removed,).toBe(undefined,);
          },
        },),
        it({
          name: 'listRepoMembers returns alphabetically by user id',
          async fn() {
            const repoId = uniqueId('mrepo',);
            const userIdA = uniqueId('ma',);
            const userIdB = uniqueId('mb',);
            const ownerId = uniqueId('mowner',);
            await insertUser({
              id: ownerId,
              login: `l-${ownerId}`,
              createdAt: 1,
            },);
            await insertRepo({
              id: repoId,
              ownerId,
              name: `n-${repoId}`,
              createdAt: 1,
            },);
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
            await upsertRepoMember({
              repoId,
              userId: userIdB,
              role: 'reader',
            },);
            await upsertRepoMember({
              repoId,
              userId: userIdA,
              role: 'reader',
            },);
            const members = await listRepoMembers(repoId,);
            expect(members.length,).toBe(2,);
            expect([...members,].map(function pickUserId(m,) {
              return m.user_id;
            },),)
              .toEqual([userIdA, userIdB,].toSorted(function compareAsc(
                a,
                b,
              ) {
                return a < b ? -1 : 1;
              },),);
          },
        },),
      ],
    },),
    describe({
      name: 'issue assignees',
      concurrency: 1,
      children: [
        it({
          name: 'assign + list + unassign',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const issueId = uniqueId('iassign',);
            await createIssueWithEvent({
              id: issueId,
              repoId,
              number: 1,
              authorId: userId,
              title: 'hello',
              createdAt: 1,
            },);
            const otherUserId = uniqueId('other',);
            await insertUser({
              id: otherUserId,
              login: `l-${otherUserId}`,
              createdAt: 1,
            },);
            await assignUserToIssue({
              issueId,
              userId: otherUserId,
            },);
            const assignees = await listIssueAssignees(issueId,);
            expect(assignees.length,).toBe(1,);
            expect(assignees[0]?.id,).toBe(otherUserId,);
            await unassignUserFromIssue({
              issueId,
              userId: otherUserId,
            },);
            const after = await listIssueAssignees(issueId,);
            expect(after.length,).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'milestones',
      concurrency: 1,
      children: [
        it({
          name: 'insert + get + listRepoMilestones + setIssueMilestone',
          async fn() {
            const { userId, repoId, } = await setupUserRepo();
            const milestoneId = uniqueId('ms',);
            await insertMilestone({
              id: milestoneId,
              repoId,
              title: 'v1',
              dueAt: 5_000,
            },);
            const ms = await getMilestone(milestoneId,);
            expect(ms?.title,).toBe('v1',);
            expect(ms?.due_at,).toBe(5_000,);

            const issueId = uniqueId('ims',);
            await createIssueWithEvent({
              id: issueId,
              repoId,
              number: 7,
              authorId: userId,
              title: 'hello',
              createdAt: 1,
            },);
            await setIssueMilestone({
              issueId,
              milestoneId,
            },);
            const linked = await getIssueMilestoneId(issueId,);
            expect(linked,).toBe(milestoneId,);

            const repoMilestones = await listRepoMilestones(repoId,);
            expect(repoMilestones.length,).toBe(1,);
            expect(repoMilestones[0]?.id,).toBe(milestoneId,);

            await clearIssueMilestone(issueId,);
            const cleared = await getIssueMilestoneId(issueId,);
            expect(cleared,).toBe(undefined,);

            // Guard against unused import warning; exercise insertLabel here too
            const labelId = uniqueId('lbl',);
            await insertLabel({
              id: labelId,
              repoId,
              name: `n-${labelId}`,
            },);
          },
        },),
      ],
    },),
  ],
},);
