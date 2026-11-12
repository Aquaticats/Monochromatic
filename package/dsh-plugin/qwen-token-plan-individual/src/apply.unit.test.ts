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
  QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,
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
 failure, and a credentials service whose reference resolution always fails.
 The failed resolution ends the mount refresh before its listing fetch, so no
 network call and no mount-race poll can happen in a test.

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
    readonly get: () => { readonly resolve: () => Promise<undefined> };
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
      get: () => ({
        resolve: async function resolve(): Promise<undefined> {
          /**
           Reference resolution failure the refresh turns into its typed
           credential failure and stops on.
           */
          throw new Error('credential reference resolution fails in every fake host',);
        },
      }),
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
        const fake = fakeContextOf({
          refuseWritesWith: new Error(UPSTREAM_NESTED_TRANSACTION_MESSAGE,),
        },);
        apply(fake.ctx as never,);
        await settleMountRefresh();
        expect(fake.attempts.length,)
          .toBe(1,);
        expect(fake.attempts[0]
          ?.ns,)
          .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,);
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
          .toBe(QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE,);
      },
    },),
    it({
      name: 'arms no plugin-manager settle listener when the mount refresh lands without a deferral',
      fn: async () => {
        const fake = fakeContextOf();
        apply(fake.ctx as never,);
        await settleMountRefresh();
        expect(fake.attempts.length,)
          .toBe(1,);
        expect(drainListenersOf(fake.listeners,))
          .toEqual([]);
      },
    },),
  ],
},);
