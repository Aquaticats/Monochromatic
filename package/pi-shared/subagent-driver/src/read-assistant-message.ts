/**
 Reads what a finished assistant message says, so the record keeps its parts without holding Pi's whole message.

 @module
 */

import type { PiRecord, } from './driver-types.ts';
import {
  textOfParts,
  thinkingLengthOf,
} from './previews-of-pi-messages.ts';
import {
  fieldsOf,
  recordsIn,
  textOf,
} from './read-pi-fields.ts';

/**
 What one assistant message says.
 */
export type AssistantMessage = {
  /**
   The message's text, joined from its text parts.
   */
  readonly text: string;

  /**
   How the message ended, such as `stop` or `error`.
   */
  readonly stop: string;

  /**
   What the model said about ending it, which is empty when it said nothing.
   */
  readonly error: string;

  /**
   The message's parts, which name the tool calls it made.
   */
  readonly parts: readonly PiRecord[];

  /**
   The characters of thinking the message holds.
   */
  readonly thinkingChars: number;

  /**
   Its usage and cost.
   */
  readonly usage: PiRecord;
};

/**
 Tells whether a `message_end` record finished an assistant's message.

 @param event - `message_end` record

 @returns whether the message is an assistant's

 @example
 ```ts
 const assistant = isAssistantMessageEnd(event,);
 ```
 */
export function isAssistantMessageEnd(event: PiRecord,): boolean {
  return fieldsOf(event.message,)
    .role
    === 'assistant';
}

/**
 Reads one finished assistant message.

 @param event - `message_end` record of an assistant's message

 @returns what the message says

 @example
 ```ts
 const message = assistantMessageOf(event,);
 ```
 */
export function assistantMessageOf(event: PiRecord,): AssistantMessage {
  /**
   The message's parts.
   */
  const parts = recordsIn(fieldsOf(event.message,)
    .content,);
  return {
    text: textOfParts(parts,),
    stop: textOf(fieldsOf(event.message,)
      .stopReason,),
    error: textOf(fieldsOf(event.message,)
      .errorMessage,),
    parts,
    thinkingChars: thinkingLengthOf(parts,),
    usage: fieldsOf(fieldsOf(event.message,)
      .usage,),
  };
}
