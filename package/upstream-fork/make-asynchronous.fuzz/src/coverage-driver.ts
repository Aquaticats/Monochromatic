/**
 Deterministic coverage driver: exercises every exported function and its
 error paths with fixed inputs, so the V8 coverage it produces is
 reproducible. Run under `NODE_V8_COVERAGE` by the `fuzz:coverage` task,
 then summarized by `coverage-report.ts`.
 
 @module
 */

import {
  blobUrl,
  createWorker,
  errorReporterSource,
  getMessageData,
  getResult,
  isNodeRuntime,
  makeAsynchronous,
  makeAsynchronousIterable,
  makeCallWorkerBody,
  makeIterableWorkerBody,
  nodeWorkerPreambleSource,
  workerConstructorFrom,
  workerThreadsSpecifier,
} from '@monochromatic-dev/upstream-fork-make-asynchronous/ts';

//region Fixtures

/**
 Expected doubled value of the fixed call argument.
 */
const DOUBLED_VALUE = 42;

/**
 Fixed call argument doubling to the expected value.
 */
const DOUBLE_ARGUMENT = 21;

/**
 Reply id carried by the message-shape check.
 */
const MESSAGE_REPLY_ID = 6;

/**
 Expected doubled value of the string `baseUrl` call.
 */
const BASED_VALUE = 4;

/**
 Call argument doubling to the string-`baseUrl` value.
 */
const BASED_ARGUMENT = 2;

/**
 Expected doubled value of the `URL` `baseUrl` call.
 */
const BASED_URL_VALUE = 6;

/**
 Call argument doubling to the `URL`-`baseUrl` value.
 */
const BASED_URL_ARGUMENT = 3;

/**
 Doubles one fixed value inside the driver worker.
 
 @param value - Value to double.
 
 @returns Doubled value.
 */
function double(value: number,): number {
  return value * 2;
}

/**
 Generator yielding two fixed values.
 
 @returns Yielded values before returning.
 */
function * pair(): Generator<number> {
  yield 1;
  yield 2;
}

/**
 Throwing function driving the failure paths.
 
 @throws Error on every call.
 */
function fail(): never {
  throw new Error('coverage driver failed',);
}

/**
 Falsy-throwing function driving the falsy-rejection path.
 
 The reply protocol keys on `error`, not the value, so falsy rejections
 must stay rejections.
 
 @throws 0 on every call.
 */
function throwZero(): never {
  // oxlint-disable-next-line eslint/no-throw-literal, typescript/only-throw-error -- falsy worker rejections are the behavior under coverage; the reply protocol keys on `error`, not the value
  throw 0;
}

/**
 Generator throwing after one value, driving the iterable failure path.
 
 @returns Yielded values before throwing.
 */
function * failAfter(): Generator<number> {
  yield 1;
  throw new Error('coverage driver iterable failed',);
}

//endregion Fixtures

//region Helpers

/**
 Runs a thunk that is expected to throw, swallowing every outcome so the
 driver keeps exercising remaining paths. Worker replies key on `error`,
 so falsy and non-`Error` rejections are expected driver traffic, not
 defects.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() {
   getResult({ error: 1, id: 0, });
 });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (swallowed) {
    void swallowed;
  }
}

/**
 Awaits a thunk that is expected to reject, swallowing every outcome so the
 driver keeps exercising remaining paths. Worker replies key on `error`,
 so falsy and non-`Error` rejections are expected driver traffic, not
 defects.
 
 @param thunk - Async operation expected to reject.
 
 @example
 ```ts
 await swallowAsync(async function bad(): Promise<void> {
   await Promise.reject(new Error('boom',),);
 });
 ```
 */
async function swallowAsync(thunk: () => Promise<void>,): Promise<void> {
  try {
    await thunk();
  }
  catch (swallowed) {
    void swallowed;
  }
}

//endregion Helpers

//region Exercise

/**
 Exercises host detection, worker sources, reply settlement, single calls
 (success, failure, falsy values, abort, unclonable arguments, baseUrl),
 iterations (success, failure, abort), and direct worker creation.
 
 @example
 ```ts
 await exercise();
 ```
 */
async function exercise(): Promise<void> {
  //region Detection and sources

  if (!isNodeRuntime())
    throw new Error('coverage driver expects the Node.js runtime',);
  if (workerThreadsSpecifier() !== 'node:worker_threads')
    throw new Error('coverage driver specifier diverged; driver inputs are stale',);
  if (errorReporterSource.length === 0)
    throw new Error('coverage driver reporter diverged; driver inputs are stale',);
  if (!nodeWorkerPreambleSource.includes('node:worker_threads',))
    throw new Error('coverage driver preamble diverged; driver inputs are stale',);
  if (!makeCallWorkerBody('function probe() {}',)
    .includes('function probe() {}',))
    throw new Error('coverage driver call body diverged; driver inputs are stale',);
  if (!makeIterableWorkerBody('function * probe() {}',)
    .includes('function * probe() {}',))
    throw new Error('coverage driver iterable body diverged; driver inputs are stale',);
  if (workerConstructorFrom({ Worker: globalThis.Worker, },) !== globalThis.Worker)
    throw new Error('coverage driver constructor diverged; driver inputs are stale',);
  /**
   Blob URL revoked after the check.
   */
  const url = blobUrl('globalThis.postMessage({});',);
  URL.revokeObjectURL(url,);

  //endregion Detection and sources

  //region Settlement

  /* oxlint-disable typescript/no-unsafe-type-assertion -- the driver asserts restored values against fixed inputs */
  if ((getResult({
    id: 1,
    output: DOUBLED_VALUE,
  },) as number) !== DOUBLED_VALUE)
    throw new Error('coverage driver result diverged; driver inputs are stale',);
  if ((getResult({
    id: 2,
    output: 0,
  },) as number) !== 0)
    throw new Error('coverage driver falsy result diverged; driver inputs are stale',);
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  swallow(function falsyFailure() {
    getResult({
      error: 0,
      id: 3,
    },);
  },);
  swallow(function namedFailure() {
    getResult({
      error: new Error('boom',),
      errorName: 'UnicornError',
      id: 4,
    },);
  },);
  swallow(function propertiedFailure() {
    getResult({
      error: new Error('boom',),
      errorProperties: { code: 'ENOENT', },
      id: 5,
    },);
  },);
  /**
   Message payload read off the Node.js event shape.
   */
  const message = getMessageData({
    value: {
      id: MESSAGE_REPLY_ID,
      output: 1,
    },
  },);
  if (message.id !== MESSAGE_REPLY_ID)
    throw new Error('coverage driver message diverged; driver inputs are stale',);

  //endregion Settlement

  //region Single calls

  /**
   Doubled value through the wrapped function.
   */
  const doubled = await makeAsynchronous({ fn: double, },)({ args: [DOUBLE_ARGUMENT], },);
  if (doubled !== DOUBLED_VALUE)
    throw new Error('coverage driver call diverged; driver inputs are stale',);

  await swallowAsync(async function throwingCall(): Promise<void> {
    await makeAsynchronous({ fn: fail, },)({ args: [], },);
  },);

  await swallowAsync(async function falsyCall(): Promise<void> {
    await makeAsynchronous({ fn: throwZero, },)({ args: [], },);
  },);

  /**
   Falsy value resolving through the wrapped function.
   */
  const zero = await makeAsynchronous({
    // oxlint-disable-next-line unicorn/consistent-function-scoping -- distinct serialized fixtures per driver section; hoisting would share one source across sections
    fn: function identity(value: number,): number {
      return value;
    },
  },)({ args: [0], },);
  if (zero !== 0)
    throw new Error('coverage driver falsy call diverged; driver inputs are stale',);

  await swallowAsync(async function preAbortedCall(): Promise<void> {
    /**
     Controller aborted before the call starts.
     */
    const controller = new AbortController();
    controller.abort(new Error('Aborted',),);
    await makeAsynchronous({ fn: double, },)
      .withSignal(controller.signal,)({ args: [1], },);
  },);

  await swallowAsync(async function unclonableCall(): Promise<void> {
    await makeAsynchronous({
      // oxlint-disable-next-line unicorn/consistent-function-scoping -- distinct serialized fixtures per driver section; hoisting would share one source across sections
      fn: function identity(value: unknown,): unknown {
        return value;
      },
    },)({
      // oxlint-disable-next-line unicorn/consistent-function-scoping -- distinct serialized unclonable marker; hoisting would share it across driver sections
      args: [function unclonable(): void {
        throw new Error('unclonable marker',);
      },],
    },);
  },);

  /**
   Call through the string `baseUrl` shape.
   */
  const based = await makeAsynchronous({
    fn: double,
    options: { baseUrl: 'file:///fixture.js', },
  },)({ args: [BASED_ARGUMENT], },);
  if (based !== BASED_VALUE)
    throw new Error('coverage driver baseUrl call diverged; driver inputs are stale',);

  /**
   Call through the `URL` `baseUrl` shape.
   */
  const basedUrl = await makeAsynchronous({
    fn: double,
    options: { baseUrl: new URL('file:///fixture.js',), },
  },)({ args: [BASED_URL_ARGUMENT], },);
  if (basedUrl !== BASED_URL_VALUE)
    throw new Error('coverage driver URL call diverged; driver inputs are stale',);

  //endregion Single calls

  //region Iterations

  /**
   Values drained through the wrapped generator.
   */
  const values: unknown[] = [];
  for await (const value of makeAsynchronousIterable({ fn: pair, },)({ args: [], }))
    values.push(value,);
  if (values.length !== 2)
    throw new Error('coverage driver iteration diverged; driver inputs are stale',);

  await swallowAsync(async function failingIteration(): Promise<void> {
    for await (const ignored of makeAsynchronousIterable({ fn: failAfter, },)({ args: [], }))
      void ignored;
  },);

  await swallowAsync(async function preAbortedIteration(): Promise<void> {
    /**
     Controller aborted before iteration starts.
     */
    const controller = new AbortController();
    controller.abort(new Error('Aborted',),);
    for await (const ignored of makeAsynchronousIterable({ fn: pair, },)
      .withSignal(controller.signal,)({ args: [], }))
      void ignored;
  },);

  //endregion Iterations

  //region Direct worker

  await swallowAsync(async function preAbortedWorker(): Promise<void> {
    /**
     Controller aborted before creation starts.
     */
    const controller = new AbortController();
    controller.abort(new Error('Aborted',),);
    await createWorker({
      content: 'globalThis.postMessage({});',
      nodePreamble: nodeWorkerPreambleSource,
      signal: controller.signal,
    },);
  },);

  /**
   Live worker answering one request, then terminated.
   */
  const worker = await createWorker({
    content: 'globalThis.onmessage = async ({data: {id}}) => { globalThis.postMessage({id, output: 1}); };',
    nodePreamble: nodeWorkerPreambleSource,
  },);
  /**
   Reply of the single request.
   */
  const reply = await worker.request([],);
  if (reply.id !== 1)
    throw new Error('coverage driver worker diverged; driver inputs are stale',);
  worker.cleanup();

  //endregion Direct worker
}

//endregion Exercise

await exercise();
