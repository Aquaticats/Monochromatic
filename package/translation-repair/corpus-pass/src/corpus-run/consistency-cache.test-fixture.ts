/**
 A store the tests of asking the consistency benches share: it keeps what it is
 handed, hands back what an earlier run settled, and refuses to store when told.
 Cat-themed invention; no corpus content appears here.

 @module
 */

import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { ConsistencyCacheValue, } from '../../dist/final/node/index.mjs';

/**
 A cache that keeps what it is handed, and refuses to store when told.

 @param resumed - what an earlier run settled

 @param persisted - sink for every question and answer handed to the store

 @param refusal - what storing fails with, absent when it works

 @returns The cache

 @example
 ```ts
 const cache = recordingCache({ resumed: new Map(), persisted: [], },);
 ```
 */
export function recordingCache(
  {
    resumed,
    persisted,
    refusal,
  }: {
    readonly resumed: ReadonlyMap<string, ConsistencyCacheValue>;
    readonly persisted: {
      readonly key: string;
      readonly serialized: string
    }[];
    readonly refusal?: Error;
  },
): SliceCache<ConsistencyCacheValue> {
  return {
    resumed,
    persist: function persist(entry,): Promise<void> {
      if (refusal !== undefined)
        return Promise.reject(refusal,);
      persisted.push(entry,);
      return Promise.resolve();
    },
  };
}
