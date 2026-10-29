import type { Logger, } from '@monochromatic-dev/logger-core/ts';

//region Capturing logger
// A LOGGER THAT KEEPS EVERY LINE, for cases that read what the code under
// test logged.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Unit test files that kept their own copy
// of these loggers, whatever they called the array, now import them from
// here, including the warning-only logger repair-assemble-withdrawal and
// stage-call each spread over their own tagged logger.

/**
 Drains nothing: every line is kept as it is emitted, so no buffer waits.

 @example
 ```ts
 await flushNothing();
 ```
 */
function flushNothing(): Promise<void> {
  return Promise.resolve();
}

/**
 Logger keeping every line, at any level, in emission order.

 @param messages - array the lines land in

 @returns Logger writing into it

 @example
 ```ts
 const messages: string[] = [];
 const l = capturingLogger({ messages, },);
 ```
 */
export function capturingLogger({ messages, }: { readonly messages: string[]; },): Logger {
  /**
   One level's writer, every level sharing the same array.

   @param message - line as the code under test wrote it
   */
  function keep(message: string,): void {
    messages.push(message,);
  }

  return {
    debug: keep,
    error: keep,
    fatal: keep,
    flush: flushNothing,
    info: keep,
    trace: keep,
    warn: keep,
  };
}

/**
 Logger paired with its own backing array, for cases with no array of their
 own to pass in.

 @returns Logger plus the array every level writes into

 @example
 ```ts
 const { logger, lines, } = capturingLoggerPair();
 ```
 */
export function capturingLoggerPair(): {
  readonly logger: Logger;
  readonly lines: readonly string[];
} {
  /**
   Lines written, in order.
   */
  const lines: string[] = [];

  return {
    lines,
    logger: capturingLogger({ messages: lines, },),
  };
}

/**
 Logger keeping every line behind the level it was written at, for cases
 whose contract includes the level: a warning a log reader looks for is not
 an information line.

 @param lines - array the `level message` lines land in

 @returns Logger writing into it

 @example
 ```ts
 const lines: string[] = [];
 const l = levelCapturingLogger({ lines, },);
 ```
 */
export function levelCapturingLogger({ lines, }: { readonly lines: string[]; },): Logger {
  /**
   Writer for one level, every level sharing the same array.

   @param level - level the writer keeps lines at

   @returns Writer prefixing each line with that level

   @example
   ```ts
   const warn = keepAt({ level: 'warn', },);
   ```
   */
  function keepAt({ level, }: { readonly level: string; },): (message: string) => void {
    return function keep(message: string,): void {
      lines.push(`${level} ${message}`,);
    };
  }

  return {
    debug: keepAt({ level: 'debug', },),
    error: keepAt({ level: 'error', },),
    fatal: keepAt({ level: 'fatal', },),
    flush: flushNothing,
    info: keepAt({ level: 'info', },),
    trace: keepAt({ level: 'trace', },),
    warn: keepAt({ level: 'warn', },),
  };
}

/**
 Logger recording only the warnings a base logger would otherwise emit,
 every other level deferring to that base logger as it is.

 @param base - logger this wraps, read for every level but warn

 @param warnings - array the warn level's messages land in

 @returns Logger behaving as the base logger everywhere but warn

 @example
 ```ts
 const warnings: string[] = [];
 const logger = warningRecordingLogger({ base: l, warnings, },);
 ```
 */
export function warningRecordingLogger(
  {
    base,
    warnings,
  }: {
    readonly base: Logger;
    readonly warnings: string[];
  },
): Logger {
  return {
    ...base,
    warn: function record(message: string,): void {
      warnings.push(message,);
    },
  };
}

//endregion Capturing logger
