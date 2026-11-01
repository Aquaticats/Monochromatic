/**
 Side-effect-free formatting for unknown diagnostic values.
 
 @module
 */

/**
 Formats primitive values without invoking caller-owned coercion hooks.
 
 Objects and functions intentionally use category placeholders. Calling
 `String` on either can execute `toString`, `valueOf`, proxy, or symbol hooks
 supplied by caller-owned state while an assertion failure is being rendered.
 
 @param value - Unknown value being rendered for diagnostics.
 
 @returns primitive text or side-effect-free reference category.
 
 @example
 ```ts
 formatUnknownValue({ custom: true, }); // '[object]'
 formatUnknownValue(-0,); // '0'
 ```
 */
export function formatUnknownValue(value: unknown,): string {
  if (value === null)
    return 'null';
  if ((typeof value) === 'string')
    return value;
  if ((typeof value) === 'number')
    return `${value}`;
  if ((typeof value) === 'bigint')
    return `${value}`;
  if ((typeof value) === 'boolean')
    return value ? 'true' : 'false';
  if ((typeof value) === 'undefined')
    return 'undefined';
  if ((typeof value) === 'symbol') {
    /**
     Primitive symbol description read without invoking custom hooks.
     */
    const { description, } = value;
    return description === undefined ? 'Symbol()' : `Symbol(${description})`;
  }
  if ((typeof value) === 'function')
    return '[function]';
  return '[object]';
}
