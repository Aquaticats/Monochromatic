/**
 Web Storage and page stand-ins for the coverage driver: an enumerable
 in-memory store, one that overflows its quota, one that fails after its
 first write, and one that throws on every access; installers that swap
 the process globals for a scope; and a page stand-in whose lifecycle
 events the driver can fire.

 @module
 */

import { installGlobalValue, } from './coverage-globals.ts';

//region Shapes

/**
 What `Storage.getItem` and `Storage.key` answer, taken from the DOM
 contract the stand-ins implement rather than declared here: the sinks
 call these members through the `Storage` type, so the stand-ins must
 answer the way the platform does.
 */
type StorageAnswer = ReturnType<Storage['getItem']>;

/**
 Page visibility states the buffered sinks distinguish.
 */
type Visibility = 'hidden' | 'visible';

/**
 Installed page stand-in: disposing removes it; the methods fire the
 lifecycle events the buffered sinks listen for.
 */
export type FakePage = Disposable & {
  /**
   Fires `pagehide` on the global.
   */
  readonly firePageHide: () => void;
  /**
   Sets the document's visibility, then fires `visibilitychange`.
   */
  readonly fireVisibility: (options: { readonly state: Visibility; },) => void;
};

//endregion Shapes

//region Stores

/**
 In-memory `Storage` with enumeration, so a sink's prior-run scan sees the
 keys it holds.

 @returns Storage stand-in.

 @example
 ```ts
 using _local = installFakeLocalStorage({ fake: createMemoryStorage() });
 ```
 */
export function createMemoryStorage(): Storage {
  /**
   Backing map in insertion order.
   */
  const backing = new Map<string, string>();
  /**
   Stand-in implementing the `Storage` surface over the map.
   */
  const storage = {
    /**
     Stored key count.
     */
    get length(): number {
      return backing.size;
    },
    clear(): void {
      backing.clear();
    },
    getItem(key: string,): StorageAnswer {
      return backing.get(key,) ?? null;
    },
    key(index: number,): StorageAnswer {
      return [...backing.keys(),][index] ?? null;
    },
    removeItem(key: string,): void {
      backing.delete(key,);
    },
    setItem(
      key: string,
      value: string,
    ): void {
      backing.set(
        key,
        value,
      );
    },
  };
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- The literal implements every Storage member the sinks call.
  return storage as unknown as Storage;
}

/**
 In-memory `Storage` that throws the browser's `QuotaExceededError` once the
 stored value lengths would exceed `budget`, so a sink's reactive eviction
 runs.

 @param budget - Total value length accepted before overflowing.

 @returns Storage stand-in.

 @example
 ```ts
 using _session = installFakeSessionStorage({ fake: createQuotaStorage({ budget: 2000 }) });
 ```
 */
export function createQuotaStorage({ budget, }: { readonly budget: number; },): Storage {
  /**
   Backing store.
   */
  const inner = createMemoryStorage();
  /**
   Occupied value length.
   */
  const used = { chars: 0, };
  /**
   Stand-in layering the quota over the memory store.
   */
  const storage = {
    /**
     Stored key count.
     */
    get length(): number {
      return inner.length;
    },
    clear(): void {
      inner.clear();
      used.chars = 0;
    },
    getItem(key: string,): StorageAnswer {
      return inner.getItem(key,);
    },
    key(index: number,): StorageAnswer {
      return inner.key(index,);
    },
    removeItem(key: string,): void {
      used.chars -= inner.getItem(key,)
        ?.length
        ?? 0;
      inner.removeItem(key,);
    },
    setItem(
      key: string,
      value: string,
    ): void {
      /**
       Occupied length after this write replaces any prior value.
       */
      const next = (used.chars - (inner.getItem(key,)
        ?.length
        ?? 0)) + value.length;
      if (next > budget)
        throw new DOMException(
          'exceeded the quota',
          'QuotaExceededError',
        );
      inner.setItem(
        key,
        value,
      );
      used.chars = next;
    },
  };
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- The literal implements every Storage member the sinks call.
  return storage as unknown as Storage;
}

/**
 In-memory `Storage` whose first `setItem` succeeds and every later one
 throws a non-quota error, so a sink's give-up report runs and its
 once-per-episode gate is exercised.

 @returns Storage stand-in.

 @example
 ```ts
 using _session = installFakeSessionStorage({ fake: createFlakyStorage() });
 ```
 */
export function createFlakyStorage(): Storage {
  /**
   Backing store.
   */
  const inner = createMemoryStorage();
  /**
   Writes attempted so far.
   */
  const calls = { setItem: 0, };
  /**
   Stand-in failing after the first write.
   */
  const storage = {
    /**
     Stored key count.
     */
    get length(): number {
      return inner.length;
    },
    clear(): void {
      inner.clear();
    },
    getItem(key: string,): StorageAnswer {
      return inner.getItem(key,);
    },
    key(index: number,): StorageAnswer {
      return inner.key(index,);
    },
    removeItem(key: string,): void {
      inner.removeItem(key,);
    },
    setItem(
      key: string,
      value: string,
    ): void {
      calls.setItem += 1;
      if (calls.setItem > 1)
        throw new Error('storage disabled mid-session',);
      inner.setItem(
        key,
        value,
      );
    },
  };
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- The literal implements every Storage member the sinks call.
  return storage as unknown as Storage;
}

/**
 Shared failure for every member of the refusing store; always throws.
 */
function refuse(): never {
  throw new Error('storage access denied',);
}

/**
 `Storage` whose every member throws, so a sink's verify probe fails and
 reports.

 @returns Storage stand-in.

 @example
 ```ts
 using _session = installFakeSessionStorage({ fake: createThrowingStorage() });
 ```
 */
export function createThrowingStorage(): Storage {
  /**
   Stand-in refusing every access.
   */
  const storage = {
    /**
     Stored key count; refused like every other member.
     */
    get length(): number {
      return refuse();
    },
    clear: refuse,
    getItem: refuse,
    key: refuse,
    removeItem: refuse,
    setItem: refuse,
  };
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- The literal implements every Storage member the sinks call.
  return storage as unknown as Storage;
}

//endregion Stores

//region Installers

/**
 Installs `fake` as `globalThis.localStorage` through its property
 descriptor (the real global is an accessor); disposing restores the
 original descriptor or removes the property when none existed.

 @param fake - Storage stand-in to install.

 @returns Disposable restoring the previous state.

 @example
 ```ts
 using _local = installFakeLocalStorage({ fake: createMemoryStorage() });
 ```
 */
export function installFakeLocalStorage({ fake, }: { readonly fake: Storage; },): Disposable {
  return installGlobalValue({
    name: 'localStorage',
    value: fake,
  },);
}

/**
 Swaps `globalThis.sessionStorage` for `fake`; disposing restores the real
 backend.

 @param fake - Storage stand-in to install.

 @returns Disposable restoring the original.

 @example
 ```ts
 using _session = installFakeSessionStorage({ fake: createMemoryStorage() });
 ```
 */
export function installFakeSessionStorage({ fake, }: { readonly fake: Storage; },): Disposable {
  /**
   Real backend to restore.
   */
  const original = globalThis.sessionStorage;
  globalThis.sessionStorage = fake;
  return {
    [Symbol.dispose](): void {
      globalThis.sessionStorage = original;
    },
  };
}

/**
 Installs a minimal page (a global `addEventListener` and a `document` with
 a visibility state) so the buffered sinks register their page-lifecycle
 flush hooks, and lets the driver fire those events; disposing removes
 both globals or restores what was there.

 @returns Installed page stand-in.

 @example
 ```ts
 using page = installFakePage();
 page.fireVisibility({ state: 'hidden' });
 ```
 */
export function installFakePage(): FakePage {
  /**
   Listeners registered on the global, by event type.
   */
  const globalListeners = new Map<string, (() => void)[]>();
  /**
   Listeners registered on the document, by event type.
   */
  const documentListeners = new Map<string, (() => void)[]>();
  /**
   Mutable document stand-in; the visibility state changes when the
   driver fires an event.
   */
  const fakeDocument: {
    visibilityState: Visibility;
    readonly addEventListener: (
      type: string,
      listener: () => void,
    ) => void;
  } = {
    addEventListener(
      type: string,
      listener: () => void,
    ): void {
      documentListeners.set(
        type,
        [
          ...(documentListeners.get(type,) ?? []),
          listener,
        ],
      );
    },
    visibilityState: 'visible',
  };
  /**
   Restores the global listener slot when disposed.
   */
  const globalSlot = installGlobalValue({
    name: 'addEventListener',
    value: function addGlobalListener(
      type: string,
      listener: () => void,
    ): void {
      globalListeners.set(
        type,
        [
          ...(globalListeners.get(type,) ?? []),
          listener,
        ],
      );
    },
  },);
  /**
   Restores the document slot when disposed.
   */
  const documentSlot = installGlobalValue({
    name: 'document',
    value: fakeDocument,
  },);
  return {
    firePageHide(): void {
      for (const listener of globalListeners.get('pagehide',) ?? [])
        listener();
    },
    fireVisibility({ state, }: { readonly state: Visibility; },): void {
      fakeDocument.visibilityState = state;
      for (const listener of documentListeners.get('visibilitychange',) ?? [])
        listener();
    },
    [Symbol.dispose](): void {
      documentSlot[Symbol.dispose]();
      globalSlot[Symbol.dispose]();
    },
  };
}

//endregion Installers
