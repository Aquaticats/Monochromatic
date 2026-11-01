/**
 Pure invariant checker for one scenario run
 (`doc/decision/cli-git-concurrent-commits.md` "Container end-to-end verification").

 @module
 */

import type {
  AttemptObservation,
  ContentState,
  LandingObservation,
  RunObservation,
  Violation,
} from './invariant-model-fixture.ts';

//region Helpers

/**
 Digest and commit ID prefix length in diagnostics.
 */
const SHORT_DIGEST = 12;

/**
 Renders content for diagnostics.

 @param content - content state

 @returns short description

 @example
 ```ts
 describeContent({ state: 'absent' }); // => 'absent'
 ```
 */
export function describeContent(content: ContentState,): string {
  return content.state === 'absent' ? 'absent' : `sha256:${content.digest
    .slice(
      0,
      SHORT_DIGEST,
    )}`;
}

/**
 Compares two content states.

 @param left - first state

 @param right - second state

 @returns whether both are absent or both carry the same digest

 @example
 ```ts
 sameContent({ left: { state: 'absent' }, right: { state: 'absent' } }); // => true
 ```
 */
export function sameContent({
  left,
  right,
}: Readonly<{
  left: ContentState;
  right: ContentState;
}>,): boolean {
  if ((left.state === 'absent') || (right.state === 'absent'))
    return left.state === right.state;
  return left.digest === right.digest;
}

/**
 Builds one violation.

 @param invariant - invariant name

 @param subject - attempt, command, or path

 @param detail - observation

 @returns violation record

 @example
 ```ts
 violation({ invariant: 'fsck-clean', subject: 'repository', detail: 'exit 1' });
 ```
 */
function violation({
  invariant,
  subject,
  detail,
}: Violation,): Violation {
  return {
    invariant,
    subject,
    detail,
  };
}

//endregion Helpers

//region Attempt checks

/**
 Checks one landed commit of an attempt.

 @param attempt - attempt owning the landing

 @param landing - landed commit facts

 @returns violations

 @example
 ```ts
 checkLanding({ attempt, landing });
 ```
 */
function checkLanding({
  attempt,
  landing,
}: Readonly<{
  attempt: AttemptObservation;
  landing: LandingObservation;
}>,): readonly Violation[] {
  /**
   Subject naming attempt and commit.
   */
  const subject = `${attempt.label} ${landing.oid
    .slice(
      0,
      SHORT_DIGEST,
    )}`;
  /**
   Paths changed outside the attempt's selection.
   */
  const outside = landing.changedPaths
    .filter(function unselected(path,) {
      return !attempt.selectedPaths
        .includes(path,);
    },);
  /**
   Paths whose landed bytes match no acceptable content.
   */
  const wrongBytes = landing.paths
    .filter(function unacceptable(path,) {
      return !path.acceptable
        .some(function matches(content,) {
          return sameContent({
            left: content,
            right: path.landed,
          },);
        },);
    },);
  /**
   Whether the attempt claims success, so exit-0-only obligations apply.
   */
  const succeeded = attempt.exitCode === 0;
  /**
   Hook facts, when recorded.
   */
  const hooks = landing.hooks ?? {};
  /**
   Candidate violations paired with whether they occurred.
   */
  const checks: readonly (readonly [
    boolean,
    Violation
  ])[] = [
    [
      !landing.onExpectedBranch,
      violation({
        invariant: 'landed-branch',
        subject,
        detail: 'the branch HEAD named at invocation does not reach the landed commit',
      },),
    ],
    [
      outside.length > 0,
      violation({
        invariant: 'landed-scope',
        subject,
        detail: `changes unselected paths ${outside.join(', ',)}`,
      },),
    ],
    ...wrongBytes.map(function bytesViolation(path,): readonly [
      boolean,
      Violation
    ] {
      return [
        true,
        violation({
          invariant: 'landed-bytes',
          subject: `${subject} ${path.path}`,
          detail: `landed ${describeContent(path.landed,)}, accepted ${path.acceptable
            .map(describeContent,)
            .join(' or ',)}`,
        },),
      ];
    },),
    [
      succeeded && attempt.requiresRemote
        && (!landing.remoteContains)
        && (!landing.onAmendedPublishedHistory),
      violation({
        invariant: 'remote-contains',
        subject,
        detail: 'the remote branch does not reach the landed commit after exit 0',
      },),
    ],
    [
      succeeded && attempt.requiresRemote
        && (!landing.remoteContains)
        && landing.onAmendedPublishedHistory
        && (!attempt.pushRejectionSurfaced),
      violation({
        invariant: 'push-rejection-surfaced',
        subject,
        detail: 'the commit landed on an amended published history, but exit 0 surfaced no non-fast-forward auto-push rejection',
      },),
    ],
    [
      hooks.commitMsgTrailer === false,
      violation({
        invariant: 'commit-msg-hook',
        subject,
        detail: 'the landed message lacks the commit-msg hook trailer',
      },),
    ],
    [
      succeeded && (hooks.postCommitRuns !== undefined)
        && (hooks.postCommitRuns !== 1),
      violation({
        invariant: 'post-commit-once',
        subject,
        detail: `post-commit ran ${String(hooks.postCommitRuns,)} times`,
      },),
    ],
    [
      hooks.signatureValid === false,
      violation({
        invariant: 'signature-valid',
        subject,
        detail: 'git verify-commit rejected the landed commit',
      },),
    ],
  ];
  return checks.flatMap(function occurred([failed, found,],) {
    return failed ? [found,] : [];
  },);
}

/**
 Checks landing count,
 landing contents,
 and event consistency of one attempt.

 @param attempt - attempt observation

 @returns violations

 @example
 ```ts
 checkAttempt(attempt);
 ```
 */
function checkAttempt(attempt: AttemptObservation,): readonly Violation[] {
  /**
   Number of history commits carrying the token.
   */
  const count = attempt.landings
    .length;
  /**
   Landing-count problem for this attempt's outcome.
   */
  const countProblem = (function landingCountProblem(): string {
    if (attempt.killed)
      return count <= 1 ? '' : `killed attempt landed ${String(count,)} times`;
    if (attempt.exitCode === 0)
      return count === 1 ? '' : `exited 0 but landed ${String(count,)} times`;
    if (attempt.landedEventOid !== undefined) {
      return (count === 1) && (attempt.landings[0]
        ?.oid
        === attempt.landedEventOid)
        ? ''
        : `commit-landed names ${attempt.landedEventOid} but history has ${String(count,)} landings`;
    }
    return count === 0 ? '' : `exited ${String(attempt.exitCode,)} without commit-landed but landed ${String(count,)} times`;
  })();
  return [
    ...(countProblem === '' ? [] : [violation({
      invariant: 'landed-once',
      subject: attempt.label,
      detail: countProblem,
    },),]),
    ...attempt.eventIssues
      .map(function eventViolation(issue,) {
        return violation({
          invariant: 'exit-events',
          subject: attempt.label,
          detail: issue,
        },);
      },),
    ...(count === 1
      ? attempt.landings
        .flatMap(function landingViolations(landing,) {
          return checkLanding({
            attempt,
            landing,
          },);
        },)
      : []),
  ];
}

//endregion Attempt checks

//region Run check

/**
 Checks every invariant of one scenario run.

 @param observation - reduced run facts

 @returns violations, empty when the run passes

 @example
 ```ts
 checkInvariants(observation).length === 0;
 ```
 */
export function checkInvariants(observation: RunObservation,): readonly Violation[] {
  return [
    ...observation.attempts
      .flatMap(checkAttempt,),
    ...observation.auxiliaries
      .flatMap(function auxiliaryViolations(auxiliary,) {
        return [
          ...(auxiliary.mustSucceed && (auxiliary.exitCode !== 0)
            ? [violation({
              invariant: 'command-succeeds',
              subject: auxiliary.label,
              detail: `exited ${String(auxiliary.exitCode,)}`,
            },),]
            : []),
          ...auxiliary.eventIssues
            .map(function eventViolation(issue,) {
              return violation({
                invariant: 'exit-events',
                subject: auxiliary.label,
                detail: issue,
              },);
            },),
        ];
      },),
    ...observation.worktree
      .flatMap(function worktreeViolation(entry,) {
        return sameContent({
          left: entry.expected,
          right: entry.actual,
        },)
          ? []
          : [violation({
            invariant: 'worktree-preserved',
            subject: entry.path,
            detail: `expected ${describeContent(entry.expected,)}, found ${describeContent(entry.actual,)}`,
          },),];
      },),
    ...observation.staged
      .flatMap(function stagedViolation(entry,) {
        if (entry.expectedByHarness)
          return [];
        return [violation({
          invariant: entry.revertsLandedContent ? 'index-no-revert' : 'index-matches-head',
          subject: entry.path,
          detail: entry.revertsLandedContent
            ? 'the real index stages content a landed commit replaced'
            : 'the real index differs from HEAD without a harness write',
        },),];
      },),
    ...observation.leftovers
      .map(function leftoverViolation(leftover,) {
        return violation({
          invariant: 'no-leftovers',
          subject: leftover,
          detail: 'remains after the run',
        },);
      },),
    ...((observation.fsck
      .exitCode
      === 0) && (observation.fsck
        .output
        .trim()
        === '')
      ? []
      : [violation({
        invariant: 'fsck-clean',
        subject: 'repository',
        detail: `exit ${String(observation.fsck
          .exitCode,)}: ${observation.fsck
            .output
            .trim()}`,
      },),]),
    ...observation.expectations
      .flatMap(function expectationViolation(expectation,) {
        return expectation.holds
          ? []
          : [violation({
            invariant: expectation.name,
            subject: 'scenario',
            detail: expectation.detail,
          },),];
      },),
  ];
}

//endregion Run check
