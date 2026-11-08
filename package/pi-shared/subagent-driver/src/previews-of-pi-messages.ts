/**
 Turns parts of Pi's messages into the short text a feed line holds.

 @module
 */

import type { PiRecord, } from './driver-types.ts';
import { textOf, } from './read-pi-fields.ts';

/**
 Shortens a value for a feed line, saying how long it was.

 @param value - value to show

 @param length - longest text to keep

 @returns the value as text, cut if long

 @example
 ```ts
 const text = preview({ value: 'abc', length: 2, },);
 ```
 */
export function preview({
  value,
  length,
}: {
  readonly value: unknown;
  readonly length: number
},): string {
  /**
   The value as text.
   */
  const text = ((typeof value) === 'string') ? value : (JSON.stringify(value,) ?? '');
  return (text.length > length) ? `${text.slice(
    0,
    length,
  )}...[${String(text.length,)}]` : text;
}

/**
 Joins the text parts of a message.

 @param parts - message's parts

 @returns the message's text

 @example
 ```ts
 const text = textOfParts([{ type: 'text', text: 'hello', },],);
 ```
 */
export function textOfParts(parts: readonly PiRecord[],): string {
  return parts.filter(function isTextPart(part,) {
    return part.type === 'text';
  },)
    .map(function textOfPart(part,) {
      return textOf(part.text,);
    },)
    .join('\n',);
}

/**
 Counts the characters of a message's thinking.

 @param parts - message's parts

 @returns the count

 @example
 ```ts
 const length = thinkingLengthOf([{ type: 'thinking', thinking: 'hm', },],);
 ```
 */
export function thinkingLengthOf(parts: readonly PiRecord[],): number {
  return parts.filter(function isThinkingPart(part,) {
    return part.type === 'thinking';
  },)
    .reduce(
    function addLength(
      sum,
      part,
    ) {
      return sum
        + textOf(part.thinking,)
          .length;
    },
    0,
    );
}
