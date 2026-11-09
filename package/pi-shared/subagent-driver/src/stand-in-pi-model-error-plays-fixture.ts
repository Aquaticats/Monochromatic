/**
 What the stand-in Pi does with a prompt or a steer in a scenario about a failed attempt: the model error texts, each attempt
 after one, the recovery a reply makes, and the prompts Pi handles without running. The plays for the other scenarios, and the
 dispatcher that chooses between them, are beside this file.

 @module
 */

import { exit, } from 'node:process';
import { setTimeout as wait, } from 'node:timers/promises';
import { POLL_MILLISECONDS, } from '../dist/final/node/index.mjs';
import { seen, } from './stand-in-pi-log-fixture.ts';
import {
  answer,
  emit,
  emitErrorWithoutSettling,
  finishRun,
  finishRunInError,
  LONG_WAIT_ERROR_TEXT,
  MODEL_ERROR_TEXT,
  modelErrorPlay,
  PERMANENT_ERROR_TEXT,
  RATE_LIMIT_ERROR_TEXT,
  RECOVERED_REPLY,
  TRANSIENT_ERROR_TEXT,
} from './stand-in-pi-records-fixture.ts';

/**
 Status a scenario exits with when it leaves on its own terms.
 */
const CLEAN_EXIT_STATUS = 0;

/**
 How many looks at the run Pi's leaving leaves room for, so a retry the driver scheduled is on its feed before Pi's exit is seen.
 */
const LOOKS_BEFORE_LEAVING = 3;

/**
 Milliseconds after the failed attempt at which Pi leaves, long enough for the driver to have scheduled the retry it owes and
 short enough that the wait is nowhere near the retry's own delay.
 */
const LEAVE_AFTER_FAILURE_MILLISECONDS = POLL_MILLISECONDS * LOOKS_BEFORE_LEAVING;

/**
 How many prompts Pi answers with a failed run before it is given a prompt that starts a run of its own.
 */
const FAILING_ATTEMPTS_BEFORE_A_LONG_RUN = 2;

/**
 Which prompt starts a run of its own, above the failing attempts.
 */
const PROMPT_THAT_STARTS_A_LONG_RUN = FAILING_ATTEMPTS_BEFORE_A_LONG_RUN + 1;

/**
 Leaves this process after a delay, so a test can watch what the driver does in the wait before it goes.

 @param milliseconds - how long to stay
 */
async function leaveAfter(milliseconds: number,): Promise<void> {
  await wait(milliseconds,);
  exit(CLEAN_EXIT_STATUS,);
}

/**
 What each scenario about a failed attempt does with a prompt or a steer. A scenario missing from every map finishes a normal run.
 */
export const modelErrorPlays: ReadonlyMap<string, (id: unknown,) => void> = new Map<string, (id: unknown,) => void>([
  [
    'model-error',
    function endInError(id,) {
      answer({ id, },);
      finishRunInError(MODEL_ERROR_TEXT,);
    },
  ],
  [
    'transient-error-then-success',
    modelErrorPlay({
      error: TRANSIENT_ERROR_TEXT,
      recovers: true,
    }),
  ],
  [
    'always-transient-error',
    modelErrorPlay({
      error: TRANSIENT_ERROR_TEXT,
      recovers: false,
    }),
  ],
  [
    'permanent-error',
    modelErrorPlay({
      error: PERMANENT_ERROR_TEXT,
      recovers: false,
    }),
  ],
  [
    'long-wait-error',
    modelErrorPlay({
      error: LONG_WAIT_ERROR_TEXT,
      recovers: false,
    }),
  ],
  [
    'rate-limit-then-success',
    modelErrorPlay({
      error: RATE_LIMIT_ERROR_TEXT,
      recovers: true,
    }),
  ],
  [
    'handled-retry',
    function handleTheRetry(id,) {
      // The first attempt fails, and every retry after it is a prompt Pi consumes without starting a run or replying.
      if (seen.prompts === 1) {
        answer({ id, },);
        finishRunInError(TRANSIENT_ERROR_TEXT,);
        return;
      }
      emit({
        type: 'response',
        id,
        success: true,
        data: { disposition: 'handled', },
      },);
    },
  ],
  [
    'busy-after-inbox',
    function startARunAndRefuseTheNextPrompt(id,) {
      if (seen.prompts <= FAILING_ATTEMPTS_BEFORE_A_LONG_RUN) {
        answer({ id, },);
        finishRunInError(TRANSIENT_ERROR_TEXT,);
        return;
      }
      if (seen.prompts === PROMPT_THAT_STARTS_A_LONG_RUN) {
        // What a message from the inbox starts: a run that produces no assistant message and does not settle.
        answer({ id, },);
        emit({ type: 'agent_start', },);
        return;
      }
      emit({
        type: 'response',
        id,
        success: false,
        error: 'a run is already going',
      },);
    },
  ],
  [
    'exits-after-error',
    function endInErrorThenLeave(id,) {
      answer({ id, },);
      finishRunInError(TRANSIENT_ERROR_TEXT,);
      void leaveAfter(LEAVE_AFTER_FAILURE_MILLISECONDS,);
    },
  ],
  [
    'recovered-error',
    function retryAfterError(id,) {
      answer({ id, },);
      // The retry answers the error before Pi settles on it, which is what a recovering retry is: a settle in between would
      // leave the run's failure standing until the retry's reply cleared it, and a watch looking in that gap would end the run.
      emitErrorWithoutSettling(MODEL_ERROR_TEXT,);
      emit({ type: 'auto_retry_start', },);
      finishRun(RECOVERED_REPLY,);
    },
  ],
],);
