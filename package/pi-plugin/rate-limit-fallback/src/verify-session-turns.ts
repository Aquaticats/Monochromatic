/**
 Long transcript turns and the compaction record both verification arms read.
 @module
 */

import type {
  AgentSession,
  SessionManager,
} from '@earendil-works/pi-coding-agent';

/**
 Phrases one long turn repeats, sized to the gate review's probe: about 34 thousand estimated tokens.
 */
const BIG_TURN_PHRASES = 5_000;

/**
 One long turn for a compaction check.
 */
export const BIG_TURN: string = 'lorem ipsum dolor sit amet '.repeat(BIG_TURN_PHRASES,);

/**
 Send the long transcript turns one after another.

 @param session - Pi session to prompt

 @param turns - how many long turns to send

 @example
 ```ts
 await sendLongTurns({ session, turns: 2 });
 ```
 */
export async function sendLongTurns({
  session,
  turns,
}: {
  readonly session: AgentSession;
  readonly turns: number;
},): Promise<void> {
  await Array.fromAsync(
    { length: turns, },
    async function sendTurn(
      _unused: unknown,
      index: number,
    ): Promise<void> {
      await session.prompt(`turn ${index + 1}: ${BIG_TURN}`,);
    },
  );
}

/**
 Count the compaction records a session branch carries.

 @param manager - the session's own manager

 @returns number of compaction entries

 @example
 ```ts
 compactionCount(sessionManager); // 0
 ```
 */
export function compactionCount(manager: SessionManager,): number {
  /**
   Compaction records seen on the branch so far.
   */
  let count = 0;
  for (const entry of manager.getBranch()) {
    if (entry.type === 'compaction')
      count += 1;
  }
  return count;
}
