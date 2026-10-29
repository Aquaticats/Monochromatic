import type { Logger, } from './types.ts';

//region Callback observation does not grant telemetry control over an operation

/**
 Fixed callback names carry no thrown value, message body or sink-delivery claim.
 
 @example
 ```ts
 const callback: LoggerCallbackName = 'warn';
 ```
 */
export type LoggerCallbackName = keyof Logger;

/**
 Canonical ordering is independent of callback invocation and failure order.
 */
const CALLBACK_ORDER: Readonly<Record<LoggerCallbackName, number>> = {
  debug: 0,
  error: 1,
  fatal: 2,
  flush: 3,
  info: 4,
  trace: 5,
  warn: 6,
};

/**
 Forwarding preserves caller tags, message order and receiver without eagerly reading callbacks.
 Snapshots report abnormal completion, not whether a sink stored a message.
 
 @example
 ```ts
 const observed = observeLoggerCallbacks(l);
 observed.logger.warn('operation refused');
 const callbacks = observed.snapshot();
 ```
 */
export type LoggerCallbackObservation = {
  /**
   Only callback exceptions are isolated; blocking, process exit and external mutations remain possible.
   */
  readonly logger: Logger;
  /**
   Returns an immutable detached snapshot; repeated failures occupy only one fixed callback-name slot.
   */
  readonly snapshot: () => readonly LoggerCallbackName[];
};

/**
 Observes caller logger exceptions without turning telemetry into operation control flow.
 Every requested message is forwarded once with its original receiver and existing tags.
 Synchronous callback/getter throws and awaited flush rejection become fixed-name observations.
 No thrown value is inspected, retained, serialized or used as a cause.
 Void logging methods must obey their synchronous contract; an unobserved rejecting Promise is not covered.
 No flush, retry, timer, sink or default logger is introduced.
 
 @param l - caller-owned logger whose callbacks remain lazy and receiver-bound
 
 @returns Forwarding logger and bounded immutable callback-observation snapshots
 
 @example
 ```ts
 const observed = observeLoggerCallbacks(l);
 observed.logger.info('persisted output');
 const abnormalCallbacks = observed.snapshot();
 ```
 */
export function observeLoggerCallbacks(l: Logger): LoggerCallbackObservation {
  /**
   Membership is restricted to Logger's fixed callback names, never arbitrary caught data.
   */
  const failures = new Set<LoggerCallbackName>();

  /**
   Logging primitives deliberately preserve existing tags rather than add a wrapper-specific prefix.
   
   @param method - fixed synchronous Logger method
   
   @returns Receiver-preserving message forwarding with observable exception containment
   */
  function forward(method: Exclude<LoggerCallbackName, 'flush'>): (message: string) => void {
    /**
     Both method lookup and invocation remain inside the same exception boundary.
     
     @param message - already serialized caller message forwarded unchanged
     */
    return function forwardMessage(message: string): void {
      try {
        l[method](message);
      }
      catch (error) {
        // Deliberately discard the value without inspection; retain its fixed abnormal-completion event.
        void error;
        failures.add(method);
      }
    };
  }

  /**
   Flush remains explicit; property access, invocation, thenable adoption and rejection are observed together.
   The requested attempt completes without propagating its rejection.
   */
  async function flush(): Promise<void> {
    try {
      await l.flush();
    }
    catch (error) {
      // Deliberately discard the value; the snapshot records rejection without recursively invoking a failed logger.
      void error;
      failures.add('flush');
    }
  }

  /**
   Snapshots neither expose mutable membership nor change after subsequent logger activity.
   
   @returns Canonically ordered frozen callback names observed so far
   */
  function snapshot(): readonly LoggerCallbackName[] {
    return Object.freeze([...failures].toSorted(function compareCallbacks(
      left: LoggerCallbackName,
      right: LoggerCallbackName,
    ): number {
      return CALLBACK_ORDER[left] - CALLBACK_ORDER[right];
    }));
  }

  return Object.freeze({
    logger: Object.freeze({
      debug: forward('debug'),
      error: forward('error'),
      fatal: forward('fatal'),
      flush,
      info: forward('info'),
      trace: forward('trace'),
      warn: forward('warn'),
    }),
    snapshot,
  });
}

//endregion Callback observation does not grant telemetry control over an operation
