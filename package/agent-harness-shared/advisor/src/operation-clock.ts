/**
 Abort-aware operation waits that never require a provider to acknowledge cancellation. @module
 */
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Distinguishes local scheduling cutoffs from actual provider completion.
 */
export const ADVISOR_CLOCK_BOUNDARY: unique symbol = Symbol('advisor/local-scheduling-boundary');

//region Wait lifecycle

/**
 Wait for provider completion, a local scheduling boundary, or caller cancellation.
 
 @param pending - observed provider outcomes; rejections remain observed by the race
 
 @param untilMs - absolute local wakeup boundary
 
 @param signal - caller cancellation capability
 
 @param now - clock used to compute the remaining wait
 
 @returns provider outcome or local-boundary sentinel
 
 @mutates pending - Promise.race attaches settlement observers to provider promises
 
 @mutates signal - installs and removes a caller abort listener
 
 @example
 ```ts
 const event = await waitForAdvisorEvent({ pending, untilMs, signal });
 ```
 */
export async function waitForAdvisorEvent<T>({
  pending,
  untilMs,
  signal,
  now = Date.now,
}: {
  readonly pending: readonly Promise<T>[];
  readonly untilMs: number;
  readonly signal?: ForeignHostCapability<AbortSignal>;
  readonly now?: () => number;
},): Promise<T | typeof ADVISOR_CLOCK_BOUNDARY> {
  /**
   Wakeup primitive shared by the timer and cancellation listener.
   */
  const {
    promise,
    resolve,
  } = Promise.withResolvers<typeof ADVISOR_CLOCK_BOUNDARY>();
  /**
   Resolve the local wait without throwing away already collected reviews.
   */
  function wake(): void {
    resolve(ADVISOR_CLOCK_BOUNDARY,);
  }
  /**
   Timer exists only for this scheduler wait.
   */
  const timer = setTimeout(
    wake,
    Math.max(
      1,
      untilMs - now(),
    ),
  );
  signal?.addEventListener(
    'abort',
    wake,
    { once: true, },
  );
  /**
   Remove both wakeup sources when any branch settles.
   */
  using cleanup = {
    [Symbol.dispose](): void {
      clearTimeout(timer,);
      signal?.removeEventListener(
        'abort',
        wake,
      );
    },
  };
  if (signal?.aborted === true)
    wake();
  return await Promise.race([
    ...pending,
    promise,
  ],);
}

/**
 Own a cancellation capability whose scope exit always requests provider cleanup.
 
 @returns controller and deterministic disposal
 
 @example
 ```ts
 using cancellation = createAdvisorCancellation();
 ```
 */
export function createAdvisorCancellation(): {
  readonly controller: AbortController;
  readonly [Symbol.dispose]: () => void
} {
  /**
   Controller is private to the current operation, never reused between calls.
   */
  const controller = new AbortController();
  return {
    controller,
    [Symbol.dispose](): void { controller.abort(); },
  };
}

//endregion Wait lifecycle
