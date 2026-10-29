/**
 * Tests the no-throw, no-leak guarantees of the probes. Bun's runtime
 * exposes neither `indexedDB`, `navigator.storage.getDirectory`, nor
 * `localStorage` by default, so every probe returns `false` in this
 * environment without further setup. The localStorage tests stub the
 * global to exercise the success path and the throwing path.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { probeStorage, } from './storage-probe.ts';

/**
 * Minimal localStorage shape the probe uses.
 */
type FakeStorage = {
  readonly setItem: (
    key: string,
    value: string,
  ) => void;
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors the Web Storage `Storage.getItem` signature (returns `string | null` for an absent key); the fake must match it to install as `globalThis.localStorage`
  readonly getItem: (key: string,) => string | null;
  readonly removeItem: (key: string,) => void;
};

/**
 * Container for an installed fake localStorage; restores the prior
 * value of `globalThis.localStorage` on `Symbol.dispose`. Use with
 * `using` so the global never leaks across tests.
 */
type StubHandle = Disposable;

/**
 * Installs `fake` as `globalThis.localStorage` and returns a disposable
 * that removes it on scope exit (including throws). Removing rather
 * than restoring is correct here because Bun does not expose
 * `localStorage` natively; the property would not exist before the
 * test ran.
 *
 * @param fake - object exposing the localStorage subset the probe uses
 *
 * @returns disposable that detaches the stub
 */
function stubLocalStorage(fake: FakeStorage,): StubHandle {
  // Reflect.set bypasses the static `Storage` typing of
  // `globalThis.localStorage` without resorting to `any`-casts that
  // trip both no-explicit-any and no-unsafe-member-access. The probe
  // calls the three methods on `FakeStorage`, which is structurally
  // compatible with the parts of `Storage` it touches.
  Reflect.set(
    globalThis,
    'localStorage',
    fake,
  );
  return {
    [Symbol.dispose]: function dispose(): void {
      Reflect.deleteProperty(
        globalThis,
        'localStorage',
      );
    },
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: probeStorage.name,
      concurrency: 1,
      children: [
        it({
          name: 'returns all-false in a Bun env without browser storage globals',
          fn: async () => {
            const caps = await probeStorage();
            expect(caps.idb,).toBe(false,);
            expect(caps.opfs,).toBe(false,);
            expect(caps.localStorage,).toBe(false,);
          },
        },),

        it({
          name: 'returns localStorage=true when a working stub is installed',
          fn: async () => {
            const store = new Map<string, string>();
            using _ = stubLocalStorage({
              setItem(
                key,
                value,
              ) {
                store.set(
                  key,
                  value,
                );
              },
              getItem(key,) {
                return store.get(key,) ?? null;
              },
              removeItem(key,) {
                store.delete(key,);
              },
            },);
            const caps = await probeStorage();
            expect(caps.localStorage,).toBe(true,);
          },
        },),

        it({
          name: 'returns localStorage=false when setItem throws (private mode)',
          fn: async () => {
            using _ = stubLocalStorage({
              setItem() {
                throw new Error('quota exceeded',);
              },
              getItem() {
                return null;
              },
              removeItem() {},
            },);
            const caps = await probeStorage();
            expect(caps.localStorage,).toBe(false,);
          },
        },),

        it({
          name: 'returns localStorage=false when getItem returns the wrong value',
          fn: async () => {
            using _ = stubLocalStorage({
              setItem() {},
              getItem() {
                return 'wrong';
              },
              removeItem() {},
            },);
            const caps = await probeStorage();
            expect(caps.localStorage,).toBe(false,);
          },
        },),

        it({
          name: 'never throws even when storage globals throw',
          fn: async () => {
            using _ = stubLocalStorage({
              setItem(): never {
                throw new Error('boom',);
              },
              getItem(): never {
                throw new Error('boom',);
              },
              removeItem(): never {
                throw new Error('boom',);
              },
            },);
            let caught: unknown = null;
            try {
              await probeStorage();
            }
            catch (error) {
              caught = error;
            }
            expect(caught,).toBeNull();
          },
        },),
      ],
    },),
  ],
},);
