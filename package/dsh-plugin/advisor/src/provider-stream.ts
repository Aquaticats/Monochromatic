/**
 DSH LLM streaming consumption mapped onto the pi-ai assistant event
 protocol. @module
 */

import type {
  AssistantMessage,
  AssistantMessageEvent,
  TextContent,
  ThinkingContent,
  ToolCall,
} from '@earendil-works/pi-ai';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type {
  GenerateOptions,
} from '@deepseek-ai/dsh-llm';
import {
  emptyAdvisorUsage,
} from './message-map.ts';
import {
  advisorStopReasonForFinish,
  buildAdvisorAssistantMessage,
} from './provider-stream-finish.ts';
import {
  publishAdvisorChunkEvent,
} from './provider-stream-chunk.ts';
import {
  createDshAdvisorStreamDriver,
  type DshAdvisorStreamSurface,
} from './stream-driver.ts';
import { mapDshUsageToPiUsage, } from './usage-map.ts';
import type {
  DshAdvisorLlmSurface,
} from './types.ts';

//region Types

/**
 Inputs for streaming one Advisor completion through the DSH runtime.
 */
export type StartDshAdvisorStreamOptions = {
  /**
   DSH LLM surface owning the provider route.
   */
  readonly llm: DshAdvisorLlmSurface;
  /**
   Assembled one-shot request for the Advisor call.
   */
  readonly generate: GenerateOptions;
};

//endregion Types

//region Public API

/**
 Start one Advisor completion over `ctx.llm.stream` and expose it as a pi-ai
 assistant event stream.

 Events map structurally from DSH chunks: block starts, deltas, and ends carry
 the real text and tool-call data, and every partial carries the latest
 usage. Usage-only updates publish an empty text delta because the shared
 seam reads `partial.usage` from partial events (documented gap).

 @param options - runtime and assembled request

 @returns stream surface iterated for usage updates and resolved with the
 terminal message

 @example
 ```ts
 const stream = startDshAdvisorStream({ llm, generate });
 for await (const event of stream) { ... }
 const message = await stream.result();
 ```
 */
export function startDshAdvisorStream(
  options: StartDshAdvisorStreamOptions,
): DshAdvisorStreamSurface {
  /**
   Push driver publishing mapped events and the terminal message.
   */
  const driver = createDshAdvisorStreamDriver();
  void consumeDshAdvisorStream({
    llm: options.llm,
    generate: options.generate,
    publish: driver.publish,
    conclude: driver.conclude,
  },);
  return driver.surface;
}

//endregion Public API

//region Stream consumption

/**
 Consume one DSH stream and publish its mapped pi-ai event protocol.

 Never throws: every terminal path concludes the driver with a classified
 assistant message. The publisher handles record the mapped protocol as the
 stream advances.

 @param llm - DSH LLM surface owning the request

 @param generate - one-shot request being consumed

 @param publish - event publisher driving the pi-ai stream surface

 @param conclude - terminal publisher closing the driver
 */
async function consumeDshAdvisorStream(
  {
    llm,
    generate,
    publish,
    conclude,
  }: {
    readonly llm: DshAdvisorLlmSurface;
    readonly generate: GenerateOptions;
    readonly publish: (event: AssistantMessageEvent) => void;
    readonly conclude: (message: AssistantMessage) => void;
  },
): Promise<void> {
  /**
   Assembled pi-ai content blocks in completion order.
   */
  const content: (TextContent | ThinkingContent | ToolCall)[] = [];
  /**
   Response assembly state mutated per stream chunk.
   */
  const state = {
    usage: emptyAdvisorUsage(),
    timestampMs: Date.now(),
  };
  /**
   Partial message snapshot carrying the latest usage for seam updates.

   @returns partial assistant message over the current response state
   */
  function partial(): AssistantMessage {
    return buildAdvisorAssistantMessage({
      generate,
      content,
      usage: state.usage,
      stopReason: 'pending',
      timestampMs: state.timestampMs,
    },);
  }
  publish({
    type: 'start',
    partial: partial(),
  },);
  try {
    for await (const chunk of llm
      .stream(generate)) {
      if (chunk.type
        === 'finish') {
        /**
         Stop classification for this terminal finish.
         */
        const classification = advisorStopReasonForFinish(chunk.reason,);
        /**
         Terminal assistant message for this response.
         */
        const terminal = buildAdvisorAssistantMessage({
          generate,
          content,
          usage: state.usage,
          stopReason: classification.kind === 'done'
            ? classification.doneReason
            : classification.failureReason,
          ...(classification.kind === 'failure'
            ? { errorMessage: classification.errorMessage, }
            : {}),
          timestampMs: state.timestampMs,
        },);
        if (classification.kind
          === 'done')
          publish({
            type: 'done',
            reason: classification.doneReason,
            message: terminal,
          },);
        else
          publish({
            type: 'error',
            reason: classification.failureReason,
            error: terminal,
          },);
        conclude(terminal,);
        return;
      }
      if (chunk.type
        === 'usage') {
        state.usage = mapDshUsageToPiUsage(chunk.usage,);
        publish({
          type: 'text_delta',
          contentIndex: 0,
          delta: '',
          partial: partial(),
        },);
        continue;
      }
      publishAdvisorChunkEvent({
        chunk,
        content,
        publish,
        partial: partial(),
      },);
    }
    conclude(
      buildAdvisorAssistantMessage({
        generate,
        content,
        usage: state.usage,
        stopReason: 'error',
        errorMessage: 'advisor: DSH stream ended without a terminal finish',
        timestampMs: state.timestampMs,
      },),
    );
  }
  catch (error) {
    conclude(
      buildAdvisorAssistantMessage({
        generate,
        content,
        usage: state.usage,
        stopReason: 'error',
        errorMessage: `advisor: DSH stream failed: ${caughtValueText(error,)}`,
        timestampMs: state.timestampMs,
      },),
    );
  }
}

//endregion Stream consumption
