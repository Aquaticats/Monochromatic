import type { Logger, } from '@monochromatic-dev/logger-core/ts';

/**
 A logger that keeps every message per level, so tests can assert which event lines were logged.
 */
export type RecordingLogger = Logger & Readonly<{
  /**
   Messages per level name.
   */
  messages: Readonly<Record<'debug' | 'error' | 'info' | 'warn', string[]>>;
}>;

/**
 Flush for the recording logger: there is nothing buffered to flush.
 */
async function flushNothing(): Promise<void> {
  await Promise.resolve();
}

/**
 Creates a logger that records instead of printing.

 @returns the recording logger

 @example
 ```ts
 const l = recordingLogger();
 l.warn('ALERT x',);
 l.messages.warn; // ['ALERT x']
 ```
 */
export function recordingLogger(): RecordingLogger {
  /**
   Recorded messages per level.
   */
  const messages: Record<'debug' | 'error' | 'info' | 'warn', string[]> = {
    debug: [],
    error: [],
    info: [],
    warn: [],
  };
  /**
   Builds a method that records into one level.

   @param level - level to record into

   @returns the logging method
   */
  function recorder(level: 'debug' | 'error' | 'info' | 'warn',): (message: string,) => void {
    return function record(message: string,): void {
      messages[level]
        .push(message,);
    };
  }
  return {
    trace: recorder('debug',),
    debug: recorder('debug',),
    info: recorder('info',),
    warn: recorder('warn',),
    error: recorder('error',),
    fatal: recorder('error',),
    flush: flushNothing,
    messages,
  };
}
