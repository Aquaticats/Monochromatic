/**
 Primitive target classification before delegating dynamic Sinon overloads. @module
 */

/**
 The property argument needs Sinon's own overload validation rather than contextual routing.
 */
export const SINON_VALIDATES_PROPERTY: unique symbol = Symbol('property argument requires Sinon validation',);

/**
 Recognizes property-bearing values without excluding callable objects.

 @param value - dynamic Sinon argument

 @returns whether property operations can target this value

 @example
 ```ts
 if (isSandboxTarget(value)) inspect(value);
 ```
 */
export function isSandboxTarget(value: unknown,): value is object {
  return (((typeof value) === 'object') && (value !== null)) || ((typeof value) === 'function');
}

/**
 Normalizes primitive keys without invoking arbitrary coercion code.

 @param value - dynamic property argument

 @returns primitive property key or SINON_VALIDATES_PROPERTY for other overloads

 @example
 ```ts
 const key = sandboxPropertyKey(args[1]);
 ```
 */
export function sandboxPropertyKey(value: unknown,): string | symbol {
  if (((typeof value) === 'string') || ((typeof value) === 'symbol'))
    return value;
  return (typeof value) === 'number' ? String(value,) : SINON_VALIDATES_PROPERTY;
}
