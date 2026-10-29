/**
 Test-only clock helpers shared by this package's unit tests.
 
 Upstream `quick-lru` reads `Date.now()` directly, so the fork does too and
 expiry tests control time by installing a fake `Date.now`. The fake clock
 replaces a global, so every suite that installs one runs sequentially
 (`describe({ concurrency: 1 })`), and `using` hands the real clock back at
 scope exit.
 
 @module
 */

//region Fake clock

/**
 Controllable replacement for `Date.now`: reads return the fake time until
 `restore` or disposal puts the real clock back.
 
 @example
 ```ts
 using clock = installFakeClock({ startMilliseconds: 1_000, });
 cache.set({ key: 'a', value: 1, maxAge: 500, },);
 clock.advance(600,);
 cache.get('a',); // => undefined
 ```
 */
export type FakeClock = {
  /**
   Current fake time in milliseconds since the epoch.
   */
  readonly now: number;
  /**
   Moves fake time forward (or back) by the given amount.
   */
  readonly advance: (milliseconds: number,) => void;
  /**
   Puts the real `Date.now` back, exactly as disposal does.
   */
  readonly restore: () => void;
  /**
   Puts the real `Date.now` back at `using` scope exit.
   */
  readonly [Symbol.dispose]: () => void;
};

/**
 Installs a fake `Date.now` reading from a controllable holder.
 
 @param options - Fake time to start at; realistic values keep `expiry`
 timestamps away from upstream's falsy-expiry quirks unless a test aims at
 them deliberately.
 
 @returns Clock handle whose `restore` and disposal hand the real `Date.now`
 back.
 
 @example
 ```ts
 using clock = installFakeClock({ startMilliseconds: 1_700_000_000_000, });
 clock.advance(250,);
 ```
 */
export function installFakeClock(options: {
  /**
   Fake time to start at, in milliseconds since the epoch.
   */
  readonly startMilliseconds: number,
},): FakeClock {
  /**
   Real `Date.now`, captured so restoration is exact.
   */
  const realDateNow = Date.now;
  /**
   Fake time holder; a container because repository lint bans function-root
   `let` bindings.
   */
  const time = {
    milliseconds: options.startMilliseconds,
  };

  /**
   Replacement for `Date.now` reading the fake time holder.
   
   @returns Current fake time in milliseconds since the epoch.
   */
  function readFakeNow(): number {
    return time.milliseconds;
  }

  /**
   Puts the real `Date.now` back.
   */
  function restore(): void {
    Date.now = realDateNow;
  }

  Date.now = readFakeNow;

  return {
    /**
     Current fake time in milliseconds since the epoch.
     */
    get now(): number {
      return time.milliseconds;
    },
    advance: function advance(milliseconds: number,): void {
      time.milliseconds += milliseconds;
    },
    restore,
    [Symbol.dispose]: restore,
  };
}

//endregion Fake clock
