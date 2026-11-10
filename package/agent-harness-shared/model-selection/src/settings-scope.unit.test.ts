/**
 Unit tests for pi settings scope loading.
 
 @module
 */

import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { homedir, } from 'node:os';
import {
  isAbsolute,
  join,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  getSettingsPaths,
  loadSettingsScopePatterns,
} from '../dist/final/node/index.mjs';
import { captureAsyncError, } from './test-fixtures.ts';

//region Fixtures

/** Global enabled model pattern fixture. */
const GLOBAL_PATTERN = 'cheap/*';

/** Project enabled model pattern fixture. */
const PROJECT_PATTERN = 'expensive/reviewer';

/**
 Set one environment variable for the duration of a `using` scope.

 @param name - environment variable name

 @param value - value to set, or `undefined` to remove the variable

 @returns disposable restoring the previous state

 @mutates process.env - sets the named variable and restores it on dispose
 */
function withEnvironmentVariable(
  {
    name,
    value,
  }: {
    readonly name: string;
    readonly value?: string;
  },
): Disposable {
  /**
   Value the variable had before the scope started.
   */
  const previous = process.env[name];
  if (value === undefined)
    Reflect.deleteProperty(process.env, name,);
  else
    process.env[name] = value;
  return {
    [Symbol.dispose]: function restoreEnvironmentVariable() {
      if (previous === undefined)
        Reflect.deleteProperty(process.env, name,);
      else
        process.env[name] = previous;
    },
  };
}

//endregion Fixtures

await describe({
  name: loadSettingsScopePatterns.name,
  children: [
    it({
      name: 'returns unrestricted scope when settings are absent',
      fn: async function testAbsentSettings(): Promise<void> {
        await using directory = await mkdtempDisposable(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        const root = directory.path;
        expect(await loadSettingsScopePatterns({
          cwd: join(
            root,
            'repo',
          ),
          home: join(
            root,
            'home',
          ),
        },),)
          .toEqual({},);
      },
    },),
    it({
      name: 'reads global settings from the HOME environment when no home option is given',
      fn: async function testHomeEnvironmentFallback(): Promise<void> {
        await using directory = await mkdtempDisposable(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        const root = directory.path;
        const home = join(
          root,
          'home',
        );
        using homeVariable = withEnvironmentVariable({
          name: 'HOME',
          value: home,
        },);
        await mkdir(
          join(
            home,
            '.pi',
            'agent',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            home,
            '.pi',
            'agent',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [GLOBAL_PATTERN,], },),
        );
        expect((await loadSettingsScopePatterns({
          cwd: join(
            root,
            'repo',
          ),
        },)).patterns,).toEqual([GLOBAL_PATTERN,],);
      },
    },),
    it({
      name: 'resolves the global settings path from the runtime home when HOME is unset',
      fn: async function testRuntimeHomeFallback(): Promise<void> {
        using homeVariable = withEnvironmentVariable({ name: 'HOME', },);
        const { globalPath, } = getSettingsPaths({ cwd: process.cwd(), },);
        expect(globalPath,).toBe(join(
          homedir(),
          '.pi',
          'agent',
          'settings.json',
        ),);
        expect(isAbsolute(globalPath,),).toBe(true,);
      },
    },),
    it({
      name: 'prefers project enabledModels over global enabledModels',
      fn: async function testProjectSettingsPrecedence(): Promise<void> {
        await using directory = await mkdtempDisposable(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        const root = directory.path;
        const home = join(
          root,
          'home',
        );
        const cwd = join(
          root,
          'repo',
        );
        await Promise.all([
          mkdir(
            join(
              home,
              '.pi',
              'agent',
            ),
            { recursive: true, },
          ),
          mkdir(
            join(
              cwd,
              '.pi',
            ),
            { recursive: true, },
          ),
        ],);
        await writeFile(
          join(
            home,
            '.pi',
            'agent',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [GLOBAL_PATTERN,], },),
        );
        await writeFile(
          join(
            cwd,
            '.pi',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [
            '',
            PROJECT_PATTERN,
          ], },),
        );
        expect((await loadSettingsScopePatterns({ cwd, home, },)).patterns,).toEqual([
          PROJECT_PATTERN,
        ],);
      },
    },),
    it({
      name: 'throws for invalid enabledModels',
      fn: async function testInvalidSettings(): Promise<void> {
        await using directory = await mkdtempDisposable(join(
          scratchRoot(),
          'pi-shared-settings-test-',
        ),);
        const root = directory.path;
        const cwd = join(
          root,
          'repo',
        );
        await mkdir(
          join(
            cwd,
            '.pi',
          ),
          { recursive: true, },
        );
        await writeFile(
          join(
            cwd,
            '.pi',
            'settings.json',
          ),
          JSON.stringify({ enabledModels: [1,], },),
        );
        const error = await captureAsyncError(function loadInvalidSettings() {
          return loadSettingsScopePatterns({
            cwd,
            home: join(
              root,
              'home',
            ),
          },);
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain('invalid project settings',);
      },
    },),
  ],
},);
