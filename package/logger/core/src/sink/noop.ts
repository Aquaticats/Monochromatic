import type {
  LogRecord,
  Sink,
} from '../types.ts';

/**
 Lazy immutable completion adds no asynchronous work at module import.
 */
const memo: { completed?: Promise<void>; } = {};

/**
 Noop verification: the sink is always available, so it never throws nor
 needs setup.
 
 @returns Always-resolved `true`.
 */
function verify(): Promise<boolean> {
  return Promise.resolve(true,);
}

/**
 Discards a log record. Matches the `Sink['write']` signature.
 
 @param _record - Log record to discard.
 */
function write(_record: LogRecord,): Promise<void> {
  // Intentionally discards all logs; resolves immediately to match the async Sink contract.
  memo.completed ??= Promise.resolve();
  return memo.completed;
}

/**
 Builds a noop sink that discards every record and always verifies as
 available. Stateless, so the returned adapters share the same functions;
 the factory shape merely matches the other sinks. Useful as a stand-in
 that disables logging without removing log calls.
 
 @returns Sink that discards all records and exposes no `flush` (nothing
 is buffered).
 
 @example
 ```ts
 const { logger } = createLogger({ sinks: [createNoopSink()] });
 logger.info('goes nowhere');
 ```
 */
export function createNoopSink(): Sink {
  return {
    verify,
    write,
  };
}
