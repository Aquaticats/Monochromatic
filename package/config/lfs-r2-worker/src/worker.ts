/**
 Wrangler entry: the only module whose exports workerd inspects.

 workerd rejects a main module that exports anything other than handler
 objects and entrypoint classes ("Incorrect type for map entry"), so the
 library surface lives in `index.ts` and this module exports the handler
 alone.

 @module
 */

import {
  createLogger,
  type Logger,
  sinks,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { handleRequest, } from './index.ts';
import type {
  ExecutionContextLike,
  WorkerEnv,
} from './store.ts';

/**
 Memo for the root logger. Empty until the first request, so importing this
 module runs no sink verification: the verification uses timers, which workerd
 forbids in global scope (issue #493).
 */
const memo: { root?: Logger; } = {};

/**
 Root logger every request handler derives from, built on the first request.

 It writes to the console sink alone. The default logger also probes
 IndexedDB, sessionStorage and localStorage, none of which exist in workerd, and
 that discovery runs inside the first request of every new isolate, where the
 Workers Free plan allows 10 ms of CPU in total (2 to 4 ms measured live).

 @returns logger tagged for this Worker
 */
function rootLogger(): Logger {
  if (memo.root !== undefined) {
    return memo.root;
  }
  /**
   Logger writing to the console sink alone.
   */
  const { logger, } = createLogger({ sinks: [sinks.createConsoleSink(),], },);
  memo.root = tagged({
    tag: 'lfs-r2-worker',
    l: logger,
  },);
  return memo.root;
}

/**
 Parameters for {@link flushOnExit}.
 */
type FlushOnExitParams = {
  /**
   Execution context that outlives the response.
   */
  readonly ctx: ExecutionContextLike;
  /**
   Root logger whose flush is scheduled.
   */
  readonly l: Logger;
};

/**
 Disposable that hands the root logger's flush to `waitUntil` when the
 handler scope ends, on success and failure alike.

 @param ctx - execution context that outlives the response

 @param l - root logger whose flush is scheduled

 @returns disposable whose dispose schedules the flush
 */
function flushOnExit({
  ctx,
  l,
}: FlushOnExitParams,): Disposable {
  return {
    [Symbol.dispose](): void {
      ctx.waitUntil(l.flush(),);
    },
  };
}

/**
 Worker entry wrangler invokes for every request.
 */
const worker = {
  /**
   Route the request, logging and rethrowing any unexpected failure so the
   platform reports it as a 500 instead of hiding it.

   @param request - inbound HTTP request

   @param env - bindings and secrets from `wrangler.toml`

   @param ctx - execution context whose `waitUntil` keeps the log flush alive
     past the response

   @returns response for the matched route

   @example
   ```ts
   export default worker;
   ```
   */
  async fetch(
    request: Request,
    env: WorkerEnv,
    ctx: ExecutionContextLike,
  ): Promise<Response> {
    /**
     Root logger, built by the first request of this isolate.
     */
    const l = rootLogger();
    /**
     Flush scheduled when this scope exits.
     */
    using flushAtExit = flushOnExit({
      ctx,
      l,
    },);
    try {
      return await handleRequest({
        request,
        env,
        l,
      },);
    }
    catch (error) {
      l.error(`unhandled failure for ${request.method} ${request.url}: ${String(error,)}`,);
      throw error;
    }
  },
};

export default worker;
