'use strict';

const Stream = require('node:stream',);

module.exports = Stream.Readable;
for (const key of Object.keys(Stream,)) {
  Object.defineProperty(module.exports, key, {
    value: Stream[key],
    writable: true,
    enumerable: true,
    configurable: true,
  },);
}
module.exports.Stream = Stream;
