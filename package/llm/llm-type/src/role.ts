/**
 Chat message author roles for OpenAI-compatible chat-completion APIs.
 
 `CHAT_ROLES` is the runtime source of truth; `ChatRole` derives from it so
 the union and the validation array can never drift apart.
 
 @example
 ```ts
 import {
   CHAT_ROLES,
   type ChatRole,
 } from '@monochromatic-dev/llm-llm-type';
 
 const role: ChatRole = 'user';
 const isRole = CHAT_ROLES.includes(role,);
 ```
 
 @module
 */

/**
 Author roles an OpenAI-compatible chat-completion request accepts.
 
 Ordered system, user, assistant to mirror a typical conversation turn.
 Tool and function roles are intentionally omitted until a consumer needs them.
 
 @example
 ```ts
 for (const role of CHAT_ROLES) console.log(role,);
 ```
 */
export const CHAT_ROLES = [
  'system',
  'user',
  'assistant',
] as const;

/**
 Union of valid chat message author roles, derived from {@link CHAT_ROLES}.
 
 Derived from the runtime array so a new role is added in exactly one place.
 
 @example
 ```ts
 const role: ChatRole = 'assistant';
 ```
 */
export type ChatRole = typeof CHAT_ROLES[number];
