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
import { editorOptions, } from './editor-attempt-fixture.ts';
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
  // Nothing else commits, so the held commit never loses a landing race.
  replayPlumbing: 'unused',
  summary: 'the checked-out branch changes while a commit is held in its message editor; the commit must not land on the new branch and the repository must stay consistent',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: ['a.txt',],
      },),
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
      ...editorOptions({ repository: context.repository, },),
    },);
    await holdAt({
      repository: context.repository,
      token: held.token,
      event: 'editor-open',
    },);
    /**
     Whether the editor was reached before the commit settled.
     */
    const arrival = await reached({
      repository: context.repository,
      token: held.token,
      running: held.running,
      event: 'editor-open',
    },);
    /**
     Tip of the branch switched to, before the switch.
     */
    const sideBefore = (await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'refs/heads/side',
      ],
    },)).trim();
    /**
     Branch switch through the wrapper, which refuses it while tracked files are modified.
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
    // The wrapper's policy refuses that switch, so a person or a tool that bypasses the wrapper switches with real Git.
    await realGit({
      repository: context.repository,
      args: [
        'switch',
        '--quiet',
        'side',
      ],
    },);
    await releaseAt({
      repository: context.repository,
      token: held.token,
      event: 'editor-open',
    },);
    /**
     Finished held commit.
     */
    const record = await held.finished;
    /**
     Tip of the branch switched to, after the held commit finished.
     */
    const sideAfter = (await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'refs/heads/side',
      ],
    },)).trim();
    return [
      {
        name: 'held-commit-reached-editor',
        holds: arrival === 'marker',
        detail: `the held commit ${arrival} before its editor opened`,
      },
      {
        name: 'wrapper-switch-refused',
        holds: switched.exitCode !== 0,
        detail: 'the wrapper switched branches while tracked files were modified',
      },
      {
        name: 'commit-not-on-switched-branch',
        holds: sideAfter === sideBefore,
        detail: `the branch switched to moved from ${sideBefore} to ${sideAfter}; the held commit exited ${String(record.outcome
          .exitCode,)}`,
      },
    ];
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
