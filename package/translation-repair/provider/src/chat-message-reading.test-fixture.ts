import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import { messageText, } from '../dist/final/node/index.mjs';

//region Chat message reading
// READING ONE ROLE'S TEXT OUT OF A BUILT EXCHANGE, for cases that check a
// prompt builder's output rather than a live reply.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Prompt and wire test files that kept
// their own copy of these readers now import them from here.

/**
 User message of a plan, as text.

 @param messages - messages the builder returned

 @returns Last message's text

 @throws {@link Error} when the builder returned no message

 @example
 ```ts
 const content = userText({ messages, },);
 ```
 */
export function userText({ messages, }: { readonly messages: readonly ChatMessage[]; },): string {
  /**
   Last message, which is the user turn.
   */
  const asked = messages.at(-1,);
  if (asked === undefined)
    throw new Error('the builder returned no message',);
  return messageText({ message: asked, },);
}

/**
 Joins the system half of an exchange, which is where standing rules live.

 @param messages - exchange to read

 @returns Every system message, joined

 @example
 ```ts
 const sheet = systemOf({ messages: buildTranslateMessages({ sourceText, },).messages, },);
 ```
 */
export function systemOf(
  { messages, }: {
    readonly messages: readonly {
      readonly role: string;
      readonly content: string;
    }[];
  },
): string {
  return messages
    .filter(function isSystem(message,): boolean {
      return message.role === 'system';
    },)
    .map(function toContent(message,): string {
      return message.content;
    },)
    .join('\n',);
}

//endregion Chat message reading
