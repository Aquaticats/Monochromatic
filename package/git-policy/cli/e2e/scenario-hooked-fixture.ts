/**
 Concurrent agents in hooked,
 signed,
 and lint-staged repositories.

 @module
 */

import { synthesizeText, } from './content-fixture.ts';
import { MAX_JITTER_MS, } from './scenario-concurrent-paths-fixture.ts';
import {
  disjointPlans,
  finishAll,
  repositoryOptions,
  type RepositoryOverrides,
  runDisjointWorkers,
  seedTexts,
  startWorkers,
  workerPaths,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  ALL_HOOK_EVENTS,
  stagePartially,
} from './scenario-setup-fixture.ts';
import {
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Worker count for hook and signing scenarios.
 */
export const HOOKED_WORKERS = 4;

/**
 Text size of rewritten files.
 */
const FILE_BYTES = 500;

//endregion Constants

//region Workload

/**
 Hooked or signed concurrent commits of disjoint files.

 @param context - scenario context

 @returns expectations

 @example
 ```ts
 await runHookedConcurrent(context);
 ```
 */
async function runHookedConcurrent(context: ScenarioContext,): ReturnType<ScenarioDefinition['run']> {
  /**
   Finished attempts.
   */
  const attempts = await runDisjointWorkers({
    context,
    plans: disjointPlans(HOOKED_WORKERS,),
    maxJitterMs: MAX_JITTER_MS,
  },);
  return [allSucceeded({
    name: 'all-commits-succeed',
    attempts,
  },),];
}

/**
 Repository setup for hooked or signed concurrent commits.

 @param overrides - hook,
 worktree,
 or signing options

 @returns setup function

 @example
 ```ts
 hookedRepository({ hooks: 'config', hookEvents: ALL_HOOK_EVENTS });
 ```
 */
function hookedRepository(overrides: RepositoryOverrides,): ScenarioDefinition['repository'] {
  return function setup(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: workerPaths({
          prefix: 'w',
          count: HOOKED_WORKERS,
        },),
      },),
      ...overrides,
    },);
  };
}

//endregion Workload

//region Scenarios

/**
 Config-based hooks under concurrency.
 The former `hooks-hookdir-concurrent` twin (traditional hooks in the hooks directory) is removed:
 traditional hooks are unsupported by the wrapper,
 and this scenario ran the identical workload and assertions.
 */
const configConcurrent: ScenarioDefinition = {
  name: 'hooks-config-concurrent',
  group: 'concurrency',
  summary: `${String(HOOKED_WORKERS,)} agents commit at once with all four commit hooks registered in config`,
  minimumGit: '2.54.0',
  repository: hookedRepository({
    hooks: 'config',
    hookEvents: ALL_HOOK_EVENTS,
  },),
  run: runHookedConcurrent,
};

/**
 SSH signing under concurrency.
 */
const signingConcurrent: ScenarioDefinition = {
  name: 'ssh-signing-concurrent',
  group: 'concurrency',
  summary: `${String(HOOKED_WORKERS,)} agents make SSH-signed commits at once`,
  repository: hookedRepository({ signing: true, },),
  run: runHookedConcurrent,
};

/**
 lint-staged hooks under concurrency in a linked worktree.
 */
const lintStagedConcurrent: ScenarioDefinition = {
  name: 'lint-staged-stash-concurrent',
  group: 'concurrency',
  summary: 'lint-staged backup-stash hook while 3 explicit-path agents and 1 partially staged index commit run at once (linked worktree)',
  minimumGit: '2.54.0',
  repository: hookedRepository({
    worktree: 'linked',
    hooks: 'config',
    hookEvents: ['pre-commit',],
    hookMode: 'lint-staged',
  },),
  async run(context,) {
    /**
     Path the index commit takes.
     */
    const [indexPath = 'w-3.txt',] = workerPaths({
      prefix: 'w',
      count: HOOKED_WORKERS,
    },)
      .slice(-1,);
    await stagePartially({
      context,
      path: indexPath,
    },);
    /**
     Explicit-path plans for the other files.
     */
    const plans = disjointPlans(HOOKED_WORKERS - 1,);
    await Promise.all(plans.flatMap(function planPaths(plan,) {
      return plan.paths
        .map(async function rewrite(path,) {
          await writeWorktree({
            ...context,
            path,
            bytes: synthesizeText({
              random: context.random
                .fork(plan.label,),
              size: FILE_BYTES,
            },),
          },);
        },);
    },),);
    /**
     Explicit agents and the index agent started together.
     */
    const [explicit, index,] = await Promise.all([
      startWorkers({
        context,
        plans,
        maxJitterMs: MAX_JITTER_MS,
      },),
      startAttempt({
        ...context,
        label: 'index',
        paths: [indexPath,],
        mode: 'index',
      },),
    ],);
    /**
     Finished attempts.
     */
    const attempts = await finishAll([
      ...explicit,
      index,
    ],);
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts,
    },),];
  },
};

/**
 Hooked,
 signed,
 and lint-staged scenarios in report order.
 */
export const HOOKED_SCENARIOS: readonly ScenarioDefinition[] = [
  configConcurrent,
  signingConcurrent,
  lintStagedConcurrent,
];

//endregion Scenarios
