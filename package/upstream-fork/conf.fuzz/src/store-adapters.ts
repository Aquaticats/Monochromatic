/**
 Store adapters exposing one uniform operation surface over this package's
 fork and upstream `conf`,
 so the same generated operation sequence can drive both.

 Each adapter constructs its store in a disposable temp directory,
 runs the sequence,
 and returns one observation record:
 per-operation results
 (read values,
 probes,
 thrown error names and message texts),
 final store contents,
 item count,
 and the config file's parsed `JSON`.

 @module
 */

import Conf from '../upstream-conf/index.ts';

import {
  createConf,
} from '@monochromatic-dev/upstream-fork-conf/ts';

import { createTempStoreDir, } from './store-fixture.ts';
import {
  type StoreObservation,
  runSequence,
} from './store-observe.ts';
import {
  type GetLookup,
  type StoreSurface,
  entriesRecord,
} from './store-surface.ts';
import {
  type JsonRecord,
  type JsonValue,
  STORE_DEFAULTS,
  type StoreOperationSequence,
} from './store-operations.ts';

//region Adapters

/**
 Runs one operation sequence against this package's fork.

 @param sequence - Generated operations to run in order.

 @returns Observation of the fork's behavior.

 @example
 ```ts
 const observation = await runOnFork({ operations: [], });
 ```
 */
export async function runOnFork(sequence: StoreOperationSequence,): Promise<StoreObservation> {
  /**
   Disposable directory holding this run's config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Fork store under test.
   */
  const config = createConf<Record<string, JsonValue>>({
    cwd: dir.path,
    defaults: STORE_DEFAULTS,
  });

  return runSequence({
    surface: {
      path: config.path,
      get: function forkGet(input: {
        readonly key: string;
        readonly useDefault: boolean;
        readonly defaultValue: JsonValue;
      },): GetLookup {
        /**
         Fork's answer for the requested key.
         */
        const value = input.useDefault
          ? config.get({
            key: input.key,
            defaultValue: input.defaultValue,
          },)
          : config.get(input.key,);
        return (value === undefined)
          ? { kind: 'missing', }
          : {
            kind: 'found',
            value,
          };
      },
      setOne: function forkSetOne(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.set(input,);
      },
      setMany: function forkSetMany(input: {
        readonly entries: readonly (readonly [
          string,
          JsonValue
        ])[];
      },): void {
        config.set({
          values: entriesRecord(input.entries,),
        },);
      },
      has: function forkHas(key: string,): boolean {
        return config.has(key,);
      },
      delete: function forkDelete(key: string,): void {
        config.delete(key,);
      },
      appendToArray: function forkAppend(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.appendToArray(input,);
      },
      reset: function forkReset(input: {
        readonly keys: readonly string[];
      },): void {
        config.reset(input,);
      },
      clear: function forkClear(): void {
        config.clear();
      },
      replaceStore: function forkReplaceStore(input: {
        readonly value: JsonRecord;
      },): void {
        config.store = input.value;
      },
      readStore: function forkReadStore(): JsonRecord {
        return config.store;
      },
      readSize: function forkReadSize(): number {
        return config.size;
      },
    },
    sequence,
  });
}

/**
 Runs one operation sequence against the vendored upstream `conf` snapshot.

 @param sequence - Generated operations to run in order.

 @returns Observation of upstream's behavior.

 @example
 ```ts
 const observation = await runOnUpstream({ operations: [], });
 ```
 */
export async function runOnUpstream(sequence: StoreOperationSequence,): Promise<StoreObservation> {
  /**
   Disposable directory holding this run's config file.
   */
  await using dir = await createTempStoreDir();
  /**
   Upstream store under comparison.
   */
  const config = new Conf<Record<string, JsonValue>>({
    cwd: dir.path,
    defaults: STORE_DEFAULTS,
  });

  return runSequence({
    surface: {
      path: config.path,
      get: function upstreamGet(input: {
        readonly key: string;
        readonly useDefault: boolean;
        readonly defaultValue: JsonValue;
      },): GetLookup {
        /**
         Upstream's answer for the requested key.
         */
        const value = input.useDefault
          ? config.get(
            input.key,
            input.defaultValue,
          )
          : config.get(input.key,);
        return (value === undefined)
          ? { kind: 'missing', }
          : {
            kind: 'found',
            value,
          };
      },
      setOne: function upstreamSetOne(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.set(
          input.key,
          input.value,
        );
      },
      setMany: function upstreamSetMany(input: {
        readonly entries: readonly (readonly [
          string,
          JsonValue
        ])[];
      },): void {
        config.set(entriesRecord(input.entries,),);
      },
      has: function upstreamHas(key: string,): boolean {
        return config.has(key,);
      },
      delete: function upstreamDelete(key: string,): void {
        config.delete(key,);
      },
      appendToArray: function upstreamAppend(input: {
        readonly key: string;
        readonly value: JsonValue;
      },): void {
        config.appendToArray(
          input.key,
          input.value,
        );
      },
      reset: function upstreamReset(input: {
        readonly keys: readonly string[];
      },): void {
        config.reset(...input.keys,);
      },
      clear: function upstreamClear(): void {
        config.clear();
      },
      replaceStore: function upstreamReplaceStore(input: {
        readonly value: JsonRecord;
      },): void {
        config.store = input.value;
      },
      readStore: function upstreamReadStore(): JsonRecord {
        return config.store;
      },
      readSize: function upstreamReadSize(): number {
        return config.size;
      },
    },
    sequence,
  });
}

//endregion Adapters
