/**
 Internal logger error reporting helpers.
 
 Logger internals cannot report failures through the logger itself without
 risking recursion, so these helpers format caught values and write directly
 to the host console.
 
 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { neutralizeControlCharacters, } from './sink/console-control-chars.ts';

/**
 Report fallback directly because a sink cannot recursively log through itself.

 @param directory - verified destination receiving records
 
 @param reason - evidence explaining why the project directory is unavailable

 @example
 ```ts
 reportLoggerFallback({ directory: '/fixture-state/log', reason: 'Project is read-only' });
 ```
 */
export function reportLoggerFallback({
  directory,
  reason,
}: {
  readonly directory: string;
  readonly reason: string;
},): void {
  if ((typeof console) === 'undefined')
    return;
  /**
   Host console method avoids recursive sink discovery while reporting fallback.
   */
  const report = (typeof console.warn) === 'function'
    ? console.warn
    : ((typeof console.error) === 'function' ? console.error : console.log);
  if ((typeof report) !== 'function')
    return;
  /**
   Diagnostic paths and caught values remain readable without executing terminal control sequences.
   */
  const message = `logger file sink: ${reason}; using fallback ${JSON.stringify(directory,)}`;
  report.call(
    console,
    neutralizeControlCharacters(message,),
  );
}

/**
 Reports a logger-internal caught value without going back through logger
 sinks, formatting it via {@link caughtValueText}.
 
 @param context - Human-readable operation that caught the value.
 
 @param error - Caught value to include in the diagnostic.
 
 @mutates error - `caughtValueText` may invoke string-conversion hooks.
 
 @example
 ```ts
 reportLoggerInternalError({
   context: 'console sink verify failed',
   error: new Error('blocked'),
 });
 ```
 */
export function reportLoggerInternalError(
  {
    context,
    error,
  }: {
    readonly context: string;
    readonly error: unknown;
  },
): void {
  if ((typeof console) === 'undefined')
    return;

  // Internal failures cannot go through a sink; use whatever host console
  // method is available without assuming a browser-style `console.warn`.
  /**
   Available console method for reporting the internal failure.
   */
  /**
   Host console method avoids recursive sink discovery while reporting fallback.
   */
  const report = (typeof console.warn) === 'function'
    ? console.warn
    : ((typeof console.error) === 'function' ? console.error : console.log);
  if ((typeof report) !== 'function')
    return;

  /**
   Internal failures cross the same terminal boundary as ordinary sink records.
   */
  const message = `logger internal error: ${context}: ${caughtValueText(error,)}`;
  report.call(
    console,
    neutralizeControlCharacters(message,),
  );
}
