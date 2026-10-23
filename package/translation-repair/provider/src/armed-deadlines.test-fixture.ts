/**
 Fixture that reads which per-call deadlines a client armed, for the cases that pin the length of a deadline without
 waiting for it. Nothing here reaches a provider.

 @module
 */

/**
 Shortest delay the fixture reports, in milliseconds: the per-call deadlines are minutes long, and shorter delays
 belong to pacing and backoff timers the case does not ask about.
 */
const DEADLINE_FLOOR_MS = 100_000;

/**
 The two-argument form of the timer function that a per-call deadline arms with.
 */
type TimerFunction = (
  callback: Parameters<typeof globalThis.setTimeout>[0],
  delay?: number,
) => ReturnType<typeof globalThis.setTimeout>;

/**
 Puts a function in the place of the global `setTimeout` until the returned handle is disposed.

 @param replacement - function that takes the timer function's place

 @returns Handle whose disposal puts the original back, however the scope ends

 @example
 ```ts
 using restore = replacingSetTimeout({ replacement: recordingSetTimeout, },);
 ```
 */
function replacingSetTimeout(
  { replacement, }: { readonly replacement: TimerFunction; },
): Disposable {
  /**
   The timer function the process had before the replacement.
   */
  const original = globalThis.setTimeout;

  /**
   Puts a timer function in the global slot.

   @param timerFunction - function to install

   @example
   ```ts
   install(original,);
   ```
   */
  function install(timerFunction: TimerFunction,): void {
    Object.defineProperty(
      globalThis,
      'setTimeout',
      {
        value: timerFunction,
        configurable: true,
        writable: true,
      },
    );
  }
  install(replacement,);
  return {
    [Symbol.dispose](): void {
      install(original,);
    },
  };
}

/**
 Runs work and reports every timer of a deadline's length that the work armed, in the order armed.

 THE GLOBAL `setTimeout` IS WRAPPED FOR THE LENGTH OF THE WORK AND PUT BACK AFTER IT, however it ends; the wrapper
 passes every call through, so the deadline's own timer still arms and is cleared by the client's disposal. A case
 that uses this runs alone in its file, since the wrapper sees every timer of the process.

 @param run - work to run

 @returns Work's result and the delays armed at or above the fixture's floor

 @example
 ```ts
 const { armed, } = await armedDeadlinesDuring({ run: async () => await askCat({ client, modelId, },), },);
 ```
 */
export async function armedDeadlinesDuring<Result,>(
  { run, }: { readonly run: () => Promise<Result>; },
): Promise<{
  readonly result: Result;
  readonly armed: readonly number[];
}> {
  /**
   Delays seen, in the order armed.
   */
  const armed: number[] = [];

  /**
   The timer function the process had before the wrapper.
   */
  const original = globalThis.setTimeout;

  /**
   Holds the original timer function out of the global slot until the scope ends. The replacement records the delay
   of a timer long enough to be a deadline, then arms it as usual.
   */
  using _restore = replacingSetTimeout({
    replacement: function recordingSetTimeout(
      callback,
      delay,
    ) {
      if ((delay !== undefined) && (delay >= DEADLINE_FLOOR_MS))
        armed.push(delay,);
      return original(
        callback,
        delay,
      );
    },
  },);
  return {
    result: await run(),
    armed,
  };
}
