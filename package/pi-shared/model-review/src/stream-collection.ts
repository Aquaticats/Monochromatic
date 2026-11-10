/**
 Structured-tool and direct-JSON stream collection.
 
 @module
 */

import type {
  AssistantMessageEvent,
  ToolCall,
} from '@earendil-works/pi-ai';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { extractStructuredJson, } from './json.ts';

//region Result types and sentinels -- collected stream outcome

/**
 Sentinel for stream without tool call.
 */
const NO_TOOL_CALL: unique symbol = Symbol('structured review tool call absent',);

/**
 Collected initial structured stream result.
 
 @example
 ```ts
 const result: CollectedStructuredStream = { kind: 'noToolCall', textContent: '' };
 ```
 */
type CollectedStructuredStream =
  | {
    /**
     Structured tool discriminant.
     */
    readonly kind: 'toolCall';
    /**
     Unknown tool arguments retained for caller parser.
     */
    readonly arguments: unknown;
  }
    | {
    /**
     Missing-tool discriminant.
     */
      readonly kind: 'noToolCall';
    /**
     Concatenated finalized text blocks.
     */
      readonly textContent: string;
    };

//endregion Result types and sentinels -- collected stream outcome

//region Empty-output error -- distinguishes empty output from malformed JSON

/**
 Error distinguishing empty direct JSON from malformed JSON.
 
 @example
 ```ts
 throw new EmptyStructuredReviewTextError();
 ```
 */
class EmptyStructuredReviewTextError extends Error {
  /**
   Create stable empty-output error.
   
   @example
   ```ts
   new EmptyStructuredReviewTextError();
   ```
   */
  constructor() {
    super('Structured reviewer direct JSON returned no text',);
    this.name = 'EmptyStructuredReviewTextError';
  }
}

//endregion Empty-output error -- distinguishes empty output from malformed JSON

//region Provider failure error -- terminal provider event surfaced as a throw

/**
 Terminal provider failure carried by a stream error event.
 
 A provider that refuses, throttles, or aborts a request ends its stream with an
 `error` event instead of a tool call. Callers retry another candidate or report
 the failure when a request throws, so the event becomes this error rather than
 an omitted-tool reply built from text the provider never sent.
 
 @example
 ```ts
 throw new StructuredReviewProviderError({
   stopReason: 'error',
   errorMessage: '429: rate limited',
 });
 ```
 */
class StructuredReviewProviderError extends Error {
  /**
   Provider diagnostic sent with the failing response, absent when it sent none.
   */
  readonly errorMessage?: string;
  /**
   Terminal stop reason the provider reported for this stream.
   */
  readonly stopReason: 'aborted' | 'error';

  /**
   Create provider failure carrying the provider's own diagnostic.
   
   @param stopReason - terminal stop reason the provider reported
   
   @param errorMessage - provider diagnostic sent with the failing response
   
   @example
   ```ts
   new StructuredReviewProviderError({ stopReason: 'error', errorMessage: '429: rate limited' });
   ```
   */
  constructor(
    {
      stopReason,
      errorMessage,
    }: {
      readonly stopReason: 'aborted' | 'error';
      readonly errorMessage?: string;
    },
  ) {
    super(
      errorMessage === undefined
        ? `Structured reviewer provider failed (stopReason "${stopReason}") and sent no error message`
        : `Structured reviewer provider failed (stopReason "${stopReason}"): ${errorMessage}`,
    );
    this.name = 'StructuredReviewProviderError';
    this.stopReason = stopReason;
    if (errorMessage !== undefined)
      this.errorMessage = errorMessage;
  }
}

//endregion Provider failure error -- terminal provider event surfaced as a throw

//region Collection -- one stream folded into tool arguments or finalized text

/**
 Consume stream and collect expected tool arguments or finalized text.
 
 @param stream - reviewer event stream
 
 @param expectedToolName - exact structured tool name
 
 @returns structured arguments or missing-tool text
 
 @mutates stream - async iteration consumes supplied stream
 
 @throws when reviewer calls unexpected tool, or {@link StructuredReviewProviderError} when the provider fails
 
 @example
 ```ts
 await collectStructuredStream({ stream, expectedToolName: 'submit_review' });
 ```
 */
async function collectStructuredStream(
  {
    stream,
    expectedToolName,
  }: {
    readonly stream: ForeignBorrowed<AsyncIterable<AssistantMessageEvent>>;
    readonly expectedToolName: string;
  },
): Promise<CollectedStructuredStream> {
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- Async stream fold updates one tool latch and concatenated finalized text. */
  /**
   Last structured tool call or absent sentinel.
   */
  let toolCall: ToolCall | typeof NO_TOOL_CALL = NO_TOOL_CALL;
  /**
   Finalized text emitted by stream.
   */
  let textContent = '';
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  for await (const event of stream) {
    if (event.type === 'error') {
      /**
       Trimmed provider diagnostic, absent when the provider sent none.
       */
      const errorMessage = event.error
        .errorMessage
        ?.trim();
      throw new StructuredReviewProviderError({
        stopReason: event.reason,
        ...((errorMessage === undefined) || (errorMessage === '')
          ? {}
          : { errorMessage, }),
      },);
    }
    if (event.type === 'toolcall_end') {
      /**
       Complete tool call emitted by reviewer.
       */
      const { toolCall: emittedToolCall, } = event;
      if (emittedToolCall.name !== expectedToolName) {
        throw new Error(
          `Structured reviewer called unexpected tool "${emittedToolCall.name}" instead of "${expectedToolName}"`,
        );
      }
      toolCall = emittedToolCall;
    }
    if (event.type === 'text_end')
      textContent += event.content;
  }

  if (toolCall !== NO_TOOL_CALL) {
    return {
      kind: 'toolCall',
      arguments: toolCall.arguments,
    };
  }
  return {
    kind: 'noToolCall',
    textContent,
  };
}

//endregion Collection -- one stream folded into tool arguments or finalized text

//region Public API -- direct JSON collection

/**
 Collect direct JSON stream and parse unknown value.
 
 @param stream - direct JSON reviewer event stream
 
 @param expectedToolName - tolerated expected tool name
 
 @returns parsed unknown reviewer value
 
 @mutates stream - async iteration consumes supplied stream
 
 @throws {@link EmptyStructuredReviewTextError} for empty output
 
 @throws {@link StructuredReviewProviderError} when the provider fails
 
 @example
 ```ts
 await collectDirectJson({ stream, expectedToolName: 'submit_review' });
 ```
 */
async function collectDirectJson(
  {
    stream,
    expectedToolName,
  }: {
    readonly stream: ForeignBorrowed<AsyncIterable<AssistantMessageEvent>>;
    readonly expectedToolName: string;
  },
): Promise<unknown> {
  /**
   Collected retry stream.
   */
  const result = await collectStructuredStream({
    stream,
    expectedToolName,
  },);
  if (result.kind === 'toolCall')
    return result.arguments;
  if (result.textContent === '')
    throw new EmptyStructuredReviewTextError();
  return extractStructuredJson(result.textContent,);
}

//endregion Public API -- direct JSON collection

export {
  collectDirectJson,
  collectStructuredStream,
  EmptyStructuredReviewTextError,
  StructuredReviewProviderError,
};
export type { CollectedStructuredStream, };
