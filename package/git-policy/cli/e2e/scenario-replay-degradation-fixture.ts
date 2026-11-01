/**
 Replay degradation on a Git without replay plumbing
 (`git merge-tree --merge-base`, Git 2.40.0):
 a commit that loses a landing race fails fast with `concurrent-commit/head-moved` and exit `1`,
 never `transaction-failed`,
 and the invariant checker proves nothing was corrupted or lost.
 These scenarios run only on such a Git
 (`SPEC.md` "Compatibility and degradation").

 @module
 */

import { synthesizeText, } from './content-fixture.ts';
import {
  reached,
  releaseAt,
} from './barrier-fixture.ts';
import { extractPolicyEvents, } from './jsonl-event-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import {
  disjointPlans,
  repositoryOptions,
  runDisjointWorkers,
  seedTexts,
  workerPaths,
} from './scenario-helper-fixture.ts';
import type {
  Expectations,
  ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Finding a commit that cannot replay reports.
 */
const HEAD_MOVED = 'concurrent-commit/head-moved';

/**
 Worker count range for the burst scenario.
 */
const WORKER_RANGE = {
  min: 4,
  max: 8,
} as const;

/**
 Largest worker start offset for the burst scenario.
 */
const MAX_JITTER_MS = 40;

//endregion Constants

//region Helpers

/**
 Whether an attempt failed exactly as a commit that cannot replay must:
 exit `1` with one `head-moved` core finding and nothing else.

 @param attempt - finished attempt

 @returns whether the failure is the fail-fast finding
 */
function failedFast(attempt: AttemptRecord,): boolean {
  /**
   Event types and codes in order.
   */
  const events = extractPolicyEvents(attempt.outcome
    .stderr,)
    .events
    .map(function describe(event,) {
      return `${event.type}:${event.code ?? ''}`;
    },);
  return (attempt.outcome
    .exitCode
    === 1) && (events.length === 1)
    && (events[0] === `core-finding:${HEAD_MOVED}`);
}

/**
 Expectation that every attempt landed or failed fast.

 @param attempts - finished attempts

 @returns expectation record
 */
function landedOrFailedFast(attempts: readonly AttemptRecord[],): Expectations[number] {
  /**
   Attempts that ended any other way.
   */
  const other = attempts.filter(function unexpected(attempt,) {
    return (attempt.outcome
      .exitCode
      !== 0) && (!failedFast(attempt,));
  },);
  return {
    name: 'landed-or-failed-fast',
    holds: other.length === 0,
    detail: other.map(function describe(attempt,) {
      return `${attempt.label} exited ${String(attempt.outcome
        .exitCode,)}: ${attempt.outcome
          .stderr
          .trim()
          .split('\n',)
          .slice(-2,)
          .join(' | ',)}`;
    },)
      .join('; ',),
  };
}

//endregion Helpers

//region Scenarios

/**
 One commit held in its editor while another lands, so it must replay.
 */
const failsFast: ScenarioDefinition = {
  name: 'replay-unavailable-fails-fast',
  group: 'concurrency',
  summary: 'a commit that loses the landing race on a Git without merge-tree --merge-base fails fast with head-moved',
  replayPlumbing: 'absent',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: [
        'held.txt',
        'winner.txt',
      ],
    },), },);
  },
  async run(context,) {
    await Promise.all([
      'held.txt',
      'winner.txt',
    ].map(async function rewrite(path,) {
      await writeWorktree({
        ...context,
        path,
        bytes: synthesizeText({
          random: context.random
            .fork(path,),
          size: 400,
        },),
      },);
    },),);
    /**
     Commit held in its editor until the winner landed.
     */
    const held = await startAttempt({
      ...context,
      label: 'held',
      paths: ['held.txt',],
      mode: 'explicit',
      extraArgs: ['-e',],
      env: { GIT_EDITOR: context.repository
        .editorProgram, },
    },);
    /**
     Whether the held commit reached its editor.
     */
    const inEditor = await reached({
      repository: context.repository,
      token: held.token,
      running: held.running,
      event: 'editor',
    },);
    /**
     Winner, landing while the held commit waits.
     */
    const winner = await (await startAttempt({
      ...context,
      label: 'winner',
      paths: ['winner.txt',],
      mode: 'explicit',
    },)).finished;
    /**
     Branch after the winner landed.
     */
    const headAfterWinner = (await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'HEAD',
      ],
    },)).trim();
    await releaseAt({
      repository: context.repository,
      token: held.token,
      event: 'editor',
    },);
    /**
     Settled held commit.
     */
    const heldRecord = await held.finished;
    /**
     Branch after the held commit settled.
     */
    const headAfterHeld = (await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'HEAD',
      ],
    },)).trim();
    return [
      {
        name: 'held-reached-editor',
        holds: inEditor === 'marker',
        detail: `the held commit's editor barrier ended with ${inEditor}`,
      },
      {
        name: 'winner-lands',
        holds: winner.outcome
          .exitCode
          === 0,
        detail: `winner exited ${String(winner.outcome
          .exitCode,)}: ${winner.outcome
            .stderr
            .trim()}`,
      },
      {
        name: 'held-fails-fast-with-head-moved',
        holds: failedFast(heldRecord,),
        detail: `held exited ${String(heldRecord.outcome
          .exitCode,)}: ${heldRecord.outcome
            .stderr
            .trim()}`,
      },
      {
        name: 'held-lands-nothing',
        holds: headAfterHeld === headAfterWinner,
        detail: `HEAD moved from ${headAfterWinner} to ${headAfterHeld}`,
      },
    ];
  },
};

/**
 Disjoint concurrent commits: every one lands or fails fast.
 */
const burst: ScenarioDefinition = {
  name: 'replay-unavailable-disjoint-burst',
  group: 'concurrency',
  summary: '4 to 8 agents commit disjoint paths on a Git without merge-tree --merge-base; each lands or fails fast',
  replayPlumbing: 'absent',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: workerPaths({
        prefix: 'w',
        count: WORKER_RANGE.max,
      },),
    },), },);
  },
  async run(context,) {
    /**
     Finished attempts.
     */
    const attempts = await runDisjointWorkers({
      context,
      plans: disjointPlans(context.random
        .integer(WORKER_RANGE,),),
      maxJitterMs: MAX_JITTER_MS,
    },);
    return [
      landedOrFailedFast(attempts,),
      {
        name: 'some-commit-lands',
        holds: attempts.some(function landed(attempt,) {
          return attempt.outcome
            .exitCode
            === 0;
        },),
        detail: 'no commit landed',
      },
    ];
  },
};

//endregion Scenarios

/**
 Replay degradation scenarios.
 */
export const REPLAY_DEGRADATION_SCENARIOS: readonly ScenarioDefinition[] = [
  failsFast,
  burst,
];
