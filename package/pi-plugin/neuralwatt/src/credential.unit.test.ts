/**
 Tests for the NeuralWatt credential seam.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  NEURALWATT_API_KEY_ENV,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';
import {
  setTimeout as sleep,
} from 'node:timers/promises';
import type {
  EnvLocalJsonEnvironment,
  ReadEnvLocalJsonOptions,
  ReadEnvLocalJsonResult,
} from '@monochromatic-dev/env-local-json-core/ts';
import type {
  Logger,
} from '@monochromatic-dev/logger-core/ts';
import type {
  AuthContext,
} from '@earendil-works/pi-ai';
import {
  createNeuralWattApiKeyAuth,
  CREDENTIAL_REVALIDATION_INTERVAL_MS,
  deriveRepoEnvLocalJsonPath,
  ENV_LOCAL_JSON_PATH_VARIABLE,
  NEURALWATT_CREDENTIAL_GUIDANCE,
  readNeuralWattStoreValue,
  resolveNeuralWattCredential,
  STORE_READ_ATTEMPTS,
  type NeuralWattStoreRead,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Recorded logger whose messages the secret-leak guard inspects.
 */
type CapturingLogger = {
  readonly logger: Logger;
  readonly messages: string[];
};

/**
 Recorder collecting messages into one list.

 @param messages - list the recorder appends to

 @returns a logger-level callback appending to that list
 */
function messageRecorder(
  messages: string[],
): (message: string) => void {
  return function record(message: string,): void {
    messages.push(message,);
  };
}

/**
 Build a logger that records every message instead of emitting it.

 @returns the logger and the message list it appends to
 */
function createCapturingLogger(): CapturingLogger {
  /**
   Messages every level appends to.
   */
  const messages: string[] = [];
  /**
   Recorder shared by every level of this logger.
   */
  const record = messageRecorder(messages,);
  return {
    logger: {
      debug: record,
      error: record,
      fatal: record,
      flush: function flush(): Promise<void> {
        return Promise.resolve();
      },
      info: record,
      trace: record,
      warn: record,
    },
    messages,
  };
}

/**
 Existence probe over a known file list, for derivation tests.

 @param existing - absolute paths that count as existing files

 @returns a probe answering membership in that list
 */
function fixtureFileExists(
  existing: readonly string[],
): (path: string) => Promise<boolean> {
  return function fileExists(path: string,): Promise<boolean> {
    return Promise.resolve(existing.includes(path,));
  };
}

/**
 Fake store reader recording the options it was called with.

 @param value - stored value the read reports under the credential name

 @param path - store path the read reports when it loads

 @returns the reader and the recorded call options
 */
function fixtureStoreReader({
  value,
  path,
}: {
  readonly value?: string;
  readonly path?: string;
},): {
  readonly reader: (options: ReadEnvLocalJsonOptions) => Promise<ReadEnvLocalJsonResult>;
  readonly calls: ReadEnvLocalJsonOptions[];
} {
  /**
   Options every call was made with.
   */
  const calls: ReadEnvLocalJsonOptions[] = [];
  return {
    reader: function reader(
      options: ReadEnvLocalJsonOptions,
    ): Promise<ReadEnvLocalJsonResult> {
      calls.push(options,);
      return Promise.resolve({
        values: value === undefined ? {} : { [NEURALWATT_API_KEY_ENV]: value, },
        loaded: path !== undefined,
        ...(path === undefined ? {} : { path, }),
      });
    },
    calls,
  };
}

/**
 pi auth context whose environment answers from a fixture record.

 @param environment - fixture environment values by name

 @returns the auth context pi would hand `resolve`
 */
function fixtureAuthContext(
  environment: EnvLocalJsonEnvironment,
): AuthContext {
  return {
    env: function env(name: string,) {
      return Promise.resolve(environment[name],);
    },
    fileExists: function fileExists(path: string,) {
      void path;
      return Promise.resolve(false,);
    },
  };
}

/**
 Build the auth seam around a fixture store reader.

 @param logger - capturing logger the tests assert on

 @param readStore - store reader the seam resolves through

 @param now - clock the seam schedules revalidation against

 @returns the api-key auth object under test
 */
function fixtureApiKeyAuth({
  logger,
  readStore,
  now,
}: {
  readonly logger: Logger;
  readonly readStore: (options: ReadEnvLocalJsonOptions) => Promise<ReadEnvLocalJsonResult>;
  readonly now?: () => number;
},): ReturnType<typeof createNeuralWattApiKeyAuth> {
  return createNeuralWattApiKeyAuth({
    logger,
    environment: {},
    moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
    fileExists: fixtureFileExists([],),
    readStore,
    ...(now === undefined ? {} : { now, }),
  },);
}

//endregion Fixtures

//region Resolution chain

await describe({
  name: resolveNeuralWattCredential.name,
  children: [
    it({
      name: 'prefers the pi credential over every other home',
      fn: async () => {
        const resolution = await resolveNeuralWattCredential({
          sources: {
            credentialKey: 'pi-key',
            environmentKey: 'environment-key',
            readStoreValue: function readStoreValue(): Promise<NeuralWattStoreRead> {
              return Promise.resolve({
                value: 'store-key',
                path: '/store',
              });
            },
          },
        },);
        expect(resolution.key).toBe('pi-key');
        expect(resolution.source).toBe('pi-credential');
      },
    }),
    it({
      name: 'prefers the encrypted store over the environment',
      fn: async () => {
        const resolution = await resolveNeuralWattCredential({
          sources: {
            environmentKey: 'environment-key',
            readStoreValue: function readStoreValue(): Promise<NeuralWattStoreRead> {
              return Promise.resolve({
                value: 'store-key',
                path: '/store',
              });
            },
          },
        },);
        expect(resolution.key).toBe('store-key');
        expect(resolution.source).toBe('encrypted-store');
        expect(resolution.storePath).toBe('/store');
      },
    }),
    it({
      name: 'falls back to the environment when the store carries no value',
      fn: async () => {
        const resolution = await resolveNeuralWattCredential({
          sources: {
            environmentKey: 'environment-key',
            readStoreValue: function readStoreValue(): Promise<NeuralWattStoreRead> {
              return Promise.resolve({});
            },
          },
        },);
        expect(resolution.key).toBe('environment-key');
        expect(resolution.source).toBe('environment');
      },
    }),
    it({
      name: 'reports absence when no home answers',
      fn: async () => {
        const resolution = await resolveNeuralWattCredential({
          sources: {
            readStoreValue: function readStoreValue(): Promise<NeuralWattStoreRead> {
              return Promise.resolve({});
            },
          },
        },);
        expect(resolution.key).toBe(undefined);
        expect(resolution.source).toBe('none');
      },
    }),
    it({
      name: 'trims whitespace so it never counts as a key',
      fn: async () => {
        const resolution = await resolveNeuralWattCredential({
          sources: {
            credentialKey: '   ',
            environmentKey: '  environment-key  ',
            readStoreValue: function readStoreValue(): Promise<NeuralWattStoreRead> {
              return Promise.resolve({ value: '  ', });
            },
          },
        },);
        expect(resolution.key).toBe('environment-key');
        expect(resolution.source).toBe('environment');
      },
    }),
  ],
});

//endregion Resolution chain

//region Store path derivation

await describe({
  name: deriveRepoEnvLocalJsonPath.name,
  children: [
    it({
      name: 'derives the repository store from any build layout of the package',
      fn: async () => {
        /**
         Manifest that stops the ancestor walk at the package root.
         */
        const manifest = '/repo/package/pi-plugin/neuralwatt/package.json';
        /**
         Store the repository root is expected to carry.
         */
        const store = '/repo/.env.local.json';
        const fromSource = await deriveRepoEnvLocalJsonPath({
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([
            manifest,
            store,
          ],),
        },);
        const fromDist = await deriveRepoEnvLocalJsonPath({
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/dist/final/node/index.mjs',
          fileExists: fixtureFileExists([
            manifest,
            store,
          ],),
        },);
        expect(fromSource.path).toBe(store);
        expect(fromDist.path).toBe(store);
      },
    }),
    it({
      name: 'reports no path when the repository store is absent',
      fn: async () => {
        const derived = await deriveRepoEnvLocalJsonPath({
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([
            '/repo/package/pi-plugin/neuralwatt/package.json',
          ],),
        },);
        expect(derived.path).toBe(undefined);
      },
    }),
    it({
      name: 'reports no path when no package manifest exists above the module',
      fn: async () => {
        const derived = await deriveRepoEnvLocalJsonPath({
          moduleUrl: 'file:///stranded/module.ts',
          fileExists: fixtureFileExists([],),
        },);
        expect(derived.path).toBe(undefined);
      },
    }),
  ],
});

//endregion Store path derivation

//region Store reader

await describe({
  name: readNeuralWattStoreValue.name,
  children: [
    it({
      name: 'hides the credential name from the reader environment so the store wins',
      fn: async () => {
        const { reader, calls, } = fixtureStoreReader({ value: 'store-key', });
        const stored = await readNeuralWattStoreValue({
          environment: {
            [NEURALWATT_API_KEY_ENV]: 'stale-environment-key',
            [ENV_LOCAL_JSON_PATH_VARIABLE]: '/store/.env.local.json',
            PATH: '/usr/bin',
          },
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([],),
          readStore: reader,
        },);
        expect(stored.value).toBe('store-key');
        const [options] = calls;
        expect(calls).toHaveLength(1);
        expect(Object.hasOwn(options?.env ?? {}, NEURALWATT_API_KEY_ENV)).toBe(false);
        expect(options?.env?.PATH).toBe('/usr/bin');
        expect(options?.keys).toEqual([NEURALWATT_API_KEY_ENV]);
      },
    }),
    it({
      name: 'passes no explicit path when the environment names a store',
      fn: async () => {
        const { reader, calls, } = fixtureStoreReader({});
        await readNeuralWattStoreValue({
          environment: {
            [ENV_LOCAL_JSON_PATH_VARIABLE]: '/override/.env.local.json',
          },
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([
            '/repo/package/pi-plugin/neuralwatt/package.json',
            '/repo/.env.local.json',
          ],),
          readStore: reader,
        },);
        const [call] = calls;
        expect(call?.path).toBe(undefined);
      },
    }),
    it({
      name: 'passes the derived repository store when no override exists',
      fn: async () => {
        const { reader, calls, } = fixtureStoreReader({});
        await readNeuralWattStoreValue({
          environment: {},
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([
            '/repo/package/pi-plugin/neuralwatt/package.json',
            '/repo/.env.local.json',
          ],),
          readStore: reader,
        },);
        const [call] = calls;
        expect(call?.path).toBe('/repo/.env.local.json');
      },
    }),
    it({
      name: 'passes no path when the derived candidate is absent, never a hard error',
      fn: async () => {
        const { reader, calls, } = fixtureStoreReader({});
        await readNeuralWattStoreValue({
          environment: {},
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([
            '/repo/package/pi-plugin/neuralwatt/package.json',
          ],),
          readStore: reader,
        },);
        const [call] = calls;
        expect(call?.path).toBe(undefined);
      },
    }),
    it({
      name: 'reports the store path only when the reader loaded a store',
      fn: async () => {
        const loading = fixtureStoreReader({
          value: 'store-key',
          path: '/store/.env.local.json',
        },);
        const absent = fixtureStoreReader({});
        const loaded = await readNeuralWattStoreValue({
          environment: {},
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([],),
          readStore: loading.reader,
        },);
        const empty = await readNeuralWattStoreValue({
          environment: {},
          moduleUrl: 'file:///repo/package/pi-plugin/neuralwatt/src/credential.ts',
          fileExists: fixtureFileExists([],),
          readStore: absent.reader,
        },);
        expect(loaded.path).toBe('/store/.env.local.json');
        expect(empty.path).toBe(undefined);
      },
    }),
  ],
});

//endregion Store reader

//region Pi auth object

await describe({
  name: createNeuralWattApiKeyAuth.name,
  children: [
    it({
      name: 'resolves the winning home and names it as the pi source label',
      fn: async () => {
        const { logger, messages, } = createCapturingLogger();
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: fixtureStoreReader({
            value: 'store-key',
            path: '/store',
          },).reader,
        },);
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({},),
          signal: new AbortController().signal,
        },);
        expect(resolved?.auth.apiKey).toBe('store-key');
        expect(resolved?.source).toBe('/store');
        for (const message of messages) {
          expect(message.includes('store-key',)).toBe(false);
        }
      },
    }),
    it({
      name: 'reports no auth and logs guidance when no home answers',
      fn: async () => {
        const { logger, messages, } = createCapturingLogger();
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: fixtureStoreReader({},).reader,
        },);
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({},),
          signal: new AbortController().signal,
        },);
        expect(resolved).toBe(undefined);
        expect(messages.some(function namesGuidance(message,) {
          return message.includes(NEURALWATT_CREDENTIAL_GUIDANCE,);
        },)).toBe(true);
      },
    }),
    it({
      name: 'falls through to the environment when the store read fails',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: function failingReader(): Promise<ReadEnvLocalJsonResult> {
            return Promise.reject(new Error('store unavailable',));
          },
        },);
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({
            [NEURALWATT_API_KEY_ENV]: 'environment-key',
          },),
          signal: new AbortController().signal,
        },);
        expect(resolved?.auth.apiKey).toBe('environment-key');
        expect(resolved?.source).toBe(NEURALWATT_API_KEY_ENV);
      },
    }),
    it({
      name: 'keeps the resolved key when a later store read times out',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: function timingOutReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            if (reads === 1) {
              return Promise.resolve({
                loaded: true,
                path: '/store',
                values: { [NEURALWATT_API_KEY_ENV]: 'store-key-1', },
              },);
            }
            return Promise.reject(new Error('sops could not decrypt (timeout)',));
          },
        },);
        const ctx = fixtureAuthContext({},);
        const first = await auth.resolve({
          ctx,
          signal: new AbortController().signal,
        },);
        const second = await auth.resolve({
          ctx,
          signal: new AbortController().signal,
        },);
        expect(first?.auth.apiKey).toBe('store-key-1');
        expect(second?.auth.apiKey).toBe('store-key-1');
        expect(reads).toBe(1);
      },
    }),
    it({
      name: 'retries a first store read that times out and still resolves the key',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: function flakyReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            if (reads === 1)
              return Promise.reject(new Error('sops could not decrypt (timeout)',));
            return Promise.resolve({
              loaded: true,
              path: '/store',
              values: { [NEURALWATT_API_KEY_ENV]: 'store-key-1', },
            },);
          },
        },);
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({},),
          signal: new AbortController().signal,
        },);
        expect(resolved?.auth.apiKey).toBe('store-key-1');
        expect(reads).toBe(2);
      },
    }),
    it({
      name: 'stops retrying the store read when the caller aborts',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        const controller = new AbortController();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: function abortedReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            controller.abort(new Error('request cancelled',),);
            return Promise.reject(new Error('sops decrypt aborted',));
          },
        },);
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({},),
          signal: controller.signal,
        },);
        expect(resolved).toBe(undefined);
        expect(reads).toBe(1);
      },
    }),
    it({
      name: 'shares one store read between concurrent resolutions',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: async function countingReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            return {
              loaded: true,
              path: '/store',
              values: { [NEURALWATT_API_KEY_ENV]: 'store-key-1', },
            };
          },
        },);
        const ctx = fixtureAuthContext({},);
        const { signal, } = new AbortController();
        const resolvedBoth = await Promise.all([
          auth.resolve({
            ctx,
            signal,
          },),
          auth.resolve({
            ctx,
            signal,
          },),
        ],);
        expect(reads).toBe(1);
        expect(resolvedBoth.map(function readKey(resolved,) {
          return resolved?.auth.apiKey;
        },)).toEqual([
          'store-key-1',
          'store-key-1',
        ],);
      },
    }),
    it({
      name: 'reports absence once the bounded store read attempts are spent',
      fn: async () => {
        const { logger, messages, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: function timingOutReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            return Promise.reject(new Error('sops could not decrypt (timeout)',));
          },
        },);
        const resolved = await auth.resolve({
          ctx: fixtureAuthContext({},),
          signal: new AbortController().signal,
        },);
        expect(resolved).toBe(undefined);
        expect(reads).toBe(STORE_READ_ATTEMPTS);
        expect(messages.some(function namesGuidance(message,) {
          return message.includes(NEURALWATT_CREDENTIAL_GUIDANCE,);
        },)).toBe(true);
      },
    }),
    it({
      name: 'picks up a key the store names after it named none',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        const auth = fixtureApiKeyAuth({
          logger,
          readStore: function lateReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            return Promise.resolve({
              loaded: true,
              path: '/store',
              values: reads === 1 ? {} : { [NEURALWATT_API_KEY_ENV]: 'store-key-1', },
            },);
          },
        },);
        const ctx = fixtureAuthContext({},);
        const before = await auth.resolve({
          ctx,
          signal: new AbortController().signal,
        },);
        const after = await auth.resolve({
          ctx,
          signal: new AbortController().signal,
        },);
        expect(before).toBe(undefined);
        expect(after?.auth.apiKey).toBe('store-key-1');
        expect(reads).toBe(2);
      },
    }),
    it({
      name: 'replaces the served key with a rotated one once the revalidation window passes',
      fn: async () => {
        const { logger, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        /**
         Clock the seam schedules revalidation against.
         */
        let time = 0;
        /**
         Signal the second read raises once the seam asks for it.
         */
        const rotatedRead = Promise.withResolvers<void>();
        const auth = fixtureApiKeyAuth({
          logger,
          now: function clock(): number {
            return time;
          },
          readStore: function rotatedReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            if (reads === 2)
              rotatedRead.resolve();
            return Promise.resolve({
              loaded: true,
              path: '/store',
              values: { [NEURALWATT_API_KEY_ENV]: `store-key-${String(reads,)}`, },
            },);
          },
        },);
        const ctx = fixtureAuthContext({},);
        const { signal, } = new AbortController();
        const first = await auth.resolve({
          ctx,
          signal,
        },);
        time += CREDENTIAL_REVALIDATION_INTERVAL_MS + 1;
        const servedFromMemory = await auth.resolve({
          ctx,
          signal,
        },);
        await rotatedRead.promise;
        /**
         Microtasks the read chain still holds, drained so the rotated read is
         adopted before the next resolution.
         */
        await sleep(0,);
        const rotated = await auth.resolve({
          ctx,
          signal,
        },);
        expect(first?.auth.apiKey).toBe('store-key-1');
        expect(servedFromMemory?.auth.apiKey).toBe('store-key-1');
        expect(rotated?.auth.apiKey).toBe('store-key-2');
        expect(reads).toBe(2);
      },
    }),
    it({
      name: 'keeps the last good key when the revalidation read fails, and says so',
      fn: async () => {
        const { logger, messages, } = createCapturingLogger();
        /**
         Store reads the seam made, in order.
         */
        let reads = 0;
        /**
         Clock the seam schedules revalidation against.
         */
        let time = 0;
        /**
         Signal the last read attempt raises once the seam asks for it.
         */
        const everyAttempt = Promise.withResolvers<void>();
        const auth = fixtureApiKeyAuth({
          logger,
          now: function clock(): number {
            return time;
          },
          readStore: function timingOutReader(): Promise<ReadEnvLocalJsonResult> {
            reads += 1;
            if (reads === (1 + STORE_READ_ATTEMPTS))
              everyAttempt.resolve();
            if (reads === 1) {
              return Promise.resolve({
                loaded: true,
                path: '/store',
                values: { [NEURALWATT_API_KEY_ENV]: 'store-key-1', },
              },);
            }
            return Promise.reject(new Error('sops could not decrypt (timeout)',));
          },
        },);
        const ctx = fixtureAuthContext({},);
        const { signal, } = new AbortController();
        const first = await auth.resolve({
          ctx,
          signal,
        },);
        time += CREDENTIAL_REVALIDATION_INTERVAL_MS + 1;
        const servedFromMemory = await auth.resolve({
          ctx,
          signal,
        },);
        await everyAttempt.promise;
        /**
         Microtasks the failing refresh still holds, drained so it reports
         before the assertions.
         */
        await sleep(0,);
        const afterFailure = await auth.resolve({
          ctx,
          signal,
        },);
        expect(first?.auth.apiKey).toBe('store-key-1');
        expect(servedFromMemory?.auth.apiKey).toBe('store-key-1');
        expect(afterFailure?.auth.apiKey).toBe('store-key-1');
        expect(reads).toBe(1 + STORE_READ_ATTEMPTS);
        expect(messages.some(function keepsLastGoodKey(message,) {
          return message.includes('keeping the NeuralWatt credential resolved earlier',);
        },)).toBe(true);
        for (const message of messages) {
          expect(message.includes('store-key-1',)).toBe(false);
        }
      },
    }),
  ],
});

//endregion Pi auth object
