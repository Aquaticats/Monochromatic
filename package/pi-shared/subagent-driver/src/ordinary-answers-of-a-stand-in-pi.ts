/**
 What the stand-in Pi answers its ordinary commands with, so a run's state, its command names and its statistics all have shapes to
 read.

 @module
 */

import type { PiRecord, } from './driver-types.ts';

/**
 What Pi answers each ordinary command with.

 @param sessionFile - the session file this stand-in keeps, which its own answers name

 @returns the answers, by command

 @example
 ```ts
 const answers = ordinaryAnswersOf({ sessionFile: '/runs/b-2/stand-in-session-a1b2.jsonl', },);
 ```
 */
export function ordinaryAnswersOf({ sessionFile, }: { readonly sessionFile: string },): ReadonlyMap<string, PiRecord> {
  return new Map<string, PiRecord>([
    [
      'get_state',
      {
        model: { id: 'stand-in-model', },
        thinkingLevel: 'xhigh',
        // Pi's own `get_state` answer names the session file it keeps, which is where this run's masking finds it at rest.
        sessionFile,
      },
    ],
    [
      'get_commands',
      { commands: [
        { name: 'bash', },
        { name: 'edit', },
      ], },
    ],
    [
      'get_session_stats',
      {
        sessionFile,
        cost: 0.5,
      },
    ],
  ],);
}
