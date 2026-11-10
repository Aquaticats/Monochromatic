/**
 Pins the arrangement behind the sync and async observable pair: both variants wrap one value
 store, so the container, its update ordering, and its snapshot live in a single module.
 
 Rule `TWS` asks a two-sided behaviour to name which arrangement applies. The arrangement chosen
 here is a shared core, and this test reads the factory sources to fail on a hand-copied second
 container or on a declaration that stops naming the core it wraps.
 
 @module
 */

import { readFileSync, } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Reads one module beside this test, so the assertion never depends on the working directory.
 
 @param fileName - Module file name in this folder
 
 @returns module source text
 */
function sourceOf(fileName: string,): string {
  return readFileSync(new URL(fileName, import.meta.url,), 'utf8',);
}

/**
 Sync variant factory source, read once for the assertions below.
 */
const SYNC_SOURCE = sourceOf('./create-observable.ts',);

/**
 Async variant factory source, read once for the assertions below.
 */
const ASYNC_SOURCE = sourceOf('./create-observable-async.ts',);

/**
 The store module the two variants share.
 */
const STORE_SOURCE = sourceOf('./value-store.ts',);

/**
 Both factory sources, each checked by the same assertions.
 */
const FACTORY_SOURCES: readonly string[] = [SYNC_SOURCE, ASYNC_SOURCE,];

await describe({
  name: 'sync and async variants share one value store',
  children: [
    it({
      name: 'each factory imports createValueStore from the store module',
      fn: async () => {
        for (const source of FACTORY_SOURCES) {
          expect(source,).toContain('from \'./value-store.ts\'',);
          expect(source,).toContain('createValueStore',);
        }
      },
    },),

    it({
      name: 'the store module is the only place that declares a value container',
      fn: async () => {
        expect(STORE_SOURCE,).toContain('const state',);
        for (const source of FACTORY_SOURCES)
          expect(source,).not.toContain('const state',);
      },
    },),

    it({
      name: 'each declaration names the shared core it wraps',
      fn: async () => {
        for (const source of FACTORY_SOURCES)
          expect(source,).toContain('{@link createValueStore}',);
      },
    },),
  ],
},);
