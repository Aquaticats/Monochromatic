/**
 Presence index: which oids the R2 bucket holds, answered in one D1 query.

 One R2 `head` per object costs 0.25 to 0.45 ms of CPU, so a batch of 100
 exceeds the 10 ms the Workers Free plan allows per request. A D1 query that
 matches 500 oids at once costs 0 to 2 ms (measured on the live account, see
 `doc/troubleshooting/lfs-r2-worker-error-1102.md`). R2 stays the source of
 truth: a row proves the object exists, a missing row proves nothing, and the
 callers confirm misses against R2 where an absent object would be reported.

 @module
 */

import type {
  DatabaseRow,
  WorkerEnv,
} from './store.ts';

/**
 SQL selecting the indexed subset of a JSON array of oids (`?1`).
 */
export const SELECT_PRESENT_SQL = 'SELECT oid FROM present WHERE oid IN (SELECT value FROM json_each(?1))';

/**
 SQL recording every oid of a JSON array (`?1`) as present; idempotent.
 */
export const INSERT_PRESENT_SQL = 'INSERT OR IGNORE INTO present (oid) SELECT value FROM json_each(?1)';

/**
 Most oids sent in one query. 500 oids serialize to about 33 KB, well under
 D1's statement size limit, and one batch rarely exceeds it.
 */
export const INDEX_CHUNK_SIZE = 500;

/**
 Answer of an index lookup.
 */
export type PresenceLookup = IndexedLookup | UnavailableLookup;

/**
 Answer when the environment has no index, so the caller must ask the bucket.
 */
export type UnavailableLookup = {
  /**
   Discriminant.
   */
  readonly kind: 'unavailable';
};

/**
 Answer when the index responded.
 */
export type IndexedLookup = {
  /**
   Discriminant.
   */
  readonly kind: 'indexed';
  /**
   Oids the index holds; every other oid is unconfirmed, not necessarily absent.
   */
  readonly known: ReadonlySet<string>;
};

/**
 Split `oids` into runs of at most {@link INDEX_CHUNK_SIZE}.

 @param oids - oids to split

 @returns consecutive runs covering `oids` in order
 */
function chunkOids(oids: readonly string[],): readonly (readonly string[])[] {
  return Array.from(
    { length: Math.ceil(oids.length / INDEX_CHUNK_SIZE,), },
    function sliceChunk(
      _unused: unknown,
      chunkIndex: number,
    ): readonly string[] {
      return oids.slice(
        chunkIndex * INDEX_CHUNK_SIZE,
        (chunkIndex + 1) * INDEX_CHUNK_SIZE,
      );
    },
  );
}

/**
 Read the `oid` column of one index row.

 The column arrives untyped because D1 hands back whatever the table holds, and a
 row without a text oid would otherwise become the string `undefined` in the set
 of present oids, promising an object the bucket does not hold. Throw instead, so
 the caller falls back to asking the bucket.

 @param row - row as the database handed it back

 @returns the oid the row names

 @throws TypeError when the column is not text
 */
function oidOfRow(row: DatabaseRow,): string {
  /**
   The `oid` column.
   */
  const { oid, } = row;
  if ((typeof oid) !== 'string') {
    throw new TypeError('presence index row has no text oid',);
  }
  return oid;
}

/**
 Parameters for {@link lookupPresent}.
 */
export type LookupPresentParams = {
  /**
   Worker env that may carry the index.
   */
  readonly env: WorkerEnv;
  /**
   Oids to look up.
   */
  readonly oids: readonly string[];
};

/**
 Ask the index which of `oids` the bucket holds.

 @param env - Worker env that may carry the index

 @param oids - oids to look up

 @returns the indexed subset, or `unavailable` when the env has no index

 @throws whatever the D1 binding throws, so the caller can log it and fall back to the bucket

 @example
 ```ts
 const lookup = await lookupPresent({ env, oids });
 ```
 */
export async function lookupPresent({
  env,
  oids,
}: LookupPresentParams,): Promise<PresenceLookup> {
  /**
   Index database, when bound.
   */
  const { INDEX: database, } = env;
  if (database === undefined) {
    return { kind: 'unavailable', };
  }
  /**
   One result set per chunk.
   */
  const answers = await Promise.all(chunkOids(oids,)
    .map(async function queryChunk(chunk: readonly string[],) {
      return await database
        .prepare(SELECT_PRESENT_SQL,)
        .bind(JSON.stringify(chunk,),)
        .all();
    },),);
  return {
    kind: 'indexed',
    known: new Set(answers.flatMap(function toOids(answer,) {
      return answer.results
        .map(function toOid(row,): string {
          return oidOfRow(row,);
        },);
    },),),
  };
}

/**
 Parameters for {@link recordPresent}.
 */
export type RecordPresentParams = {
  /**
   Worker env that may carry the index.
   */
  readonly env: WorkerEnv;
  /**
   Oids the bucket is known to hold.
   */
  readonly oids: readonly string[];
};

/**
 Record `oids` as present in the index; a no-op when the env has no index or `oids` is empty.

 Call it only after the bucket confirmed the objects, so a failed write never
 leaves an index row for an object the bucket lacks.

 @param env - Worker env that may carry the index

 @param oids - oids the bucket is known to hold

 @throws whatever the D1 binding throws, so the caller can log it; the object is stored either way

 @example
 ```ts
 await recordPresent({ env, oids: [oid] });
 ```
 */
export async function recordPresent({
  env,
  oids,
}: RecordPresentParams,): Promise<void> {
  /**
   Index database, when bound.
   */
  const { INDEX: database, } = env;
  if (database === undefined) {
    return;
  }
  await Promise.all(chunkOids(oids,)
    .map(async function insertChunk(chunk: readonly string[],) {
      return await database
        .prepare(INSERT_PRESENT_SQL,)
        .bind(JSON.stringify(chunk,),)
        .run();
    },),);
}
