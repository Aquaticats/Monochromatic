'use strict';

module.exports = new Proxy(function silentStub() {}, {
  get() {
    return module.exports;
  },
  has() {
    return false;
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
