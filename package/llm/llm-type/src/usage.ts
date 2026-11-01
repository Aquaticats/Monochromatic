/**
 Token usage statistics for OpenAI-compatible chat-completion responses.
 
 Field names stay snake_case to match the wire format, so a parsed response
 body satisfies the type without remapping. A consumer that needs usage
 intersects it onto a {@link ChatCompletionResponse}, for example
 `ChatCompletionResponse & { readonly usage: CompletionUsage }`.
 
 @example
 ```ts
 import type { CompletionUsage } from '@monochromatic-dev/llm-llm-type';
 
 const usage: CompletionUsage = { prompt_tokens: 12, completion_tokens: 34, };
 ```
 
 @module
 */

/**
 Prompt and completion token counts reported alongside a completion.
 
 `total_tokens` is optional because not every OpenAI-compatible server
 includes it; the two component counts are always present when a server
 returns a `usage` block.
 
 @example
 ```ts
 const usage: CompletionUsage = {
   prompt_tokens: 100,
   completion_tokens: 42,
   total_tokens: 142,
 };
 ```
 */
export type CompletionUsage = {
  /**
   Tokens consumed by the prompt.
   */
  readonly prompt_tokens: number;
  /**
   Tokens generated in the completion.
   */
  readonly completion_tokens: number;
  /**
   Sum of prompt and completion tokens, absent when the server omits it.
   */
  readonly total_tokens?: number;
};
