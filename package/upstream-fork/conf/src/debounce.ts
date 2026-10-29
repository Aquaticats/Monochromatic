/**
 Trailing-edge debounce for change coalescing.
 
 Replaces upstream `conf`'s `debounce-fn` dependency: each trigger restarts
 the wait,
 and the wrapped call runs once after the triggers stop.
 
 @module
 */

//region Types

/**
 One debounced call wrapper.
 
 @example
 ```ts
 const debounced = debounce({ fn: handleChange, wait: 100, });
 debounced.trigger();
 debounced.cancel();
 ```
 */
export type DebouncedCall = {
  /**
   Restarts the wait; the wrapped call runs `wait` ms after the last
   trigger.
   */
  readonly trigger: () => void;
  /**
   Drops any pending run so the wrapped call never fires for stale events.
   */
  readonly cancel: () => void;
};

//endregion Types

//region Factory

/**
 Wraps a call so rapid triggers collapse into one trailing run.
 
 @param fn - Call to coalesce; invoked with no arguments.
 
 @param wait - Quiet period in milliseconds before the run.
 
 @returns Wrapper exposing `trigger` and `cancel`.
 
 @example
 ```ts
 const debounced = debounce({
   fn: function refresh(): void {
     console.log('refreshed');
   },
   wait: 100,
 });
 debounced.trigger();
 debounced.trigger();
 // 'refreshed' logs once, 100 ms after the second trigger
 ```
 */
export function debounce({
  fn,
  wait,
}: {
  readonly fn: () => void;
  readonly wait: number;
},): DebouncedCall {
  /**
   Pending trailing run; its absence means nothing is scheduled.
   */
  const pending: {
    timer?: ReturnType<typeof setTimeout>;
  } = {};
  return {
    trigger: function trigger(): void {
      if (pending.timer !== undefined)
        clearTimeout(pending.timer,);
      pending.timer = setTimeout(
        function runAfterQuietPeriod(): void {
          delete pending.timer;
          fn();
        },
        wait,
      );
    },
    cancel: function cancel(): void {
      if (pending.timer === undefined)
        return;
      clearTimeout(pending.timer,);
      delete pending.timer;
    },
  };
}

//endregion Factory
