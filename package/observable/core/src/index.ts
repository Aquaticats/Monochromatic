/**
 Observable value containers with method-based get and set and change notification.
 
 Two variants: {@link createObservable} (synchronous handler, void setter) and
 {@link createObservableAsync} (awaitable handler, promise-returning setter).
 Both update state before invoking the change handler.
 
 The internal `create-observable-core.ts` builds the one container both variants wrap; its
 factory reaches the entry only as `@internal`, for its own test.
 
 @packageDocumentation
 */

export { createObservable, } from './create-observable.ts';
export type { Observable, } from './create-observable.ts';

export { createObservableAsync, } from './create-observable-async.ts';
export type { ObservableAsync, } from './create-observable-async.ts';

/**
 @internal
 */
export {
  createObservableCore,
  type ObservableContainer,
} from './create-observable-core.ts';
