/**
 Pins an interleaving by freezing the scenario's commit service.

 A commit prepares in the wrapper (private index, policies) and only then connects to the service.
 While the service is frozen, a client that finished preparing sits connected and waits,
 so every client started under a freeze has prepared against the branch as it is,
 and none lands before the service runs again.
 The scenario then edits the worktree between two starts, as a second agent would,
 and resumes the service to let the queued commits land in the order they connected.

 @module
 */

import { waitForConnection, } from './client-connection-fixture.ts';
import type { AttemptMode, } from './ledger-fixture.ts';
import type { ScenarioContext, } from './scenario-model-fixture.ts';
import {
  type StartedAttempt,
  startAttempt,
} from './worker-fixture.ts';

//region Types

/**
 A commit started while the service was frozen.
 */
export type QueuedAttempt = Readonly<{
  /**
   The started commit.
   */
  attempt: StartedAttempt;
  /**
   Whether it connected to the service before settling,
   which proves its preparation ended while the service was frozen.
   */
  queued: boolean;
}>;

//endregion Types

//region Queue

/**
 Starts a commit and waits until it has prepared and connected to the frozen service.

 @param context - scenario context,
 whose service the caller froze

 @param label - unique label

 @param paths - explicit paths

 @param mode - selection mode

 @returns the started commit and whether it is queued

 @example
 ```ts
 context.repository.service.stall();
 const first = await startQueued({ context, label: 'first', paths: ['a.txt'] });
 ```
 */
export async function startQueued({
  context,
  label,
  paths,
  mode = 'explicit',
}: Readonly<{
  context: ScenarioContext;
  label: string;
  paths: readonly string[];
  mode?: Exclude<AttemptMode, 'foreign'>;
}>,): Promise<QueuedAttempt> {
  /**
   The started commit.
   */
  const attempt = await startAttempt({
    ...context,
    label,
    paths,
    mode,
  },);
  return {
    attempt,
    queued: (await waitForConnection(attempt.running,)) === 'connected',
  };
}

/**
 Expectation that every commit was queued behind the frozen service before it ran again.

 @param queued - commits started under the freeze

 @returns the expectation record

 @example
 ```ts
 queuedExpectation([first, second]);
 ```
 */
export function queuedExpectation(queued: readonly QueuedAttempt[],): Readonly<{
  name: string;
  holds: boolean;
  detail: string;
}> {
  /**
   Commits that settled or timed out before connecting.
   */
  const unqueued = queued.filter(function missed(entry,) {
    return !entry.queued;
  },);
  return {
    name: 'agents-queued-in-order',
    holds: unqueued.length === 0,
    detail: `${unqueued.map(function label(entry,) {
      return entry.attempt
        .token;
    },)
      .join(', ',)} did not connect to the frozen service`,
  };
}

//endregion Queue
