/**
 Putting a run away after it ended: asking Pi for its own statistics, aborting a Pi still at work, ending it, and closing the
 feed within their own bounds.

 @module
 */

import type { ConnectionToPi, } from './connection-to-pi.ts';
import {
  SHUTDOWN_GRACE_MILLISECONDS,
  SHUTDOWN_STAGES,
  TEARDOWN_ANSWER_MILLISECONDS,
  TEARDOWN_MILLISECONDS,
} from './driver-constants.ts';
import { millisecondsLeftUntil, } from './deadline.ts';
import type {
  CommandAnswer,
  RecordOfPiRun,
} from './driver-types.ts';

/**
 What putting a run away leaves for the summary.
 */
export type RunPutAway = {
  /**
   Pi's own session statistics, or what Pi answered instead of them, or why it answered nothing.
   */
  readonly sessionStats: unknown;

  /**
   Whether Pi's leaving could be confirmed.
   */
  readonly stopped: boolean;
};

/**
 What asking Pi for its statistics left: Pi's own data, Pi's refusal with its text, or the driver's bound when Pi answered
 nothing.

 @param answer - how waiting for the statistics ended

 @returns what the summary carries as the session's statistics
 */
function sessionStatsOf(answer: CommandAnswer,): unknown {
  if (answer.ending !== 'answered') {
    return answer.problem;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const {
    data,
    error,
  } = answer.record;
  if (data !== undefined) {
    return data;
  }
  return error ?? null;
}

/**
 Puts a run away once it is over: the statistics are asked for while Pi can still answer, a Pi still at work is aborted, and Pi
 is ended within its own bounds. Every stage shares what the teardown has left, so one slow stage cannot spend another's, and the
 whole of it is bounded even when Pi answers nothing.

 @param connection - the running Pi

 @param record - the run's record, which the statistics and any problem are noted on

 @returns the statistics and whether Pi's leaving could be confirmed

 @example
 ```ts
 const putAway = await putTheRunAway({ connection, record, },);
 ```
 */
export async function putTheRunAway(
  {
    connection,
    record,
  }: {
    readonly connection: ConnectionToPi;
    readonly record: RecordOfPiRun
  },
): Promise<RunPutAway> {
  /**
   When putting the run away must be over, counted from now: the wall clock bounds the run, and this bounds everything after it.
   */
  const teardownDeadline = Date.now() + TEARDOWN_MILLISECONDS;

  /**
   How long one stage of putting the run away may take: its own bound, or what is left of the teardown's.

   @param stageMilliseconds - the stage's own bound

   @returns the milliseconds this stage may take
   */
  function withinTeardown(stageMilliseconds: number,): number {
    return Math.min(
      stageMilliseconds,
      millisecondsLeftUntil(teardownDeadline,),
    );
  }

  /**
   Pi's own statistics, asked for while it can still answer.
   */
  const stats = await connection.send({
    command: { type: 'get_session_stats', },
    answerMilliseconds: withinTeardown(TEARDOWN_ANSWER_MILLISECONDS,),
    deadline: teardownDeadline,
  },);
  if (record.isRunning()) {
    await connection.send({
      command: { type: 'abort', },
      answerMilliseconds: withinTeardown(TEARDOWN_ANSWER_MILLISECONDS,),
      deadline: teardownDeadline,
    },);
  }
  /**
   How Pi's ending went: every stage of it shares what the teardown has left, so one stage cannot spend another's.
   */
  const shutDown = await connection.shutDown({
    graceMilliseconds: Math.max(
      1,
      Math.floor(withinTeardown(SHUTDOWN_GRACE_MILLISECONDS * SHUTDOWN_STAGES,) / SHUTDOWN_STAGES),
    ),
  },);
  if (!shutDown.stopped) {
    record.note({
      kind: 'problem',
      problem: shutDown.reason,
    },);
  }
  return {
    sessionStats: sessionStatsOf(stats,),
    stopped: shutDown.stopped,
  };
}
