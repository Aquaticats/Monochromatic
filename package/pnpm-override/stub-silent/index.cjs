'use strict';

/**
 Text every primitive coercion of the stub produces, so a consumer that interpolates the stub into a template literal or
 concatenates it reads a marker instead of crashing.
 */
const PRIMITIVE_TEXT = '[stub-silent]';

/**
 Primitive coercion of the stub, which `Symbol.toPrimitive` resolves to.

 @returns The marker text, `PRIMITIVE_TEXT`
 */
const toPrimitive = () => PRIMITIVE_TEXT;

module.exports = new Proxy(function silentStub() {}, {
  get(target, key) {
    // A callable `then` makes the stub a thenable whose callbacks are never invoked, so awaiting the stub, a property
    // chain of it, or a call result of it would never settle.
    if (key === 'then' || key === 'catch' || key === 'finally')
      return undefined;
    // Answering Symbol.toPrimitive with the stub itself makes `String(stub)` and template interpolation throw
    // "Cannot convert object to primitive value".
    if (key === Symbol.toPrimitive)
      return toPrimitive;
    return module.exports;
  },
  has(target, key) {
    // A Proxy must not report a non-configurable own key of its target as absent, and a plain function's only such key
    // is `prototype`.
    const descriptor = Reflect.getOwnPropertyDescriptor(target, key);
    return descriptor !== undefined && !descriptor.configurable;
  },
  set() {
    return true;
  },
  apply() {
    return module.exports;
  },
  construct() {
    return module.exports;
  },
},);
