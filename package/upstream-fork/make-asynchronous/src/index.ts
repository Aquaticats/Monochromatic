/**
 TypeScript fork of [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by Sindre Sorhus (MIT): run a serialized function in a worker thread
 without blocking the main thread.
 
 Rewritten to this repository's TypeScript standards with the upstream
 `make-asynchronous` 2.1.0 worker semantics preserved: each call and each
 iteration spawns its own worker, the wrapped function is serialized and
 cannot close over outer scope, results and failures (including falsy
 values) travel back keyed by request id, and `withSignal` fails the worker
 with the signal's reason. The only API-shape deviations are
 lint-mandated: factories and calls take destructured objects
 (`makeAsynchronous({ fn, options })`, `fn({ args })`), and the iterable
 factory takes `{ fn, options }` with `{ args }` per iteration.
 
 @example
 ```ts
 import { makeAsynchronous, } from '\@monochromatic-dev/upstream-fork-make-asynchronous';
 
 const fn = makeAsynchronous({
   fn: function double(value: number,): number {
     return value * 2;
   },
 });
 await fn({ args: [2], }); // => 4
 ```
 
 @packageDocumentation
 */

export {
  makeAsynchronous,
  type AnyFunction,
  type AsyncCall,
  type AsyncForm,
  type AsyncSettled,
  type AsyncWrapped,
} from './make-asynchronous.ts';

export {
  type AsyncIterableForm,
  type AsyncIterableWrapped,
  type IterableCall,
  type IterableFunction,
  type IterableValue,
  makeAsynchronousIterable,
} from './make-asynchronous-iterable.ts';

export { type MakeAsynchronousOptions, } from './options.ts';

export { getResult, } from './result.ts';

export {
  blobUrl,
  createWorker,
  getMessageData,
  isMessageEvent,
  isNodeRuntime,
  isWorkerReply,
  type LiveWorker,
  type NodeWorkerHandle,
  type WorkerConstructor,
  type WorkerHandle,
  type WorkerListener,
  type WorkerMessage,
  type WorkerReply,
  type WorkerRequest,
  workerConstructorFrom,
  workerThreadsSpecifier,
} from './worker-lifecycle.ts';

export {
  errorReporterSource,
  makeCallWorkerBody,
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
} from './worker-source.ts';
