import { ABSENT, } from './constants.ts';
import type { BackendResult, } from './consensus.ts';

/**
 Heal async backends to the canonical serialized value by deleting or setting mismatches.
 
 Internal helper for the async store; not part of the package public API.
 
 @param results - all backend results
 
 @param canonicalSerialized - canonical serialized value, or {@link ABSENT} when consensus is absence
 
 @param key - key to heal
 
 @example
 ```ts
 await healBackends({ results, canonicalSerialized: '"correct-value"', key: 'my-key' });
 ```
 */
export async function healBackends({
  results,
  canonicalSerialized,
  key,
}: Readonly<{
  results: readonly BackendResult<
    {
      readonly set: (
        key: string,
        value: string,
      ) => unknown;
      readonly delete: (key: string,) => unknown;
    }
    >[];
  canonicalSerialized: string | typeof ABSENT;
  key: string;
}>,): Promise<void> {
  await Promise.all(
    results.map(async function heal({
      value,
      backend,
    },) {
      if (canonicalSerialized === ABSENT) {
        if (value !== ABSENT)
          await backend.delete(key,);
        return;
      }

      if (value !== canonicalSerialized) {
        await backend.set(
          key,
          canonicalSerialized,
        );
      }
    },),
  );
}

/**
 Heal sync backends to the canonical serialized value by deleting or setting mismatches.
 
 Internal helper for the sync store; not part of the package public API.
 
 @param results - all backend results
 
 @param canonicalSerialized - canonical serialized value, or {@link ABSENT} when consensus is absence
 
 @param key - key to heal
 
 @example
 ```ts
 healBackendsSync({ results, canonicalSerialized: '"correct-value"', key: 'my-key' });
 ```
 */
export function healBackendsSync({
  results,
  canonicalSerialized,
  key,
}: Readonly<{
  results: readonly BackendResult<
    {
      readonly set: (
        key: string,
        value: string,
      ) => unknown;
      readonly delete: (key: string,) => unknown;
    }
    >[];
  canonicalSerialized: string | typeof ABSENT;
  key: string;
}>,): void {
  for (const {
    value,
    backend,
  } of results) {
    if (canonicalSerialized === ABSENT) {
      if (value !== ABSENT)
        backend.delete(key,);
      continue;
    }

    if (value !== canonicalSerialized) {
      backend.set(
        key,
        canonicalSerialized,
      );
    }
  }
}
