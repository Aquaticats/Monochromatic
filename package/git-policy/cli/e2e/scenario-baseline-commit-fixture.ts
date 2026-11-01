/**
 Single-commit baselines for commit shapes:
 explicit paths,
 index commits,
 and sequential trace replay.
 Each must pass against today's build,
 which proves the harness sound for these shapes.

 @module
 */

import {
  editText,
  synthesizeText,
} from './content-fixture.ts';
import {
  allSucceeded,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  repositoryOptions,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  seedTracePaths,
  TRACE_MAX_CHANGES,
} from './scenario-setup-fixture.ts';
import { runTraceOperations, } from './trace-execution-fixture.ts';
import {
  planTraceOperations,
  selectTraceWindow,
} from './trace-replay-fixture.ts';
import {
  readWorktree,
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Trace window length for baseline replay.
 */
const BASELINE_TRACE_COMMITS = 12;

/**
 Text size of files the baselines write.
 */
const FILE_BYTES = 400;

//endregion Constants

//region Scenarios

/**
 Modify,
 add,
 and delete through three sequential explicit-path commits.
 */
const explicitCommit: ScenarioDefinition = {
  name: 'baseline-explicit-commit',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'modify, add, and delete one path each through sequential explicit-path commits',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: [
        'modified.txt',
        'gone.txt',
      ],
    },), },);
  },
  async run(context,) {
    /**
     Current bytes of the modified file.
     */
    const current = (await readWorktree({
      repository: context.repository,
      path: 'modified.txt',
    },)).bytes ?? Buffer.from('x\n',);
    await writeWorktree({
      ...context,
      path: 'modified.txt',
      bytes: editText({
        random: context.random,
        current,
        added: 3,
        deleted: 2,
      },),
    },);
    /**
     Modify attempt.
     */
    const modify = await (await startAttempt({
      ...context,
      label: 'modify',
      paths: ['modified.txt',],
      mode: 'explicit',
    },)).finished;
    await writeWorktree({
      ...context,
      path: 'added.txt',
      bytes: synthesizeText({
        random: context.random,
        size: FILE_BYTES,
      },),
    },);
    await runWrapper({
      ...context,
      label: 'add added.txt',
      args: [
        'add',
        '--',
        'added.txt',
      ],
      mustSucceed: true,
    },);
    context.ledger
      .recordStaged({
        path: 'added.txt',
        staged: true,
      },);
    /**
     Add attempt.
     */
    const add = await (await startAttempt({
      ...context,
      label: 'add',
      paths: ['added.txt',],
      mode: 'explicit',
    },)).finished;
    await writeWorktree({
      ...context,
      path: 'gone.txt',
    },);
    /**
     Delete attempt.
     */
    const remove = await (await startAttempt({
      ...context,
      label: 'delete',
      paths: ['gone.txt',],
      mode: 'explicit',
    },)).finished;
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts: [
        modify,
        add,
        remove,
      ],
    },),];
  },
};

/**
 Index commit of two staged paths beside an unrelated unstaged edit.
 */
const indexCommit: ScenarioDefinition = {
  name: 'baseline-index-commit',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'git add two paths, keep an unrelated unstaged edit, commit --no-only',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: [
        'one.txt',
        'two.txt',
        'tail.txt',
      ],
    },), },);
  },
  async run(context,) {
    await Promise.all([
      'one.txt',
      'two.txt',
      'tail.txt',
    ].map(async function rewrite(path,) {
      await writeWorktree({
        ...context,
        path,
        bytes: synthesizeText({
          random: context.random
            .fork(path,),
          size: FILE_BYTES,
        },),
      },);
    },),);
    await runWrapper({
      ...context,
      label: 'add one two',
      args: [
        'add',
        '--',
        'one.txt',
        'two.txt',
      ],
      mustSucceed: true,
    },);
    [
      'one.txt',
      'two.txt',
    ].forEach(function staged(path,) {
      context.ledger
        .recordStaged({
          path,
          staged: true,
        },);
    },);
    /**
     Index attempt.
     */
    const attempt = await (await startAttempt({
      ...context,
      label: 'index',
      paths: [
        'one.txt',
        'two.txt',
      ],
      mode: 'index',
    },)).finished;
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts: [attempt,],
    },),];
  },
};

/**
 Sequential trace replay.
 */
const traceSequential: ScenarioDefinition = {
  name: 'baseline-trace-sequential',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: `replay ${String(BASELINE_TRACE_COMMITS,)} consecutive trace commits one at a time`,
  // Config hooks need Git 2.54.0.
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: ['README.txt',],
      },),
      hooks: 'config',
      hookEvents: ['pre-commit',],
    },);
  },
  async run(context,) {
    /**
     Window operations and the seed files they need.
     */
    const {
      operations,
      seeds,
    } = planTraceOperations(selectTraceWindow({
      trace: context.trace,
      random: context.random
        .fork('window',),
      length: BASELINE_TRACE_COMMITS,
      maxChanges: TRACE_MAX_CHANGES,
    },),);
    await seedTracePaths({
      context,
      seeds,
    },);
    /**
     Finished attempts.
     */
    const attempts = await runTraceOperations({
      context,
      operations,
      concurrency: 1,
    },);
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts,
    },),];
  },
};

/**
 Commit-shape baselines in report order.
 */
export const BASELINE_COMMIT_SCENARIOS: readonly ScenarioDefinition[] = [
  explicitCommit,
  indexCommit,
  traceSequential,
];

//endregion Scenarios
