/**
 `SIGKILL` to one committing agent at each point of its commit that a harness can observe from outside,
 while another agent commits, followed by recovery through an ordinary wrapper invocation and a follow-up commit.

 A commit has four client-side points before the service owns it:
 the message editor open (preparation is under way, the message is not final),
 the message editor done (the message is final, nothing is sent),
 the commit queued at a frozen service (prepared and connected, nothing is sent),
 and the commit landed (the service is frozen the instant the branch moves, so the agent dies in its exchange with it).
 A seeded time offset can land anywhere, including inside the exchange with the service.
 The first three points leave nothing to land, so the victim's commit must not appear;
 the later ones leave a landed commit that must appear once.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';

import {
  holdAt,
  reached,
} from './barrier-fixture.ts';
import { synthesizeText, } from './content-fixture.ts';
import { editorOptions, } from './editor-attempt-fixture.ts';
import { killWhenLanded, } from './landing-kill-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { landedCommit, } from './scenario-reservation-helper-fixture.ts';
import {
  repositoryOptions,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type Expectations,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  queuedExpectation,
  startQueued,
} from './scenario-queue-fixture.ts';
import { planFaultOffset, } from './seeded-schedule-fixture.ts';
import { readBranchTip, } from './work-record-fixture.ts';
import {
  runWrapper,
  type StartedAttempt,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Seeded kill window after start, in milliseconds.
 */
const KILL_WINDOW = {
  minMs: 20,
  maxMs: 600,
} as const;

/**
 The points a client is killed at.
 */
type KillPoint = 'after-landing' | 'editor-open' | 'editor-written' | 'offset' | 'queued';

/**
 What the kill leaves of the victim's commit.
 `nothing` for a point before the commit reaches the service,
 `landed` for a point after it landed,
 `either` for a time offset.
 */
type VictimOutcome = 'either' | 'landed' | 'nothing';

/**
 Outcome each point leaves.
 */
const OUTCOMES: Readonly<Record<KillPoint, VictimOutcome>> = {
  'after-landing': 'landed',
  'editor-open': 'nothing',
  'editor-written': 'nothing',
  offset: 'either',
  queued: 'nothing',
};

//endregion Constants

//region Workload

/**
 Starts the victim and brings it to the kill point.

 @param context - scenario context

 @param point - where the victim is killed

 @returns the victim, a bystander start, and what the wait for the point came to

 @example
 ```ts
 await reachPoint({ context, point: 'editor-open' });
 ```
 */
async function reachPoint({
  context,
  point,
}: Readonly<{
  context: ScenarioContext;
  point: KillPoint;
}>,): Promise<Readonly<{
  victim: StartedAttempt;
  bystander: () => Promise<StartedAttempt>;
  expectations: Expectations;
}>> {
  /**
   Starts the bystander, a plain explicit-path commit of its own file.

   @returns the started bystander
   */
  async function startBystander(): Promise<StartedAttempt> {
    return await startAttempt({
      ...context,
      label: 'bystander',
      paths: ['bystander.txt',],
      mode: 'explicit',
    },);
  }
  if ((point === 'editor-open') || (point === 'editor-written')) {
    /**
     Victim that opens the scripted editor.
     */
    const victim = await startAttempt({
      ...context,
      label: 'victim',
      paths: ['victim.txt',],
      mode: 'explicit',
      ...editorOptions({ repository: context.repository, },),
    },);
    await holdAt({
      repository: context.repository,
      token: victim.token,
      event: point,
    },);
    /**
     Whether the editor reached the point.
     */
    const arrival = await reached({
      repository: context.repository,
      token: victim.token,
      running: victim.running,
      event: point,
    },);
    return {
      victim,
      bystander: startBystander,
      expectations: [{
        name: 'victim-reached-point',
        holds: arrival === 'marker',
        detail: `victim ${arrival} before its editor reached ${point}`,
      },],
    };
  }
  if (point === 'queued') {
    context.repository
      .service
      .stall();
    /**
     Victim prepared and connected to the frozen service.
     */
    const victim = await startQueued({
      context,
      label: 'victim',
      paths: ['victim.txt',],
    },);
    return {
      victim: victim.attempt,
      // The bystander queues behind the victim, so both are accepted before the service runs again.
      bystander: async function queueBystander() {
        return (await startQueued({
          context,
          label: 'bystander',
          paths: ['bystander.txt',],
        },)).attempt;
      },
      expectations: [queuedExpectation([victim,],),],
    };
  }
  /**
   Tip of the branch before the victim starts.
   */
  const head = await readBranchTip(context.repository,);
  /**
   Victim that commits plainly.
   */
  const victim = await startAttempt({
    ...context,
    label: 'victim',
    paths: ['victim.txt',],
    mode: 'explicit',
  },);
  if (point === 'offset') {
    await sleep(planFaultOffset({
      random: context.random
        .fork('kill',),
      ...KILL_WINDOW,
    },),);
    return {
      victim,
      bystander: startBystander,
      expectations: [],
    };
  }
  /**
   Whether the branch moved before the victim settled; the service stays frozen until the caller resumes it.
   */
  const arrival = await killWhenLanded({
    context,
    victim,
    from: head,
  },);
  return {
    victim,
    bystander: startBystander,
    expectations: [{
      name: 'victim-reached-point',
      holds: arrival === 'moved',
      detail: `victim ${arrival} before its commit landed`,
    },],
  };
}

/**
 Expectation that the victim's commit is on the branch exactly as often as its kill point allows.

 @param context - scenario context

 @param outcome - what the point leaves

 @param victim - the finished victim

 @returns the expectation record

 @example
 ```ts
 await victimOutcome({ context, outcome: 'nothing', victim });
 ```
 */
async function victimOutcome({
  context,
  outcome,
  victim,
}: Readonly<{
  context: ScenarioContext;
  outcome: VictimOutcome;
  victim: AttemptRecord;
}>,): Promise<Expectations[number]> {
  /**
   Commits carrying the victim's token.
   */
  const landed = (await landedCommit({
    context,
    attempt: victim,
  },)).split('\n',)
    .filter(function nonEmpty(line,) {
      return line !== '';
    },);
  return {
    name: `victim-commit-${outcome}`,
    holds: outcome === 'either'
      ? landed.length <= 1
      : landed.length === (outcome === 'landed' ? 1 : 0),
    detail: `${String(landed.length,)} commits carry the victim's token`,
  };
}

/**
 Kills the victim at a point,
 lets a bystander finish,
 recovers,
 and commits again.

 @param context - scenario context

 @param point - where the victim is killed

 @returns expectations

 @example
 ```ts
 await killAndRecover({ context, point: 'editor-open' });
 ```
 */
async function killAndRecover({
  context,
  point,
}: Readonly<{
  context: ScenarioContext;
  point: KillPoint;
}>,): Promise<Expectations> {
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
        size: 800,
      },),
    },);
  },),);
  /**
   Victim at its kill point.
   */
  const {
    victim,
    bystander: startBystander,
    expectations,
  } = await reachPoint({
    context,
    point,
  },);
  /**
   Bystander started while the victim is at the point.
   */
  const bystander = await startBystander();
  // The victim's kill is already done at the points that freeze the service; the caller runs it again at the others.
  victim.kill();
  if ((point === 'queued') || (point === 'after-landing'))
    context.repository
      .service
      .resume();
  /**
   Settled victim and bystander.
   */
  const [victimRecord, bystanderRecord,] = await Promise.all([
    victim.finished,
    bystander.finished,
  ],);
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
      size: 400,
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
  return [
    ...expectations,
    allSucceeded({
      name: 'bystander-succeeds',
      attempts: [bystanderRecord,],
    },),
    allSucceeded({
      name: 'follow-up-commit-succeeds',
      attempts: [followUp,],
    },),
    await victimOutcome({
      context,
      outcome: OUTCOMES[point],
      victim: victimRecord,
    },),
    ...point === 'offset' ? [] : [{
      name: 'victim-was-killed',
      holds: victimRecord.killed,
      detail: `the victim exited ${String(victimRecord.outcome
        .exitCode,)} before the kill reached it`,
    },],
  ];
}

/**
 Builds the kill scenario for one point.

 @param point - where the victim is killed

 @param summary - one-line description for the report

 @returns scenario definition

 @example
 ```ts
 killScenario({ point: 'editor-written', summary: 'kill with the message final' });
 ```
 */
function killScenario({
  point,
  summary,
}: Readonly<{
  point: KillPoint;
  summary: string;
}>,): ScenarioDefinition {
  return {
    name: `sigkill-${point}`,
    group: 'concurrency',
    // Before the service owns the victim's commit it never lands, and the bystander never loses a race to it;
    // a seeded offset can kill during the exchange with the service after the bystander started.
    replayPlumbing: 'unused',
    summary,
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
      return await killAndRecover({
        context,
        point,
      },);
    },
  };
}

//endregion Workload

/**
 Client kill scenarios in the order a commit passes the points.
 */
export const SIGKILL_SCENARIOS: readonly ScenarioDefinition[] = [
  killScenario({
    point: 'editor-open',
    summary: 'SIGKILL one agent while its message editor is open, before anything reaches the service, while another commits, then recover',
  },),
  killScenario({
    point: 'editor-written',
    summary: 'SIGKILL one agent after its editor wrote the final message, before anything reaches the service, while another commits, then recover',
  },),
  killScenario({
    point: 'queued',
    summary: 'SIGKILL one agent whose prepared commit waits at a frozen service while another queues behind it, then recover',
  },),
  killScenario({
    point: 'after-landing',
    summary: 'SIGKILL one agent the instant its commit landed, while another commits, then recover',
  },),
  killScenario({
    point: 'offset',
    summary: 'SIGKILL one committing agent at a seeded offset while another commits, then recover',
  },),
];
