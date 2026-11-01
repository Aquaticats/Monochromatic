/**
 Request projection from the pi-ai simple-stream context to a DSH
 `GenerateOptions` request. @module
 */

import type {
  Model,
  Api,
  ImageContent,
  TextContent,
  TranscriptContext,
} from '@earendil-works/pi-ai';
import type {
  GenerateOptions,
  ReasoningEffortId,
  RequestMessage,
} from '@deepseek-ai/dsh-llm';
import type {
  SessionId,
} from '@deepseek-ai/dsh-session/types';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { advisorContentText, } from './conversation-text.ts';

//region Types

/**
 Inputs for projecting one Advisor request onto the DSH seam.
 */
export type BuildDshAdvisorGenerateOptionsOptions = {
  /**
   Normalized pi-ai transcript context carrying the system prompt and user
   message.
   */
  readonly context: TranscriptContext;
  /**
   Selected Advisor model whose provider and id name the DSH route.
   */
  readonly model: Model<Api>;
  /**
   Reasoning effort chosen by the DSH greatest-below-max rule, when the route
   advertises efforts.
   */
  readonly reasoningEffort?: ReasoningEffortId;
  /**
   Session identity stamped onto the request for provider routing.
   */
  readonly sessionId: SessionId;
  /**
   Per-call output token budget from `maxAdvisorOutputTokens`.
   */
  readonly maxTokens?: number;
  /**
   Sampling temperature forwarded from the simple-stream options.
   */
  readonly temperature?: number;
  /**
   Cancellation signal forwarded from the simple-stream options.
   */
  readonly signal?: AbortSignal;
};

//endregion Types

//region Public API

/**
 Build the DSH request for one Advisor completion.

 The Advisor request is a one-shot call: the serialized conversation rides in
 a single user message and the system prompt maps to the `system` slot.
 `purpose` stays unset (capability gap 9), `sessionId` stamps the host session
 identity onto the request so routes that key request routing on it accept the
 call, and `maxTokens` carries the configured `maxAdvisorOutputTokens` so an
 incapable route fails at dispatch into normal recovery.

 @param options - transcript context, model, and request controls

 @returns assembled DSH generate options

 @throws when the request context carries a message role the one-shot shape
 cannot project

 @example
 ```ts
 const generate = buildDshAdvisorGenerateOptions({ context, model, sessionId });
 ```
 */
export function buildDshAdvisorGenerateOptions(
  options: BuildDshAdvisorGenerateOptionsOptions,
): GenerateOptions {
  /**
   System prompt text assembled from leading system messages.
   */
  const systemParts: string[] = [];
  /**
   Identity-free user inputs sent as the request history.
   */
  const messages: RequestMessage[] = [];
  for (const message of options
    .context
    .messages) {
    if (message.role
      === 'system') {
      systemParts.push(advisorContentText({ content: message.content, }),);
      continue;
    }
    if (message.role
      === 'user') {
      /**
       User content as a block list regardless of pi string shorthand.
       */
      const userBlocks: readonly ForeignBorrowed<ImageContent | TextContent>[] = (typeof message.content) === 'string'
        ? [{
          type: 'text',
          text: message.content,
        },]
        : message.content;
      messages.push({
        role: 'user',
        content: userBlocks.map(function toDshUserBlock(block,) {
          return {
            type: 'text',
            text: block.type === 'text'
              ? block.text
              : `[image: ${block.mimeType}]`,
          };
        },),
      },);
      continue;
    }
    /**
     The Advisor seam sends a one-shot request; history roles belong to the
     serialized context text, not the request messages.
     */
    throw new Error(
      `advisor: request context carries unsupported role ${message.role} for a one-shot DSH request`,
    );
  }
  return {
    provider: options.model
      .provider,
    model: options.model
      .id,
    sessionId: options.sessionId,
    messages,
    ...(systemParts.length === 0
      ? {}
      : { system: systemParts.join('\n\n',), }),
    ...(options.reasoningEffort === undefined
      ? {}
      : { reasoningEffort: options.reasoningEffort, }),
    ...(options.maxTokens === undefined
      ? {}
      : { maxTokens: options.maxTokens, }),
    ...(options.temperature === undefined
      ? {}
      : { temperature: options.temperature, }),
    ...(options.signal === undefined
      ? {}
      : { signal: options.signal, }),
  };
}

//endregion Public API
