import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  wait,
} from '@monochromatic-dev/async-time-core/ts';

import {
  apply,
  NEURALWATT_SETTINGS_NAMESPACE,
} from '../dist/final/node/index.mjs';

//region Test fixtures

/**
 Exact refusal `@deepseek-ai/dsh-hmr` `runExclusive` throws when a settings
 write is issued while the caller's chain already runs inside a reload
 transaction (`dsh-hmr` `lib/index.js`). Quoted here rather than derived from
 the plugin's markers so marker drift fails this test.
 */
const UPSTREAM_NESTED_TRANSACTION_MESSAGE = 'HMR transactions cannot be nested';

/**
 One recorded settings merge write.
 */
type RecordedWrite = {
  ns: string;
  patch: Record<string, unknown>;
};

/**
 Loader-scoped registration context for the plugin's `apply`: a settings seam
 that records every write attempt, optionally refusing each one with a given
 failure, and a listing fetch that never runs because the failed write defers
 before it. Records the write attempts and the event listeners the plugin
 registers.

 @param options - failure every settings write refuses with, or absent for
 writes that land

 @returns fake context plus the attempts and listeners it saw

 @example
 ```ts
 const fake = fakeContextOf({
   refuseWritesWith: new Error('HMR transactions cannot be nested'),
 });
 apply(fake.ctx as never);
 ```
 */
function fakeContextOf({
  refuseWritesWith,
}: {
  readonly refuseWritesWith?: Error;
} = {}): {
  readonly ctx: {
    readonly settings: {
      describe: () => unknown[];
      update: (ns: string, patch: Record<string, unknown>) => Promise<void>;
    };
    readonly events: {
      readonly on: (
        name: string,
        listener: (...args: never[]) => void,
      ) => () => boolean;
      readonly once: (
        name: string,
        listener: (...args: never[]) => void,
      ) => () => boolean;
    };
  };
  readonly attempts: RecordedWrite[];
  readonly listeners: Map<string, ((...args: never[]) => void)[]>;
} {
  /**
   Write attempts the fake saw, refused or not.
   */
  const attempts: RecordedWrite[] = [];
  /**
   Event listeners the plugin registered, keyed by event name.
   */
  const listeners = new Map<string, ((...args: never[]) => void)[]>();
  /**
   Register one listener under an event name, the shared body of both `on` and
   `once` on this fake.
   */
  function registerListener(
    name: string,
    listener: (...args: never[]) => void,
  ): () => boolean {
    /**
     Listeners already registered under this event name.
     */
    const underName = listeners.get(name,) ?? [];
    underName.push(listener,);
    listeners.set(name, underName,);
    return () => true;
  }
  return {
    ctx: {
      settings: {
        describe: () => [],
        update: async function update(ns, patch) {
          attempts.push({ ns, patch, },);
          if (refuseWritesWith !== undefined) throw refuseWritesWith;
        },
      },
      events: {
        on: registerListener,
        once: registerListener,
      },
    },
    attempts,
    listeners,
  };
}

/**
 Settle the plugin's fire-and-forget mount refresh: its deferral catch runs on
 microtasks only, so one timer turn sees it.
 */
async function settleMountRefresh(): Promise<void> {
  await wait(0,);
}

/**
 Guard whose disposal restores the runtime fetch it replaced. Installing it
 makes the apply boundary (which takes no fetch injection) hermetic: a landing
 write's listing step fails like the offline gate instead of reaching the
 network.

 @returns a disposable guard over the replaced `globalThis.fetch`

 @example
 ```ts
 using fetchGuard = unreachableFetchGuard();
 ```
 */
function unreachableFetchGuard(): { readonly [Symbol.dispose]: () => void; } {
  /**
   Runtime fetch this guard replaces until its disposal.
   */
  const realFetch = globalThis.fetch;
  globalThis.fetch = function unreachableFetch(): Promise<Response> {
    return Promise.reject(new Error('network unreachable in apply tests',));
  };
  return {
    [Symbol.dispose]: function restoreFetch(): void {
      globalThis.fetch = realFetch;
    },
  };
}

/**
 Listeners the plugin registered on the plugin-manager's settle event, or a
 loud failure when it registered more than one.

 @param listeners - listeners the fake host saw

 @returns the registered drain listeners

 @example
 ```ts
 drainListenersOf(fake.listeners);
 ```
 */
function drainListenersOf(
  listeners: ReadonlyMap<string, ((...args: never[]) => void)[]>,
): readonly ((...args: never[]) => void)[] {
  /**
   Drain listeners under the plugin-manager's settle event name.
   */
  const drains = listeners.get('plugin-manager/changed',) ?? [];
  expect(drains.length,)
    .toBeLessThanOrEqual(1,);
  return drains;
}

//endregion Test fixtures

await describe({
  name: apply.name,
  children: [
    it({
      name: 'hands a nested reload-transaction write refusal to a plugin-manager settle listener, which re-runs the mount refresh outside that transaction',
      fn: async () => {
        using fetchGuard = unreachableFetchGuard();
        void fetchGuard;
        const fake = fakeContextOf({
          refuseWritesWith: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE,),
        },);
        apply(fake.ctx as never,);
        await settleMountRefresh();
        expect(fake.attempts.length,)
          .toBe(1,);
        expect(fake.attempts[0]
          ?.ns,)
          .toBe(NEURALWATT_SETTINGS_NAMESPACE,);
        /**
         Deferral listener the refused write armed, after the premise check.
         */
        const [drain] = drainListenersOf(fake.listeners,);
        if (drain === undefined) {
          throw new Error('expected one plugin-manager/changed drain listener');
        }
        drain();
        await settleMountRefresh();
        expect(fake.attempts.length,)
          .toBe(2,);
        expect(fake.attempts[1]
          ?.ns,)
          .toBe(NEURALWATT_SETTINGS_NAMESPACE,);
      },
    },),
    it({
      name: 'arms no plugin-manager settle listener when the mount refresh lands without a deferral',
      fn: async () => {
        using fetchGuard = unreachableFetchGuard();
        void fetchGuard;
        const fake = fakeContextOf();
        apply(fake.ctx as never,);
        await settleMountRefresh();
        /**
         The snapshot write landed and the unreachable fetch kept the served
         snapshot, so one attempt ran and no deferral armed a listener.
         */
        expect(fake.attempts.length,)
          .toBe(1,);
        expect(drainListenersOf(fake.listeners,))
          .toEqual([]);
      },
    },),
  ],
},);
