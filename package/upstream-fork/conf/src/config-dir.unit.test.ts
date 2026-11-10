/**
 Per-user config directory resolution tests run under injected disposable
 homes,
 asserting the freedesktop.org and Windows layouts.
 The Windows layout is platform-gated,
 so those tests override `process.platform` for the length of their own
 synchronous block and put the host descriptor back at its end.
 Those tests pin which root the resolution reads,
 not the separator or drive spelling of the joined path,
 which stays the host's because `node:path` is the host's module.
 The macOS layout is left to the host it runs on.
 
 Every override is a `using` scope declared after the home,
 so it ends before the home does and a failed assertion restores it too:
 a leaked `process.platform` would make the next test read the previous
 test's platform.
 
 @module
 */

import {
  basename,
  join,
} from 'node:path';
import process from 'node:process';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { configDirectory, } from '../dist/final/neutral/index.mjs';

import {
  createDisposableHome,
  restoreHome,
} from './test-support.ts';

//region Environment and platform overrides

/**
 Creates the injected disposable home and hands back the scope that puts the
 environment back when it ends.
 
 @returns Scope carrying the fixture home;
 disposing it restores the environment the fixture replaced.
 
 @example
 ```ts
 using home = disposableHome();
 home.home; // '/tmp/conf-fork-fixtures/<uuid>'
 ```
 */
function disposableHome(): Disposable & { readonly home: string } {
  /**
   Fixture home this scope owns the restoration of.
   */
  const handle = createDisposableHome();
  return {
    home: handle.home,
    [Symbol.dispose]() {
      restoreHome(handle,);
    },
  };
}

/**
 Points one environment variable at a new value.
 
 @param name - Environment variable name to set.
 
 @param value - Value to set it to.
 
 @returns Scope putting back the prior presence or absence when it ends;
 it must be declared after {@link disposableHome},
 because the home fixture overwrites the variables it replaced.
 
 @example
 ```ts
 using _appData = setEnvironmentVariable({ name: 'APPDATA', value: '', });
 ```
 */
function setEnvironmentVariable({
  name,
  value,
}: {
  readonly name: string;
  readonly value: string;
},): Disposable {
  /**
   Value this variable held before the override,
   or `undefined` when it was absent.
   */
  const previous = process.env[name];
  process.env[name] = value;
  return {
    [Symbol.dispose]() {
      if (previous === undefined)
        Reflect.deleteProperty(
          process.env,
          name,
        );
      else
        process.env[name] = previous;
    },
  };
}

/**
 Overrides `process.platform`,
 so a layout gated on another platform can be exercised on this host.
 
 @param platform - Platform identifier the resolution must see.
 
 @returns Scope putting the host's own descriptor back when it ends.
 
 @example
 ```ts
 using _windows = setProcessPlatform('win32',);
 ```
 */
function setProcessPlatform(platform: string,): Disposable {
  /**
   Host descriptor for `process.platform`,
   restored verbatim because it carries the host's own value.
   */
  const original = Object.getOwnPropertyDescriptor(
    process,
    'platform',
  );
  Object.defineProperty(
    process,
    'platform',
    {
      configurable: true,
      value: platform,
    },
  );
  return {
    [Symbol.dispose]() {
      if (original !== undefined)
        Object.defineProperty(
          process,
          'platform',
          original,
        );
    },
  };
}

//endregion Environment and platform overrides

//region Resolution under test

/**
 Platform this file's process started on,
 which every override must leave in place.
 */
const HOST_PLATFORM = process.platform;

/**
 Resolves the config directory for the app name and suffix every case below
 shares.
 
 @returns Config directory the resolution reports.
 
 @example
 ```ts
 resolvedFooNodejs(); // '/home/me/.config/foo-nodejs'
 ```
 */
function resolvedFooNodejs(): string {
  return configDirectory({
    projectName: 'foo',
    projectSuffix: 'nodejs',
  },);
}

//endregion Resolution under test

await describe({
  name: configDirectory.name,
  children: [
    it({
      name: 'resolves the config directory under XDG_CONFIG_HOME',
      fn: async () => {
        using home = disposableHome();
        /**
         Custom XDG root the resolution must honor.
         */
        const customXdg = join(home.home, 'custom-xdg',);
        process.env.XDG_CONFIG_HOME = customXdg;

        expect(resolvedFooNodejs(),).toBe(join(customXdg, 'foo-nodejs',),);
      },
    },),

    it({
      name: 'falls back to $HOME/.config when XDG_CONFIG_HOME is unset',
      fn: async () => {
        using home = disposableHome();
        Reflect.deleteProperty(process.env, 'XDG_CONFIG_HOME',);

        expect(resolvedFooNodejs(),).toBe(join(home.home, '.config', 'foo-nodejs',),);
      },
    },),

    it({
      name: 'falls back to $HOME/.config when XDG_CONFIG_HOME is empty',
      fn: async () => {
        using home = disposableHome();
        using _emptyXdg = setEnvironmentVariable({
          name: 'XDG_CONFIG_HOME',
          value: '',
        },);

        expect(resolvedFooNodejs(),).toBe(join(home.home, '.config', 'foo-nodejs',),);
      },
    },),

    it({
      name: 'joins the project suffix as a dash-suffixed name',
      fn: async () => {
        using home = disposableHome();

        expect(resolvedFooNodejs(),).toBe(join(home.home, '.config', 'foo-nodejs',),);
      },
    },),

    it({
      name: 'appends nothing for an empty project suffix',
      fn: async () => {
        using home = disposableHome();
        /**
         Resolved directory for the bare project name.
         */
        const resolved = configDirectory({
          projectName: 'foo',
          projectSuffix: '',
        },);

        expect(basename(resolved,),).toBe('foo',);
        expect(resolved,).toBe(join(home.home, '.config', 'foo',),);
      },
    },),

    it({
      name: 'reads %APPDATA% for the Windows layout',
      fn: async () => {
        using home = disposableHome();
        /**
         Roaming root this test supplies,
         which the resolution must prefer over the home fallback.
         */
        const roaming = join(home.home, 'roaming',);
        using _windows = setProcessPlatform('win32',);
        using _roaming = setEnvironmentVariable({
          name: 'APPDATA',
          value: roaming,
        },);

        expect(resolvedFooNodejs(),).toBe(join(roaming, 'foo-nodejs', 'Config',),);
      },
    },),

    it({
      name: 'falls back to $HOME/AppData/Roaming when APPDATA is empty',
      fn: async () => {
        using home = disposableHome();
        using _windows = setProcessPlatform('win32',);
        using _emptyAppData = setEnvironmentVariable({
          name: 'APPDATA',
          value: '',
        },);

        expect(resolvedFooNodejs(),).toBe(
          join(
            home.home,
            'AppData',
            'Roaming',
            'foo-nodejs',
            'Config',
          ),
        );
      },
    },),

    it({
      name: 'leaves the host platform in place for later tests',
      fn: async () => {
        expect(process.platform,).toBe(HOST_PLATFORM,);
      },
    },),
  ],
},);
