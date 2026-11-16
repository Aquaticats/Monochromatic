/**
 The session-file answers the stand-in Pi gives for the scenarios that are about a session file: one that names a file this run never
 made, one that names no file at all, and one whose statistics are empty.

 @module
 */

import type { PiRecord, } from './driver-types.ts';

/**
 The scenario in which the stand-in reports a session file the run was never given, standing for a misreporting or compromised peer.
 */
export const REPORTS_ANOTHER_SESSION_SCENARIO = 'reports-another-session';

/**
 What the stand-in answers for one command, when the scenario is about a session file.


 @param type - the command Pi was asked

 @param scenario - the scenario this stand-in was started with

 @param ordinary - what this stand-in answers that command with when no scenario says otherwise

 @param reported - the file the peer says it keeps

 @returns the answer, that this scenario says nothing about that command, or that it says nothing at all

 @example
 ```ts
 const answer = sessionAnswerOf({ type: 'get_state', scenario: 'reports-another-session', ordinary: {}, reported: '/runs/c/s.jsonl', },);
 ```
 */
export function sessionAnswerOf({
  type,
  scenario,
  ordinary,
  reported,
}: {
  readonly type: unknown;
  readonly scenario: unknown;
  readonly ordinary: PiRecord;
  readonly reported: string;
}): { readonly answer: PiRecord } | { readonly noData: true } | { readonly nothing: true } {
  /**
   The command, as the text Pi's own record holds.
   */
  const command = String(type,);
  /**
   The scenario, as the text the fixture was started with.
   */
  const chosen = String(scenario,);
  if ((command !== 'get_state') && (command !== 'get_session_stats')) {
    return { nothing: true, };
  }
  if (chosen === REPORTS_ANOTHER_SESSION_SCENARIO) {
    return { answer: {
      ...ordinary,
      sessionFile: reported,
    }, };
  }
  if ((chosen === 'statless') && (command === 'get_session_stats')) {
    // An answer that says nothing at all, which is a statistics answer with neither data nor an error.
    return { noData: true, };
  }
  if (chosen === 'sessionless') {
    /**
     Everything that answer holds, less the session file.
     */
    const {
      sessionFile: _unreported,
      ...rest
    } = ordinary;
    return { answer: rest, };
  }
  return { nothing: true, };
}
