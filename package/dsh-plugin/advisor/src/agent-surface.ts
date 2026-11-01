/**
 Projection from the live `Agent` to the structural surface the review
 pipeline reads. @module
 */

import {
  assembleContextFor,
  type Agent,
} from '@deepseek-ai/dsh-agent';
import type {
  AssembleContext,
} from '@deepseek-ai/dsh-system-prompt';
import type {
  WorkingDirectoryService,
} from '@deepseek-ai/dsh-working-directory';
import { appendAdvisorOperationRecord, } from './session-event.ts';
import type {
  DshAdvisorAgentSurface,
} from './types.ts';

//region Public API

/**
 Project one live agent onto the structural review surface.

 @param agent - live agent carrying the session and route identity

 @param workingDirectory - service resolving the session's committed
 directory

 @returns surface carrying the session, directory, route identity, and
 failure-accounting sink

 @example
 ```ts
 const agent = toDshAdvisorAgentSurface({ agent: exec.agent, workingDirectory: ctx.workingDirectory });
 ```
 */
export function toDshAdvisorAgentSurface(
  {
    agent,
    workingDirectory,
  }: {
    readonly agent: Agent;
    readonly workingDirectory: Pick<WorkingDirectoryService, 'get'>;
  },
): DshAdvisorAgentSurface {
  return {
    sessionId: agent.id,
    session: agent.session,
    cwd: workingDirectory
      .get(agent.session,),
    options: {
      ...(agent.options
        .provider
        === undefined
        ? {}
        : { provider: agent.options
          .provider, }),
      ...(agent.options
        .model
        === undefined
        ? {}
        : { model: agent.options
          .model, }),
    },
    whenIdle: async function whenIdle(): Promise<void> {
      await agent
        .whenIdle();
    },
    appendOperationRecord: function appendOperationRecord(operation): void {
      appendAdvisorOperationRecord({
        session: agent.session,
        operation,
      },);
    },
    assembleContext: function assembleContext(signal?: AbortSignal): AssembleContext {
      return assembleContextFor(
        agent,
        signal
      );
    },
  };
}

//endregion Public API
