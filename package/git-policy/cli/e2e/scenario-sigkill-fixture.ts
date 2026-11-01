/**
 `SIGKILL` injected at each observable transaction phase while another agent commits,
 followed by recovery through an ordinary wrapper invocation and a follow-up commit.

 Phases are hook events
 (`pre-commit`,
 `prepare-commit-msg`,
 `commit-msg`,
 `post-commit`)
 plus a seeded time offset that can land anywhere,
 including inside cli-git's landing.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';

import { synthesizeText, } from './content-fixture.ts';
import type { HookEvent, } from './hook-program-fixture.ts';
import { ALL_HOOK_EVENTS, } from './scenario-setup-fixture.ts';
import {
  repositoryOptions,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import { planFaultOffset, } from './seeded-schedule-fixture.ts';
import {
  holdAt,
  reached,
} from './barrier-fixture.ts';
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

//endregion Constants

//region Workload

/**
 Kills the victim at a phase,
 lets a bystander finish,
 recovers,
 and commits again.

 @param context - scenario context

 @param phase - hook event to kill in, or `offset` for a seeded delay

 @returns expectations

 @example
 ```ts
 await killAndRecover({ context, phase: 'pre-commit' });
 ```
 */
async function killAndRecover({
  context,
  phase,
}: Readonly<{
  context: ScenarioContext;
  phase: HookEvent | 'offset';
}>,): Promise<Awaited<ReturnType<ScenarioDefinition['run']>>> {
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
   Victim attempt.
   */
  const victim = await startAttempt({
    ...context,
    label: 'victim',
    paths: ['victim.txt',],
    mode: 'explicit',
  },);
  if (phase === 'offset')
    await sleep(planFaultOffset({
      random: context.random
        .fork('kill',),
      ...KILL_WINDOW,
    },),);
  else {
    await holdAt({
      repository: context.repository,
      token: victim.token,
      event: phase,
    },);
    await reached({
      repository: context.repository,
      token: victim.token,
      running: victim.running,
      event: phase,
    },);
  }
  /**
   Bystander started while the victim is at its phase.
   */
  const bystander: StartedAttempt = await startAttempt({
    ...context,
    label: 'bystander',
    paths: ['bystander.txt',],
    mode: 'explicit',
  },);
  victim.kill();
  /**
   Settled victim and bystander.
   */
  const [, bystanderRecord,] = await Promise.all([
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
    allSucceeded({
      name: 'bystander-succeeds',
      attempts: [bystanderRecord,],
    },),
    allSucceeded({
      name: 'follow-up-commit-succeeds',
      attempts: [followUp,],
    },),
  ];
}

/**
 Builds the kill scenario for one phase.

 @param phase - hook event or `offset`

 @returns scenario definition

 @example
 ```ts
 killScenario('commit-msg');
 ```
 */
function killScenario(phase: HookEvent | 'offset',): ScenarioDefinition {
  return {
    name: `sigkill-${phase}`,
    group: 'concurrency',
    // A hook-phase victim is killed before landing, or after it in post-commit before the bystander starts,
    // so the bystander never loses a race; a seeded offset can kill inside landing after the bystander started.
    replayPlumbing: phase === 'offset' ? 'required' : 'unused',
    // Config hooks need Git 2.54.0.
    minimumGit: '2.54.0',
    summary: phase === 'offset'
      ? 'SIGKILL one committing agent at a seeded offset while another commits, then recover'
      : `SIGKILL one committing agent inside ${phase} while another commits, then recover`,
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
        hooks: 'config',
        hookEvents: ALL_HOOK_EVENTS,
      },);
    },
    async run(context,) {
      return await killAndRecover({
        context,
        phase,
      },);
    },
  };
}

//endregion Workload

/**
 Kill scenarios in phase order.
 */
export const SIGKILL_SCENARIOS: readonly ScenarioDefinition[] = [
  ...ALL_HOOK_EVENTS.map(function phaseScenario(event,) {
    return killScenario(event,);
  },),
  killScenario('offset',),
];
