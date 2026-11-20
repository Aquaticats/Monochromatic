/**
 Pi's answers to a prompt it will not take yet: the texts it answers with, and the reads that tell an answer about the moment from
 a refusal of the message itself.

 Pi 1.5.2 answers a prompt with this error while its agent streams and the prompt named no streaming behaviour, and queues the
 prompt into that turn when one is named (`prompt()` in `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent`). The
 text says the agent is busy, never that the message is unacceptable, so a caller that reads it as a refusal ends runs nothing
 refused.

 `@earendil-works/pi-agent-core` throws errors that open with the same three words and mean something else: a prompt already being
 processed, or a reset or a continuation that met a running agent (`agent.js` in that package, "Agent is already processing a
 prompt. Use steer() or followUp() to queue messages, or wait for completion."). None of them names a streaming behaviour, and
 none of them says a turn of the agent's own took this message, so they stay refusals.

 A session that is compacting or summarising refuses a prompt before it prepares it, which refuses the moment and never the
 message.

 The wake (`kept-while-work-runs.ts`), the model-error retry (`answer-a-model-error.ts`) and each inbox delivery
 (`watch-pi-run.ts`) read Pi's answer here. The first two read only the busy answer, because a session that compacts starts no
 turn for them to wait for.

 @module
 */

import type { CommandAnswer, } from './driver-types.ts';

/**
 The start of the error Pi 1.5.2 answers a prompt with when its agent is already in a turn and the prompt named no streaming
 behaviour (`prompt()` in `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent`): "Agent is already processing.
 Specify streamingBehavior ('steer' or 'followUp') to queue the message." It says the agent is busy, never that the message is
 unacceptable.

 The continuation that names `streamingBehavior` is what tells that answer from the agent library's own error, which opens with
 the same words and names neither a queue nor a turn (`agent.js` of `@earendil-works/pi-agent-core`, "Agent is already processing
 a prompt. Use steer() or followUp() to queue messages, or wait for completion."). Only the session's answer says a turn of the
 agent's own took the message, so only that one is read as the agent being awake.
 */
const AGENT_ALREADY_PROCESSING = 'Agent is already processing. Specify streamingBehavior';

/**
 The start of the error Pi 1.5.2 throws for a prompt that arrives while a manual compaction or a branch summary runs (`prompt()` in
 `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent`): "Cannot submit a prompt while compaction is in progress.
 Wait for compaction to finish and retry." The guard is read before the prompt is prepared, so a named streaming behaviour does
 not queue the message either; Pi's automatic compaction uses another controller and passes it.
 */
const A_COMPACTION_IN_PROGRESS = 'Cannot submit a prompt while compaction is in progress';

/**
 The text Pi refused the prompt with, which is empty text when it refused nothing.

 @param answer - how waiting for Pi's answer ended

 @returns Pi's own text, or empty text when the answer is not a refusal
 */
function refusalTextOf({ answer, }: { readonly answer: CommandAnswer },): string {
  if (answer.ending !== 'answered') {
    return '';
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  if (record.success !== false) {
    return '';
  }
  /**
   What Pi said of the message.
   */
  const { error, } = record;
  return ((typeof error) === 'string') ? error : '';
}

/**
 Tells whether Pi answered that its agent is already in a turn of its own, which refuses this message for no reason of its own.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi said its agent was already processing

 @example
 ```ts
 const busy = {
   ending: 'answered',
   record: {
     success: false,
     error: "Agent is already processing. Specify streamingBehavior ('steer' or 'followUp')"
       + ' to queue the message.',
   },
 } as const;
 const awake = foundTheAgentAlreadyAwake({ answer: busy, },);
 ```
 */
export function foundTheAgentAlreadyAwake({ answer, }: { readonly answer: CommandAnswer },): boolean {
  /**
   The text Pi refused the prompt with.
   */
  const refused = refusalTextOf({ answer, },);
  return refused.startsWith(AGENT_ALREADY_PROCESSING,);
}

/**
 Tells whether Pi answered that it cannot take a prompt yet, though nothing is wrong with the message: its agent is in a turn of
 its own, or its session is being compacted or summarised.

 Each inbox delivery reads this, because an answer here leaves the message for a later look rather than keeping it aside as one Pi
 refused. The wake and the retry read {@link foundTheAgentAlreadyAwake} alone, because a compaction starts no turn of Pi's own
 for them to wait for.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi said it could not take the prompt now

 @example
 ```ts
 const compacting = {
   ending: 'answered',
   record: {
     success: false,
     error: 'Cannot submit a prompt while compaction is in progress. Wait for compaction to finish and retry.',
   },
 } as const;
 const later = foundPiRefusedTheMoment({ answer: compacting, },);
 ```
 */
export function foundPiRefusedTheMoment({ answer, }: { readonly answer: CommandAnswer },): boolean {
  if (foundTheAgentAlreadyAwake({ answer, },)) {
    return true;
  }
  /**
   The text Pi refused the prompt with.
   */
  const refused = refusalTextOf({ answer, },);
  return refused.startsWith(A_COMPACTION_IN_PROGRESS,);
}
