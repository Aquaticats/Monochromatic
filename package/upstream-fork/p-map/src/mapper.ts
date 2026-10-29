/**
 Input shapes accepted by the concurrent mappers and the mapper call
 signature.
 
 Mirrors upstream `p-map`'s `input` and `Mapper` types: inputs may be
 synchronous or asynchronous iterables of elements or element promises, and
 every iterated item is awaited before its mapper call.
 
 @module
 */

import { MapperRequiredError, } from './errors.ts';
import type { pMapSkip, } from './p-map-skip.ts';

//region Types

/**
 Inputs walked by {@link pMap} and {@link pMapIterable}: a synchronous or
 asynchronous iterable whose elements may themselves be promises, each
 awaited before its mapper call.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @example
 ```ts
 const input: MapInput<number> = [
   1,
   Promise.resolve(2,),
 ];
 ```
 */
export type MapInput<Element> =
  | AsyncIterable<Element | Promise<Element>>
    | Iterable<Element | Promise<Element>>;

/**
 Structural view of {@link MapInput} used to probe iterator availability in
 upstream `p-map`'s access order without narrowing the union first.
 
 @typeParam Element - element type after awaiting an iterated item
 */
type IteratorProbe<Element> = {
  readonly [Symbol.iterator]?: () => Iterator<Element | Promise<Element>>;
  readonly [Symbol.asyncIterator]?: () => AsyncIterator<Element | Promise<Element>>;
};

/**
 Function called for every input element with its position in the input.
 
 May return {@link pMapSkip} to drop that input's slot from the collected
 results.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @typeParam NewElement - mapper result type, awaited when thenable
 
 @example
 ```ts
 const mapper: Mapper<number, string> = function toStringMapper(value: number,): string {
   return String(value,);
 };
 ```
 */
export type Mapper<Element, NewElement> = (
  element: Element,
  index: number,
) => NewElement | typeof pMapSkip | PromiseLike<NewElement | typeof pMapSkip>;

//endregion Types

//region Validation

/**
 Validates one mapper against upstream `p-map`'s acceptance rule: any
 function value passes, everything else fails.
 
 @param mapper - Mapper value as supplied by the caller.
 
 @throws MapperRequiredError when the mapper is not a function.
 
 @example
 ```ts
 validateMapper(function identity(value: number,): number {
   return value;
 });
 ```
 */
export function validateMapper(mapper: unknown,): void {
  if ((typeof mapper) !== 'function')
    throw new MapperRequiredError();
}

//endregion Validation

//region Probes

/**
 Narrows {@link MapInput} to the structural probe both validators and
 iterator selection index into.
 
 One cast at this boundary keeps the public input type equal to upstream
 `p-map`'s union while the probe below reproduces upstream's property-access
 order exactly.
 
 @typeParam Element - element type after awaiting an iterated item
 
 @param iterable - Inputs as typed by the public API.
 
 @returns Probe view carrying both iterator slots as optional members.
 
 @example
 ```ts
 iteratorProbe([1, 2,],)[Symbol.iterator]; // => function
 ```
 */
export function iteratorProbe<Element>(iterable: MapInput<Element>,): IteratorProbe<Element> {
  return iterable as IteratorProbe<Element>;
}

//endregion Probes
