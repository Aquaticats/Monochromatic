/**
 Version predicates and step execution of `isVersionInRangeFormat`,
 `shouldPerformMigration`,
 `runMigrationSteps`,
 and `applyMigrations` over a stub {@link MigrationHost}.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  applyMigrations,
  type BeforeEachMigrationContext,
  type Conf,
  createConf,
  createPlainObject,
  isVersionInRangeFormat,
  MigrationFailedError,
  type MigrationHost,
  MissingProjectVersionError,
  runMigrationSteps,
  shouldPerformMigration,
} from '../dist/final/neutral/index.mjs';

import { createTempDirectory, } from './test-support.ts';

/**
 Failure prefix upstream `conf` puts in front of the caught reason in every
 `MigrationFailedError` message.
 
 @example
 ```ts
 MIGRATION_FAILURE_PREFIX; // 'Something went wrong during the migration! ...'
 ```
 */
const MIGRATION_FAILURE_PREFIX = 'Something went wrong during the migration! Changes applied to the store until this failed migration will be restored.';

/**
 Runs a call expected to throw and returns the caught error,
 so one invocation can carry several assertions.
 
 @param call - Call expected to throw.
 
 @returns The caught error value.
 
 @example
 ```ts
 const error = captureThrown(function boom(): void { throw new Error('x'); });
 ```
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

/**
 Extracts a printable message from a caught value,
 so message assertions read from one helper.
 
 @param error - Caught value from a call under test.
 
 @returns The error's message,
 or the value's text form when it carries none.
 
 @example
 ```ts
 caughtMessage(new Error('boom')); // => 'boom'
 ```
 */
function caughtMessage(error: unknown,): string {
  return caughtValueText(error,);
}

/**
 Mutable file state behind one stub migration host,
 mirroring the on-disk store the runner reads and restores.
 */
type StubFileState = {
  /**
   File contents including reserved bookkeeping keys.
   */
  rawStore: Record<string, unknown>;
  /**
   User-visible store contents the runner snapshots and restores.
   */
  userStore: Record<string, unknown>;
};

/**
 File and version operations recorded by one stub migration host.
 */
type StubCalls = {
  /**
   Versions passed to `recordVersion`,
   in call order.
   */
  recordVersion: string[];
  /**
   User stores passed to `writeUserStore`,
   in call order.
   */
  writeUserStore: Record<string, unknown>[];
  /**
   Raw stores passed to `writeStoreWithoutEvents`,
   in call order.
   */
  writeStoreWithoutEvents: Record<string, unknown>[];
};

/**
 One stub migration host: runner surface,
 live file state,
 and recorded calls.
 */
type MigrationHostStub = {
  /**
   Host surface under test.
   */
  readonly host: MigrationHost<Record<string, unknown>>;
  /**
   Live file state migrations mutate and the runner restores.
   */
  readonly state: StubFileState;
  /**
   Calls recorded by the stub.
   */
  readonly calls: StubCalls;
};

/**
 Builds one stub migration host over mutable file state,
 recording every write and version call.
 
 @param rawStore - Initial raw file contents including bookkeeping keys.
 
 @param configFileExists - Whether the config file pre-exists the pass.
 
 @returns Stub host with its live state and recorded calls.
 
 @example
 ```ts
 const stub = createMigrationHostStub({ rawStore: {}, configFileExists: true, });
 ```
 */
function createMigrationHostStub({
  rawStore,
  configFileExists: fileExists,
}: {
  readonly rawStore: Record<string, unknown>;
  readonly configFileExists: boolean;
},): MigrationHostStub {
  /**
   Live file state shared by the stub's reads and writes.
   */
  const state: StubFileState = {
    rawStore: structuredClone(rawStore,),
    userStore: structuredClone(rawStore,),
  };
  /**
   Calls recorded by the stub.
   */
  const calls: StubCalls = {
    recordVersion: [],
    writeUserStore: [],
    writeStoreWithoutEvents: [],
  };
  /**
   Host surface under test backed by the live state.
   */
  const host: MigrationHost<Record<string, unknown>> = {
    store: createConf({
      cwd: createTempDirectory(),
    },),
    configFileExists: function configFileExists(): boolean {
      return fileExists;
    },
    readRawStore: function readRawStore(): Record<string, unknown> {
      return state.rawStore;
    },
    readUserStore: function readUserStore(): Record<string, unknown> {
      return state.userStore;
    },
    writeUserStore: function writeUserStore(store: Record<string, unknown>,): void {
      calls.writeUserStore.push(store,);
      state.userStore = store;
    },
    writeStoreWithoutEvents: function writeStoreWithoutEvents(store: Record<string, unknown>,): void {
      calls.writeStoreWithoutEvents.push(store,);
      state.rawStore = store;
    },
    recordVersion: function recordVersion(version: string,): void {
      calls.recordVersion.push(version,);
    },
  };
  return {
    host,
    state,
    calls,
  };
}

await describe({
  name: 'migration bookkeeping',
  children: [
    it({
      name: 'isVersionInRangeFormat reports concrete versions as concrete',
      fn: async () => {
        expect(isVersionInRangeFormat('1.0.0',),).toBe(false,);
        expect(isVersionInRangeFormat('0.0.0',),).toBe(false,);
      },
    },),

    it({
      name: 'isVersionInRangeFormat reports range syntax as range format',
      fn: async () => {
        expect(isVersionInRangeFormat('>=2.0.0',),).toBe(true,);
        expect(isVersionInRangeFormat('1.x',),).toBe(true,);
        expect(isVersionInRangeFormat('*',),).toBe(true,);
      },
    },),

    it({
      name: 'shouldPerformMigration runs a concrete version after the previous migrated version at or below the target',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '1.0.0',
          previousMigratedVersion: '0.9.0',
          versionToMigrate: '1.0.0',
        },),).toBe(true,);
        expect(shouldPerformMigration({
          candidateVersion: '1.0.0',
          previousMigratedVersion: '0.9.0',
          versionToMigrate: '2.0.0',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'shouldPerformMigration skips a concrete version at or below the previous migrated version',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '1.0.0',
          previousMigratedVersion: '1.0.0',
          versionToMigrate: '2.0.0',
        },),).toBe(false,);
        expect(shouldPerformMigration({
          candidateVersion: '0.9.0',
          previousMigratedVersion: '1.0.0',
          versionToMigrate: '2.0.0',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'shouldPerformMigration skips a concrete version above the target',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '2.0.0',
          previousMigratedVersion: '0.0.0',
          versionToMigrate: '1.0.0',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'shouldPerformMigration skips a range the previous migrated version already satisfies',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '>=1.0.0',
          previousMigratedVersion: '1.2.0',
          versionToMigrate: '2.0.0',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'shouldPerformMigration runs a range the target satisfies and the previous version does not cover',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '>=1.0.0',
          previousMigratedVersion: '0.9.0',
          versionToMigrate: '1.5.0',
        },),).toBe(true,);
        expect(shouldPerformMigration({
          candidateVersion: '1.x',
          previousMigratedVersion: '2.0.0',
          versionToMigrate: '1.5.0',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'shouldPerformMigration skips a range the target does not satisfy',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '>=2.0.0',
          previousMigratedVersion: '0.9.0',
          versionToMigrate: '1.5.0',
        },),).toBe(false,);
      },
    },),

    it({
      name: 'shouldPerformMigration treats the 0.0.0 sentinel as never covering a range',
      fn: async () => {
        expect(shouldPerformMigration({
          candidateVersion: '>=0.0.0',
          previousMigratedVersion: '0.0.0',
          versionToMigrate: '1.0.0',
        },),).toBe(true,);
      },
    },),

    it({
      name: 'applyMigrations records the project version and runs no steps for a brand-new store',
      fn: async () => {
        /**
         Stub host whose config file does not exist yet.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: false,
        },);
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        applyMigrations({
          host: stub.host,
          migrations: {
            '1.0.0': function wouldRun(): void {
              ran.push('1.0.0',);
            },
          },
          projectVersion: '2.0.0',
        },);
        expect(ran,).toHaveLength(0,);
        expect(stub.calls.recordVersion,).toEqual([
          '2.0.0',
        ],);
      },
    },),

    it({
      name: 'applyMigrations runs every applicable step for an existing store without a recorded version',
      fn: async () => {
        /**
         Stub host whose existing file records no version.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        applyMigrations({
          host: stub.host,
          migrations: {
            '1.0.0': function toV1(): void {
              ran.push('1.0.0',);
            },
            '2.0.0': function toV2(): void {
              ran.push('2.0.0',);
            },
          },
          projectVersion: '2.0.0',
        },);
        expect(ran,).toEqual([
          '1.0.0',
          '2.0.0',
        ],);
        expect(stub.calls.recordVersion,).toEqual([
          '1.0.0',
          '2.0.0',
        ],);
      },
    },),

    it({
      name: 'applyMigrations throws MissingProjectVersionError when the project version is empty',
      fn: async () => {
        /**
         Stub host under test.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        /**
         Error thrown for the empty project version.
         */
        const error = captureThrown(function applyWithoutVersion(): void {
          applyMigrations({
            host: stub.host,
            migrations: {},
            projectVersion: '',
          },);
        },);
        expect(error,).toBeInstanceOf(MissingProjectVersionError,);
        expect(caughtMessage(error,),).toBe('Please specify the `projectVersion` option.',);
      },
    },),

    it({
      name: 'applyMigrations merges missing defaults into the file before the steps run',
      fn: async () => {
        /**
         Stub host whose file lacks one default key.
         */
        const stub = createMigrationHostStub({
          rawStore: {
            theme: 'dark',
          },
          configFileExists: true,
        },);
        applyMigrations({
          host: stub.host,
          migrations: {},
          projectVersion: '1.0.0',
          defaults: {
            theme: 'light',
            locale: 'en',
          },
        },);
        expect(stub.calls.writeStoreWithoutEvents,).toEqual([
          {
            theme: 'dark',
            locale: 'en',
          },
        ],);
      },
    },),

    it({
      name: 'applyMigrations leaves the file alone when its contents already cover every default',
      fn: async () => {
        /**
         Stub host whose file already carries the only default,
         built as a null-prototype store like the merge target.
         */
        const stub = createMigrationHostStub({
          rawStore: Object.assign(createPlainObject(), {
            theme: 'dark',
          },),
          configFileExists: true,
        },);
        applyMigrations({
          host: stub.host,
          migrations: {},
          projectVersion: '1.0.0',
          defaults: {
            theme: 'light',
          },
        },);
        expect(stub.calls.writeStoreWithoutEvents,).toHaveLength(0,);
      },
    },),

    it({
      name: 'runMigrationSteps runs steps in their configured key order',
      fn: async () => {
        /**
         Stub host without a recorded version.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        runMigrationSteps({
          host: stub.host,
          migrations: {
            '2.0.0': function toV2First(): void {
              ran.push('2.0.0',);
            },
            '1.0.0': function toV1Second(): void {
              ran.push('1.0.0',);
            },
          },
          projectVersion: '2.0.0',
        },);
        expect(ran,).toEqual([
          '2.0.0',
          '1.0.0',
        ],);
      },
    },),

    it({
      name: 'runMigrationSteps gives beforeEachMigration the store and step context',
      fn: async () => {
        /**
         Stub host without a recorded version.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        /**
         Step contexts received by the hook.
         */
        const hookContexts: BeforeEachMigrationContext[] = [];
        /**
         Store handles received by the hook.
         */
        const hookStores: Conf[] = [];
        runMigrationSteps({
          host: stub.host,
          migrations: {
            '1.0.0': function toV1(): void {},
            '2.0.0': function toV2(): void {},
          },
          projectVersion: '2.0.0',
          beforeEachMigration: function recordStep(details,): void {
            hookContexts.push(details.context,);
            hookStores.push(details.store,);
          },
        },);
        expect(hookContexts,).toEqual([
          {
            fromVersion: '0.0.0',
            toVersion: '1.0.0',
            finalVersion: '2.0.0',
            versions: [
              '1.0.0',
              '2.0.0',
            ],
          },
          {
            fromVersion: '1.0.0',
            toVersion: '2.0.0',
            finalVersion: '2.0.0',
            versions: [
              '1.0.0',
              '2.0.0',
            ],
          },
        ],);
        expect(hookStores[0],).toBe(stub.host.store,);
        expect(hookStores[1],).toBe(stub.host.store,);
      },
    },),

    it({
      name: 'runMigrationSteps restores the pre-step snapshot and throws MigrationFailedError when a step throws',
      fn: async () => {
        /**
         Stub host without a recorded version.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        /**
         Error thrown while the second step runs.
         */
        const error = captureThrown(function runFailingMigration(): void {
          runMigrationSteps({
            host: stub.host,
            migrations: {
              '1.0.0': function toV1(): void {
                stub.state.userStore = {
                  phase: 'one',
                };
              },
              '2.0.0': function toV2ThenThrow(): void {
                stub.state.userStore = {
                  phase: 'two',
                };
                throw new Error('boom',);
              },
            },
            projectVersion: '2.0.0',
          },);
        },);
        expect(error,).toBeInstanceOf(MigrationFailedError,);
        expect(caughtMessage(error,).startsWith(MIGRATION_FAILURE_PREFIX,),).toBe(true,);
        expect(stub.calls.writeUserStore,).toEqual([
          {
            phase: 'one',
          },
        ],);
        expect(stub.state.userStore,).toEqual({
          phase: 'one',
        },);
      },
    },),

    it({
      name: 'runMigrationSteps records concrete step versions but never records range keys',
      fn: async () => {
        /**
         Stub host without a recorded version.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        runMigrationSteps({
          host: stub.host,
          migrations: {
            '>=1.0.0': function forRange(): void {
              ran.push('>=1.0.0',);
            },
            '2.0.0': function toV2(): void {
              ran.push('2.0.0',);
            },
          },
          projectVersion: '2.0.0',
        },);
        expect(ran,).toEqual([
          '>=1.0.0',
          '2.0.0',
        ],);
        expect(stub.calls.recordVersion,).toEqual([
          '2.0.0',
        ],);
      },
    },),

    it({
      name: 'runMigrationSteps records the project version when the steps stop below it',
      fn: async () => {
        /**
         Stub host without a recorded version.
         */
        const stub = createMigrationHostStub({
          rawStore: {},
          configFileExists: true,
        },);
        runMigrationSteps({
          host: stub.host,
          migrations: {
            '1.0.0': function toV1(): void {},
          },
          projectVersion: '2.0.0',
        },);
        expect(stub.calls.recordVersion,).toEqual([
          '1.0.0',
          '2.0.0',
        ],);
        expect(stub.calls.recordVersion.at(-1,),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'runMigrationSteps treats a range-format recorded version as 0.0.0',
      fn: async () => {
        /**
         Stub host whose earlier failure left a range-format version in the
         file.
         */
        const stub = createMigrationHostStub({
          rawStore: {
            __internal__: {
              migrations: {
                version: '>=1.0.0',
              },
            },
          },
          configFileExists: true,
        },);
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        /**
         Step contexts received by the hook.
         */
        const hookContexts: BeforeEachMigrationContext[] = [];
        runMigrationSteps({
          host: stub.host,
          migrations: {
            '0.5.0': function toV050(): void {
              ran.push('0.5.0',);
            },
          },
          projectVersion: '1.0.0',
          beforeEachMigration: function recordStep(details,): void {
            hookContexts.push(details.context,);
          },
        },);
        expect(ran,).toEqual([
          '0.5.0',
        ],);
        expect(hookContexts[0]?.fromVersion,).toBe('0.0.0',);
        expect(stub.calls.recordVersion,).toEqual([
          '0.5.0',
          '1.0.0',
        ],);
      },
    },),

    it({
      name: 'runMigrationSteps skips steps the recorded version already covers',
      fn: async () => {
        /**
         Stub host whose file records an earlier migrated version.
         */
        const stub = createMigrationHostStub({
          rawStore: {
            __internal__: {
              migrations: {
                version: '1.0.0',
              },
            },
          },
          configFileExists: true,
        },);
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        runMigrationSteps({
          host: stub.host,
          migrations: {
            '1.0.0': function alreadyCovered(): void {
              ran.push('1.0.0',);
            },
            '2.0.0': function toV2(): void {
              ran.push('2.0.0',);
            },
          },
          projectVersion: '2.0.0',
        },);
        expect(ran,).toEqual([
          '2.0.0',
        ],);
        expect(stub.calls.recordVersion,).toEqual([
          '2.0.0',
        ],);
      },
    },),
  ],
},);
