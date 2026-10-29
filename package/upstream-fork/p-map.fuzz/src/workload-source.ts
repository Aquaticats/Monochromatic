/**
 Source instrumentation shared by this package's workload runners: wraps one
 source shape with pull and close telemetry, so both implementations' iterator
 usage becomes comparable.
 
 @module
 */

import type { SourceTelemetry, } from './workload.ts';

//region Instrumentation

  /**
 Opens one instrumented synchronous iterator over the fixed values.
 
 @param values - Values yielded in order; elements may carry promises.
 
 @param telemetry - Telemetry every `next` and `return` writes to.
 
 @returns Iterator recording every `next` and `return`.
 */
function openIterator(
  {
    values,
    telemetry,
  }: {
  /**
   Values yielded in order; elements may carry promises.
   */
    readonly values: readonly (string | Promise<string>)[];
  /**
   Telemetry every `next` and `return` writes to.
   */
    readonly telemetry: SourceTelemetry;
  },
): Iterator<string | Promise<string>> {
  /**
 Pull cursor shared by this iterator's `next` calls.
 */
  const cursor = {
    position: 0,
  };
  return {
    next: function instrumentedNext(): IteratorResult<string | Promise<string>> {
      telemetry.pulls += 1;
      if (cursor.position >= values.length)
        return {
          done: true,
          value: undefined,
        };
    /**
     Value at this cursor position; the bound check above proves it
     present.
     */
      const value = values[cursor.position] ?? '';
      cursor.position += 1;
      return {
        done: false,
        value,
      };
    },
    return: function instrumentedReturn(): IteratorResult<string | Promise<string>> {
      telemetry.closes += 1;
      return {
        done: true,
        value: undefined,
      };
    },
  };
}


/**
 Wraps one source shape with pull and close telemetry.
 
 @param values - Values yielded in order; elements may carry promises.
 
 @param sourceKind - Whether the instrumented source is synchronous or
 asynchronous.
 
 @param telemetry - Telemetry every `next` and `return` writes to.
 
 @returns Iterable or async iterable recording every `next` and `return`.
 
 @example
 ```ts
 const source = instrumentSource({
   values: ['c0',],
   sourceKind: 'sync',
   telemetry: { pulls: 0, closes: 0, },
 });
 ```
 */
export function instrumentSource(
  {
    values,
    sourceKind,
    telemetry,
  }: {
    /**
     Values yielded in order; elements may carry promises.
     */
    readonly values: readonly (string | Promise<string>)[];
    /**
     Whether the instrumented source is synchronous or asynchronous.
     */
    readonly sourceKind: 'sync' | 'async';
    /**
     Telemetry every `next` and `return` writes to.
     */
    readonly telemetry: SourceTelemetry;
  },
): Iterable<string | Promise<string>> | AsyncIterable<string | Promise<string>> {
  if (sourceKind === 'async')
    return {
      [Symbol.asyncIterator]: function openInstrumentedAsync(): AsyncIterator<string | Promise<string>> {
        /**
         Synchronous inner iterator this wrapper records through.
         */
        const inner = openIterator({
          values,
          telemetry,
        },);
        return {
          next: function instrumentedAsyncNext(): Promise<IteratorResult<string | Promise<string>>> {
            return Promise.resolve(inner.next(),);
          },
          return: function instrumentedAsyncReturn(): Promise<IteratorResult<string | Promise<string>>> {
            return Promise.resolve(inner.return?.()
              ?? {
                done: true,
                value: undefined,
              },);
          },
        };
      },
    };

  return {
    [Symbol.iterator]: function openInstrumentedSync(): Iterator<string | Promise<string>> {
      return openIterator({
        values,
        telemetry,
      },);
    },
  };
}

//endregion Instrumentation
