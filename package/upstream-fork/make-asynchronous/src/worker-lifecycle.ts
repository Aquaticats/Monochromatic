/**
 Worker lifecycle for the make-asynchronous fork: host detection, worker
 creation on Node.js and browsers, single-flight request tracking, abort
 handling, and reply routing.
 
 Derived from [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by Sindre Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README
 for the full attribution. Behavior matches `make-asynchronous` 2.1.0: one
 request in flight per worker, stray messages dropped, worker failures
 remembered and replayed to the pending request, abort delivered as the
 signal's reason, and cleanup terminating the worker exactly once.
 
 @module
 */

import type { MakeAsynchronousOptions, } from './options.ts';

//region Host detection

/**
 Whether the current host reports a Node.js runtime.
 
 Upstream reads `globalThis.process?.versions?.node`; the fork keeps the
 probe so browser bundles never touch `node:worker_threads`. The probe
 reads through `globalThis` so the `process` binding stays host-optional.
 
 @returns True on Node.js, false on browsers.
 
 @example
 ```ts
 isNodeRuntime(); // => true
 ```
 */
export function isNodeRuntime(): boolean {
  return ((typeof process) !== 'undefined')
    && ((typeof process.versions) === 'object')
    && ((typeof process.versions
      ?.node) === 'string');
}

/**
 Worker_threads import specifier, assembled at runtime so bundlers never see
 the `node:worker_threads` literal and try to bundle it for browsers.
 
 @returns Specifier resolving to the Node.js worker_threads module.
 
 @example
 ```ts
 workerThreadsSpecifier(); // => "node:worker_threads"
 ```
 */
export function workerThreadsSpecifier(): string {
  return [
    'node:',
    'worker_threads',
  ].join('',);
}

//endregion Host detection

//region Worker handles

/**
 Minimal main-thread worker handle the lifecycle drives.
 
 Covers both hosts with one shape: Node.js `worker_threads` workers expose
 EventEmitter methods while Web Workers expose `addEventListener`. Both
 expose `postMessage` and `terminate`.
 
 @example
 ```ts
 const handle: WorkerHandle = worker as WorkerHandle;
 ```
 */
export type WorkerHandle = {
  /**
   Posts one `{id, arguments_}` request to the worker.
   */
  readonly postMessage: (message: WorkerRequest,) => unknown;
  /**
   Stops the worker without waiting for pending replies.
   */
  readonly terminate: () => void;
};

/**
 Node.js worker handle: `worker_threads` workers are EventEmitters.
 
 @example
 ```ts
 const worker: NodeWorkerHandle = nodeWorker;
 ```
 */
export type NodeWorkerHandle = WorkerHandle & {
  /**
   Node.js EventEmitter subscription.
   */
  readonly on: (
    event: string,
    listener: WorkerListener,
  ) => void;
};

/**
 One request posted to the worker: the reply id plus the cloned arguments.
 
 @example
 ```ts
 const message: WorkerRequest = { arguments_: [2], id: 1, };
 ```
 */
export type WorkerRequest = {
  /**
   Arguments cloned into the worker and spread into the wrapped function.
   Absent on later iterable pulls, which skip re-cloning.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- later iterable pulls carry no arguments, so absence must travel as a value exactly like upstream passing `undefined`
  readonly arguments_: readonly unknown[] | undefined;
  /**
   Request the worker's reply answers.
   */
  readonly id: number;
};

/**
 Listener receiving the raw event value from either host.
 
 @example
 ```ts
 const listener: WorkerListener = function onReply(message): void {};
 ```
 */
export type WorkerListener = (message: unknown,) => void;

/**
 Constructor shape producing a {@link WorkerHandle}.
 
 @example
 ```ts
 const ctor: WorkerConstructor = Worker;
 ```
 */
export type WorkerConstructor = new (
  source: string | URL,
  options?: {
    readonly eval?: boolean;
    readonly type?: string;
    readonly workerData?: {
      // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Node.js worker_threads WorkerOptions.workerData, whose absent baseUrl is explicitly `undefined` so the preamble's `workerData.baseUrl` check reads false
      readonly baseUrl?: string | undefined;
    };
  },
) => WorkerHandle;

/**
 Live worker plus its single-flight request channel.
 
 @example
 ```ts
 const live: LiveWorker = await createWorker({ content, options, });
 ```
 */
export type LiveWorker = {
  /**
   Detaches the abort listener, revokes the browser blob URL, and
   terminates the worker. Runs on every exit from the call path through the
   `using` disposer.
   */
  readonly cleanup: () => void;
  /**
   Posts one request and resolves with the worker's raw reply envelope.
   Rejects with the remembered failure when the worker already failed, and
   throws synchronously when the arguments cannot be cloned.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- later iterable pulls carry no arguments, so absence must travel as a value exactly like upstream passing `undefined`
  readonly request: (requestArguments: readonly unknown[] | undefined,) => Promise<WorkerReply>;
  /**
   Disposer terminating the worker when the call path exits.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Raw reply envelope posted by the worker: either `{id, output}` or
 `{id, error, ...}`, matched to its request by `id`.
 
 @example
 ```ts
 const reply: WorkerReply = { id: 1, output: 42, };
 ```
 */
export type WorkerReply = {
  /**
   Request the reply answers; matched against the pending request.
   */
  readonly id?: unknown;
  /**
   Call result, present on success.
   */
  readonly output?: unknown;
  /**
   Failure value, present when the worker reports a failure.
   */
  readonly error?: unknown;
  /**
   Error name traveling beside the cloned error.
   */
  readonly errorName?: unknown;
  /**
   Extra own error properties traveling beside the cloned error.
   */
  readonly errorProperties?: unknown;
};

//endregion Worker handles

//region Extractors

/**
 Reads the message payload off either host's event shape.
 
 A `worker_threads` message arrives as the raw value while a Web Worker
 wraps it in a `MessageEvent`.
 
 @param message - Raw event value from the subscribed listener.
 
 @returns Reply envelope carried by the event.
 
 @example
 ```ts
 getMessageData({ data: reply, }); // => reply
 ```
 */
export function getMessageData(message: WorkerMessage,): WorkerReply {
  if (isNodeRuntime()) {
    /**
     Reply envelope delivered raw on Node.js hosts.
     */
    const reply = message.value;
    if (isWorkerReply(reply,))
      return reply;
    throw new TypeError('Worker message is not a reply envelope',);
  }
  /**
   Reply envelope wrapped in a MessageEvent on browsers.
   */
  const event = message.value;
  if (isMessageEvent(event,))
    return event.data;
  throw new TypeError('Worker message is not a reply envelope',);
}

/**
 Tests whether a value is a worker reply envelope: an object carrying
 either an `output` or an `error` key.
 
 @param value - Value delivered to the listener.
 
 @returns Whether the value is shaped like a worker reply.
 
 @example
 ```ts
 isWorkerReply({ id: 1, output: 42, }); // => true
 ```
 */
export function isWorkerReply(value: unknown,): value is WorkerReply {
  return ((typeof value) === 'object')
    && ((value !== null))
    && ((Object.hasOwn(
      value,
      'output',
    )) || (Object.hasOwn(
      value,
      'error',
    )));
}

/**
 Tests whether a value is a browser message event wrapping a reply.
 
 @param value - Value delivered to the listener.
 
 @returns Whether the value wraps a reply in its `data` member.
 
 @example
 ```ts
 isMessageEvent({ data: reply, }); // => true
 ```
 */
export function isMessageEvent(value: unknown,): value is { readonly data: WorkerReply; } {
  if (((typeof value) !== 'object') || ((value === null))
    || (!('data' in value)))
    return false;
  /**
   Wrapped payload that must itself be a reply envelope.
   */
  const data: unknown = (value as { readonly data: unknown; }).data;
  return isWorkerReply(data,);
}

/**
 One subscribed worker message: the raw value the listener receives.
 
 `worker_threads` delivers the reply directly while Web Workers wrap it in
 a `MessageEvent`; the name is fixed to `message` at every call site.
 
 @example
 ```ts
 const event: WorkerMessage = { value: reply, };
 ```
 */
export type WorkerMessage = {
  /**
   Raw value delivered to the listener.
   */
  readonly value: unknown;
};

/**
 Reads the `Worker` constructor off a dynamically imported
 `worker_threads` namespace.
 
 @param namespace - Module namespace from the dynamic import.
 
 @returns Worker constructor for the Node.js host.
 
 @example
 ```ts
 workerConstructorFrom(await import('node:worker_threads',));
 ```
 */
export function workerConstructorFrom(namespace: unknown,): WorkerConstructor {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- worker_threads module shape is host-provided
  return (namespace as { readonly Worker: WorkerConstructor; }).Worker;
}

/**
 Builds a blob URL holding one worker module source for browsers.
 
 @param content - Worker module source.
 
 @returns Object URL for the source blob.
 
 @mutates content - File API `URL.createObjectURL` retains the blob in
 the host blob URL store until `URL.revokeObjectURL` removes it.
 
 @example
 ```ts
 blobUrl('globalThis.postMessage({});',);
 ```
 */
export function blobUrl(content: string,): string {
  /**
   Blob holding the worker module source.
   */
  const blob = new globalThis.Blob(
    [content,],
    { type: 'text/javascript', },
  );
  return URL.createObjectURL(blob,);
}

//endregion Extractors

//region Creation

/**
 Creates one worker and its single-flight request channel.
 
 The abort check runs after the dynamic import (the only `await` before the
 worker exists), so an abortion at any point before the worker exists means
 it is never created.
 
 @param content - Worker module source.
 
 @param nodePreamble - Node.js preamble shimmed in front of the body.
 
 @param options - Base URL wiring for bare dynamic imports.
 
 @param signal - Abort signal failing the worker with its reason.
 
 @returns Live worker with cleanup and request.
 
 @mutates signal - `signal.addEventListener` retains the abort callback
 until cleanup removes it.
 
 @example
 ```ts
 const worker = await createWorker({ content, options, });
 ```
 */
export async function createWorker(
  {
    content,
    nodePreamble,
    options,
    signal,
  }: {
    readonly content: string;
    readonly nodePreamble: string;
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 createWorker default parameter `options = {}`, whose explicitly-undefined callers must still typecheck
    readonly options?: MakeAsynchronousOptions | undefined;
    // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors upstream make-asynchronous 2.1.0 createWorker `signal` parameter, whose explicitly-undefined callers must still typecheck
    readonly signal?: AbortSignal | undefined;
  },
): Promise<LiveWorker> {
  /**
   Worker constructor for this host: dynamically imported
   `worker_threads` on Node.js, the global on browsers.
   */
  const WorkerCtor = isNodeRuntime()
    ? workerConstructorFrom(
      await import(workerThreadsSpecifier(),),
    )
    : globalThis.Worker;

  // Checked after the only `await`, so an abortion at any point before the
  // worker exists means it is never created.
  signal?.throwIfAborted();

  /**
   Browser blob URL revoked by cleanup; empty while on Node.js.
   */
  const revokeState = { url: '', };
  /**
   Live worker handle once created.
   */
  const holder: { worker?: WorkerHandle; } = {};
  /**
   Whether the worker already failed; a worker can fail with any value,
   including a falsy one, so the value cannot tell whether it did.
   */
  const failureState = {
    failure: undefined as unknown,
    hasFailed: false,
  };
  /**
   Request waiting for its reply; at most one is ever in flight because a
   call makes one and an iteration waits for each reply before asking for
   the next value.
   */
  const pending: {
    request?: {
      readonly id: number;
      readonly reject: (reason: unknown,) => void;
      readonly resolve: (reply: WorkerReply,) => void;
    };
  } = {};

  /**
   Remembers the worker's failure so it can still be surfaced, and keeps a
   permanent listener so a late failure never becomes an uncaught exception.
   
   @param value - Failure value from the worker surface.
   */
  function rememberFailure(value: unknown,): void {
    if (failureState.hasFailed)
      return;

    failureState.hasFailed = true;
    failureState.failure = value;

    pending.request
      ?.reject(value,);
    delete pending.request;
  }

  /**
   Remembers one listener-delivered failure value.
   
   @param value - Failure value from the worker surface.
   */
  function rememberFailureValue(value: unknown,): void {
    rememberFailure(value,);
  }

  /**
   Remembers one worker exit as a failure naming its code.
   
   @param value - Exit code reported by the worker.
   */
  function rememberExit(value: unknown,): void {
    rememberFailure(new Error(`Worker exited with code ${String(value,)}`,),);
  }

  /**
   Treats abortion as another way for the worker to fail.
   */
  function abort(): void {
    rememberFailure(signal?.reason,);
  }

  /**
   Detaches the abort listener, revokes the browser blob URL, and terminates
   the worker.
   */
  function cleanup(): void {
    signal?.removeEventListener(
      'abort',
      abort,
    );

    if (revokeState.url !== '')
      URL.revokeObjectURL(revokeState.url,);

    holder.worker
      ?.terminate();
  }

  if (isNodeRuntime()) {
    /**
     Base URL string for bare-import resolution, or `undefined` when the
     caller passed no base URL.
     */
    const baseUrl = options?.baseUrl === undefined
      ? undefined
      : String(options.baseUrl,);
    holder.worker = new WorkerCtor(
      `${nodePreamble}${content}`,
      {
        eval: true,
        workerData: { baseUrl, },
      },
    );
  }
  else {
    revokeState.url = blobUrl(content,);

    try {
      holder.worker = new WorkerCtor(
        revokeState.url,
        { type: 'module', },
      );
    }
    catch (error) {
      // `cleanup()` is never handed out when the worker cannot be created,
      // for example because of a content security policy, so the URL is
      // revoked here.
      URL.revokeObjectURL(revokeState.url,);
      throw error;
    }
  }

  signal?.addEventListener(
    'abort',
    abort,
  );

  // A stray message that no request is waiting for is dropped, which keeps
  // the wrapped function from taking over the message channel.
  /**
   Routes one reply to its pending request by id.
   
   @param value - Raw value delivered to the listener.
   */
  function routeReply(value: unknown,): void {
    /**
     Reply envelope carried by the event.
     */
    const data = getMessageData({ value, },);

    if ((pending.request !== undefined) && (data?.id
      === pending.request
        .id)) {
      pending.request
        .resolve(data,);
      delete pending.request;
    }
  }

  if (isNodeRuntime()) {
    /**
     Live worker narrowed to its Node.js shape on this host.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the `isNodeRuntime` guard proves the Node.js shape, which the host-provided union cannot express
    const nodeWorker = holder.worker as NodeWorkerHandle;
    nodeWorker.on(
      'message',
      routeReply,
    );
    nodeWorker.on(
      'error',
      rememberFailureValue,
    );
    nodeWorker.on(
      'messageerror',
      rememberFailureValue,
    );
    // A worker can also stop without reporting an error, for example by
    // calling `process.exit()`. Node.js delivers the messages the worker
    // posted before emitting `exit`, so a reply sent just before stopping
    // still arrives.
    nodeWorker.on(
      'exit',
      rememberExit,
    );
  }
  else {
    /**
     Live worker narrowed to its Web Worker shape on this host.
     */
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- the `isNodeRuntime` guard proves the Web Worker shape, which the host-provided union cannot express
    const webWorker = holder.worker as Worker;
    webWorker.addEventListener(
      'message',
      routeReply,
    );
    webWorker.addEventListener(
      'error',
      rememberFailureValue,
    );
    webWorker.addEventListener(
      'messageerror',
      rememberFailureValue,
    );
  }

  /**
   Request counter assigning each reply its request id. The wrapped
   function shares the worker globals and can post messages of its own, so
   every reply carries the id of the request it answers.
   */
  const counter = { requestCount: 0, };

  /**
   Posts to the worker and waits for the reply, rejecting if the worker
   fails first.
   
   @param requestArguments - Arguments cloned into the worker.
   
   @returns Promise settling with the worker's raw reply envelope.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- later iterable pulls carry no arguments, so absence must travel as a value exactly like upstream passing `undefined`
  function request(requestArguments: readonly unknown[] | undefined,): Promise<WorkerReply> {
    if (failureState.hasFailed) {
      /**
       Remembered worker failure replayed to the late request, verbatim
       including falsy values.
       */
      const remembered: unknown = failureState.failure;
      // oxlint-disable-next-line typescript/prefer-promise-reject-errors -- worker failures are any value by design (upstream remembers falsy failures too); the rejection reason is the failure itself
      return Promise.reject(remembered,);
    }

    counter.requestCount += 1;
    /**
     Id of the request this reply round answers.
     */
    const id = counter.requestCount;

    /**
     Promise settled when the matching reply arrives or the worker fails.
     */
    const {
      promise,
      reject,
      resolve,
    } = Promise.withResolvers<WorkerReply>();
    pending.request = {
      id,
      reject,
      resolve,
    };

    try {
      holder.worker
        ?.postMessage({
          arguments_: requestArguments,
          id,
        },);
    }
    catch (error) {
      // Posting throws for arguments that cannot be cloned, which would
      // leave the wait dangling.
      delete pending.request;
      throw error;
    }

    return promise;
  }

  return {
    [Symbol.dispose]: cleanup,
    cleanup,
    request,
  };
}

//endregion Creation
