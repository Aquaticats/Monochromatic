/**
 Builds attempt observations:
 which history commits carry each attempt's token,
 what they landed,
 and which contents the invariant accepts.

 @module
 */

import { join, } from 'node:path';

import {
  changedPaths,
  firstParentChain,
  type HistoryCommit,
  isAncestor,
  mergeOnto,
  stateOf,
  treeBytes,
} from './git-read-fixture.ts';
import type {
  AttemptObservation,
  ContentState,
  LandingObservation,
  PathLanding,
} from './invariant-model-fixture.ts';
import { sameContent, } from './invariant-fixture.ts';
import {
  checkExitConsistency,
  extractPolicyEvents,
} from './jsonl-event-fixture.ts';
import {
  type AttemptRecord,
  type CapturedPath,
  contentOf,
} from './ledger-fixture.ts';
import {
  readOptionalText,
  runProcess,
} from './process-fixture.ts';
import type { ScenarioRepository, } from './repository-fixture.ts';

//region Types

/**
 Hook facts the scenario's setup makes checkable.
 */
export type HookChecks = Readonly<{
  /**
   A `commit-msg` hook appends a trailer.
   */
  commitMsgTrailer: boolean;
  /**
   A `post-commit` hook logs each run.
   */
  postCommitRuns: boolean;
  /**
   Commits are SSH-signed.
   */
  signature: boolean;
}>;

/**
 Trailer the shared `commit-msg` hook appends.
 */
const HOOK_TRAILER = 'E2E-Hook-Checked: yes';

/**
 Git's reason for refusing a push that does not fast-forward the remote branch.
 */
const NON_FAST_FORWARD = 'non-fast-forward';

/**
 Note the wrapper prints after every failed auto-push (`PUSH_FAILED_NOTE` in `src/native/auto_push.rs`).
 */
const PUSH_FAILED_NOTE = 'auto-push to origin failed';

/**
 Line the wrapper prints instead of that note when its reconcile of a stale push cannot publish, for example after a
 conflicting merge (`RECONCILE_FAILED_PREFIX` in `src/native/auto_push_reconcile.rs`).
 */
const RECONCILE_FAILED_LINE = 'auto-push: not published:';

/**
 Whether an attempt's stderr shows Git's non-fast-forward refusal together with one of the two ways the wrapper says
 the commit was not published.

 @param stderr - complete stderr of the attempt

 @returns whether the rejection was surfaced
 */
function surfacesPushRejection(stderr: string,): boolean {
  return stderr.includes(NON_FAST_FORWARD,)
    && [
      PUSH_FAILED_NOTE,
      RECONCILE_FAILED_LINE,
    ].some(function mentioned(note,) {
      return stderr.includes(note,);
    },);
}

//endregion Types

//region Acceptable contents

/**
 Lists every content the invariant accepts for one captured path:
 the captured bytes,
 plus each clean three-way merge of them onto the landed parent from a candidate preparation base
 (every first-parent commit from `HEAD` before invocation up to the landed parent).

 @param repository - scenario repository

 @param captured - bytes captured at invocation

 @param parent - landed commit's first parent

 @param headBefore - `HEAD` read before invocation

 @returns acceptable contents

 @example
 ```ts
 await acceptableContents({ repository, captured, parent, headBefore });
 ```
 */
async function acceptableContents({
  repository,
  captured,
  parent,
  headBefore,
}: Readonly<{
  repository: ScenarioRepository;
  captured: CapturedPath;
  parent?: string;
  headBefore: string;
}>,): Promise<readonly ContentState[]> {
  /**
   Captured content, always acceptable.
   */
  const direct = contentOf(captured.bytes,);
  if ((parent === undefined) || (captured.bytes === undefined)
    || (parent === headBefore))
    return [direct,];
  /**
   Captured bytes narrowed for the merge closure.
   */
  const other = captured.bytes;
  /**
   Landed parent's bytes.
   */
  const current = await treeBytes({
    repository,
    commit: parent,
    path: captured.path,
  },);
  if (current === 'absent')
    return [direct,];
  /**
   Candidate bases, excluding the parent itself whose merge is the captured bytes.
   */
  const bases = (await firstParentChain({
    repository,
    newest: parent,
    oldest: headBefore,
  },)).filter(function notParent(oid,) {
    return oid !== parent;
  },);
  /**
   Merge results per base.
   */
  const merges = await Promise.all(bases.map(async function mergeFrom(base,) {
    /**
     Base bytes.
     */
    const baseBytes = await treeBytes({
      repository,
      commit: base,
      path: captured.path,
    },);
    return baseBytes === 'absent' ? 'conflict' : await mergeOnto({
      repository,
      current,
      base: baseBytes,
      other,
    },);
  },),);
  return [
    direct,
    ...merges.flatMap(function cleanMerge(merge,) {
      return merge === 'conflict' ? [] : [contentOf(merge,),];
    },),
  ];
}

/**
 Lists the landed parent's bytes of a path as acceptable when a later capture of that path landed them
 (capture order,
 `SPEC.md` "Capture order"):
 another attempt that started after this one captured the same path with exactly those bytes,
 and its commit lies on the first-parent line from `HEAD` before invocation to the landed parent.
 The replayed commit then kept the landed entry of the path,
 as native sequential commits of the two captures would leave it.

 @param repository - scenario repository

 @param attempt - attempt owning the landing

 @param captured - this attempt's capture of the path

 @param parent - landed commit's first parent

 @param attempts - every recorded attempt

 @param history - local-branch history

 @returns the parent's bytes when a later capture landed them, otherwise nothing

 @example
 ```ts
 await laterCaptureContents({ repository, attempt, captured, parent, attempts, history });
 ```
 */
async function laterCaptureContents({
  repository,
  attempt,
  captured,
  parent,
  attempts,
  history,
}: Readonly<{
  repository: ScenarioRepository;
  attempt: AttemptRecord;
  captured: CapturedPath;
  parent?: string;
  attempts: readonly AttemptRecord[];
  history: readonly HistoryCommit[];
}>,): Promise<readonly ContentState[]> {
  if ((parent === undefined) || (parent === attempt.headBefore)
    || (attempt.mode !== 'explicit'))
    return [];
  /**
   Parent's content of the path.
   */
  const current = stateOf(await treeBytes({
    repository,
    commit: parent,
    path: captured.path,
  },),);
  /**
   Commits the replay moved over.
   */
  const moved = new Set((await firstParentChain({
    repository,
    newest: parent,
    oldest: attempt.headBefore,
  },)).filter(function afterHeadBefore(oid,) {
    return oid !== attempt.headBefore;
  },),);
  /**
   Whether a later explicit capture of the path with those bytes landed in between.
   */
  const landedLater = attempts.some(function laterCapture(other,) {
    return (other.startedAt > attempt.startedAt)
      && (other.mode === 'explicit')
      && other.captured
        .some(function samePathAndBytes(entry,) {
          return (entry.path === captured.path) && sameContent({
            left: contentOf(entry.bytes,),
            right: current,
          },);
        },)
      && history.some(function carriesToken(commit,) {
        return moved.has(commit.oid,)
          && commit.message
            .includes(`[${other.token}]`,);
      },);
  },);
  return landedLater ? [current,] : [];
}

//endregion Acceptable contents

//region Attempts

/**
 Counts `post-commit` hook runs per token.

 @param repository - scenario repository

 @returns runs per token

 @example
 ```ts
 await postCommitRuns(repository);
 ```
 */
export async function postCommitRuns(repository: ScenarioRepository,): Promise<ReadonlyMap<string, number>> {
  /**
   Hook log text, empty before any hook ran.
   */
  const text = await readOptionalText(join(
    repository.logDir,
    'hooks.jsonl',
  ),);
  /**
   Token of every post-commit run.
   */
  const tokens = text
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },)
    .map(function parse(line,): unknown {
      return JSON.parse(line,);
    },)
    .flatMap(function postCommitToken(entry,): readonly string[] {
      if (((typeof entry) !== 'object') || (entry === null)
        || (!('event' in entry))
        || (!('token' in entry)))
        return [];
      return (entry.event === 'post-commit') && ((typeof entry.token) === 'string') ? [entry.token,] : [];
    },);
  return new Map([...Map.groupBy(
    tokens,
    function byToken(token,) {
      return token;
    },
  ),].map(function runCount([token, tokenRuns,]: readonly [
    string,
    readonly string[]
  ],) {
    return [
      token,
      tokenRuns.length,
    ] as const;
  },),);
}

/**
 Observes one attempt.

 @param repository - scenario repository

 @param attempt - recorded attempt

 @param history - local-branch history

 @param checks - hook facts to collect

 @param runs - post-commit runs per token

 @param amendedPublished - landed amends that replaced an already-published commit

 @param attempts - every recorded attempt, for later captures of a shared path

 @returns attempt observation

 @example
 ```ts
 await observeAttempt({ repository, attempt, attempts, history, checks, runs, amendedPublished });
 ```
 */
export async function observeAttempt({
  repository,
  attempt,
  history,
  checks,
  runs,
  amendedPublished,
  attempts,
}: Readonly<{
  repository: ScenarioRepository;
  attempt: AttemptRecord;
  attempts: readonly AttemptRecord[];
  history: readonly HistoryCommit[];
  checks: HookChecks;
  runs: ReadonlyMap<string, number>;
  amendedPublished: readonly string[];
}>,): Promise<AttemptObservation> {
  /**
   Commits carrying the token.
   */
  const carrying = history.filter(function hasToken(commit,) {
    return commit.message
      .includes(`[${attempt.token}]`,);
  },);
  /**
   Decoded wrapper events; the foreign real-Git commit emits none.
   */
  const extraction = extractPolicyEvents(attempt.mode === 'foreign' ? '' : attempt.outcome
    .stderr,);
  /**
   Landed OID from a `commit-landed` event.
   */
  const landedEventOid = extraction.events
    .find(function isLanded(event,) {
      return event.type === 'commit-landed';
    },)
    ?.oid;
  /**
   Landing observations.
   */
  const landings = await Promise.all(carrying.map(async function observeLanding(commit,): Promise<LandingObservation> {
    /**
     Per-path landed and acceptable contents.
     */
    const paths = await Promise.all(attempt.captured
      .map(async function landPath(captured,): Promise<PathLanding> {
        return {
          path: captured.path,
          landed: stateOf(await treeBytes({
            repository,
            commit: commit.oid,
            path: captured.path,
          },),),
          acceptable: [
            ...await acceptableContents({
              repository,
              captured,
            // An amend replaces the commit HEAD named at invocation, so its captured bytes are the only acceptable content.
              ...(attempt.mode === 'amend' ? { parent: attempt.headBefore, } : (commit.parent === undefined ? {} : { parent: commit.parent, })),
              headBefore: attempt.headBefore,
            },),
            ...await laterCaptureContents({
              repository,
              attempt,
              captured,
              ...(commit.parent === undefined ? {} : { parent: commit.parent, }),
              attempts,
              history,
            },),
          ],
        };
      },),);
    /**
     Signature verification when commits are signed.
     */
    const signatureValid = checks.signature
      ? (await runProcess({
        command: repository.realGit,
        args: [
          'verify-commit',
          commit.oid,
        ],
        cwd: repository.worktree,
        env: repository.realEnv,
      },)).exitCode === 0
      : undefined;
    return {
      oid: commit.oid,
      onExpectedBranch: await isAncestor({
        repository,
        oid: commit.oid,
        ref: `refs/heads/${attempt.expectedBranch}`,
      },),
      // An amend's scope is measured against the commit it replaced, not against that commit's parent.
      changedPaths: await changedPaths({
        repository,
        commit: attempt.mode === 'amend' ? {
          ...commit,
          parent: attempt.headBefore,
        } : commit,
      },),
      paths,
      onAmendedPublishedHistory: (await Promise.all(amendedPublished.map(async function onAmend(amend,) {
        return (amend !== commit.oid) && await isAncestor({
          repository,
          oid: amend,
          ref: commit.oid,
        },);
      },),)).includes(true,),
      remoteContains: await isAncestor({
        repository,
        gitDir: repository.remote,
        oid: commit.oid,
        ref: `refs/heads/${attempt.expectedBranch}`,
      },),
      hooks: {
        ...(checks.commitMsgTrailer ? { commitMsgTrailer: commit.message
          .includes(HOOK_TRAILER,), } : {}),
        ...(checks.postCommitRuns ? { postCommitRuns: runs.get(attempt.token,) ?? 0, } : {}),
        ...(signatureValid === undefined ? {} : { signatureValid, }),
      },
    };
  },),);
  return {
    label: attempt.label,
    exitCode: attempt.outcome
      .exitCode,
    killed: attempt.killed,
    selectedPaths: attempt.selectedPaths,
    landings,
    eventIssues: attempt.killed
      ? []
      : [
        ...extraction.issues,
        ...checkExitConsistency({
          exitCode: attempt.outcome
            .exitCode,
          events: extraction.events,
        },),
      ],
    ...(landedEventOid === undefined ? {} : { landedEventOid, }),
    requiresRemote: attempt.requiresRemote,
    pushRejectionSurfaced: surfacesPushRejection(attempt.outcome
      .stderr,),
  };
}

//endregion Attempts
