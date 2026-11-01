/**
 Preserve failed-operation usage across Pi's thrown-tool-error boundary. @module
 */
import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  ADVISOR_OPERATION_TYPE,
  ADVISOR_TOOL_NAME,
  type AdvisorOperationSnapshot,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Failed tool operation awaiting Pi's tool-result finalization hook.
 */
export type AdvisorFailureRecord = {
  /**
   Host tool-call identity.
   */
  readonly toolCallId: string;
  /**
   Locally finalized operation and available usage.
   */
  readonly operation: AdvisorOperationSnapshot;
};

/**
 Install accounting restoration without changing Pi's error flag or leaking state between sessions.
 
 @param pi - host persistence and lifecycle registration capability
 
 @returns recorder called before rethrowing an Advisor operation error
 
 @mutates pi - registers hooks and appends durable failed-operation entries
 
 @example
 ```ts
 const onFailure = registerAdvisorFailureAccounting(pi);
 ```
 */
export function registerAdvisorFailureAccounting(pi: ForeignHostCapability<ExtensionAPI>,): (failure: AdvisorFailureRecord) => void {
  /**
   Only failures awaiting their matching tool_result event are retained.
   */
  const pending = new Map<string, AdvisorOperationSnapshot>();
  pi.on(
    'tool_result',
    function restoreAccounting(event: {
      readonly toolName: string;
      readonly toolCallId: string
    }) {
      if (event.toolName !== ADVISOR_TOOL_NAME)
        return undefined;
    /**
     Matching failed execution snapshot, never a different tool's accounting.
     */
      const operation = pending.get(event.toolCallId,);
      if (operation === undefined)
        return undefined;
      pending.delete(event.toolCallId,);
      return {
        details: { operation, },
        usage: operation.usage,
      };
    },
  );
  pi.on(
    'session_shutdown',
    function clearPending(): void { pending.clear(); },
  );
  return function recordFailure(failure: AdvisorFailureRecord,): void {
    pending.set(
      failure.toolCallId,
      failure.operation,
    );
    pi.appendEntry(
      ADVISOR_OPERATION_TYPE,
      failure.operation,
    );
  };
}
