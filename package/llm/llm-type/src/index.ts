/**
 Shared OpenAI-compatible LLM client types for the Monochromatic monorepo.
 
 The canonical home for the chat-completion shapes that recur across every
 package wrapping an OpenAI-compatible endpoint (openai / openrouter /
 anthropic-compatible / local llama-server). Consumers import these instead
 of redeclaring them, and express per-consumer variants by composition:
 
 - Text chat sends {@link ChatMessage} (`role` + string `content`).
 - Vision chat composes a message over {@link ContentPart} arrays.
 - A usage-aware response intersects {@link ChatCompletionResponse} with
   {@link CompletionUsage} (`{ usage: CompletionUsage }`).
 
 Streaming chunk types and a single client implementation are deliberately
 out of scope: no consumer streams through a shared type today, and each
 package keeps its own client.
 
 @example
 ```ts
 import {
   CHAT_ROLES,
   type ChatCompletionResponse,
   type ChatMessage,
   type ContentPart,
 } from '@monochromatic-dev/llm-llm-type';
 ```
 
 @packageDocumentation
 */

//region role

export {
  CHAT_ROLES,
  type ChatRole,
} from './role.ts';

//endregion role

//region message

export type { ChatMessage, } from './message.ts';

//endregion message

//region content-part

export type { ContentPart, } from './content-part.ts';

//endregion content-part

//region completion

export type {
  ChatCompletionChoice,
  ChatCompletionResponse,
} from './completion.ts';

//endregion completion

//region usage

export type { CompletionUsage, } from './usage.ts';

//endregion usage
