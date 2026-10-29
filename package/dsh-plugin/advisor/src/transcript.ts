/**
 Transcript port over `Session.deriveMessages()`: one compaction-aware
 snapshot per operation, converted to shared Advisor session entries. @module
 */

import type {
  Message as PiMessage,
} from '@earendil-works/pi-ai';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  estimateAdvisorInputTokens,
} from '@monochromatic-dev/agent-harness-shared-model-selection/ts';
import type {
  AdvisorAgentMessage,
  AdvisorMessageSessionEntry,
  AdvisorTranscript,
  AdvisorTranscriptSerialization,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { serializeAdvisorConversationText, } from './conversation-text.ts';
import {
  collectAdvisorToolNames,
  isPiAgentMessage,
  mapDshMessageToAdvisorMessage,
} from './message-map.ts';
import type {
  DshAdvisorMessageSource,
} from './types.ts';

//region Types

/**
 Options for capturing one transcript snapshot.
 */
export type CaptureDshAdvisorTranscriptOptions = {
  /**
   Session message source read exactly once.
   */
  readonly source: DshAdvisorMessageSource;
  /**
   Degradation sink recording an unavailable derived history for status text.
   */
  readonly onDegraded?: (reason: string) => void;
};

/**
 Captured transcript snapshot for one operation.
 */
export type DshAdvisorTranscriptSnapshot = {
  /**
   Message entries in derived order.
   */
  readonly entries: readonly AdvisorMessageSessionEntry[];
  /**
   Empty string when the derived history was readable; otherwise the captured
   failure text recorded for `/advisor status`.
   */
  readonly degradedReason: string;
};

//endregion Types

//region Serialization seam

/**
 Transcript serialization seam backed by DSH message projection and the
 shared model-selection token estimator.

 Host functions satisfy the port properties directly because the port
 parameters mirror the pi-ai message arrays the shared serializer consumes.

 @example
 ```ts
 const text = dshAdvisorTranscriptSerialization.serializeConversation(messages);
 ```
 */
export const dshAdvisorTranscriptSerialization: AdvisorTranscriptSerialization = {
  convertToLlm: function convertToLlm(
    messages: AdvisorAgentMessage[],
  ): PiMessage[] {
    return messages.filter(function keepPiAgentMessage(
      message,
    ) {
      return isPiAgentMessage(message,);
    },);
  },
  serializeConversation: function serializeConversation(
    messages: PiMessage[],
  ): string {
    return serializeAdvisorConversationText(messages,);
  },
  estimateInputTokens: estimateAdvisorInputTokens,
};

//endregion Serialization seam

//region Public API

/**
 Capture the derived session history once, degrading to an empty branch.

 The shared core reads `buildContextEntries()` once per operation; capturing
 at construction keeps that contract even if the session mutates later. A
 derived-history failure is not fatal: the focused `question` parameter
 carries the review subject and the degradation reason reaches status text.

 @param options - session source and degradation sink

 @returns entries with a degraded-reason sentinel

 @example
 ```ts
 const snapshot = captureDshAdvisorTranscript({ source: agent.session });
 ```
 */
export function captureDshAdvisorTranscript(
  options: CaptureDshAdvisorTranscriptOptions,
): DshAdvisorTranscriptSnapshot {
  try {
    /**
     Derived durable messages for this session, read once.
     */
    const messages = options
      .source
      .deriveMessages();
    /**
     Snapshot timestamp shared by every converted entry.
     */
    const timestampMs = Date.now();
    /**
     ISO timestamp recorded on each entry for the shared context filter.
     */
    const timestamp = new Date(timestampMs,)
      .toISOString();
    /**
     Tool names indexed by call id over the same derived history.
     */
    const toolNamesByCallId = collectAdvisorToolNames(messages,);
    /**
     Message entries in derived order.
     */
    const entries: AdvisorMessageSessionEntry[] = [];
    for (const message of messages) {
      entries.push({
        type: 'message',
        message: mapDshMessageToAdvisorMessage({
          message,
          toolNamesByCallId,
          timestampMs,
        },),
        timestamp,
      },);
    }
    return {
      entries,
      degradedReason: '',
    };
  }
  catch (error) {
    /**
     Captured derived-history failure text for status output.
     */
    const reason = caughtValueText(error,);
    options
      .onDegraded
      ?.(`deriveMessages failed: ${reason}`);
    return {
      entries: [],
      degradedReason: `deriveMessages failed: ${reason}`,
    };
  }
}

/**
 Build the shared Advisor transcript port over one captured DSH snapshot.

 @param options - session source and optional degradation sink

 @returns transcript port whose entries come from the captured snapshot

 @example
 ```ts
 const transcript = createDshAdvisorTranscript({ source: agent.session });
 ```
 */
export function createDshAdvisorTranscript(
  options: CaptureDshAdvisorTranscriptOptions,
): AdvisorTranscript {
  /**
   One captured snapshot standing in for the operation's session history.
   */
  const snapshot = captureDshAdvisorTranscript(options,);
  return {
    ...dshAdvisorTranscriptSerialization,
    buildContextEntries: function buildContextEntries() {
      return snapshot.entries;
    },
  };
}

//endregion Public API
