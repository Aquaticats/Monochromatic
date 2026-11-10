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

// Node builds a CommonJS module's ESM named exports from static `module.exports.<name> =` writes alone, so the
// loop above leaves every property it copies invisible to `import { Readable } from 'readable-stream'`, and that
// import fails at link time while `require` answers it. These assignments repeat each copy in the form the
// detector reads: the loop still owns the property descriptors, this block owns the ESM names, and
// `index.unit.test.ts` fails when node:stream gains a name that is missing here.
module.exports.isDestroyed = Stream.isDestroyed;
module.exports.isDisturbed = Stream.isDisturbed;
module.exports.isErrored = Stream.isErrored;
module.exports.isReadable = Stream.isReadable;
module.exports.isWritable = Stream.isWritable;
module.exports.Readable = Stream.Readable;
module.exports.Writable = Stream.Writable;
module.exports.Duplex = Stream.Duplex;
module.exports.Transform = Stream.Transform;
module.exports.PassThrough = Stream.PassThrough;
module.exports.duplexPair = Stream.duplexPair;
module.exports.pipeline = Stream.pipeline;
module.exports.addAbortSignal = Stream.addAbortSignal;
module.exports.finished = Stream.finished;
module.exports.destroy = Stream.destroy;
module.exports.compose = Stream.compose;
module.exports.setDefaultHighWaterMark = Stream.setDefaultHighWaterMark;
module.exports.getDefaultHighWaterMark = Stream.getDefaultHighWaterMark;
module.exports.promises = Stream.promises;
module.exports.Stream = Stream;
module.exports._isArrayBufferView = Stream._isArrayBufferView;
module.exports._isUint8Array = Stream._isUint8Array;
module.exports._uint8ArrayToBuffer = Stream._uint8ArrayToBuffer;
