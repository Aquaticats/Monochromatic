/**
 Reads fields of Pi's records, whose shapes come from another program and are checked here rather than assumed.

 @module
 */

import type { PiRecord, } from './driver-types.ts';

/**
 Tells whether a value is an object with fields.

 @param value - value from a parsed record

 @returns whether its fields can be read

 @example
 ```ts
 const usable = isRecord(JSON.parse('{}',),);
 ```
 */
export function isRecord(value: unknown,): value is PiRecord {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reads a field that should hold an object.

 @param value - field's value

 @returns the object, or an empty one when field holds anything else

 @example
 ```ts
 const usage = fieldsOf(message.usage,);
 ```
 */
export function fieldsOf(value: unknown,): PiRecord {
  return isRecord(value,) ? value : {};
}

/**
 Reads a field that should hold a list of objects.

 @param value - field's value

 @returns the objects it holds, or none when field holds anything else

 @example
 ```ts
 const parts = recordsIn(message.content,);
 ```
 */
export function recordsIn(value: unknown,): readonly PiRecord[] {
  return Array.isArray(value,) ? value.filter(isRecord,) : [];
}

/**
 Reads a field that should hold text.

 @param value - field's value

 @returns the text, or nothing when field holds anything else

 @example
 ```ts
 const text = textOf(part.text,);
 ```
 */
export function textOf(value: unknown,): string {
  return ((typeof value) === 'string') ? value : '';
}
