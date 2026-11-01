import { createDefaultSinks, } from '#default-sinks';

import { createLogger, } from './create-logger.ts';

import type {
  Level,
  Logger,
} from './types.ts';

/**
 Default logger instance and its readiness promise, built on first use.
 */
type DefaultInstance = {
  readonly initPromise: Promise<void>;
  readonly logger: Logger;
};

/**
 Memo for the default instance. Empty until the first log or flush call, so
 importing this module (or `tagged`, which reaches it) runs no sink
 discovery: no timers, no I/O, no storage probes. Runtimes that forbid those
 in global scope (Cloudflare Workers, issue #493) therefore pay nothing at
 import and verify their sinks inside whatever handler logs first.
 */
const memo: { current?: DefaultInstance; } = {};

/**
 Builds the default instance on first use and returns it afterwards.

 @returns Default logger and its readiness promise.

 @example
 ```ts
 const { logger } = defaultInstance();
 ```
 */
function defaultInstance(): DefaultInstance {
  memo.current ??= createLogger({ sinks: createDefaultSinks(), },);
  return memo.current;
}

/**
 Builds one level method that forwards to the default instance, creating it
 on the first call.

 @param level - Severity the method logs at.

 @returns Forwarding level method.
 */
function forward(level: Level,): (message: string,) => void {
  return function logAtLevel(message: string,): void {
    defaultInstance()
      .logger[level](message,);
  };
}

/**
 Awaits the default instance's own `flush`, creating the instance first so a
 flush before any log still verifies the sinks and drains them.
 */
async function flush(): Promise<void> {
  await defaultInstance()
    .logger
    .flush();
}

/**
 Multi-sink logger that writes to all available backends, built lazily on
 the first call. Startup records replay to async sinks that verify after
 the log call. Log calls throw only when initialization proves no backend is
 available, which the console sink prevents in every supported runtime.
 `flush()` awaits verification internally, so no readiness promise is
 exported: awaiting one at module top level was the mistake this design
 removes.

 @example
 ```ts
 import { logger, } from '\@monochromatic-dev/logger-core';

 logger.error('unexpected shutdown',);
 await logger.flush();
 ```
 */
export const logger: Logger = {
  debug: forward('debug',),
  error: forward('error',),
  fatal: forward('fatal',),
  flush,
  info: forward('info',),
  trace: forward('trace',),
  warn: forward('warn',),
};
