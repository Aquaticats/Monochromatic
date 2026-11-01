/**
 Terminal classification and assistant-message construction for the DSH
 Advisor stream mapping. @module
 */

import type {
  AssistantMessage,
  StopReason,
  TextContent,
  ThinkingContent,
  ToolCall,
  Usage,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  FinishReason,
  GenerateOptions,
} from '@deepseek-ai/dsh-llm';
import {
  DSH_CREDIT_EXHAUSTION_CODES,
  DSH_CREDIT_EXHAUSTION_MARKER,
  DSH_MODEL_API,
} from './constants.ts';

//region Terminal message construction

/**
 Failure text for one terminal DSH failure record.

 Quota codes carry the shared credit-exhaustion marker so provider exclusion
 keeps pi parity.

 @param reason - terminal failure reason with provider-neutral code

 @returns diagnostic text recorded on the terminal message
 */
function finishFailureText(
  reason: ForeignBorrowed<Extract<FinishReason, {
    kind: 'error' | 'aborted';
  }>>,
): string {
  /**
   Whether the failure code means exhausted quota or balance.
   */
  const exhaustedCredit = DSH_CREDIT_EXHAUSTION_CODES.some(
    function matchesCreditCode(code,) {
      return reason.failure
        .code
        .toUpperCase()
        === code;
    },);
  return exhaustedCredit
    ? `${reason.failure
      .message}; ${DSH_CREDIT_EXHAUSTION_MARKER}`
    : reason.failure
      .message;
}

/**
 Classify one terminal DSH finish reason.

 @param reason - finish reason from the terminal stream chunk

 @returns done-event or failure-event classification with diagnostic text

 @example
 ```ts
 const classification = advisorStopReasonForFinish(chunk.finish);
 ```
 */
export function advisorStopReasonForFinish(
  reason: ForeignBorrowed<FinishReason>,
): {
  readonly kind: 'done';
  readonly doneReason: 'stop' | 'length' | 'toolUse';
} | {
  readonly kind: 'failure';
  readonly failureReason: 'error' | 'aborted';
  readonly errorMessage: string;
} {
  if (reason.kind
    === 'stop')
    return {
      kind: 'done',
      doneReason: 'stop',
    };
  if (reason.kind
    === 'tool-calls')
    return {
      kind: 'done',
      doneReason: 'toolUse',
    };
  if (reason.kind
    === 'max-tokens')
    return {
      kind: 'done',
      doneReason: 'length',
    };
  if (reason.kind
    === 'aborted')
    return {
      kind: 'failure',
      failureReason: 'aborted',
      errorMessage: finishFailureText(reason,),
    };
  if (reason.kind
    === 'error')
    return {
      kind: 'failure',
      failureReason: 'error',
      errorMessage: finishFailureText(reason,),
    };
  return {
    kind: 'failure',
    failureReason: 'error',
    errorMessage: `unmapped DSH finish reason: ${JSON.stringify(reason,)}`,
  };
}

/**
 Build one pi-ai assistant message over the collected response state.

 @param generate - one-shot request carrying the route identity

 @param content - assembled pi-ai content blocks in completion order

 @param usage - collected usage counters for the response

 @param stopReason - classified terminal or pending stop reason

 @param errorMessage - failure text recorded on the message, when any

 @param timestampMs - message timestamp in milliseconds

 @returns terminal or partial assistant message

 @example
 ```ts
 const message = buildAdvisorAssistantMessage({
   generate,
   content,
   usage,
   stopReason: 'stop',
   timestampMs,
 });
 ```
 */
export function buildAdvisorAssistantMessage(
  {
    generate,
    content,
    usage,
    stopReason,
    errorMessage,
    timestampMs,
  }: ForeignBorrowed<{
    readonly generate: GenerateOptions;
    readonly content: (TextContent | ThinkingContent | ToolCall)[];
    readonly usage: Usage;
    readonly stopReason: StopReason;
    readonly errorMessage?: string;
    readonly timestampMs: number;
  }>,
): AssistantMessage {
  return {
    role: 'assistant',
    content,
    api: DSH_MODEL_API,
    provider: generate.provider,
    model: generate.model,
    usage,
    stopReason,
    ...(errorMessage === undefined
      ? {}
      : { errorMessage, }),
    timestamp: timestampMs,
  };
}

//endregion Terminal message construction
