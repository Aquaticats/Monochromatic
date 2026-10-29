/**
 Reduces a finished scenario's repositories and ledger to a {@link RunObservation}.
 All reads use real Git by absolute path,
 so observation never re-enters the wrapper or its recovery.

 @module
 */

import {
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  type HistoryCommit,
  isAncestor,
  readHistory,
  stateOf,
  treeBytes,
} from './git-read-fixture.ts';
import type {
  AuxiliaryObservation,
  ContentState,
  RunObservation,
  StagedDifference,
} from './invariant-model-fixture.ts';
import { sameContent, } from './invariant-fixture.ts';
import { classifyLeftovers, } from './leftover-fixture.ts';
import {
  checkExitConsistency,
  extractPolicyEvents,
} from './jsonl-event-fixture.ts';
import {
  type AttemptRecord,
  contentOf,
  type WorkloadLedger,
} from './ledger-fixture.ts';
import {
  type HookChecks,
  observeAttempt,
  postCommitRuns,
} from './observation-attempt-fixture.ts';
import {
  runBytes,
  runProcess,
} from './process-fixture.ts';
import {
  realGit,
  type ScenarioRepository,
} from './repository-fixture.ts';
import { readWorktree, } from './worker-fixture.ts';

//region Repository state

/**
 Lists leftover transaction state:
 `refs/cli-git/` refs plus the entries {@link classifyLeftovers} reports.

 @param repository - scenario repository

 @returns leftover descriptions

 @example
 ```ts
 await findLeftovers(repository);
 ```
 */
async function findLeftovers(repository: ScenarioRepository,): Promise<readonly string[]> {
  /**
   Private refs, loose or packed.
   */
  const refs = (await realGit({
    repository,
    args: [
      'for-each-ref',
      '--format=%(refname)',
      'refs/cli-git/',
    ],
  },))
    .split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },);
  /**
   Every entry under the common directory, relative to it.
   */
  const entries = await readdir(
    repository.commonDir,
    { recursive: true, },
  );
  return [
    ...refs,
    ...classifyLeftovers(entries,)
      .map(function relative(entry,) {
        return `.git/${entry}`;
      },),
  ];
}

/**
 Lists real-index paths that differ from `HEAD` and classifies each.

 @param repository - scenario repository

 @param stagedByHarness - paths the harness deliberately left staged

 @param replaced - contents per path that landed commits replaced

 @returns staged differences

 @example
 ```ts
 await findStagedDifferences({ repository, stagedByHarness, replaced });
 ```
 */
async function findStagedDifferences({
  repository,
  stagedByHarness,
  replaced,
}: Readonly<{
  repository: ScenarioRepository;
  stagedByHarness: ReadonlySet<string>;
  replaced: ReadonlyMap<string, readonly ReturnType<typeof contentOf>[]>;
}>,): Promise<readonly StagedDifference[]> {
  /**
   Paths whose index entry differs from `HEAD`.
   */
  const paths = (await realGit({
    repository,
    args: [
      'diff',
      '--cached',
      '--no-renames',
      '--name-only',
      '-z',
      'HEAD',
    ],
  },))
    .split('\0',)
    .filter(function nonEmpty(path,) {
      return path !== '';
    },);
  return await Promise.all(paths.map(async function classify(path,): Promise<StagedDifference> {
    /**
     Stage-0 entry line.
     */
    const listed = await realGit({
      repository,
      args: [
        'ls-files',
        '--stage',
        '-z',
        '--',
        path,
      ],
    },);
    /**
     Staged blob ID, absent for a staged deletion.
     */
    const oid = listed === '' ? undefined : listed.split(' ',)[1];
    /**
     Staged content.
     */
    const staged = oid === undefined
      ? contentOf()
      : contentOf(await runBytes({
        command: repository.realGit,
        args: [
          'cat-file',
          'blob',
          oid,
        ],
        cwd: repository.worktree,
        env: repository.realEnv,
      },),);
    return {
      path,
      expectedByHarness: stagedByHarness.has(path,),
      revertsLandedContent: (replaced.get(path,) ?? []).some(function equalsReplaced(content,) {
        return sameContent({
          left: content,
          right: staged,
        },);
      },),
    };
  },),);
}

/**
 Lists the landed amends that replaced an already-published commit:
 commits after them cannot fast-forward the remote.

 @param repository - scenario repository

 @param attempts - recorded attempts

 @param history - local-branch history

 @returns landed amend commits

 @example
 ```ts
 await findAmendedPublished({ repository, attempts, history });
 ```
 */
async function findAmendedPublished({
  repository,
  attempts,
  history,
}: Readonly<{
  repository: ScenarioRepository;
  attempts: readonly AttemptRecord[];
  history: readonly HistoryCommit[];
}>,): Promise<readonly string[]> {
  /**
   Landed amend commits with the commit each replaced.
   */
  const amends = attempts
    .filter(function landedAmend(attempt,) {
      return (attempt.mode === 'amend') && (attempt.outcome
        .exitCode
        === 0);
    },)
    .flatMap(function landedCommits(attempt,) {
      return history
        .filter(function carries(commit,) {
          return commit.message
            .includes(`[${attempt.token}]`,);
        },)
        .map(function replaced(commit,): Readonly<{
          oid: string;
          replaced: string;
        }> {
          return {
            oid: commit.oid,
            replaced: attempt.headBefore,
          };
        },);
    },);
  /**
   Whether the remote had each replaced commit.
   */
  const published = await Promise.all(amends.map(async function wasPublished(amend,) {
    return await isAncestor({
      repository,
      gitDir: repository.remote,
      oid: amend.replaced,
      ref: `refs/heads/${repository.branch}`,
    },);
  },),);
  return amends
    .filter(function isPublished(
      _amend,
      index,
    ) {
      return published[index] === true;
    },)
    .map(function oidOf(amend,) {
      return amend.oid;
    },);
}

//endregion Repository state

//region Run

/**
 Gathers the complete observation of one scenario run.

 @param repository - scenario repository

 @param ledger - scenario ledger

 @param checks - hook facts to collect

 @param expectations - scenario-specific expectations

 @returns observation for {@link checkInvariants}

 @example
 ```ts
 await observeRun({ repository, ledger, checks, expectations: [] });
 ```
 */
export async function observeRun({
  repository,
  ledger,
  checks,
  expectations,
}: Readonly<{
  repository: ScenarioRepository;
  ledger: WorkloadLedger;
  checks: HookChecks;
  expectations: RunObservation['expectations'];
}>,): Promise<RunObservation> {
  /**
   Ledger contents.
   */
  const snapshot = ledger.snapshot();
  /**
   Local-branch history.
   */
  const history = await readHistory(repository,);
  /**
   Post-commit runs per token.
   */
  const runs = await postCommitRuns(repository,);
  /**
   Landed amends that replaced a commit the remote already had.
   */
  const amendedPublished = await findAmendedPublished({
    repository,
    attempts: snapshot.attempts,
    history,
  },);
  /**
   Attempt observations.
   */
  const attempts = await Promise.all(snapshot.attempts
    .map(async function observe(attempt,) {
      return await observeAttempt({
        repository,
        attempt,
        attempts: snapshot.attempts,
        history,
        checks,
        runs,
        amendedPublished,
      },);
    },),);
  /**
   Parent-side contents replaced by landed commits, per path.
   */
  const replacedEntries = await Promise.all(history.flatMap(function landedByAttempt(commit,) {
    return snapshot.attempts
      .filter(function carries(attempt,) {
        return commit.message
          .includes(`[${attempt.token}]`,)
          && (commit.parent !== undefined);
      },)
      .flatMap(function replacedPaths(attempt,) {
        return attempt.selectedPaths
          .map(async function replacedContent(path,) {
            return [
              path,
              stateOf(await treeBytes({
                repository,
                commit: commit.parent ?? commit.oid,
                path,
              },),),
            ] as const;
          },);
      },);
  },),);
  /**
   Replaced contents grouped by path.
   */
  const replaced = new Map([...Map.groupBy(
    replacedEntries,
    function byPath([path,],) {
      return path;
    },
  ),].map(function contents([path, entries,],) {
    return [
      path,
      entries.map(function content([, state,],) {
        return state;
      },),
    ] as const;
  },),);
  /**
   Auxiliary command observations.
   */
  const auxiliaries = snapshot.auxiliaries
    .map(function observeAuxiliary(auxiliary,): AuxiliaryObservation {
    /**
     Decoded events for wrapper commands.
     */
      const extraction = extractPolicyEvents(auxiliary.wrapper ? auxiliary.outcome
        .stderr : '',);
      return {
        label: auxiliary.label,
        exitCode: auxiliary.outcome
          .exitCode,
        mustSucceed: auxiliary.mustSucceed,
        eventIssues: [
          ...extraction.issues,
          ...checkExitConsistency({
            exitCode: auxiliary.outcome
              .exitCode,
            events: extraction.events,
          },),
        ],
      };
    },);
  /**
   Final worktree contents of tracked harness paths.
   */
  const worktree = await Promise.all([...snapshot.worktree,].map(async function compare([path, expected,]: readonly [
    string,
    ContentState
  ],) {
    return {
      path,
      expected,
      actual: contentOf((await readWorktree({
        repository,
        path,
      },)).bytes,),
    };
  },),);
  /**
   `git fsck` settlement.
   */
  const fsck = await runProcess({
    command: repository.realGit,
    args: [
      'fsck',
      '--strict',
      '--no-dangling',
      '--no-progress',
    ],
    cwd: repository.worktree,
    env: repository.realEnv,
  },);
  return {
    attempts,
    auxiliaries,
    worktree,
    staged: await findStagedDifferences({
      repository,
      stagedByHarness: snapshot.staged,
      replaced,
    },),
    leftovers: await findLeftovers(repository,),
    fsck: {
      exitCode: fsck.exitCode,
      output: `${fsck.stdout}${fsck.stderr}`,
    },
    expectations,
  };
}

//endregion Run
