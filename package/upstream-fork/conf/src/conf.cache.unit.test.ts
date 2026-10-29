/**
 Cache option behavior: memory reads,
 per-instance isolation,
 cache dropping on writes and watcher events,
 concurrent-writer merging,
 and interaction with migrations and encryption.
 
 @module
 */

import {
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { wait, } from '@monochromatic-dev/async-time-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createConf,
  MigrationFailedError,
  type Options,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  writeConfigFile,
} from './test-support.ts';

/**
 Upper bound on watcher waits;
 Linux `fs.watchFile` polls about every 5007 milliseconds.
 */
const WATCH_TIMEOUT_MILLISECONDS = 25_000;

/**
 Poll interval for bounded condition waits.
 */
const POLL_INTERVAL_MILLISECONDS = 100;

/**
 Reads the config file the way another process would see it.
 
 @param configPath - Absolute config file path.
 
 @returns Parsed file contents.
 */
function readConfigFile(configPath: string,): Record<string, unknown> {
  return JSON.parse(readFileSync(
    configPath,
    'utf8',
  ),) as Record<string, unknown>;
}

/**
 Polls until the condition passes or the deadline expires,
 so watch-based tests stay bounded.
 
 @param check - Condition polled until it passes.
 
 @param timeoutMilliseconds - Upper bound on the wait.
 */
async function waitUntil({
  check,
  timeoutMilliseconds,
}: {
  readonly check: () => boolean;
  readonly timeoutMilliseconds: number;
},): Promise<void> {
  /**
   Moment the bounded wait started.
   */
  const startedAt = Date.now();
  while (!check()) {
    if ((Date.now() - startedAt) >= timeoutMilliseconds)
      throw new Error(`Condition was not met within ${timeoutMilliseconds} milliseconds`);
    /* oxlint-disable-next-line eslint/no-await-in-loop -- sequential polling between condition checks; parallel promises cannot express one bounded polling wait. */
    await wait(POLL_INTERVAL_MILLISECONDS,);
  }
}

/**
 Runs a call expected to throw and returns the caught error,
 so one invocation can carry several assertions.
 
 @param call - Call expected to throw.
 
 @returns The caught error value.
 */
function captureThrown(call: () => unknown,): unknown {
  try {
    call();
  }
  catch (error) {
    return error;
  }
  throw new Error('Expected the call to throw, but it returned.',);
}

await describe({
  name: 'cache option',
  children: [
    it({
      name: 'reads from memory instead of the file',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store serving reads from memory.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        writeConfigFile({
          directory,
          data: {
            foo: 'changed-by-another-process',
          },
        },);
        expect(conf.get('foo',),).toBe('bar',);
      },
    },),

    it({
      name: 'still reads the file when the cache is disabled',
      fn: async () => {
        /**
         Directory whose file will change behind the uncached store.
         */
        const directory = createTempDirectory();
        /**
         Store serving every read from the file.
         */
        const conf = createConf({
          cwd: directory,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        writeConfigFile({
          directory,
          data: {
            foo: 'changed-by-another-process',
          },
        },);
        expect(conf.get('foo',),).toBe('changed-by-another-process',);
      },
    },),

    it({
      name: 'reads from memory when the config file becomes unreadable mid-flight',
      fn: async () => {
        /**
         Directory whose config file will become a directory below.
         */
        const directory = createTempDirectory();
        /**
         Store serving reads from memory.
         */
        const cached = createConf({
          cwd: directory,
          cache: true,
        },);
        cached.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(cached.get('foo',),).toBe('bar',);
        /**
         Store without a cache for contrast.
         */
        const uncached = createConf({
          cwd: directory,
        },);
        expect(uncached.get('foo',),).toBe('bar',);

        rmSync(cached.path,);
        mkdirSync(cached.path,);

        expect(cached.get('foo',),).toBe('bar',);
        /**
         Read failure surfaced to the uncached store.
         */
        const error = captureThrown(function readUnreadableFile(): unknown {
          return uncached.get('foo',);
        },);
        expect(error,).toHaveProperty(
          'code',
          'EISDIR',
        );
      },
    },),

    it({
      name: 'does not share the cache between instances',
      fn: async () => {
        /**
         Directory shared by two independent cached stores.
         */
        const directory = createTempDirectory();
        /**
         First cached store on the shared path.
         */
        const first = createConf({
          cwd: directory,
          cache: true,
        },);
        first.set({
          key: 'foo',
          value: 'first',
        },);

        /**
         Second cached store on the same path,
         which must read the file rather than the first store's cache.
         */
        const second = createConf({
          cwd: directory,
          cache: true,
        },);
        expect(second.get('foo',),).toBe('first',);
        expect(second.size,).toBe(1,);

        /**
         Cached store on a different path,
         which must never see the shared store's contents.
         */
        const other = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        expect(other.get('foo',),).toBeUndefined();
        expect(other.size,).toBe(0,);

        first.set({
          key: 'bar',
          value: 'second',
        },);
        expect(first.get('bar',),).toBe('second',);
        expect(other.get('bar',),).toBeUndefined();
      },
    },),

    it({
      name: 'drops the cache when an item is set',
      fn: async () => {
        /**
         Store whose cache is dropped by its own writes.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);
        conf.set({
          key: 'baz',
          value: 'qux',
        },);
        expect(conf.get('baz',),).toBe('qux',);
      },
    },),

    it({
      name: 'does not lose changes made by another process when setting an item',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store whose write must merge against the file.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);

        writeConfigFile({
          directory,
          data: {
            foo: 'bar',
            external: 'kept',
          },
        },);
        conf.set({
          key: 'baz',
          value: 'qux',
        },);

        expect(readConfigFile(conf.path,),).toEqual({
          foo: 'bar',
          external: 'kept',
          baz: 'qux',
        },);
        expect(conf.get('external',),).toBe('kept',);
      },
    },),

    it({
      name: 'does not lose changes made by another process when appending to an array',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store whose append must merge against the file.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
        },);
        conf.set({
          key: 'items',
          value: [
            'first',
          ],
        },);
        expect(conf.get('items',),).toEqual([
          'first',
        ],);

        writeConfigFile({
          directory,
          data: {
            items: [
              'first',
              'external',
            ],
          },
        },);
        conf.appendToArray({
          key: 'items',
          value: 'second',
        },);

        expect(readConfigFile(conf.path,)
          .items,).toEqual([
            'first',
            'external',
            'second',
          ],);
      },
    },),

    it({
      name: 'uses the cache without dot notation',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store treating every key as one literal string.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          accessPropertiesByDotNotation: false,
        },);
        conf.set({
          key: 'items',
          value: [
            'first',
          ],
        },);
        conf.set({
          key: 'foo.bar',
          value: 'literal',
        },);
        expect(conf.get('items',),).toEqual([
          'first',
        ],);

        writeConfigFile({
          directory,
          data: {
            items: [
              'first',
              'external',
            ],
            'foo.bar': 'changed',
            extra: true,
          },
        },);

        expect(conf.get('foo.bar',),).toBe('literal',);
        expect(conf.get('items',),).toEqual([
          'first',
        ],);
        expect(conf.has('extra',),).toBe(false,);
        expect(conf.size,).toBe(2,);

        conf.appendToArray({
          key: 'items',
          value: 'second',
        },);
        expect(readConfigFile(conf.path,)
          .items,).toEqual([
            'first',
            'external',
            'second',
          ],);
      },
    },),

    it({
      name: 'does not lose changes made by another process when deleting an item',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store whose delete must merge against the file.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);

        writeConfigFile({
          directory,
          data: {
            foo: 'bar',
            external: 'kept',
          },
        },);
        conf.delete('foo',);

        expect(readConfigFile(conf.path,),).toEqual({
          external: 'kept',
        },);
        expect(conf.get('external',),).toBe('kept',);
      },
    },),

    it({
      name: 'drops the cache when the store is cleared',
      fn: async () => {
        /**
         Store whose clear must drop the cache.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);
        conf.clear();
        expect(conf.get('foo',),).toBeUndefined();
      },
    },),

    it({
      name: 'uses the cache for every read API',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store whose every read surface must serve the cache.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
        },);
        conf.set({
          values: {
            foo: 'bar',
            nested: {
              value: 1,
            },
          },
        },);
        expect(conf.size,).toBe(2,);

        writeConfigFile({
          directory,
          data: {
            foo: 'changed',
            nested: {
              value: 2,
            },
            extra: true,
          },
        },);

        expect(Object.keys(conf.store,),).toEqual([
          'foo',
          'nested',
        ],);
        expect(conf.get('foo',),).toBe('bar',);
        expect(conf.get('nested',),).toEqual({
          value: 1,
        },);
        expect(conf.has('foo',),).toBe(true,);
        expect(conf.has('extra',),).toBe(false,);
        expect(conf.size,).toBe(2,);
        expect([...conf],).toEqual([
          [
            'foo',
            'bar',
          ],
          [
            'nested',
            {
              value: 1,
            },
          ],
        ],);
      },
    },),

    it({
      name: 'reports the changed value to onDidChange from the cache',
      fn: async () => {
        /**
         Store whose change reporting must not be stale.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        /**
         Changes reported for `foo`.
         */
        const history: Record<string, unknown>[] = [];
        conf.onDidChange({
          key: 'foo',
          callback: function recordChange(change: Record<string, unknown>,): void {
            history.push(change,);
          },
        },);
        conf.set({
          key: 'foo',
          value: 'baz',
        },);
        expect(history,).toEqual([
          {
            newValue: 'baz',
            oldValue: 'bar',
          },
        ],);
      },
    },),

    it({
      name: 'reports the changed store to onDidAnyChange from the cache',
      fn: async () => {
        /**
         Store whose whole-store reporting must not be stale.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        /**
         Store changes reported after the subscription.
         */
        const history: Record<string, unknown>[] = [];
        conf.onDidAnyChange(function recordStoreChange(change: Record<string, unknown>,): void {
          history.push(change,);
        },);
        conf.set({
          key: 'baz',
          value: 'qux',
        },);
        expect(history,).toEqual([
          {
            newValue: {
              foo: 'bar',
              baz: 'qux',
            },
            oldValue: {
              foo: 'bar',
            },
          },
        ],);
      },
    },),

    it({
      name: 'applies the defaults option while the cache holds them',
      fn: async () => {
        /**
         Directory whose file will change behind the cached store.
         */
        const directory = createTempDirectory();
        /**
         Store whose defaults fill the cache at construction.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          defaults: {
            foo: 'default',
          },
        },);
        expect(conf.get('foo',),).toBe('default',);
        expect(readConfigFile(conf.path,),).toEqual({
          foo: 'default',
        },);

        writeConfigFile({
          directory,
          data: {
            foo: 'external',
          },
        },);
        expect(conf.get('foo',),).toBe('default',);
      },
    },),

    it({
      name: 'drops the cache when the store is replaced',
      fn: async () => {
        /**
         Store whose replacement must drop the cache.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        conf.store = {
          baz: 'qux',
        };
        expect(conf.get('foo',),).toBeUndefined();
        expect(conf.get('baz',),).toBe('qux',);
        expect(readConfigFile(conf.path,),).toEqual({
          baz: 'qux',
        },);
      },
    },),

    it({
      name: 'never writes changes made to the objects it returns',
      fn: async () => {
        /**
         Store handing out its cached objects.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'nested',
          value: {
            count: 1,
          },
        },);

        /**
         Cached nested object the test mutates deliberately.
         */
        const nested = conf.get('nested',) as {
          count: number;
        };
        nested.count = 2;
        expect((conf.get('nested',) as {
          count: number;
        }).count,).toBe(2,);

        conf.set({
          key: 'other',
          value: 'value',
        },);
        expect(readConfigFile(conf.path,),).toEqual({
          nested: {
            count: 1,
          },
          other: 'value',
        },);
        expect(conf.get('nested',),).toEqual({
          count: 1,
        },);
      },
    },),

    it({
      name: 'deserializes once for many reads',
      fn: async () => {
        /**
         Deserialization count proving the cache parses the file once.
         */
        let deserializeCallCount = 0;
        /**
         Store with a counting custom deserializer.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
          serialize: function serialize(value: Record<string, unknown>,): string {
            return `foo=${String(value.foo,)}`;
          },
          deserialize: function deserialize(text: string,): Record<string, unknown> {
            deserializeCallCount += 1;
            return {
              foo: text.slice('foo='.length,),
            };
          },
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);
        expect(readFileSync(
          conf.path,
          'utf8',
        ),).toBe('foo=bar',);

        /**
         Deserialization count before the repeated cached reads.
         */
        const callsBeforeRepeatedReads = deserializeCallCount;
        expect(callsBeforeRepeatedReads > 0,).toBe(true,);
        for (let index = 0; index < 10; index += 1)
          expect(conf.get('foo',),).toBe('bar',);
        expect(deserializeCallCount,).toBe(callsBeforeRepeatedReads,);
      },
    },),

    it({
      name: 'is not poisoned by a serialization failure',
      fn: async () => {
        /**
         Serialize call count controlling the one deliberate failure.
         */
        let serializeCallCount = 0;
        /**
         Store whose second serialize call throws once.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
          serialize: function serialize(value: Record<string, unknown>,): string {
            serializeCallCount += 1;
            if (serializeCallCount === 2)
              throw new TypeError('serialize exploded');
            return JSON.stringify(value,);
          },
          deserialize: function deserialize(text: string,): Record<string, unknown> {
            return JSON.parse(text,) as Record<string, unknown>;
          },
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(function setFailingValue(): void {
          conf.set({
            key: 'foo',
            value: 'boom',
          },);
        },).toThrow(TypeError,);

        expect(conf.get('foo',),).toBe('bar',);
        expect(readConfigFile(conf.path,),).toEqual({
          foo: 'bar',
        },);

        conf.set({
          key: 'foo',
          value: 'baz',
        },);
        expect(conf.get('foo',),).toBe('baz',);
      },
    },),

    it({
      name: 'caches an empty store when the config file is invalid',
      fn: async () => {
        /**
         Directory whose config file is invalid JSON.
         */
        const directory = createTempDirectory();
        writeFileSync(
          `${directory}/config.json`,
          '🦄',
        );
        /**
         Store caching the cleared empty store.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          clearInvalidConfig: true,
        },);
        expect(conf.size,).toBe(0,);
        expect(conf.get('foo',),).toBeUndefined();

        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);
      },
    },),

    it({
      name: 'caches decrypted data',
      fn: async () => {
        /**
         Directory shared by the cached and the writing store.
         */
        const directory = createTempDirectory();
        /**
         Store caching the decrypted contents.
         */
        const cached = createConf({
          cwd: directory,
          cache: true,
          encryptionKey: 'secret-key',
        },);
        cached.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(cached.get('foo',),).toBe('bar',);

        /**
         Second store writing encrypted data behind the cache.
         */
        const writer = createConf({
          cwd: directory,
          encryptionKey: 'secret-key',
        },);
        writer.set({
          key: 'foo',
          value: 'changed',
        },);
        expect(cached.get('foo',),).toBe('bar',);
      },
    },),

    it({
      name: 'drops the cache when watch sees a change',
      fn: async () => {
        /**
         Directory shared by the watching and the writing store.
         */
        const directory = createTempDirectory();
        /**
         Store whose watcher must drop the cache before reporting.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          watch: true,
        },);
        conf.set({
          key: 'foo',
          value: '🐴',
        },);
        expect(conf.get('foo',),).toBe('🐴',);

        /**
         Set once the watcher reports the external change.
         */
        let changeSeen = false;
        conf.onDidChange({
          key: 'foo',
          callback: function markSeen(): void {
            changeSeen = true;
          },
        },);

        /**
         Second store changing the file behind the watcher.
         */
        const writer = createConf({
          cwd: directory,
        },);
        writer.set({
          key: 'foo',
          value: '🦄',
        },);
        await waitUntil({
          check: function changeArrived(): boolean {
            return changeSeen;
          },
          timeoutMilliseconds: WATCH_TIMEOUT_MILLISECONDS,
        });
        expect(conf.get('foo',),).toBe('🦄',);

        conf.closeWatcher();
        writer.closeWatcher();
      },
    },),

    it({
      name: 'applies schema defaults when the file is already migrated',
      fn: async () => {
        /**
         Directory whose file already records the current version.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: '1.0.0',
              },
            },
          },
        },);
        /**
         Store whose no-op migration still exposes schema defaults.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function noOpStep(): void {},
          },
          schema: {
            foo: {
              type: 'number',
              default: 42,
            },
          },
        },);
        expect(conf.get('foo',),).toBe(42,);
        expect(conf.has('foo',),).toBe(true,);
      },
    },),

    it({
      name: 'applies schema type coercion when the file is already migrated',
      fn: async () => {
        /**
         Directory whose file already records the current version.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: '1.0.0',
              },
            },
            port: '8080',
          },
        },);
        /**
         Store whose validation coerces the stored string to a number.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function noOpStep(): void {},
          },
          schema: {
            port: {
              type: 'number',
            },
          },
          ajvOptions: {
            coerceTypes: true,
          },
        },);
        expect(conf.get('port',),).toBe(8_080,);
        expect(typeof conf.get('port',),).toBe('number',);
      },
    },),

    it({
      name: 'sees the result of a migration that ran',
      fn: async () => {
        /**
         Directory whose file carries data from before the migration.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: '0.0.0',
              },
            },
            legacy: 'value',
          },
        },);
        /**
         Store whose migration renames the legacy key.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function rename(store,): void {
              store.set({
                key: 'renamed',
                value: store.get('legacy',),
              },);
              store.delete('legacy',);
            },
          },
        },);
        expect(conf.get('renamed',),).toBe('value',);
        expect(conf.get('legacy',),).toBeUndefined();
        expect(conf.size,).toBe(1,);
        expect([...conf],).toEqual([
          [
            'renamed',
            'value',
          ],
        ],);

        /**
         File contents another process would see.
         */
        const persisted = readConfigFile(conf.path,);
        expect(persisted.renamed,).toBe('value',);
        expect(persisted.legacy,).toBeUndefined();
        expect(persisted.__internal__,).toEqual({
          migrations: {
            version: '1.0.0',
          },
        },);
      },
    },),

    it({
      name: 'sees its own writes during a migration',
      fn: async () => {
        /**
         Directory whose file records the pre-migration version.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: '0.0.0',
              },
            },
          },
        },);
        /**
         Values the migration observed between its own writes.
         */
        const observed: unknown[] = [];
        /**
         Store whose migration reads back its own writes.
         */
        const conf = createConf({
          cwd: directory,
          cache: true,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function readBack(store,): void {
              store.set({
                key: 'step',
                value: 'first',
              },);
              observed.push(store.get('step',),);
              store.set({
                key: 'step',
                value: 'second',
              },);
              observed.push(store.get('step',),);
            },
          },
        },);
        expect(observed,).toEqual([
          'first',
          'second',
        ],);
        expect(conf.get('step',),).toBe('second',);
      },
    },),

    it({
      name: 'leaves no cached data behind when a migration fails',
      fn: async () => {
        /**
         Directory whose migration will fail after one write.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: '0.0.0',
              },
            },
            stable: 'value',
          },
        },);
        expect(function constructFailingPass(): unknown {
          return createConf({
            cwd: directory,
            cache: true,
            projectVersion: '1.0.0',
            migrations: {
              '1.0.0': function failsAfterWrite(store,): void {
                store.set({
                  key: 'partial',
                  value: true,
                },);
                throw new Error('Intentional failure');
              },
            },
          },);
        },).toThrow(MigrationFailedError,);
        expect(function constructFailingPass(): unknown {
          return createConf({
            cwd: directory,
            cache: true,
            projectVersion: '1.0.0',
            migrations: {
              '1.0.0': function failsAfterWrite(store,): void {
                store.set({
                  key: 'partial',
                  value: true,
                },);
                throw new Error('Intentional failure');
              },
            },
          },);
        },).toThrow('Something went wrong during the migration',);

        /**
         Store reading the rolled-back file,
         proving no poisoned cache survived.
         */
        const recovered = createConf({
          cwd: directory,
          cache: true,
        },);
        expect(recovered.get('stable',),).toBe('value',);
        expect(recovered.get('partial',),).toBeUndefined();
      },
    },),

    it({
      name: 'is not poisoned by a schema violation',
      fn: async () => {
        /**
         Store whose rejected write must not stick in the cache.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
          schema: {
            foo: {
              type: 'string',
            },
          },
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        expect(function setWrongType(): void {
          conf.set({
            key: 'foo',
            value: 42,
          },);
        },).toThrow('Config schema violation: `foo` must be string',);
        expect(conf.get('foo',),).toBe('bar',);
        expect(readConfigFile(conf.path,),).toEqual({
          foo: 'bar',
        },);
      },
    },),

    it({
      name: 'is not poisoned by a failed multi-item set',
      fn: async () => {
        /**
         Store whose multi-item write fails part way through validation.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          cache: true,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);

        expect(function setWithUnsupportedValue(): void {
          conf.set({
            values: {
              foo: 'changed',
              invalid: undefined,
            },
          },);
        },).toThrow('Setting a value of type `undefined` for key `invalid` is not allowed as it\'s not supported by JSON',);
        expect(conf.get('foo',),).toBe('bar',);
        expect(readConfigFile(conf.path,),).toEqual({
          foo: 'bar',
        },);
      },
    },),

    it({
      name: 'does not leak the internal key to the object it was given when an assignment fails',
      fn: async () => {
        /**
         Store options whose schema rejects the assignment below;
         typed so the store accepts the deliberate violation.
         */
        const options: Options<Record<string, unknown>> = {
          cwd: createTempDirectory(),
          cache: true,
          projectVersion: '1.0.0',
          schema: {
            a: {
              type: 'number',
              default: 1,
            },
          },
          migrations: {
            '1.0.0': function noOpStep(): void {},
          },
        };
        /**
         Store carrying bookkeeping under the reserved key.
         */
        const conf = createConf(options,);

        /**
         Cached object handed back to the caller and mutated by it.
         */
        const handedBack: Record<string, unknown> = conf.store;
        handedBack.a = 'not-a-number';

        expect(function assignViolatingStore(): void {
          conf.store = handedBack;
        },).toThrow('Config schema violation: `a` must be number',);
        expect(Object.keys(conf.store,),).toEqual([
          'a',
        ],);
        expect(conf.size,).toBe(1,);
        expect([...conf],).toEqual([
          [
            'a',
            'not-a-number',
          ],
        ],);
      },
    },),
  ],
},);
