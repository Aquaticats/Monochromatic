/**
 Repository-wide interference:
 a foreign `index.lock` holder,
 and `gc --prune=now` during preparation.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';

import {
  holdAt,
  reached,
  releaseAt,
} from './barrier-fixture.ts';
import { synthesizeText, } from './content-fixture.ts';
import { startForeignCommit, } from './foreign-commit-fixture.ts';
import { MAX_JITTER_MS, } from './scenario-concurrent-paths-fixture.ts';
import {
  disjointPlans,
  finishAll,
  repositoryOptions,
  rewritePaths,
  seedTexts,
  startWorkers,
  workerPaths,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Explicit-path agents running beside the interference.
 */
const BYSTANDERS = 3;

/**
 How long the foreign holder keeps `index.lock` after bystanders start.
 */
const FOREIGN_HOLD_MS = 1_500;

/**
 Text size of the held commit's file, large enough to be a distinct loose object.
 */
const HELD_BYTES = 2_000;

/**
 Text size of other written files.
 */
const FILE_BYTES = 300;

//endregion Constants

//region Scenarios

/**
 Foreign `index.lock` holder.
 */
const foreignLock: ScenarioDefinition = {
  name: 'foreign-index-lock-holder',
  group: 'concurrency',
  summary: 'real Git by absolute path with core.lockfilePid=true runs commit --all with a waiting editor (holding index.lock) while 3 agents commit',
  // `core.lockfilePid` arrived in Git 2.54.0 (Documentation/RelNotes/2.54.0.adoc); older Git leaves no owner evidence.
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: [
        ...workerPaths({
          prefix: 'w',
          count: BYSTANDERS,
        },),
        'foreign.txt',
      ],
    },), },);
  },
  async run(context,) {
    await writeWorktree({
      ...context,
      path: 'foreign.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('foreign',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Foreign holder.
     */
    const foreign = await startForeignCommit({
      ...context,
      label: 'foreign',
      paths: ['foreign.txt',],
      lockfilePid: true,
    },);
    /**
     Whether the editor started, so the lock is held.
     */
    const editor = await reached({
      repository: context.repository,
      token: foreign.token,
      running: foreign.running,
      event: 'editor',
    },);
    /**
     Bystander plans, written only after `commit --all` staged its paths.
     */
    const plans = disjointPlans(BYSTANDERS,);
    await rewritePaths({
      context,
      plans,
    },);
    /**
     Bystanders started while the lock is held.
     */
    const bystanders = await startWorkers({
      context,
      plans,
      maxJitterMs: MAX_JITTER_MS,
    },);
    await sleep(FOREIGN_HOLD_MS,);
    await releaseAt({
      repository: context.repository,
      token: foreign.token,
      event: 'editor',
    },);
    /**
     Finished attempts.
     */
    const finished = await finishAll([
      ...bystanders,
      foreign,
    ],);
    return [
      {
        name: 'foreign-holder-started',
        holds: editor === 'marker',
        detail: `editor wait ended with ${editor}`,
      },
      allSucceeded({
        name: 'all-commits-succeed',
        attempts: finished,
      },),
    ];
  },
};

/**
 `gc --prune=now` during preparation.
 */
const gcPrune: ScenarioDefinition = {
  name: 'gc-prune-during-commits',
  group: 'concurrency',
  summary: 'git gc --prune=now runs while one commit is held in pre-commit and 3 others commit',
  // Config hooks need Git 2.54.0.
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          ...workerPaths({
            prefix: 'w',
            count: BYSTANDERS,
          },),
          'held.txt',
        ],
      },),
      hooks: 'config',
      hookEvents: ['pre-commit',],
    },);
  },
  async run(context,) {
    await writeWorktree({
      ...context,
      path: 'held.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('held',),
        size: HELD_BYTES,
      },),
    },);
    /**
     Commit held in preparation during gc.
     */
    const held = await startAttempt({
      ...context,
      label: 'held',
      paths: ['held.txt',],
      mode: 'explicit',
    },);
    await holdAt({
      repository: context.repository,
      token: held.token,
      event: 'pre-commit',
    },);
    await reached({
      repository: context.repository,
      token: held.token,
      running: held.running,
      event: 'pre-commit',
    },);
    /**
     Bystander plans.
     */
    const plans = disjointPlans(BYSTANDERS,);
    await rewritePaths({
      context,
      plans,
    },);
    /**
     Bystanders and gc started together.
     */
    const [bystanders,] = await Promise.all([
      startWorkers({
        context,
        plans,
        maxJitterMs: MAX_JITTER_MS,
      },),
      runWrapper({
        ...context,
        label: 'gc --prune=now',
        args: [
          'gc',
          '--quiet',
          '--prune=now',
        ],
        mustSucceed: true,
      },),
    ],);
    await releaseAt({
      repository: context.repository,
      token: held.token,
      event: 'pre-commit',
    },);
    /**
     Finished attempts.
     */
    const finished = await finishAll([
      held,
      ...bystanders,
    ],);
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts: finished,
    },),];
  },
};

/**
 Lock and gc scenarios in report order.
 */
export const LOCK_GC_SCENARIOS: readonly ScenarioDefinition[] = [
  foreignLock,
  gcPrune,
];

//endregion Scenarios
