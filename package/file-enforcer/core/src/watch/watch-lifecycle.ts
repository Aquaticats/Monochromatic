/**
 Watch-mode lifecycle state and cleanup helpers.
 */
export type WatchModeLifecycle = Readonly<{
  /**
   Promise that rejects when watch mode fails closed.
   */
  failure: Promise<never>;

  /**
   Returns whether watch mode has already failed closed.
   */
  hasFailed: () => boolean;

  /**
   Fails watch mode closed and tears down active state.
   */
  fail: (failureError: unknown) => void;

  /**
   Registers watcher abort controller for later teardown.
   */
  registerController: (args: {
    readonly controller: AbortController;
    readonly dir: string;
  }) => void;

  /**
   Aborts all active watchers and clears their registry.
   */
  closeAllWatchers: () => void;

  /**
   Schedules one debounce timer, clearing any older timer first.
   */
  scheduleDebounce: (args: {
    readonly callback: () => void;
    readonly delayMs: number;
  }) => void;

  /**
   Clears pending debounce timer state.
   */
  clearDebounceTimer: () => void;
}>;

//region Lifecycle factory

/**
 Creates state holder for watcher teardown, debounce cleanup, and fail-closed
 rejection: `fail` tears down active state through
 {@link WatchModeLifecycle.closeAllWatchers} and
 {@link WatchModeLifecycle.clearDebounceTimer} before rejecting.
 
 @returns Watch-mode lifecycle helpers.
 
 @example
 ```ts
 const lifecycle = createWatchModeLifecycle();
 ```
 */
export function createWatchModeLifecycle(): WatchModeLifecycle {
  /**
   Active AbortControllers for each watched directory.
   */
  const controllers = new Map<string, AbortController>();
  /**
   Single-key holder for active debounce timer.
   */
  const debounceTimerHolder = new Map<'timer', ReturnType<typeof setTimeout>>();
  /**
   Promise resolver pair used to fail watch mode closed.
   */
  const watchModeFailure = Promise.withResolvers<never>();
  /**
   Single-key holder marking watch mode as failed.
   */
  const watchModeFailureState = new Map<'failed', true>();

  /**
   Clears active debounce timer.
   
   @example
   ```ts
   clearDebounceTimer();
   ```
   */
  function clearDebounceTimer(): void {
    /**
     Active debounce timer handle, or absent between bursts.
     */
    const activeTimer = debounceTimerHolder.get('timer',);
    if (activeTimer !== undefined)
      clearTimeout(activeTimer,);
    debounceTimerHolder.delete('timer',);
  }

  /**
   Closes every registered watcher controller.
   
   @example
   ```ts
   closeAllWatchers();
   ```
   */
  function closeAllWatchers(): void {
    controllers.forEach(function abortController(controller,): void {
      controller.abort();
    },);
    controllers.clear();
  }

  return {
    failure: watchModeFailure.promise,

    hasFailed(): boolean {
      return watchModeFailureState.has('failed',);
    },

    /**
     Fails watch mode once and closes active watchers.
     
     @param failureError - rejection reason retained by failure promise
     
     @mutates failureError through watchModeFailure.reject rejection retention
     */
    fail(failureError: unknown,): void {
      if (watchModeFailureState.has('failed',))
        return;

      watchModeFailureState.set(
        'failed',
        true,
      );
      clearDebounceTimer();
      closeAllWatchers();
      watchModeFailure.reject(failureError,);
    },

    registerController(
      {
        dir,
        controller,
      }: {
        readonly controller: AbortController;
        readonly dir: string;
      },
    ): void {
      controllers.set(
        dir,
        controller,
      );
    },

    closeAllWatchers,

    scheduleDebounce(
      {
        callback,
        delayMs,
      }: {
        readonly callback: () => void;
        readonly delayMs: number;
      },
    ): void {
      clearDebounceTimer();
      debounceTimerHolder.set(
        'timer',
        setTimeout(
          callback,
          delayMs,
        ),
      );
    },

    clearDebounceTimer,
  };
}

//endregion Lifecycle factory
