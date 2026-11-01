/**
 Wrangler entry: the only module whose exports workerd inspects.

 workerd rejects a main module that exports anything other than handler
 objects and entrypoint classes ("Incorrect type for map entry"), so the
 library surface lives in `index.ts` and this module exports the handler
 alone.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { handleRequest, } from './index.ts';
import type {
  ExecutionContextLike,
  WorkerEnv,
} from './store.ts';

/**
 Root logger every request handler derives from. The default logger builds
 its sinks on the first log call, which happens inside a handler, so
 importing it here runs nothing in global scope.
 */
const l = tagged({ tag: 'lfs-r2-worker', },);

/**
 Disposable that hands the root logger's flush to `waitUntil` when the
 handler scope ends, on success and failure alike.

 @param ctx - execution context that outlives the response

 @returns disposable whose dispose schedules the flush
 */
function flushOnExit(ctx: ExecutionContextLike,): Disposable {
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
     Flush scheduled when this scope exits.
     */
    using flushAtExit = flushOnExit(ctx,);
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
