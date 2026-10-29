/**
 Migration behavior: version bookkeeping,
 range keys,
 step ordering,
 rollback,
 hooks,
 internal-key handling,
 and schema-driven coercion.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createConf,
  MigrationFailedError,
  MissingProjectVersionError,
  type BeforeEachMigrationContext,
  type Migrations,
} from '../dist/final/neutral/index.mjs';

import {
  createTempDirectory,
  readMigrationVersion,
  writeConfigFile,
} from './test-support.ts';

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
  name: 'migrations',
  children: [
    it({
      name: 'records the project version as the initial migrated version for a new store',
      fn: async () => {
        /**
         Directory whose store does not exist yet.
         */
        const directory = createTempDirectory();
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        /**
         Store that starts at its project version without running steps.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          migrations: {
            '2.0.0': function wouldRun(store,): void {
              ran.push('2.0.0',);
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(ran,).toHaveLength(0,);
        expect(conf.get('foo',),).toBeUndefined();
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'does not run migration steps for a store that does not exist yet',
      fn: async () => {
        /**
         Directory whose store does not exist yet.
         */
        const directory = createTempDirectory();
        /**
         Versions of the steps that ran.
         */
        const ran: string[] = [];
        /**
         Store whose defaults stand in for the fresh data migrations must not touch.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          defaults: {
            fromDefaults: true,
          },
          migrations: {
            '1.0.0': function firstStep(): void {
              ran.push('1.0.0',);
            },
            '>=2.0.0': function rangeStep(): void {
              ran.push('>=2.0.0',);
            },
          },
        },);
        expect(ran,).toEqual([],);
        expect(conf.get('fromDefaults',),).toBe(true,);
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'runs migration steps for an existing store without a recorded version',
      fn: async () => {
        /**
         Directory holding a pre-migration config file without bookkeeping.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            legacy: 'value',
          },
        },);
        /**
         Store migrating the legacy file forward.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          migrations: {
            '2.0.0': function migrateLegacy(store,): void {
              store.set({
                key: 'migrated',
                value: true,
              },);
            },
          },
        },);
        expect(conf.get('migrated',),).toBe(true,);
        expect(conf.get('legacy',),).toBe('value',);
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'records the project version after a migration pass',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         First store migrating to version 1.0.0.
         */
        const first = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(first.get('foo',),).toBe('bar',);
        expect(readMigrationVersion({
          configPath: first.path,
        },),).toBe('1.0.0',);

        /**
         Second store migrating from 1.0.0 to 2.0.0.
         */
        const second = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
            '1.0.1': function toOnePointOne(store,): void {
              store.set({
                key: 'foo',
                value: 'baz',
              },);
            },
            '2.0.0': function toTwo(store,): void {
              store.set({
                key: 'foo',
                value: 'bazel',
              },);
            },
          },
        },);
        expect(second.get('foo',),).toBe('bazel',);
        expect(readMigrationVersion({
          configPath: second.path,
        },),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'does not run migrations when the project version is unchanged',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store migrating to version 1.0.0.
         */
        const first = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(first.get('foo',),).toBe('bar',);

        /**
         Store reopened at the same version,
         whose step must not run again.
         */
        const second = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toOneAgain(store,): void {
              store.set({
                key: 'foo',
                value: 'baz',
              },);
            },
          },
        },);
        expect(second.get('foo',),).toBe('bar',);
      },
    },),

    it({
      name: 'runs range-condition migrations and records the concrete project version',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store whose only migration key is a range.
         */
        const first = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '>=1.0.0': function rangeOne(store,): void {
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(first.get('foo',),).toBe('bar',);
        expect(readMigrationVersion({
          configPath: first.path,
        },),).toBe('1.0.0',);

        /**
         Store whose later pass sees new range keys.
         */
        const second = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          migrations: {
            '>=1.0.0 <2.0.0': function rangeBelowTwo(store,): void {
              store.set({
                key: 'foo',
                value: 'baz',
              },);
            },
            '>=2.0.0': function rangeTwo(store,): void {
              store.set({
                key: 'foo',
                value: 'bazel',
              },);
            },
          },
        },);
        expect(second.get('foo',),).toBe('bazel',);
        expect(readMigrationVersion({
          configPath: second.path,
        },),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'runs migration keys whose ranges cover the target version and skips future keys',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Order of the migration keys that ran.
         */
        const tracker: string[] = [];
        /**
         Store mixing range and concrete migration keys.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '1.8.0',
          migrations: {
            '>=1.0.0 <2.0.0': function rangeOne(store,): void {
              tracker.push('range1',);
              store.set({
                key: 'range1',
                value: true,
              },);
            },
            '>=1.5.0': function rangeTwo(store,): void {
              tracker.push('range2',);
              store.set({
                key: 'range2',
                value: true,
              },);
            },
            '1.8.0': function exact(store,): void {
              tracker.push('exact',);
              store.set({
                key: 'exact',
                value: true,
              },);
            },
            '>2.0.0': function future(store,): void {
              tracker.push('future',);
              store.set({
                key: 'future',
                value: true,
              },);
            },
          },
        },);
        expect(tracker,).toEqual([
          'range1',
          'range2',
          'exact',
        ],);
        expect(conf.get('future',),).toBeUndefined();
      },
    },),

    it({
      name: 'runs migration steps in ascending version order',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Order of the migration keys that ran.
         */
        const tracker: string[] = [];
        /**
         Store whose steps run across several versions.
         */
        createConf({
          cwd: directory,
          projectVersion: '3.0.0',
          migrations: {
            '1.0.0': function toOne(): void {
              tracker.push('v1',);
            },
            '1.5.0': function toOneFive(): void {
              tracker.push('v1.5',);
            },
            '2.0.0': function toTwo(): void {
              tracker.push('v2',);
            },
            '3.0.0': function toThree(): void {
              tracker.push('v3',);
            },
          },
        },);
        expect(tracker,).toEqual([
          'v1',
          'v1.5',
          'v2',
          'v3',
        ],);
      },
    },),

    it({
      name: 'records non-numeric version keys and never reruns them',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store whose project version is a prerelease.
         */
        const first = createConf({
          cwd: directory,
          projectVersion: '1.0.0-alpha',
          migrations: {
            '1.0.0-alpha': function toAlpha(store,): void {
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(first.get('foo',),).toBe('bar',);
        expect(readMigrationVersion({
          configPath: first.path,
        },),).toBe('1.0.0-alpha',);

        /**
         Store moving from the alpha to the beta prerelease.
         */
        const second = createConf({
          cwd: directory,
          projectVersion: '1.0.0-beta',
          migrations: {
            '1.0.0-alpha': function toAlphaAgain(store,): void {
              store.set({
                key: 'foo',
                value: 'baz',
              },);
            },
            '1.0.0-beta': function toBeta(store,): void {
              store.set({
                key: 'foo',
                value: 'bazel',
              },);
            },
          },
        },);
        expect(second.get('foo',),).toBe('bazel',);
        expect(readMigrationVersion({
          configPath: second.path,
        },),).toBe('1.0.0-beta',);
      },
    },),

    it({
      name: 'never records a range version even when a later step fails',
      fn: async () => {
        /**
         Mirrors upstream `migrations - should not record a range version when
         a later migration fails`: recording the range key would make every
         later construction throw `Invalid Version`.
         */
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Migrations whose range key runs before a failing concrete step.
         */
        const migrations: Migrations<Record<string, unknown>> = {
          '>=1.0.0': function rangeStep(store,): void {
            store.set({
              key: 'rangeMigrationRan',
              value: true,
            },);
          },
          '1.5.0': function failingStep(): void {
            throw new Error('a later migration failed');
          },
        };
        expect(function constructFailingPass(): unknown {
          return createConf({
            cwd: directory,
            projectVersion: '2.0.0',
            migrations,
          },);
        },).toThrow('a later migration failed',);
        expect(function constructFailingPassAgain(): unknown {
          return createConf({
            cwd: directory,
            projectVersion: '2.0.0',
            migrations,
          },);
        },).toThrow('a later migration failed',);

        /**
         Store opened without migrations to inspect the recorded version.
         */
        const reopened = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
        },);
        expect(readMigrationVersion({
          configPath: reopened.path,
        },),).toBeUndefined();
      },
    },),

    it({
      name: 'recovers when the file holds a range version from an earlier failed pass',
      fn: async () => {
        /**
         Directory whose bookkeeping was poisoned with a range version.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            __internal__: {
              migrations: {
                version: '>=1.0.0',
              },
            },
          },
        },);
        /**
         Store that must treat the range version as unknown instead of throwing.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          migrations: {
            '1.5.0': function migrate(store,): void {
              store.set({
                key: 'migrated',
                value: true,
              },);
            },
          },
        },);
        expect(conf.get('migrated',),).toBe(true,);
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('2.0.0',);
      },
    },),

    it({
      name: 'rolls a failed step back to its pre-step snapshot and keeps earlier steps',
      fn: async () => {
        /**
         Directory holding the pre-migration store.
         */
        const directory = createTempDirectory();
        /**
         Store seeding the value the migration changes.
         */
        const seed = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
        },);
        seed.set({
          key: 'foo',
          value: 'bar',
        },);

        expect(function constructFailingPass(): unknown {
          return createConf({
            cwd: directory,
            projectVersion: '2.0.0',
            migrations: {
              '1.5.0': function succeeds(store,): void {
                store.set({
                  key: 'foo',
                  value: 'baz',
                },);
              },
              '2.0.0': function fails(): void {
                throw new Error('Oops! This migration failed');
              },
            },
          },);
        },).toThrow(MigrationFailedError,);
        expect(function constructFailingPass(): unknown {
          return createConf({
            cwd: directory,
            projectVersion: '2.0.0',
            migrations: {
              '1.5.0': function succeeds(store,): void {
                store.set({
                  key: 'foo',
                  value: 'baz',
                },);
              },
              '2.0.0': function fails(): void {
                throw new Error('Oops! This migration failed');
              },
            },
          },);
        },).toThrow('Something went wrong during the migration',);
        expect(function constructFailingPass(): unknown {
          return createConf({
            cwd: directory,
            projectVersion: '2.0.0',
            migrations: {
              '1.5.0': function succeeds(store,): void {
                store.set({
                  key: 'foo',
                  value: 'baz',
                },);
              },
              '2.0.0': function fails(): void {
                throw new Error('Oops! This migration failed');
              },
            },
          },);
        },).toThrow('Oops! This migration failed',);

        /**
         Store reopened without migrations to inspect the restored state.
         */
        const reopened = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
        },);
        expect(reopened.get('foo',),).toBe('baz',);
        expect(readMigrationVersion({
          configPath: reopened.path,
        },),).toBe('1.5.0',);
      },
    },),

    it({
      name: 'throws MissingProjectVersionError when migrations are configured without a projectVersion',
      fn: async () => {
        /**
         Store options carrying migrations but no target version.
         */
        const directory = createTempDirectory();
        expect(function constructWithoutVersion(): unknown {
          return createConf({
            cwd: directory,
            migrations: {
              '1.0.0': function toOne(): void {},
            },
          },);
        },).toThrow(MissingProjectVersionError,);
        expect(function constructWithoutVersion(): unknown {
          return createConf({
            cwd: directory,
            migrations: {
              '1.0.0': function toOne(): void {},
            },
          },);
        },).toThrow('Please specify the `projectVersion` option.',);
      },
    },),

    it({
      name: 'does not throw when no migrations are configured and no projectVersion is given',
      fn: async () => {
        /**
         Store without migrations and without a target version.
         */
        const directory = createTempDirectory();
        expect(function constructWithoutVersion(): unknown {
          return createConf({
            cwd: directory,
          },);
        },).not.toThrow();
        /**
         Store that must stay usable afterwards.
         */
        const conf = createConf({
          cwd: directory,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(conf.get('foo',),).toBe('bar',);
      },
    },),

    it({
      name: 'calls beforeEachMigration with the step context and stops at the throwing step',
      fn: async () => {
        /**
         Mirrors upstream `migrations - error handling and hook integration`.
         */
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Contexts the hook observed,
         in call order.
         */
        const contexts: BeforeEachMigrationContext[] = [];
        /**
         Construction wrapped so the thrown error can carry several assertions.
         */
        const error = captureThrown(function constructFailingPass(): void {
          createConf({
            cwd: directory,
            projectVersion: '2.0.0',
            beforeEachMigration: function hook({ context, }: {
              readonly context: BeforeEachMigrationContext;
            },): void {
              contexts.push(context,);
              if (context.toVersion === '1.5.0')
                throw new Error('Hook prevented migration');
            },
            migrations: {
              '1.0.0': function toOne(store,): void {
                store.set({
                  key: 'v1',
                  value: true,
                },);
              },
              '1.5.0': function toOneFive(store,): void {
                store.set({
                  key: 'v1.5',
                  value: true,
                },);
              },
              '2.0.0': function toTwo(store,): void {
                store.set({
                  key: 'v2',
                  value: true,
                },);
              },
            },
          },);
        },);
        expect(error,).toBeInstanceOf(MigrationFailedError,);
        expect(error,).toHaveProperty('message',);
        expect(contexts,).toEqual([
          {
            fromVersion: '0.0.0',
            toVersion: '1.0.0',
            finalVersion: '2.0.0',
            versions: [
              '1.0.0',
              '1.5.0',
              '2.0.0',
            ],
          },
          {
            fromVersion: '1.0.0',
            toVersion: '1.5.0',
            finalVersion: '2.0.0',
            versions: [
              '1.0.0',
              '1.5.0',
              '2.0.0',
            ],
          },
        ],);
      },
    },),

    it({
      name: 'preserves the internal key when the store is replaced wholesale',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store whose bookkeeping must survive store replacement.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'test',
                value: 'value1',
              },);
            },
          },
        },);
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('1.0.0',);

        conf.store = {
          newData: 'test',
        };
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('1.0.0',);

        conf.store = {};
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('1.0.0',);

        /**
         Store without dot notation whose replacement must also preserve bookkeeping.
         */
        const withoutDotNotation = createConf({
          cwd: createTempDirectory(),
          projectVersion: '1.0.0',
          accessPropertiesByDotNotation: false,
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'test',
                value: 'value',
              },);
            },
          },
        },);
        withoutDotNotation.store = {
          newData: 'test',
        };
        expect(readMigrationVersion({
          configPath: withoutDotNotation.path,
        },),).toBe('1.0.0',);
      },
    },),

    it({
      name: 'migrates nested keys through dot notation and literal keys without it',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store whose migration writes a nested key through dot notation.
         */
        const withDotNotation = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'nested.deep',
                value: 'value',
              },);
            },
          },
        },);
        expect(withDotNotation.get('nested.deep',),).toBe('value',);

        /**
         Store whose migration treats dotted keys as literal strings.
         */
        const literalDirectory = createTempDirectory();
        writeConfigFile({
          directory: literalDirectory,
          data: {},
        },);
        const withoutDotNotation = createConf({
          cwd: literalDirectory,
          projectVersion: '1.0.0',
          accessPropertiesByDotNotation: false,
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'migrated',
                value: true,
              },);
              store.set({
                key: 'key.with.dots',
                value: 'literal',
              },);
            },
          },
        },);
        expect(withoutDotNotation.get('key.with.dots',),).toBe('literal',);
        expect(withoutDotNotation.get('migrated',),).toBe(true,);
      },
    },),

    it({
      name: 'lets a migration fix schema-invalid data before validation runs',
      fn: async () => {
        /**
         Mirrors upstream `migrations - validation should not prevent
         migrations from running`.
         */
        /**
         Directory whose file violates the schema the store will enforce.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            age: '25',
            active: 'true',
          },
        },);
        /**
         Store whose migration repairs the violations before validation.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          schema: {
            age: {
              type: 'number',
            },
            active: {
              type: 'boolean',
            },
          },
          migrations: {
            '1.0.0': function repair(store,): void {
              store.set({
                key: 'age',
                value: 25,
              },);
              store.set({
                key: 'active',
                value: true,
              },);
            },
          },
        },);
        expect(conf.get('age',),).toBe(25,);
        expect(conf.get('active',),).toBe(true,);
      },
    },),

    it({
      name: 'coerces values to the schema types during migration',
      fn: async () => {
        /**
         Directory whose file holds strings where the schema wants numbers and booleans.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            port: '8080',
            enabled: 'true',
            count: '42',
          },
        },);
        /**
         Store whose migration coerces each value to its schema type.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          schema: {
            port: {
              type: 'number',
            },
            enabled: {
              type: 'boolean',
            },
            count: {
              type: 'number',
            },
          },
          migrations: {
            '1.0.0': function coerce(store,): void {
              store.set({
                key: 'port',
                value: 8_080,
              },);
              store.set({
                key: 'enabled',
                value: true,
              },);
              store.set({
                key: 'count',
                value: 42,
              },);
            },
          },
        },);
        expect(conf.get('port',),).toBe(8_080,);
        expect(conf.get('enabled',),).toBe(true,);
        expect(conf.get('count',),).toBe(42,);
      },
    },),

    it({
      name: 'transforms string data into the schema object shape during migration',
      fn: async () => {
        /**
         Directory whose file holds one packed string.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            settings: 'key1=value1,key2=value2',
          },
        },);
        /**
         Store whose migration unpacks the string into the schema shape.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          schema: {
            settings: {
              type: 'object',
              properties: {
                key1: {
                  type: 'string',
                },
                key2: {
                  type: 'string',
                },
              },
            },
          },
          migrations: {
            '1.0.0': function transform(store,): void {
              store.set({
                key: 'settings',
                value: {
                  key1: 'value1',
                  key2: 'value2',
                },
              },);
            },
          },
        },);
        expect(conf.get('settings',),).toEqual({
          key1: 'value1',
          key2: 'value2',
        },);
      },
    },),

    it({
      name: 'fixes multiple schema violations in one migration',
      fn: async () => {
        /**
         Directory whose file violates several schema properties at once.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {
            enabled: 'yes',
            count: '42',
          },
        },);
        /**
         Store whose migration repairs both violations.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '2.0.0',
          schema: {
            enabled: {
              type: 'boolean',
            },
            count: {
              type: 'number',
            },
          },
          migrations: {
            '2.0.0': function repair(store,): void {
              store.set({
                key: 'enabled',
                value: true,
              },);
              store.set({
                key: 'count',
                value: 42,
              },);
            },
          },
        },);
        expect(conf.get('enabled',),).toBe(true,);
        expect(conf.get('count',),).toBe(42,);
      },
    },),

    it({
      name: 'does not expose the internal key through the public surface',
      fn: async () => {
        /**
         Directory holding a pre-migration config file.
         */
        const directory = createTempDirectory();
        writeConfigFile({
          directory,
          data: {},
        },);
        /**
         Store whose bookkeeping must stay invisible.
         */
        const conf = createConf({
          cwd: directory,
          projectVersion: '1.0.0',
          migrations: {
            '1.0.0': function toOne(store,): void {
              store.set({
                key: 'foo',
                value: 'bar',
              },);
            },
          },
        },);
        expect(conf.store,).toEqual({
          foo: 'bar',
        },);
        expect(conf.get('__internal__',),).toBeUndefined();
        expect(conf.has('__internal__',),).toBe(false,);
        expect(conf.size,).toBe(1,);
        expect([...conf],).toEqual([
          [
            'foo',
            'bar',
          ],
        ],);
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBe('1.0.0',);
      },
    },),

    it({
      name: 'does not create migration metadata when migrations are not configured',
      fn: async () => {
        /**
         Store with a project version but no migrations.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          projectVersion: '1.0.0',
        },);
        expect(conf.has('__internal__',),).toBe(false,);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        expect(readMigrationVersion({
          configPath: conf.path,
        },),).toBeUndefined();
      },
    },),
  ],
},);
