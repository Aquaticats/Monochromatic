/**
 `SIGKILL` to the commit service itself, at each instant of a landing batch that the service exposes,
 while two agents commit, followed by the service manager's restart, the clients' retries, and a follow-up commit.

 The service lands a batch in a fixed order: it writes the batch's objects, journals the intent, moves the branch by
 one compare-and-swap, updates the real index, and acknowledges the clients.
 The service ends itself at a named point of that order when `CLI_GIT_TEST_ONLY_SERVICE_FAULT` names it
 (`src/native/service_fault.rs`, which the release executable already carries; this suite adds no hook).
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
 Seeded window of the external kill after the service runs again, in milliseconds.
 A batch of several commits takes the service longer than the window, so the kill lands inside it.
 */
const SERVICE_KILL_WINDOW = {
  minMs: 0,
  maxMs: 5,
} as const;

/**
 Agents committing in the external-kill scenario.
 */
const KILLED_SERVICE_AGENTS = 6;

/**
 Text size of rewritten files.
 */
const FILE_BYTES = 400;

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
async function recoverAndCommit(context: ScenarioContext,): Promise<Expectations> {
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
  // The restart is the service manager's: it follows the end of the armed process, however long the agents take.
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
 Kills the service from outside at a seeded instant inside a batch, then restarts it.
 Every agent is queued at the frozen service first,
 so the service lands them as one batch when it runs again and the kill comes while that batch is in progress.

 @param context - scenario context

 @returns expectations

 @example
 ```ts
 await killServiceAtOffset(context);
 ```
 */
async function killServiceAtOffset(context: ScenarioContext,): Promise<Expectations> {
  /**
   Disjoint explicit-path agents.
   */
  const plans = disjointPlans(KILLED_SERVICE_AGENTS,);
  await rewritePaths({
    context,
    plans,
  },);
  /**
   The scenario's service.
   */
  const { service, } = context.repository;
  service.stall();
  /**
   Agents prepared and queued at the frozen service, one after another.
   */
  const queued = await plans.reduce<Promise<readonly QueuedAttempt[]>>(
    async function queueNext(
      previous,
      plan,
    ) {
      return [
        ...await previous,
        await startQueued({
          context,
          label: plan.label,
          paths: plan.paths,
        },),
      ];
    },
    Promise.resolve([],),
  );
  service.resume();
  /**
   Seeded wait between the service running again and its kill.
   */
  const offset = planFaultOffset({
    random: context.random
      .fork('kill',),
    ...SERVICE_KILL_WINDOW,
  },);
  await sleep(offset,);
  /**
   Agents still waiting for their answer when the service dies.
   */
  const waiting = queued.filter(function unsettled(entry,) {
    return !entry.attempt
      .running
      .isSettled();
  },);
  await service.kill();
  await service.restart();
  /**
   Settled agents.
   */
  const settled = await finishAll(queued.map(function attemptOf(entry,) {
    return entry.attempt;
  },),);
  return [
    queuedExpectation(queued,),
    {
      name: 'service-killed-with-agents-waiting',
      holds: waiting.length > 0,
      detail: `every agent had its answer before the service was killed ${String(offset,)} ms after it ran again`,
    },
    allSucceeded({
      name: 'agents-succeed-after-restart',
      attempts: settled,
    },),
    ...await recoverAndCommit(context,),
  ];
}

/**
 Kills the service once it claimed the after-commit record of a landed commit.
 The claim is the earliest observable instant of the service's own after-commit work,
 so the kill comes while the push and the upkeep run.

 @param context - scenario context

 @returns expectations

 @example
 ```ts
 await killServiceDuringWork(context);
 ```
 */
async function killServiceDuringWork(context: ScenarioContext,): Promise<Expectations> {
  await writeWorktree({
    ...context,
    path: 'victim.txt',
    bytes: synthesizeText({
      random: context.random
        .fork('victim',),
      size: FILE_BYTES,
    },),
  },);
  /**
   Agent whose commit lands before the kill.
   */
  const victim = await startAttempt({
    ...context,
    label: 'victim',
    paths: ['victim.txt',],
    mode: 'explicit',
  },);
  /**
   Whether the service claimed a record before the agent settled.
   */
  const arrival = await waitForPushRecord({
    repository: context.repository,
    running: victim.running,
    states: ['claimed',],
  },);
  await context.repository
    .service
    .kill();
  await context.repository
    .service
    .restart();
  return [
    {
      name: 'service-killed-with-work-claimed',
      holds: arrival === 'record',
      detail: `the agent ${arrival} before the service claimed its record`,
    },
    allSucceeded({
      name: 'agent-succeeds',
      attempts: [await victim.finished,],
    },),
    ...await recoverAndCommit(context,),
  ];
}

/**
 Builds the scenario for one fault point.

 @param spec - point and the instant it names

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

/**
 External kill of the service at a seeded instant.
 */
const serviceOffset: ScenarioDefinition = {
  name: 'sigkill-service-offset',
  group: 'concurrency',
  replayPlumbing: 'unused',
  summary: `SIGKILL the commit service at a seeded offset inside a batch of ${String(KILLED_SERVICE_AGENTS,)} queued commits, restart it, recover`,
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          ...workerPaths({
            prefix: 'w',
            count: KILLED_SERVICE_AGENTS,
          },),
          'follow-up.txt',
        ],
      },),
      afterCommitRecords: true,
    },);
  },
  run: killServiceAtOffset,
};

/**
 External kill of the service while it works on a landed commit's after-commit records.
 */
const serviceDuringWork: ScenarioDefinition = {
  name: 'sigkill-service-during-work',
  group: 'concurrency',
  replayPlumbing: 'unused',
  summary: 'SIGKILL the commit service once it claimed a landed commit\'s push record, restart it, recover',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'victim.txt',
          'follow-up.txt',
        ],
      },),
      afterCommitRecords: true,
    },);
  },
  run: killServiceDuringWork,
};

//endregion Workload

/**
 Service kill scenarios in the order a batch reaches its points.
 */
export const PHASE_KILL_SCENARIOS: readonly ScenarioDefinition[] = [
  ...FAULT_POINTS.map(faultScenario,),
  serviceOffset,
  serviceDuringWork,
];
