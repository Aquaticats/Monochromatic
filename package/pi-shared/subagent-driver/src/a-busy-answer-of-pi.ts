/**
 Pi's answer to a prompt that met an agent already in a turn: the text it answers with, and the read that tells that answer from a
 refusal of the message itself.

 Pi 1.5.2 answers a prompt with this error while its agent streams and the prompt named no streaming behaviour, and queues the
 prompt into that turn when one is named (`prompt()` in `dist/core/agent-session.js` of `@earendil-works/pi-coding-agent`). The
 text says the agent is busy, never that the message is unacceptable, so a caller that reads it as a refusal ends runs nothing
 refused.

 @module
 */

import type { CommandAnswer, } from './driver-types.ts';

/**
 The start of the error Pi 1.5.2 answers a prompt with when its agent is already in a turn and the prompt named no streaming
 behaviour: "Agent is already processing. Specify streamingBehavior ('steer' or 'followUp') to queue the message." It says the
 agent is busy, never that the message is unacceptable.
 */
const AGENT_ALREADY_PROCESSING = 'Agent is already processing';

/**
 Tells whether Pi answered that its agent is already in a turn of its own, which refuses this message for no reason of its own.

 @param answer - how waiting for Pi's answer ended

 @returns whether Pi said its agent was already processing

 @example
 ```ts
 const busy = { ending: 'answered', record: { success: false, error: 'Agent is already processing' }, } as const;
 const awake = foundTheAgentAlreadyAwake({ answer: busy, },);
 ```
 */
export function foundTheAgentAlreadyAwake({ answer, }: { readonly answer: CommandAnswer },): boolean {
  if (answer.ending !== 'answered') {
    return false;
  }
  /**
   Pi's own record, which is what an answered wait carries.
   */
  const { record } = answer;
  if (record.success !== false) {
    return false;
  }
  /**
   What Pi said of the message.
   */
  const { error, } = record;
  return ((typeof error) === 'string') && error.startsWith(AGENT_ALREADY_PROCESSING,);
}
