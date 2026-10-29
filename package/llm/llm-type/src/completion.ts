/**
 Non-streaming chat-completion response shapes for OpenAI-compatible APIs.
 
 Designed as a superset of what each consumer reads: every field beyond
 `choices[].message.content` is optional, so a consumer's unchecked
 `as ChatCompletionResponse` cast keeps compiling and `message.content`
 stays a non-null `string`. A consumer that also needs token usage composes
 {@link ChatCompletionResponse} with {@link CompletionUsage}, for example
 `ChatCompletionResponse & { readonly usage: CompletionUsage }`, rather than
 redeclaring the response.
 
 @example
 ```ts
 import type { ChatCompletionResponse } from '@monochromatic-dev/llm-llm-type';
 
 const data = (await response.json()) as ChatCompletionResponse;
 const text = data.choices[0]?.message.content ?? '';
 ```
 
 @module
 */

import type { ChatRole, } from './role.ts';

/**
 Single choice returned in a non-streaming chat completion.
 
 Only `message.content` is required, because that is the one field every
 consumer reads; `role` is optional so a response carrying it still matches
 while content-only responses (vision callers) match too. This keeps every
 existing `as ChatCompletionResponse` cast valid without a null branch.
 
 @example
 ```ts
 const choice: ChatCompletionChoice = { message: { content: 'Done.', }, };
 ```
 */
export type ChatCompletionChoice = {
  /**
   Generated message for this choice.
   */
  readonly message: {
    /**
     Author role ({@link ChatRole}) echoed by the API, absent when a consumer does not read it.
     */
    readonly role?: ChatRole;
    /**
     Generated text body of the choice.
     */
    readonly content: string;
  };
};

/**
 Full non-streaming chat-completion response.
 
 @example
 ```ts
 const data = (await response.json()) as ChatCompletionResponse;
 const [first,] = data.choices;
 ```
 */
export type ChatCompletionResponse = {
  /**
   Completion choices; `n=1` requests return a single-element array.
   */
  readonly choices: readonly ChatCompletionChoice[];
};
