import { logger as defaultLogger, } from './logger.ts';
import type { Logger, } from './types.ts';

/**
 Wraps a logger so every message is prefixed with `[tag] `.
 Callers typically pass `myFn.name` as tag to keep prefixes
 in sync with refactors.
 
 @param tag - Prefix string inserted before each message
 
 @param l - Base logger to wrap; defaults to the module-level {@link logger}
   singleton
 
 @returns Logger whose methods prepend `[tag] ` to every message
 
 @example
 ```ts
 import { tagged } from '\@monochromatic-dev/logger-core/tagged';
 
 function handleRequest({ l }: { l: Logger }): void {
   l.info('received');
 }
 
 handleRequest({ l: tagged({ tag: handleRequest.name }) });
 // logs: [handleRequest] received
 ```
 
 @example
 ```ts
 // Composing tags: the outermost wrap (`l2` here) prepends to the message
 // last, so its tag ends up rightmost. The innermost wrap (`l1`) hits the
 // underlying logger first, so its tag is leftmost. The chain reads
 // root-first: outer wrap = inner tag position.
 const l1 = tagged({ tag: 'http' });
 const l2 = tagged({ tag: 'retry', l: l1 });
 l2.info('attempt 3');
 // logs: [http] [retry] attempt 3
 ```
 */
export function tagged({
  tag,
  l = defaultLogger,
}: {
  readonly l?: Logger;
  readonly tag: string;
},): Logger {
  /**
   Bracketed tag prepended to every message; built once so each log call does one concatenation.
   */
  const prefix = `[${tag}] `;
  return {
    debug: function debug(message: string,): void {
      l.debug(`${prefix}${message}`,);
    },
    error: function error(message: string,): void {
      l.error(`${prefix}${message}`,);
    },
    fatal: function fatal(message: string,): void {
      l.fatal(`${prefix}${message}`,);
    },
    flush: function flush(): Promise<void> {
      return l.flush();
    },
    info: function info(message: string,): void {
      l.info(`${prefix}${message}`,);
    },
    trace: function trace(message: string,): void {
      l.trace(`${prefix}${message}`,);
    },
    warn: function warn(message: string,): void {
      l.warn(`${prefix}${message}`,);
    },
  };
}
