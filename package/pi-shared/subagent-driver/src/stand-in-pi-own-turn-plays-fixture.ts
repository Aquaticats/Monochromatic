/**
 What the stand-in Pi does when the work a run left running ends: it starts a turn of its own to read the result, as Pi does when
 a tool it ran in the background reports, and answers the message the driver sends at that same moment the way Pi answers a prompt
 that meets a running turn.

 Pi 1.5.2 refuses such a prompt when the client named no streaming behaviour, and queues it into the running turn when the client
 named one (`prompt()` in `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent`). The turn's own reply is the run's
 last answer either way, so a driver that reads the refusal as the run's end loses that reply and cuts the turn off.

 The turn starts while the message is being taken, not on the work's own exit, because the driver asks what the run left running
 only when its view of Pi says Pi rests: a turn it had already seen would keep the run waiting instead of waking the agent. That is
 the race the run of 2026-11-19 lost, where Pi wrote `turn_start` 33 ms before the driver sent its wake.

 @module
 */

import type { PiRecord, } from '../dist/final/node/index.mjs';
import {
  seen,
  writeLogLine,
} from './stand-in-pi-log-fixture.ts';
import {
  answer,
  assistantReply,
  emit,
  finishRun,
} from './stand-in-pi-records-fixture.ts';
import {
  WORK_MILLISECONDS,
  startWorkTheToolReports,
} from './stand-in-pi-work-plays-fixture.ts';

/**
 The scenario whose agent starts a turn of its own when the work it left running ends.
 */
export const WAKES_ITSELF_SCENARIO = 'wakes-itself-when-the-work-ends';

/**
 What Pi 1.5.2 answers a prompt with while its agent is streaming and the prompt named no streaming behaviour, word for word.
 */
export const ALREADY_PROCESSING_TEXT: string = 'Agent is already processing. Specify streamingBehavior (\'steer\' or \'followUp\') to queue the message.';

/**
 The reply of the turn the agent started by itself, which the run must end with.
 */
export const OWN_TURN_REPLY = 'the build ended while I was reading it, and the task goes on';

/**
 Whether the agent's own turn is open, which is the state a wake that meets it finds.
 */
const ownTurn: { open: boolean } = { open: false, };

/**
 Starts the agent's turn of its own, as Pi does when a tool reports the end of work it ran in the background.
 */
function startOwnTurn(): void {
  if (ownTurn.open) {
    return;
  }
  ownTurn.open = true;
  emit({ type: 'agent_start', },);
  emit({ type: 'turn_start', },);
  void writeLogLine('marker own-turn-started',);
}

/**
 Ends the agent's own turn, once a message has been queued into it.

 A turn whose message was refused is left open: a driver that reads the refusal as the run's end stops Pi before any turn could
 end, which is what the incident of 2026-11-19 recorded as an aborted assistant message.
 */
function endOwnTurn(): void {
  ownTurn.open = false;
  emit(assistantReply(OWN_TURN_REPLY,),);
  emit({ type: 'agent_end', },);
  emit({ type: 'agent_settled', },);
}

/**
 Answers a command with Pi's refusal, in the shape `prompt()` answers one.

 @param id - command's identifier

 @param error - what Pi refused it with
 */
function answerRefusal({
  id,
  error,
}: {
  readonly id: unknown;
  readonly error: string
},): void {
  emit({
    type: 'response',
    id,
    success: false,
    error,
  },);
}

/**
 Answers the message the driver sends when the work ends, as Pi answers a prompt that meets a turn already running.

 Pi takes commands in the order they arrive, so a turn that began as the work ended is what this message meets. Starting it here
 is how the stand-in reproduces that ordering: the driver had already sent the message when this turn opened.

 @param command - the prompt or steer Pi was sent
 */
async function answerTheWake(command: PiRecord,): Promise<void> {
  startOwnTurn();
  await writeLogLine('marker wake-met-an-open-turn',);
  if (command.streamingBehavior === undefined) {
    answerRefusal({
      id: command.id,
      error: ALREADY_PROCESSING_TEXT,
    },);
    return;
  }
  /**
   How the message named the queueing, as Pi's protocol writes it, or that it named something this stand-in does not know.
   */
  const named = ((typeof command.streamingBehavior) === 'string')
    ? command.streamingBehavior
    : 'a streaming behaviour that is not text';
  answer({
    id: command.id,
    data: { disposition: 'queued', },
  },);
  await writeLogLine(`marker wake-queued ${named}`,);
  endOwnTurn();
}

/**
 Plays the first prompt as a turn that leaves work running, and the message the driver sends when that work ends as one that
 meets the turn the agent started by itself.

 @param command - the prompt or steer Pi was sent

 @example
 ```ts
 await leaveWorkRunningAndWakeItself({ id: 'c1', },);
 ```
 */
async function leaveWorkRunningAndWakeItself(command: PiRecord,): Promise<void> {
  if (seen.prompts > 1) {
    await answerTheWake(command,);
    return;
  }
  answer({ id: command.id, },);
  await startWorkTheToolReports({ milliseconds: WORK_MILLISECONDS, },);
  finishRun('started the build and ended my turn',);
}

/**
 What this scenario does with a prompt.

 @param command - the prompt or steer Pi was sent
 */
export const ownTurnPlays: ReadonlyMap<string, (command: PiRecord,) => Promise<void>> = new Map([
  [
    WAKES_ITSELF_SCENARIO,
    leaveWorkRunningAndWakeItself,
  ],
],);
