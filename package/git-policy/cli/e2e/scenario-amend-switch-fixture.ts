/**
 History-rewriting interference:
 an amend racing concurrent commits,
 and a branch switch while a commit prepares.

 @module
 */

import {
  holdAt,
  reached,
  releaseAt,
} from './barrier-fixture.ts';
import { synthesizeText, } from './content-fixture.ts';
import { realGit, } from './repository-fixture.ts';
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
  amendReplacedBase,
  commitAmendBase,
} from './scenario-setup-fixture.ts';
import {
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Explicit-path agents running beside the amend.
 */
const BYSTANDERS = 3;

/**
 Text size of written files.
 */
const FILE_BYTES = 300;

//endregion Constants

//region Scenarios

/**
 Amend racing concurrent commits.
 */
const amendDuringCommits: ScenarioDefinition = {
  name: 'amend-during-commits',
  group: 'concurrency',
  summary: 'one agent amends while 3 agents commit; the amend may land only if HEAD did not move',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: [
        ...workerPaths({
          prefix: 'w',
          count: BYSTANDERS,
        },),
        'amend.txt',
      ],
    },), },);
  },
  async run(context,) {
    /**
     Parent of the token-free setup commit the amend targets.
     */
    const { parent, } = await commitAmendBase({
      context,
      path: 'amend.txt',
    },);
    /**
     Bystander plans.
     */
    const plans = disjointPlans(BYSTANDERS,);
    await rewritePaths({
      context,
      plans,
    },);
    await writeWorktree({
      ...context,
      path: 'amend.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('amend',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Bystanders and the amend started together.
     */
    const [bystanders, amend,] = await Promise.all([
      startWorkers({
        context,
        plans,
        maxJitterMs: MAX_JITTER_MS,
      },),
      startAttempt({
        ...context,
        label: 'amend',
        paths: ['amend.txt',],
        mode: 'amend',
      },),
    ],);
    /**
     Finished bystanders.
     */
    const finished = await finishAll(bystanders,);
    return [
      allSucceeded({
        name: 'bystanders-succeed',
        attempts: finished,
      },),
      await amendReplacedBase({
        context,
        amend: await amend.finished,
        parent,
      },),
    ];
  },
};

/**
 Branch switch while a commit prepares.
 */
const branchSwitch: ScenarioDefinition = {
  name: 'branch-switch-during-commit',
  group: 'concurrency',
  // The held commit fails with branch-switched before any replay.
  replayPlumbing: 'unused',
  summary: 'git switch to another branch while a commit is held in pre-commit; the commit must not land on the new branch',
  // Config hooks need Git 2.54.0.
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: ['a.txt',],
      },),
      hooks: 'config',
      hookEvents: ['pre-commit',],
    },);
  },
  async run(context,) {
    await realGit({
      repository: context.repository,
      args: [
        'branch',
        'side',
      ],
    },);
    await realGit({
      repository: context.repository,
      args: [
        'push',
        '--quiet',
        '--set-upstream',
        'origin',
        'side',
      ],
    },);
    await realGit({
      repository: context.repository,
      args: [
        'branch',
        '--quiet',
        '--set-upstream-to=origin/main',
        'main',
      ],
    },);
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
     Commit held while the switch runs.
     */
    const held = await startAttempt({
      ...context,
      label: 'held',
      paths: ['a.txt',],
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
     Branch switch through the wrapper.
     */
    const switched = await runWrapper({
      ...context,
      label: 'switch side',
      args: [
        'switch',
        'side',
      ],
      mustSucceed: false,
    },);
    await releaseAt({
      repository: context.repository,
      token: held.token,
      event: 'pre-commit',
    },);
    /**
     Finished held commit.
     */
    const record = await held.finished;
    return [{
      name: 'switch-invalidates-commit',
      holds: (switched.exitCode !== 0) || (record.outcome
        .exitCode
        !== 0),
      detail: 'both the branch switch and the held commit exited 0',
    },];
  },
};

/**
 Amend and branch-switch scenarios in report order.
 */
export const AMEND_SWITCH_SCENARIOS: readonly ScenarioDefinition[] = [
  amendDuringCommits,
  branchSwitch,
];

//endregion Scenarios
