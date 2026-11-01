/**
 Single-commit baselines for configured hooks,
 signing,
 fault injection after the landing,
 and the checker's own positive control.
 Each must pass against today's build.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { synthesizeText, } from './content-fixture.ts';
import { hooksIgnored, } from './hooks-ignored-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import {
  repositoryOptions,
  runSequentialWorkers,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import { ALL_HOOK_EVENTS, } from './scenario-setup-fixture.ts';
import { landedCommit, } from './scenario-reservation-helper-fixture.ts';
import { settleServiceWork, } from './service-fixture.ts';
import { waitForPushRecord, } from './work-record-fixture.ts';
import {
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Text size of files the baselines write.
 */
const FILE_BYTES = 400;

//endregion Constants

//region Scenarios

/**
 Two sequential commits in a hooked or signed repository.

 @param context - scenario context

 @returns expectations

 @example
 ```ts
 await runHookedSequence(context);
 ```
 */
async function runHookedSequence(context: ScenarioContext,): ReturnType<ScenarioDefinition['run']> {
  /**
   Finished attempts.
   */
  const attempts = await runSequentialWorkers({
    context,
    plans: [
      {
        label: 'hooked-a',
        paths: ['a.txt',],
      },
      {
        label: 'hooked-b',
        paths: ['b.txt',],
      },
    ],
  },);
  return [allSucceeded({
    name: 'all-commits-succeed',
    attempts,
  },),];
}

/**
 Two sequential commits in a repository that configures all four commit hooks.
 The wrapper runs no external hook program, so the scenario holds the new contract instead of the retired one:
 the commits land, every commit warns that configured hooks are unsupported,
 the hook program never runs, and the service settles one push record per landed commit
 (the durable trace of the after-commit work that the `post-commit` hook once observed).
 */
const configHooks: ScenarioDefinition = {
  name: 'baseline-hooks-config',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'configured hooks are not run and are warned about; the commits land and settle one push record each',
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      hooks: 'config',
      hookEvents: ALL_HOOK_EVENTS,
      afterCommitRecords: true,
    },);
  },
  async run(context,) {
    /**
     Finished attempts.
     */
    const attempts = await runSequentialWorkers({
      context,
      plans: [
        {
          label: 'hooked-a',
          paths: ['a.txt',],
        },
        {
          label: 'hooked-b',
          paths: ['b.txt',],
        },
      ],
    },);
    return [
      allSucceeded({
        name: 'all-commits-succeed',
        attempts,
      },),
      ...await hooksIgnored({
        context,
        attempts,
      },),
    ];
  },
};

/**
 SSH-signed sequential commits.
 */
const sshSigning: ScenarioDefinition = {
  name: 'baseline-ssh-signing',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'two sequential SSH-signed commits',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      signing: true,
    },);
  },
  run: runHookedSequence,
};

/**
 Kill once the commit landed and its push record exists,
 the point the retired `post-commit` hook marked,
 then recover and commit again.
 The record is written after the landing and before the service is told about it,
 so the kill leaves a landed commit whose push waits for the next wake.
 This proves the kill machinery and leftover checks against a point today's build already survives;
 kills before the landing are design scenarios (`scenario-sigkill-fixture.ts`).
 */
const sigkillAfterLanding: ScenarioDefinition = {
  name: 'baseline-sigkill-after-landing',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'SIGKILL one commit once it landed and its push record exists, run git status to recover, commit again',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      afterCommitRecords: true,
    },);
  },
  async run(context,) {
    await writeWorktree({
      ...context,
      path: 'a.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('a',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Attempt killed after its landing.
     */
    const victim = await startAttempt({
      ...context,
      label: 'killed',
      paths: ['a.txt',],
      mode: 'explicit',
    },);
    /**
     Whether the push record appeared before the attempt settled.
     */
    const arrival = await waitForPushRecord({
      repository: context.repository,
      running: victim.running,
    },);
    victim.kill();
    /**
     Settled victim.
     */
    const victimRecord = await victim.finished;
    await runWrapper({
      ...context,
      label: 'recovery status',
      args: [
        'status',
        '--short',
      ],
      mustSucceed: true,
    },);
    await writeWorktree({
      ...context,
      path: 'b.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('b',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Follow-up attempt.
     */
    const followUp = await (await startAttempt({
      ...context,
      label: 'follow-up',
      paths: ['b.txt',],
      mode: 'explicit',
    },)).finished;
    await settleServiceWork(context.repository
      .commonDir,);
    /**
     The killed commit, which the landing put on the branch.
     */
    const landed = (await landedCommit({
      context,
      attempt: victimRecord,
    },)).trim();
    return [
      {
        name: 'victim-reached-point',
        holds: arrival === 'record',
        detail: `victim ${arrival} before its push record appeared`,
      },
      {
        name: 'victim-was-killed',
        holds: victimRecord.killed,
        detail: `the victim exited ${String(victimRecord.outcome
          .exitCode,)} before the kill reached it`,
      },
      {
        name: 'killed-commit-landed-once',
        holds: /^[0-9a-f]{40,64}$/v.test(landed,),
        detail: `the killed commit's token names ${landed === '' ? 'no commit' : 'several commits'}`,
      },
      allSucceeded({
        name: 'follow-up-commit-succeeds',
        attempts: [followUp,],
      },),
    ];
  },
};

/**
 Positive control for the observer and checker:
 one commit lands through the wrapper,
 then real Git tampers with it so that five invariants must be reported.
 */
const checkerPositiveControl: ScenarioDefinition = {
  name: 'baseline-checker-positive-control',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'land one commit, then tamper with real Git; the checker must report exactly the planted violations',
  expectedViolations: [
    'index-no-revert',
    'landed-bytes',
    'no-leftovers',
    'remote-contains',
    'worktree-preserved',
  ],
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: ['a.txt',],
    },), },);
  },
  async run(context,) {
    /**
     Seed blob of the path, which the landed commit replaces.
     */
    const seedBlob = (await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'HEAD:a.txt',
      ],
    },)).trim();
    await writeWorktree({
      ...context,
      path: 'a.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('landed',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Landed attempt.
     */
    const landed = await (await startAttempt({
      ...context,
      label: 'landed',
      paths: ['a.txt',],
      mode: 'explicit',
    },)).finished;
    // The service pushes after the commit returned; tampering before that push ends would amend what it pushes.
    await settleServiceWork(context.repository
      .commonDir,);
    // Tampering bypasses the ledger on purpose, so the worktree no longer holds what the harness wrote.
    await writeFile(
      join(
        context.repository
          .worktree,
        'a.txt',
      ),
      synthesizeText({
        random: context.random
          .fork('tampered',),
        size: FILE_BYTES,
      },),
    );
    await realGit({
      repository: context.repository,
      args: [
        'commit',
        '--quiet',
        '--amend',
        '--no-edit',
        '--no-verify',
        '--',
        'a.txt',
      ],
    },);
    await realGit({
      repository: context.repository,
      args: [
        'update-index',
        '--cacheinfo',
        `100644,${seedBlob},a.txt`,
      ],
    },);
    await writeFile(
      join(
        context.repository
          .commonDir,
        'refs',
        'heads',
        'stale.lock',
      ),
      '',
    );
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts: [landed,],
    },),];
  },
};

/**
 Hook,
 signing,
 fault,
 and checker baselines in report order.
 */
export const BASELINE_HOOK_SCENARIOS: readonly ScenarioDefinition[] = [
  configHooks,
  sshSigning,
  sigkillAfterLanding,
  checkerPositiveControl,
];

//endregion Scenarios
