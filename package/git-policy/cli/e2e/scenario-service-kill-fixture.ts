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
import type { SeededRandom, } from './seeded-random-fixture.ts';
import type { RepositoryOptions, } from './repository-model-fixture.ts';
import {
  claimsAtKill,
  claimsRunExpectation,
} from './scenario-claim-recovery-fixture.ts';
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
import { waitForServiceWork, } from './service-settle-fixture.ts';
import {
  readBranchTip,
  waitForPushRecord,
} from './work-record-fixture.ts';
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

/**
 How long a claim of a killed service stays out of reach of every other instance, in milliseconds:
 the service's `CLAIM_LEASE` (`src/native/landing/service_work.rs`),
 which `service-kill-lease.unit.test.ts` reads from that file so a change of the lease fails there with its reason.
 A restarted service leaves a claim alone until its lease runs out,
 because a service that is still running renews it and the claim's holder is never told apart by process ID.
 */
export const CLAIM_LEASE_MS = 120_000;

/**
 What the recovery of a killed service's claims may take beyond the lease, in milliseconds:
 the service looks at the repository again when the lease is over (`work_loop` in `src/native/service/service_server.rs`),
 requeues the claims, and then runs the push and the upkeep.
 A passing run took one second beyond the lease (2026-11-04, seed 1, container of two CPUs).
 */
export const CLAIM_RECOVERY_MARGIN_MS = 30_000;

/**
 The longest the scenario waits for the claims of the killed service to be run, in milliseconds.
 Together with the rest of the workload it stays under the scenario timeout (`SCENARIO_TIMEOUT_MS`).
 */
export const CLAIM_RECOVERY_BUDGET_MS: number = CLAIM_LEASE_MS + CLAIM_RECOVERY_MARGIN_MS;

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
 Whether the commit after the kill wakes the restarted service.
 */
type FollowUp = 'commits' | 'none';

/**
 Kills the service once it claimed the after-commit record of a landed commit.
 The claim is the earliest observable instant of the service's own after-commit work,
 so the kill comes while the push and the upkeep run.
 Then it restarts the service and waits for the dead service's claims to be taken back.

 @param context - scenario context

 @param followUp - `commits` makes a follow-up commit after the restart, whose wake reaches the service;
 `none` runs no wrapper command and sends no wake,
 so the claims can only be run by the restarted service itself

 @returns expectations

 @example
 ```ts
 await killServiceDuringWork({ context, followUp: 'none' });
 ```
 */
async function killServiceDuringWork({
  context,
  followUp,
}: Readonly<{
  context: ScenarioContext;
  followUp: FollowUp;
}>,): Promise<Expectations> {
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
  /**
   The commit the claimed records are about, the only one on the branch so far.
   */
  const commit = await readBranchTip(context.repository,);
  await context.repository
    .service
    .kill();
  /**
   When the service died, with its claims on the landed commit's records.
   */
  const killedAt = Date.now();
  /**
   What the dead service held.
   */
  const held = await claimsAtKill({
    repository: context.repository,
    commit,
  },);
  await context.repository
    .service
    .restart();
  /**
   Expectations of the follow-up commit; a restarted service with no follow-up gets no wake at all.
   */
  const recovered = followUp === 'commits' ? await recoverAndCommit(context,) : [];
  /**
   Records still waiting after the lease and the margin.
   */
  const unfinished = await waitForServiceWork({
    gitDir: context.repository.gitDir,
    deadlineMilliseconds: Math.max(
      0,
      CLAIM_RECOVERY_BUDGET_MS - (Date.now() - killedAt),
    ),
  },);
  /**
   How long after the kill the wait ended.
   */
  const recoveredAfter = Date.now() - killedAt;
  return [
    {
      name: 'service-killed-with-work-claimed',
      holds: arrival === 'record',
      detail: `the agent ${arrival} before the service claimed its record`,
    },
    held.expectation,
    allSucceeded({
      name: 'agent-succeeds',
      attempts: [await victim.finished,],
    },),
    ...recovered,
    {
      name: 'killed-service-claims-recovered-within-lease',
      holds: (unfinished.length === 0) && (recoveredAfter <= CLAIM_RECOVERY_BUDGET_MS),
      detail: `${unfinished.length === 0 ? 'nothing' : unfinished.join(', ',)} unfinished ${String(recoveredAfter,)} ms after the kill, `
        + `a lease of ${String(CLAIM_LEASE_MS,)} ms and ${String(CLAIM_RECOVERY_MARGIN_MS,)} ms of margin`,
    },
    await claimsRunExpectation({
      repository: context.repository,
      commit,
      claimed: held.claimed,
    },),
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
 Repository setup of the scenarios that kill the service while it works on a landed commit.

 @param random - the scenario's seeded source

 @returns repository options

 @example
 ```ts
 repositoryOf(random);
 ```
 */
function repositoryOf(random: SeededRandom,): RepositoryOptions {
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
}

/**
 External kill of the service while it works on a landed commit's after-commit records,
 followed by a commit whose wake reaches the restarted service.
 */
const serviceDuringWork: ScenarioDefinition = {
  name: 'sigkill-service-during-work',
  group: 'concurrency',
  replayPlumbing: 'unused',
  summary: 'SIGKILL the commit service once it claimed a landed commit\'s push record, restart it, recover, and wait out the claim\'s lease',
  repository: repositoryOf,
  async run(context,) {
    return await killServiceDuringWork({
      context,
      followUp: 'commits',
    },);
  },
};

/**
 The same kill with nobody committing afterwards: no wrapper command runs and no wake reaches the restarted service.
 */
const serviceThenIdle: ScenarioDefinition = {
  name: 'sigkill-service-then-idle',
  group: 'concurrency',
  replayPlumbing: 'unused',
  summary: 'SIGKILL the commit service once it claimed a landed commit\'s push record, restart it, send it nothing, and wait out the claim\'s lease',
  repository: repositoryOf,
  async run(context,) {
    return await killServiceDuringWork({
      context,
      followUp: 'none',
    },);
  },
};

//endregion Workload

/**
 Service kill scenarios at instants the harness chooses.
 */
export const SERVICE_KILL_SCENARIOS: readonly ScenarioDefinition[] = [
  serviceOffset,
  serviceDuringWork,
  serviceThenIdle,
];
