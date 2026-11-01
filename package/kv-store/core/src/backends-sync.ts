/**
 Backend querying for synchronous stores. Internal helper; not part of the
 package public API.
 */

import { ABSENT, } from './constants.ts';
import type { BackendResult, } from './consensus.ts';
import type { SyncStorageBackend, } from './types.ts';

/**
 Query all sync backends for a key and return typed results with priority info.
 
 @param backends - sync storage backends to query
 
 @param key - lookup key
 
 @returns results from all backends
 
 @example
 ```ts
 const results = queryAllBackendsSync({ backends, key: 'my-key' });
 ```
 */
export function queryAllBackendsSync({
  backends,
  key,
}: Readonly<{
  backends: readonly [
    SyncStorageBackend,
    ...SyncStorageBackend[],
  ];
  key: string;
}>,): [
  BackendResult<SyncStorageBackend>,
  ...BackendResult<SyncStorageBackend>[],
] {
  /**
   Per-backend query results assembled in priority-respecting order.
   */
  const results = backends.map(function queryBackend(backend,) {
    /**
     Raw value returned by this backend before nullish normalisation.
     */
    const raw = backend.get(key,);
    return {
      value: raw ?? ABSENT,
      priority: backend.priority
        ?? 0,
      backend,
    };
  },);
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- non-empty backends guarantees non-empty results
  return results as [
    BackendResult<SyncStorageBackend>,
    ...BackendResult<SyncStorageBackend>[],
  ];
}
