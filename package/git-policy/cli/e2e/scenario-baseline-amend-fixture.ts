/**
 Single-commit baselines that rewrite history or run a lint-staged hook:
 amend of a setup commit,
 and a partially staged index commit through lint-staged's stash sequence.
 Each must pass against today's build.

 @module
 */

import { synthesizeText, } from './content-fixture.ts';
import {
  repositoryOptions,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  amendReplacedBase,
  commitAmendBase,
  stagePartially,
} from './scenario-setup-fixture.ts';
import {
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
 Amend of a token-free setup commit.
 */
const amend: ScenarioDefinition = {
  name: 'baseline-amend',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'amend a setup commit with new bytes for its path',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: ['amend.txt',],
    },), },);
  },
  async run(context,) {
    /**
     Parent of the setup commit.
     */
    const { parent, } = await commitAmendBase({
      context,
      path: 'amend.txt',
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
     Amend attempt.
     */
    const attempt = await (await startAttempt({
      ...context,
      label: 'amend',
      paths: ['amend.txt',],
      mode: 'amend',
    },)).finished;
    return [
      allSucceeded({
        name: 'all-commits-succeed',
        attempts: [attempt,],
      },),
      await amendReplacedBase({
        context,
        amend: attempt,
        parent,
      },),
    ];
  },
};

/**
 lint-staged-style hook on a partially staged index commit in a linked worktree.
 */
const lintStaged: ScenarioDefinition = {
  name: 'baseline-lint-staged',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'lint-staged backup stash and hide-unstaged hook on one partially staged index commit (linked worktree)',
  // Config hooks need Git 2.54.0.
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: ['partial.txt',],
      },),
      worktree: 'linked',
      hooks: 'config',
      hookEvents: ['pre-commit',],
      hookMode: 'lint-staged',
    },);
  },
  async run(context,) {
    await stagePartially({
      context,
      path: 'partial.txt',
    },);
    /**
     Index attempt.
     */
    const attempt = await (await startAttempt({
      ...context,
      label: 'lint-staged',
      paths: ['partial.txt',],
      mode: 'index',
    },)).finished;
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts: [attempt,],
    },),];
  },
};

/**
 Amend and lint-staged baselines in report order.
 */
export const BASELINE_AMEND_SCENARIOS: readonly ScenarioDefinition[] = [
  amend,
  lintStaged,
];

//endregion Scenarios
