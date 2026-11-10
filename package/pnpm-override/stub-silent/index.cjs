'use strict';

module.exports = new Proxy(function silentStub() {}, {
  get() {
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
