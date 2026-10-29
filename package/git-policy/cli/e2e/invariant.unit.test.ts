import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { checkInvariants, } from './invariant-fixture.ts';
import type {
  AttemptObservation,
  ContentState,
  LandingObservation,
  RunObservation,
} from './invariant-model-fixture.ts';
import { judge, } from './scenario-model-fixture.ts';

//region Fixtures

/**
 Content with a fixed digest.

 @param digest - digest text

 @returns present content

 @example
 ```ts
 present('a');
 ```
 */
function present(digest: string,): ContentState {
  return { state: 'present', digest, };
}

/**
 Landing that satisfies every landing invariant.
 */
const GOOD_LANDING: LandingObservation = {
  oid: 'a'.repeat(40,),
  onExpectedBranch: true,
  changedPaths: ['a.txt',],
  paths: [{ path: 'a.txt', landed: present('captured',), acceptable: [present('captured',), present('merged',),], },],
  remoteContains: true,
  onAmendedPublishedHistory: false,
  hooks: { commitMsgTrailer: true, postCommitRuns: 1, signatureValid: true, },
};

/**
 Successful attempt with one good landing.
 */
const GOOD_ATTEMPT: AttemptObservation = {
  label: 'w0',
  exitCode: 0,
  killed: false,
  selectedPaths: ['a.txt',],
  landings: [GOOD_LANDING,],
  eventIssues: [],
  requiresRemote: true,
  pushRejectionSurfaced: false,
};

/**
 Observation that satisfies every invariant.
 */
const CLEAN: RunObservation = {
  attempts: [GOOD_ATTEMPT,],
  auxiliaries: [{ label: 'add', exitCode: 0, mustSucceed: true, eventIssues: [], },],
  worktree: [{ path: 'a.txt', expected: present('captured',), actual: present('captured',), },],
  staged: [{ path: 'b.txt', expectedByHarness: true, revertsLandedContent: false, },],
  leftovers: [],
  fsck: { exitCode: 0, output: '', },
  expectations: [{ name: 'all-commits-succeed', holds: true, detail: '', },],
};

/**
 Checks an observation with one attempt replaced.

 @param attempt - replacement attempt

 @returns violated invariant names

 @example
 ```ts
 invariantsFor({ ...GOOD_ATTEMPT, exitCode: 1 });
 ```
 */
function invariantsFor(attempt: AttemptObservation,): readonly string[] {
  return checkInvariants({ ...CLEAN, attempts: [attempt,], },).map(function name(violation,) {
    return violation.invariant;
  },);
}

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: checkInvariants.name,
      children: [
        it({
          name: 'accepts a clean run, including merged content and harness-staged paths',
          fn: async () => {
            expect(checkInvariants(CLEAN,),).toEqual([],);
            expect(invariantsFor({
              ...GOOD_ATTEMPT,
              landings: [{ ...GOOD_LANDING, paths: [{ path: 'a.txt', landed: present('merged',), acceptable: [present('captured',), present('merged',),], },], },],
            },),).toEqual([],);
          },
        },),
        it({
          name: 'requires exit-0 attempts to land exactly once',
          fn: async () => {
            expect(invariantsFor({ ...GOOD_ATTEMPT, landings: [], },),).toEqual(['landed-once',],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, landings: [GOOD_LANDING, GOOD_LANDING,], },),).toEqual(['landed-once',],);
          },
        },),
        it({
          name: 'forbids failed attempts from landing unless commit-landed names the commit',
          fn: async () => {
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: 1, },),).toEqual(['landed-once',],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: 1, landings: [], },),).toEqual([],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: 2, landedEventOid: GOOD_LANDING.oid, },),).toEqual([],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: 2, landedEventOid: 'b'.repeat(40,), },),).toEqual(['landed-once',],);
          },
        },),
        it({
          name: 'lets a killed attempt land at most once',
          fn: async () => {
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: -1, killed: true, landings: [], },),).toEqual([],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: -1, killed: true, },),).toEqual([],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, exitCode: -1, killed: true, landings: [GOOD_LANDING, GOOD_LANDING,], },),).toEqual(['landed-once',],);
          },
        },),
        it({
          name: 'reports wrong bytes, wrong branch, and out-of-scope paths',
          fn: async () => {
            expect(invariantsFor({
              ...GOOD_ATTEMPT,
              landings: [{
                ...GOOD_LANDING,
                onExpectedBranch: false,
                changedPaths: ['a.txt', 'other.txt',],
                paths: [{ path: 'a.txt', landed: { state: 'absent', }, acceptable: [present('captured',),], },],
              },],
            },),).toEqual(['landed-branch', 'landed-scope', 'landed-bytes',],);
          },
        },),
        it({
          name: 'requires remote containment and hook facts only where they apply',
          fn: async () => {
            const missing = { ...GOOD_LANDING, remoteContains: false, hooks: { commitMsgTrailer: false, postCommitRuns: 2, signatureValid: false, }, };
            expect(invariantsFor({ ...GOOD_ATTEMPT, landings: [missing,], },),)
              .toEqual(['remote-contains', 'commit-msg-hook', 'post-commit-once', 'signature-valid',],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, requiresRemote: false, landings: [{ ...GOOD_LANDING, remoteContains: false, hooks: {}, },], },),)
              .toEqual([],);
          },
        },),
        it({
          name: 'exempts commits on an amended published history from remote containment only when the non-fast-forward rejection was surfaced',
          fn: async () => {
            const amended = { ...GOOD_LANDING, remoteContains: false, onAmendedPublishedHistory: true, };
            expect(invariantsFor({ ...GOOD_ATTEMPT, landings: [amended,], pushRejectionSurfaced: true, },),).toEqual([],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, landings: [amended,], },),).toEqual(['push-rejection-surfaced',],);
            expect(invariantsFor({ ...GOOD_ATTEMPT, landings: [{ ...amended, onAmendedPublishedHistory: false, },], pushRejectionSurfaced: true, },),)
              .toEqual(['remote-contains',],);
          },
        },),
        it({
          name: 'reports event issues of attempts and auxiliaries',
          fn: async () => {
            expect(invariantsFor({ ...GOOD_ATTEMPT, eventIssues: ['exit 0 with a blocking event',], },),).toEqual(['exit-events',],);
            expect(checkInvariants({
              ...CLEAN,
              auxiliaries: [{ label: 'add', exitCode: 128, mustSucceed: true, eventIssues: ['malformed',], },],
            },).map(function name(violation,) {
              return violation.invariant;
            },),).toEqual(['command-succeeds', 'exit-events',],);
          },
        },),
        it({
          name: 'reports lost worktree edits, index reverts, drift, leftovers, fsck, and expectations',
          fn: async () => {
            expect(checkInvariants({
              ...CLEAN,
              worktree: [{ path: 'a.txt', expected: present('captured',), actual: { state: 'absent', }, },],
              staged: [
                { path: 'a.txt', expectedByHarness: false, revertsLandedContent: true, },
                { path: 'c.txt', expectedByHarness: false, revertsLandedContent: false, },
              ],
              leftovers: ['.git/index.lock',],
              fsck: { exitCode: 0, output: 'broken link', },
              expectations: [{ name: 'all-commits-succeed', holds: false, detail: 'w1 exited 2', },],
            },).map(function name(violation,) {
              return violation.invariant;
            },),).toEqual(['worktree-preserved', 'index-no-revert', 'index-matches-head', 'no-leftovers', 'fsck-clean', 'all-commits-succeed',],);
          },
        },),
      ],
    },),
    describe({
      name: judge.name,
      children: [
        it({
          name: 'passes ordinary scenarios only without violations',
          fn: async () => {
            expect(judge({ definition: {}, violations: [], },).passed,).toBe(true,);
            expect(judge({ definition: {}, violations: [{ invariant: 'x', subject: 's', detail: 'd', },], },).passed,).toBe(false,);
          },
        },),
        it({
          name: 'passes positive controls only on the exact planted set',
          fn: async () => {
            const planted = [
              { invariant: 'b', subject: 's', detail: 'd', },
              { invariant: 'a', subject: 's', detail: 'd', },
              { invariant: 'a', subject: 't', detail: 'd', },
            ];
            expect(judge({ definition: { expectedViolations: ['a', 'b',], }, violations: planted, },).passed,).toBe(true,);
            const missed = judge({ definition: { expectedViolations: ['a', 'b', 'c',], }, violations: planted, },);
            expect(missed.passed,).toBe(false,);
            expect(missed.violations[0]?.invariant,).toBe('positive-control',);
            expect(judge({ definition: { expectedViolations: ['a',], }, violations: [], },).passed,).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
