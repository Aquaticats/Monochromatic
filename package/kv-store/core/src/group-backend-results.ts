/**
 Owned grouping for backend consensus records.
 
 @module
 */

import type { BackendResult, } from './consensus.ts';

/**
 Backend result fields accepted as grouping keys.
 */
type BackendResultGroupField = 'priority' | 'value';

/**
 Groups backend results by one scalar consensus field.
 
 @typeParam Field - Result field selected as group key.
 
 @typeParam TBackend - Storage backend type carried by result records.
 
 @param results - Backend results traversed in source order.
 
 @param field - Scalar result field used as group key.
 
 @returns results grouped by selected field.
 
 @example
 ```ts
 const grouped = groupBackendResults({ results, field: 'value' });
 ```
 */
export function groupBackendResults<
  const Field extends BackendResultGroupField,
  TBackend,
  >({
    results,
    field,
  }: {
    readonly results: readonly BackendResult<TBackend>[];
    readonly field: Field;
  },): Map<BackendResult<TBackend>[Field], BackendResult<TBackend>[]> {
  /**
   Mutable local groups built without exposing mutation to callers.
   */
  const grouped = new Map<
    BackendResult<TBackend>[Field],
    BackendResult<TBackend>[]
    >();
  for (const result of results) {
    /**
     Scalar group key read from immutable result record.
     */
    const key = result[field];
    /**
     Existing local bucket when key was already observed.
     */
    const bucket = grouped.get(key,);
    if (bucket === undefined) {
      grouped.set(
        key,
        [result,],
      );
      continue;
    }
    bucket.push(result,);
  }
  return grouped;
}
