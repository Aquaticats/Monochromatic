/**
 Durable `plugin:advisor/operation` session records: the DSH form of pi's
 `pi-advisor.operation` failure accounting. @module
 */

import type {
  AdvisorOperationSnapshot,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import {
  appendPluginRecord,
  type Session,
} from '@deepseek-ai/dsh-session';

declare module '@deepseek-ai/dsh-session/types' {
  /* oxlint-disable typescript/consistent-type-definitions -- declaration merging on the dsh-session PluginRecordMap interface is the sanctioned way to declare plugin record payloads, so this augmentation must stay an interface */
  /**
   Plugin record map extension declaring the durable `plugin:advisor/operation`
   record kind for the dsh-session appendPluginRecord API.
   */
  interface PluginRecordMap {
    /**
     Operation ledger snapshot recorded when an Advisor operation completes, so
     attempt states, diagnostics, and usage survive the rendered-text-only
     session record.
     */
    'plugin:advisor/operation': AdvisorOperationSnapshot;
  }
  /* oxlint-enable typescript/consistent-type-definitions */
}

//region Public API

/**
 Persist one Advisor operation to the session log as an ignorable plugin
 record.

 `appendPluginRecord` marks the record `ignorable` and namespaces it under
 `plugin:`, so every reader that lacks this plugin skips it instead of
 refusing the whole log the way an unknown required event does. The agent
 lifecycle attaches the session-log writer, so this append is durable exactly
 like agent-loop events. Every completed operation appends here because the
 session record stores rendered tool text only and the canonical tool JSON
 value carrying `details` is not durable.

 @param session - session log receiving the durable record

 @param operation - operation ledger snapshot

 @example
 ```ts
 appendAdvisorOperationRecord({ session: exec.agent.session, operation: error.operation });
 ```
 */
export function appendAdvisorOperationRecord(
  {
    session,
    operation,
  }: {
    readonly session: Session;
    readonly operation: AdvisorOperationSnapshot;
  },
): void {
  appendPluginRecord(
    session,
    'plugin:advisor/operation',
    operation,
  );
}

//endregion Public API
