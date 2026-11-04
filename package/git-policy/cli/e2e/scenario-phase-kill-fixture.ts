/**
 `SIGKILL` to the commit service itself, at each instant of a landing batch that the service exposes,
 while two agents commit, followed by the service manager's restart, the clients' retries, and a follow-up commit.

 The service lands a batch in a fixed order: it writes the batch's objects, journals the intent, moves the branch by
 one compare-and-swap, updates the real index, and acknowledges the clients.
 The service ends itself at a named point of that order when `CLI_GIT_TEST_ONLY_SERVICE_FAULT` names it
 (`src/native/service/service_fault.rs`, which the release executable already carries; this suite adds no hook).
 The container has no service manager, so the harness restarts the service after it ended, as socket activation
 restarts a crashed unit, and the clients retry with their request identity.
 Whatever the instant, each commit must land exactly once, the index and worktree must agree with the branch,
 the repository must hold no stray state, and the push must reach the remote.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';

import { synthesizeText, } from './content-fixture.ts';
import {
  disjointPlans,
  finishAll,
  repositoryOptions,
  rewritePaths,
  seedTexts,
  workerPaths,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type Expectations,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  type QueuedAttempt,
  queuedExpectation,
  startQueued,
} from './scenario-queue-fixture.ts';
import { planFaultOffset, } from './seeded-schedule-fixture.ts';
import { waitForPushRecord, } from './work-record-fixture.ts';
import {
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Fault points of a batch, in the order the service reaches them,
 each with the instant it names.
 */
const FAULT_POINTS = [
  {
    point: 'after-objects',
    instant: 'the batch\'s objects exist and nothing is journaled',
  },
  {
    point: 'after-intent',
    instant: 'the intent is journaled and the branch has not moved',
  },
  {
    point: 'after-ref',
    instant: 'the branch moved and the real index is not updated',
  },
  {
    point: 'after-index',
    instant: 'the real index is updated and the intent is open',
  },
  {
    point: 'before-ack',
    instant: 'everything is done and no client is told',
  },
] as const;

/**
 Text size of rewritten files.
 */
export const FILE_BYTES = 400;

//endregion Constants

//region Workload

/**
 Recovery that follows a kill: a status through the wrapper, then a follow-up commit.

 @param context - scenario context

 @returns the follow-up expectation

 @example
 ```ts
 await recoverAndCommit(context);
 ```
 */
export async function recoverAndCommit(context: ScenarioContext,): Promise<Expectations> {
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
    path: 'follow-up.txt',
    bytes: synthesizeText({
      random: context.random
        .fork('follow-up',),
      size: FILE_BYTES,
    },),
  },);
  /**
   Follow-up after recovery.
   */
  const followUp = await (await startAttempt({
    ...context,
    label: 'follow-up',
    paths: ['follow-up.txt',],
    mode: 'explicit',
  },)).finished;
  return [allSucceeded({
    name: 'follow-up-commit-succeeds',
    attempts: [followUp,],
  },),];
}

/**
 Arms the service with a fault, lets two agents commit through it, restarts it when it ended, and recovers.

 @param context - scenario context

 @param point - fault point

 @returns expectations

 @example
 ```ts
 await killServiceAt({ context, point: 'after-ref' });
 ```
 */
async function killServiceAt({
  context,
  point,
}: Readonly<{
  context: ScenarioContext;
  point: string;
}>,): Promise<Expectations> {
  /**
   The scenario's service.
   */
  const { service, } = context.repository;
  await service.restart(point,);
  await Promise.all([
    'victim.txt',
    'bystander.txt',
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
  /**
   The restart, which is the service manager's: it follows the end of the armed process, however long the agents take.
   */
  const restarted = service.restartAfterExit();
  /**
   Both agents start together; the one whose batch reaches the point loses its service.
   */
  const [victim, bystander,] = await Promise.all([
    startAttempt({
      ...context,
      label: 'victim',
      paths: ['victim.txt',],
      mode: 'explicit',
    },),
    startAttempt({
      ...context,
      label: 'bystander',
      paths: ['bystander.txt',],
      mode: 'explicit',
    },),
  ],);
  /**
   Settled agents.
   */
  const settled = await finishAll([
    victim,
    bystander,
  ],);
  /**
   Whether the armed service ended itself at the point.
   */
  const fired = service.log()
    .includes(`test fault at ${point}`,);
  if (fired)
    await restarted;
  return [
    {
      name: 'service-fault-fired',
      holds: fired,
      detail: `the service never reached ${point}: ${service.log()}`,
    },
    allSucceeded({
      name: 'agents-succeed-after-restart',
      attempts: settled,
    },),
    ...await recoverAndCommit(context,),
  ];
}

/**
 Builds the scenario for one fault point.

 @param point - fault point

 @param instant - the instant of a batch that the point names

 @returns scenario definition

 @example
 ```ts
 faultScenario({ point: 'after-ref', instant: 'the branch moved' });
 ```
 */
function faultScenario({
  point,
  instant,
}: Readonly<{
  point: string;
  instant: string;
}>,): ScenarioDefinition {
  return {
    name: `sigkill-service-${point}`,
    group: 'concurrency',
    // The agents' commits touch different files, so a retry after the restart never has a branch to lose a race on.
    replayPlumbing: 'unused',
    summary: `SIGKILL the commit service at ${point} (${instant}) while two agents commit, restart it, recover`,
    repository(random,) {
      return repositoryOptions({
        seedFiles: seedTexts({
          random,
          paths: [
            'victim.txt',
            'bystander.txt',
            'follow-up.txt',
          ],
        },),
        afterCommitRecords: true,
      },);
    },
    async run(context,) {
      return await killServiceAt({
        context,
        point,
      },);
    },
  };
}

//endregion Workload

/**
 Service fault-point scenarios in the order a batch reaches its points.
 */
export const PHASE_KILL_SCENARIOS: readonly ScenarioDefinition[] = FAULT_POINTS.map(faultScenario,);
