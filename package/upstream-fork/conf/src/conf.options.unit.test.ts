/**
 Option handling of `createConf` ported from upstream `conf`'s test suite:
 file naming,
 custom serialization,
 path resolution,
 literal-key mode,
 created-file permissions,
 and missing-project-name rejection.
 
 @module
 */

import {
  existsSync,
  statSync,
  writeFileSync,
} from 'node:fs';

import {
  basename,
  extname,
  join,
  sep,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createConf,
  MissingProjectNameError,
} from '../dist/final/neutral/index.mjs';

import {
  createDisposableHome,
  createTempDirectory,
  restoreHome,
} from './test-support.ts';

/**
 Fixture value shared by the read and write cases,
 matching upstream `conf`'s test fixture.
 */
const FIXTURE = '🦄';

/**
 Runs a call expected to throw and returns the captured error so class and message text can be asserted.
 
 @param call - Call that must throw.
 
 @returns Captured thrown error.
 */
function captureThrown(call: () => unknown,): Error {
  try {
    call();
  }
  catch (error) {
    return error as Error;
  }
  throw new Error('expected the call to throw, but it returned',);
}

await describe({
  name: createConf.name,
  children: [
    //region File naming

    it({
      name: '`configName` option',
      fn: async () => {
        /**
         Config file base name the caller requests instead of `config`.
         */
        const configName = 'alt-config';
        /**
         Store persisting under the caller-chosen file name.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          configName,
        },);
        expect(conf.get('foo',),).toBeUndefined();
        conf.set({
          key: 'foo',
          value: FIXTURE,
        },);
        expect(conf.get('foo',),).toBe(FIXTURE,);
        expect(basename(
          conf.path,
          '.json',
        ),).toBe(configName,);
        expect(existsSync(conf.path,),).toBe(true,);
      },
    },),
    it({
      name: '`fileExtension` option',
      fn: async () => {
        /**
         File extension the caller requests instead of `json`.
         */
        const fileExtension = 'alt-ext';
        /**
         Store persisting under the caller-chosen extension.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          fileExtension,
        },);
        expect(conf.get('foo',),).toBeUndefined();
        conf.set({
          key: 'foo',
          value: FIXTURE,
        },);
        expect(conf.get('foo',),).toBe(FIXTURE,);
        expect(extname(conf.path,),).toBe(`.${fileExtension}`,);
      },
    },),
    it({
      name: '`fileExtension` option = empty string',
      fn: async () => {
        /**
         Config file base name kept bare when no extension is wanted.
         */
        const configName = 'unicorn';
        /**
         Store persisting to an extensionless file.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          fileExtension: '',
          configName,
        },);
        expect(basename(conf.path,),).toBe(configName,);
      },
    },),
    it({
      name: '`fileExtension` option - leading dots are stripped',
      fn: async () => {
        /**
         Store persisting under a `.json`-style caller extension.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          fileExtension: '.alt-ext',
        },);
        expect(extname(conf.path,),).toBe('.alt-ext',);
      },
    },),

    //endregion File naming

    //region Custom formats

    it({
      name: '`serialize` and `deserialize` options',
      fn: async () => {
        /**
         Text the custom serializer writes for the fixture store.
         */
        const serialized = `foo:${FIXTURE}`;
        /**
         Store object the custom round trip must return.
         */
        const deserialized = {
          foo: FIXTURE,
        };
        /**
         Call counts proving each hook runs exactly once through the round trip.
         */
        const counts = {
          serialize: 0,
          deserialize: 0,
        };
        /**
         Custom serializer asserting it receives the store object.
         */
        const serialize = (value: Record<string, unknown>,): string => {
          counts.serialize += 1;
          expect(value,).toEqual(deserialized,);
          return serialized;
        };
        /**
         Custom deserializer asserting it receives the serialized text.
         */
        const deserialize = (text: string,): Record<string, unknown> => {
          counts.deserialize += 1;
          expect(text,).toBe(serialized,);
          return deserialized;
        };
        /**
         Store persisting through the custom format.
         */
        const conf = createConf({
          cwd: createTempDirectory(),
          serialize,
          deserialize,
        },);
        expect(conf.store,).toEqual({},);
        conf.store = deserialized;
        expect(conf.store,).toEqual(deserialized,);
        expect(counts.serialize,).toBe(1,);
        expect(counts.deserialize,).toBe(1,);
      },
    },),

    //endregion Custom formats

    //region Path resolution

    it({
      name: '`projectName` option',
      fn: async () => {
        /**
         Disposable home the config directory must resolve inside.
         */
        const home = createDisposableHome();
        /**
         Restore the injected home environment when the test ends.
         */
        using restore = {
          [Symbol.dispose](): void {
            restoreHome(home,);
          },
        };
        /**
         Project name appearing in the resolved config path.
         */
        const projectName = 'conf-fixture-project-name';
        /**
         Store resolving its config path through the project name.
         */
        const conf = createConf({ projectName, },);
        expect(conf.get('foo',),).toBeUndefined();
        conf.set({
          key: 'foo',
          value: FIXTURE,
        },);
        expect(conf.get('foo',),).toBe(FIXTURE,);
        expect(conf.path.includes(projectName,),).toBe(true,);
        expect(conf.path.startsWith(home.home,),).toBe(true,);
        expect(existsSync(conf.path,),).toBe(true,);
      },
    },),
    it({
      name: 'no `suffix` option',
      fn: async () => {
        /**
         Disposable home the config directory must resolve inside.
         */
        const home = createDisposableHome();
        /**
         Restore the injected home environment when the test ends.
         */
        using restore = {
          [Symbol.dispose](): void {
            restoreHome(home,);
          },
        };
        /**
         Store whose directory keeps the default `-nodejs` suffix.
         */
        const conf = createConf({ projectName: 'conf-suffixless-project', },);
        expect(conf.path.includes('-nodejs',),).toBe(true,);
        expect(conf.path.startsWith(home.home,),).toBe(true,);
      },
    },),
    it({
      name: 'with `suffix` option set to empty string',
      fn: async () => {
        /**
         Disposable home the config directory must resolve inside.
         */
        const home = createDisposableHome();
        /**
         Restore the injected home environment when the test ends.
         */
        using restore = {
          [Symbol.dispose](): void {
            restoreHome(home,);
          },
        };
        /**
         Empty suffix dropping the `-nodejs` directory suffix.
         */
        const projectSuffix = '';
        /**
         Project name that must appear bare in the config path.
         */
        const projectName = 'conf-temp1-project';
        /**
         Store whose directory keeps the bare project name.
         */
        const conf = createConf({
          projectSuffix,
          projectName,
        },);
        /**
         Path segments one of which must be the bare project name.
         */
        const configPathSegments = conf.path.split(sep,);
        expect(configPathSegments.indexOf(projectName,),).toBeGreaterThan(-1,);
      },
    },),
    it({
      name: 'with `projectSuffix` option set to non-empty string',
      fn: async () => {
        /**
         Disposable home the config directory must resolve inside.
         */
        const home = createDisposableHome();
        /**
         Restore the injected home environment when the test ends.
         */
        using restore = {
          [Symbol.dispose](): void {
            restoreHome(home,);
          },
        };
        /**
         Caller-chosen suffix joining the directory name.
         */
        const projectSuffix = 'new-projectSuffix';
        /**
         Project name the suffix attaches to.
         */
        const projectName = 'conf-temp2-project';
        /**
         Store whose directory carries the suffixed project name.
         */
        const conf = createConf({
          projectSuffix,
          projectName,
        },);
        /**
         Directory name expected in the resolved config path.
         */
        const expectedRootName = `${projectName}-${projectSuffix}`;
        /**
         Path segments one of which must be the suffixed project name.
         */
        const configPathSegments = conf.path.split(sep,);
        expect(configPathSegments.indexOf(expectedRootName,),).toBeGreaterThan(-1,);
      },
    },),
    it({
      name: '`cwd` option overrides `projectName` option',
      fn: async () => {
        /**
         Caller-supplied directory the store must keep.
         */
        const cwd = createTempDirectory();
        /**
         Store whose cwd wins over the empty project name.
         */
        const conf = createConf({
          cwd,
          projectName: '',
        },);
        expect(conf.path.startsWith(cwd,),).toBe(true,);
        expect(conf.get('foo',),).toBeUndefined();
        conf.set({
          key: 'foo',
          value: FIXTURE,
        },);
        expect(conf.get('foo',),).toBe(FIXTURE,);
      },
    },),

    //endregion Path resolution

    //region Literal-key mode

    it({
      name: '.get() - without dot notation',
      fn: async () => {
        /**
         Store whose keys stay literal instead of addressing nested properties.
         */
        const config = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
        },);
        expect(config.get('foo',),).toBeUndefined();
        expect(config.get({
          key: 'foo',
          defaultValue: FIXTURE,
        },),).toBe(FIXTURE,);
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        expect(config.get('foo',),).toBe(FIXTURE,);
      },
    },),
    it({
      name: '.set() - without dot notation',
      fn: async () => {
        /**
         Store whose dotted keys stay literal on write and read.
         */
        const config = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
        },);
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        config.set({
          key: 'baz.boo',
          value: FIXTURE,
        },);
        expect(config.get('foo',),).toBe(FIXTURE,);
        expect(config.get('baz.boo',),).toBe(FIXTURE,);
      },
    },),
    it({
      name: '.set() - with object - without dot notation',
      fn: async () => {
        /**
         Store whose multi-item write keeps nested values whole under literal keys.
         */
        const config = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
        },);
        config.set({
          values: {
            foo1: 'bar1',
            foo2: 'bar2',
            baz: {
              boo: 'foo',
              foo: {
                bar: 'baz',
              },
            },
          },
        },);
        expect(config.get('foo1',),).toBe('bar1',);
        expect(config.get('foo2',),).toBe('bar2',);
        expect(config.get('baz',),).toEqual({
          boo: 'foo',
          foo: {
            bar: 'baz',
          },
        },);
        // Dotted reads no longer descend into the nested value.
        expect(config.get('baz.boo',),).toBeUndefined();
        expect(config.get('baz.foo.bar',),).toBeUndefined();
      },
    },),
    it({
      name: '.has() - without dot notation',
      fn: async () => {
        /**
         Store whose dotted keys probe as literal keys.
         */
        const config = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
        },);
        config.set({
          key: 'foo',
          value: FIXTURE,
        },);
        config.set({
          key: 'baz.boo',
          value: FIXTURE,
        },);
        expect(config.has('foo',),).toBe(true,);
        expect(config.has('baz.boo',),).toBe(true,);
        expect(config.has('missing',),).toBe(false,);
      },
    },),
    it({
      name: '.delete() - without dot notation',
      fn: async () => {
        /**
         Store whose literal dotted keys are deleted one full key at a time.
         */
        const config = createConf({
          cwd: createTempDirectory(),
          accessPropertiesByDotNotation: false,
        },);
        config.set({
          key: 'foo',
          value: 'bar',
        },);
        config.set({
          key: 'baz.boo',
          value: true,
        },);
        config.set({
          key: 'baz.foo.bar',
          value: 'baz',
        },);
        config.delete('foo',);
        expect(config.get('foo',),).toBeUndefined();
        config.delete('baz.boo',);
        expect(config.get('baz.boo',),).toBeUndefined();
        // Deleting a literal prefix key leaves the longer literal key intact.
        config.delete('baz.foo',);
        expect(config.get('baz.foo',),).toBeUndefined();
        expect(config.get('baz.foo.bar',),).toBe('baz',);
        config.set({
          key: 'foo.bar.baz',
          value: { awesome: 'icecream', },
        },);
        config.set({
          key: 'foo.bar.zoo',
          value: { awesome: 'redpanda', },
        },);
        config.delete('foo.bar.baz',);
        expect(config.get('foo.bar.zoo',),).toEqual({ awesome: 'redpanda', },);
      },
    },),

    //endregion Literal-key mode

    //region File permissions

    it({
      name: 'configFileMode',
      fn: async () => {
        /**
         Directory holding the created config file and the umask probe beside it.
         */
        const directory = createTempDirectory();
        /**
         Store requesting owner-only permissions on its created file.
         */
        const conf = createConf({
          cwd: directory,
          configFileMode: 0o600,
        },);
        conf.set({
          key: 'foo',
          value: 'bar',
        },);
        /**
         Probe file created with a fully open mode whose granted bits reveal the process umask.
         */
        const probePath = join(
          directory,
          'umask-probe.bin',
        );
        writeFileSync(
          probePath,
          '',
          {
            mode: 0o777,
          },
        );
        /**
         Probe permission bits after the umask reduction.
         */
        const probeMode = statSync(probePath,).mode & 0o777;
        /**
         Process umask inferred from the probe's reduction.
         */
        const umask = 0o777 & (~probeMode);
        /**
         Permission bits the config file actually carries.
         */
        const grantedMode = statSync(conf.path,).mode & 0o777;
        expect(grantedMode,).toBe(0o600 & (~umask),);
      },
    },),

    //endregion File permissions

    //region Option validation

    it({
      name: 'MissingProjectNameError when neither cwd nor projectName',
      fn: async () => {
        /**
         Error thrown when construction carries no location at all.
         */
        const missingError = captureThrown((): unknown => createConf({},),);
        expect(missingError,).toBeInstanceOf(MissingProjectNameError,);
        expect(missingError.name,).toBe('MissingProjectNameError',);
        expect(missingError.message,).toBe('Please specify the `projectName` option.',);
        /**
         Error thrown when projectName is present but empty.
         */
        const emptyNameError = captureThrown((): unknown => createConf({ projectName: '', },),);
        expect(emptyNameError,).toBeInstanceOf(MissingProjectNameError,);
        expect(emptyNameError.message,).toBe('Please specify the `projectName` option.',);
      },
    },),

    //endregion Option validation
  ],
},);
