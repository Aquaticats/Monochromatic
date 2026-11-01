/**
 `SIGKILL` to the commit service from outside, at instants a harness can observe or seed,
 while agents commit, followed by the service manager's restart, the clients' retries, and a follow-up commit.

 The fault-point scenarios (`scenario-phase-kill-fixture.ts`) kill the service at named instants of a landing batch.
 These kill it where the harness chooses:
 at a seeded offset inside a batch of commits queued at a frozen service,
 and once the service claimed a landed commit's after-commit record.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';

import { synthesizeText, } from './content-fixture.ts';
import {
  FILE_BYTES,
  recoverAndCommit,
} from './scenario-phase-kill-fixture.ts';
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
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

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

//endregion Constants

//region Workload

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
 Service kill scenarios at instants the harness chooses.
 */
export const SERVICE_KILL_SCENARIOS: readonly ScenarioDefinition[] = [
  serviceOffset,
  serviceDuringWork,
];
