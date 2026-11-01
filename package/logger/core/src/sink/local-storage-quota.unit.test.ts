import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _detectLocalStorageQuotaChars as detectLocalStorageQuotaChars,
} from '@monochromatic-dev/logger-core';

/**
 Temporarily sets `globalThis` keys to the supplied values, restoring each to
 its prior value (or deleting keys that were absent) when the returned guard
 leaves `using` scope, so a runtime-detection test can impersonate Deno, Bun,
 or a browser without leaking the fake globals into later tests.
 
 @param overrides - Global keys to install for the duration of the scope.
 
 @returns Disposable that restores the original globals on exit.
 */
function withGlobalOverrides(overrides: Record<string, unknown>,): Disposable {
  const host = globalThis as unknown as Record<string, unknown>;
  const saved = Object.entries(overrides,)
    .map(function captureAndSet([key, value,],) {
      const had = key in host;
      const prior = host[key];
      host[key] = value;
      return {
        key,
        had,
        prior,
      };
    },);
  return {
    [Symbol.dispose](): void {
      for (const { key, had, prior, } of saved) {
        if (had)
          host[key] = prior;
        else
          // `delete host[key]` (dynamic key) is banned; this removes the key so
          // an absent-before global (e.g. `Deno`) does not leak into later tests.
          Reflect.deleteProperty(host, key,);
      }
    },
  };
}

await describe({
  name: detectLocalStorageQuotaChars.name,
  // Serial because every test mutates process-global runtime markers.
  concurrency: 1,
  children: [
    it({
      name: 'reads node web storage as 5 MiB',
      fn: async () => {
        expect(detectLocalStorageQuotaChars(),)
          .toBe(5_242_880,);
      },
    },),

    it({
      name: 'reads Deno web storage as its measured just-under-10-MiB figure',
      fn: async () => {
        using _override = withGlobalOverrides({ Deno: {}, },);
        expect(detectLocalStorageQuotaChars(),)
          .toBe(10_477_569,);
      },
    },),

    it({
      name: 'leaves Bun uncapped since it exposes no localStorage',
      fn: async () => {
        using _override = withGlobalOverrides({ Bun: {}, },);
        expect(detectLocalStorageQuotaChars(),)
          .toBe(Number.POSITIVE_INFINITY,);
      },
    },),

    it({
      name: 'reads a browser engine as 5 MiB',
      fn: async () => {
        // No node markers, but a DOM: the browser bucket, measured on Chromium
        // and assumed shared by Firefox and WebKit.
        using _override = withGlobalOverrides({ process: undefined, document: {}, },);
        expect(detectLocalStorageQuotaChars(),)
          .toBe(5_242_880,);
      },
    },),

    it({
      name: 'leaves an unrecognized runtime uncapped',
      fn: async () => {
        using _override = withGlobalOverrides({ process: undefined, },);
        expect(detectLocalStorageQuotaChars(),)
          .toBe(Number.POSITIVE_INFINITY,);
      },
    },),
  ],
},);
