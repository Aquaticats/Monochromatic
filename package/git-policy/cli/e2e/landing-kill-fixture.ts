/**
 Kill of a committing agent at the instant its commit landed.

 The wrapper learns that the commit landed from the service, then writes the push record and tells the service about it,
 and the service answers only when it runs.
 The harness therefore freezes the service the moment the branch moves:
 the agent is still alive, blocked in its exchange with the service,
 and the kill reaches it there instead of racing its exit.

 @module
 */

import type { ScenarioContext, } from './scenario-model-fixture.ts';
import type { StartedAttempt, } from './worker-fixture.ts';
import { waitForBranchMove, } from './work-record-fixture.ts';

/**
 Waits for the branch to move, freezes the service, and kills the agent.
 The caller resumes the service.

 @param context - scenario context

 @param victim - the agent to kill

 @param from - the tip before the agent started

 @returns what the wait for the landing came to

 @example
 ```ts
 const arrival = await killWhenLanded({ context, victim, from: head });
 context.repository.service.resume();
 ```
 */
export async function killWhenLanded({
  context,
  victim,
  from,
}: Readonly<{
  context: ScenarioContext;
  victim: StartedAttempt;
  from: string;
}>,): Promise<'moved' | 'settled' | 'timeout'> {
  /**
   Whether the branch moved before the agent settled.
   */
  const arrival = await waitForBranchMove({
    repository: context.repository,
    from,
    running: victim.running,
  },);
  context.repository
    .service
    .stall();
  victim.kill();
  return arrival;
}
