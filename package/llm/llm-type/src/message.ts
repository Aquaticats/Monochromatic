/**
 Plain-text chat message envelope for OpenAI-compatible chat-completion APIs.
 
 Multimodal consumers (vision requests carrying image parts) compose over
 {@link ContentPart} instead of widening this `content` field; keeping the
 canonical envelope text-only avoids forcing a content-type branch on the
 majority of callers that only send text.
 
 @example
 ```ts
 import type { ChatMessage } from '@monochromatic-dev/llm-llm-type';
 
 const messages: readonly ChatMessage[] = [
   { role: 'system', content: 'You are concise.', },
   { role: 'user', content: 'Summarise this.', },
 ];
 ```
 
 @module
 */

import type { ChatRole, } from './role.ts';

/**
 Single text message in a chat conversation.
 
 Fields are `readonly` because a message, once enqueued, is never mutated in
 place by any consumer; an array of these stays assignable to a mutable
 `ChatMessage[]` when a caller builds the turn list incrementally.
 
 @example
 ```ts
 const turn: ChatMessage = { role: 'user', content: 'Hello.', };
 ```
 */
export type ChatMessage = {
  /**
   Author of the message, narrowing the OpenAI-compatible `role` field.
   */
  readonly role: ChatRole;
  /**
   Plain-text body of the message.
   */
  readonly content: string;
};
