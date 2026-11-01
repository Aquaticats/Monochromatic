/**
 Commits interleaved with `git add` index writers in a seeded order.

 @module
 */

import { synthesizeText, } from './content-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import {
  disjointPlans,
  repositoryOptions,
  seedTexts,
  workerPaths,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import { interleaveSequences, } from './seeded-schedule-fixture.ts';
import {
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Committing agents and staging agents each.
 */
const AGENTS = 4;

/**
 Text size of written files.
 */
const FILE_BYTES = 400;

//endregion Constants

//region Workload

/**
 Reports whether the real index holds exactly the worktree bytes of a path.

 @param context - scenario context

 @param path - repository path

 @returns whether the stage-0 blob equals the worktree blob

 @example
 ```ts
 await indexHoldsWorktree({ context, path: 'staged-0.txt' });
 ```
 */
async function indexHoldsWorktree({
  context,
  path,
}: Readonly<{
  context: ScenarioContext;
  path: string;
}>,): Promise<boolean> {
  /**
   Mode and staged blob ID fields.
   */
  const [, stagedOid,] = (await realGit({
    repository: context.repository,
    args: [
      'ls-files',
      '--stage',
      '--',
      path,
    ],
  },)).split(' ',);
  /**
   Worktree blob ID.
   */
  const worktreeOid = (await realGit({
    repository: context.repository,
    args: [
      'hash-object',
      '--',
      path,
    ],
  },)).trim();
  return stagedOid === worktreeOid;
}

/**
 Launches commits and staging runs in a seeded interleaving.

 @param context - scenario context

 @param staged - new paths the staging agents add

 @returns settlement promises of every launched agent

 @example
 ```ts
 await launchInterleaved({ context, staged });
 ```
 */
async function launchInterleaved({
  context,
  staged,
}: Readonly<{
  context: ScenarioContext;
  staged: readonly string[];
}>,): Promise<readonly Promise<unknown>[]> {
  /**
   Commit plans.
   */
  const plans = disjointPlans(AGENTS,);
  /**
   Seeded order of commit starts and staging runs.
   */
  const order = interleaveSequences({
    random: context.random
      .fork('interleave',),
    lengths: [
      plans.length,
      staged.length,
    ],
  },);
  return await order.reduce<Promise<readonly Promise<unknown>[]>>(
    async function launch(
      previous,
      step,
    ) {
    /**
     Work launched so far.
     */
      const launched = await previous;
    /**
     Commit plan when this step starts a commit.
     */
      const plan = step.actor === 0 ? plans[step.position] : undefined;
      if (plan !== undefined)
        return [
          ...launched,
          (await startAttempt({
            ...context,
            label: plan.label,
            paths: plan.paths,
            mode: 'explicit',
          },)).finished,
        ];
    /**
     Path this staging agent adds.
     */
      const path = staged[step.position] ?? '';
      context.ledger
        .recordStaged({
          path,
          staged: true,
        },);
      return [
        ...launched,
        runWrapper({
          ...context,
          label: `add ${path}`,
          args: [
            'add',
            '--',
            path,
          ],
          mustSucceed: true,
        },),
      ];
    },
    Promise.resolve([],),
  );
}

//endregion Workload

//region Scenarios

/**
 Commits interleaved with `git add` index writers.
 */
const indexWriters: ScenarioDefinition = {
  name: 'interleaved-index-writers',
  group: 'concurrency',
  summary: '4 committing agents interleaved with 4 agents staging new files through git add',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: workerPaths({
        prefix: 'w',
        count: AGENTS,
      },),
    },), },);
  },
  async run(context,) {
    /**
     New files the staging agents add.
     */
    const staged = workerPaths({
      prefix: 'staged',
      count: AGENTS,
    },);
    await Promise.all([
      ...workerPaths({
        prefix: 'w',
        count: AGENTS,
      },),
      ...staged,
    ].map(async function write(path,) {
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
    await Promise.all(await launchInterleaved({
      context,
      staged,
    },),);
    /**
     Auxiliary commands after every agent settled.
     */
    const {
      auxiliaries,
      attempts,
    } = context.ledger
      .snapshot();
    /**
     Paths whose add exited 0 but whose index entry no longer holds the added bytes.
     */
    const lost = (await Promise.all(staged.map(async function lostEntry(path,) {
      /**
       Whether this path's add exited 0.
       */
      const added = auxiliaries.some(function addSucceeded(auxiliary,) {
        return (auxiliary.label === `add ${path}`) && (auxiliary.outcome
          .exitCode
          === 0);
      },);
      return added && (!(await indexHoldsWorktree({
        context,
        path,
      },))) ? [path,] : [];
    },),)).flat();
    return [
      allSucceeded({
        name: 'all-commits-succeed',
        attempts,
      },),
      {
        name: 'staged-entries-kept',
        holds: lost.length === 0,
        detail: `staged entries lost: ${lost.join(', ',)}`,
      },
    ];
  },
};

/**
 Index-writer scenarios in report order.
 */
export const INDEX_WRITER_SCENARIOS: readonly ScenarioDefinition[] = [indexWriters,];

//endregion Scenarios
