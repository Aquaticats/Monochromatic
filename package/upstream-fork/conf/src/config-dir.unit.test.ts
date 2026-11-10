/**
 Per-user config directory resolution tests run under injected disposable
 homes,
 asserting the freedesktop.org and Windows layouts.
 The Windows layout is platform-gated,
 so those tests override `process.platform` for the length of their own
 synchronous block and put the host descriptor back at its end.
 The macOS layout is left to the host it runs on.
 
 @module
 */

import path from 'node:path';
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
 Points one environment variable at a new value.
 
 @param name - Environment variable name to set.
 
 @param value - Value to set it to.
 
 @returns Restoration putting back the prior presence or absence;
 call it from the same synchronous block as the set,
 after {@link restoreHome},
 so the next test sees this file's own environment.
 
 @example
 ```ts
 const restore = setEnvironmentVariable({ name: 'APPDATA', value: '', });
 restore();
 ```
 */
function setEnvironmentVariable({
  name,
  value,
}: {
  readonly name: string;
  readonly value: string;
},): () => void {
  /**
   Value this variable held before the override,
   or `undefined` when it was absent.
   */
  const previous = process.env[name];
  process.env[name] = value;
  return function restoreEnvironmentVariable(): void {
    if (previous === undefined)
      Reflect.deleteProperty(
        process.env,
        name,
      );
    else
      process.env[name] = previous;
  };
}

/**
 Overrides `process.platform`,
 so a layout gated on another platform can be exercised on this host.
 
 @param platform - Platform identifier the resolution must see.
 
 @returns Restoration putting the host's own descriptor back.
 
 @example
 ```ts
 const restore = setProcessPlatform('win32',);
 restore();
 ```
 */
function setProcessPlatform(platform: string,): () => void {
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
  return function restoreProcessPlatform(): void {
    if (original !== undefined)
      Object.defineProperty(
        process,
        'platform',
        original,
      );
  };
}

//endregion Environment and platform overrides

await describe({
  name: configDirectory.name,
  children: [
    it({
      name: 'resolves the config directory under XDG_CONFIG_HOME',
      fn: async () => {
        /**
         Injected home whose environment the resolution reads.
         */
        const handle = createDisposableHome();
        /**
         Custom XDG root the resolution must honor.
         */
        const customXdg = path.join(handle.home, 'custom-xdg',);
        process.env.XDG_CONFIG_HOME = customXdg;

        expect(configDirectory({
          projectName: 'foo',
          projectSuffix: 'nodejs',
        },),).toBe(path.join(customXdg, 'foo-nodejs',),);

        restoreHome(handle,);
      },
    },),

    it({
      name: 'falls back to $HOME/.config when XDG_CONFIG_HOME is unset',
      fn: async () => {
        /**
         Injected home the fallback resolves beneath.
         */
        const handle = createDisposableHome();
        Reflect.deleteProperty(process.env, 'XDG_CONFIG_HOME',);

        expect(configDirectory({
          projectName: 'foo',
          projectSuffix: 'nodejs',
        },),).toBe(path.join(handle.home, '.config', 'foo-nodejs',),);

        restoreHome(handle,);
      },
    },),

    it({
      name: 'falls back to $HOME/.config when XDG_CONFIG_HOME is empty',
      fn: async () => {
        /**
         Injected home the fallback resolves beneath.
         */
        const handle = createDisposableHome();
        /**
         Puts back the XDG root this test blanks.
         */
        const restoreXdg = setEnvironmentVariable({
          name: 'XDG_CONFIG_HOME',
          value: '',
        },);

        expect(configDirectory({
          projectName: 'foo',
          projectSuffix: 'nodejs',
        },),).toBe(path.join(handle.home, '.config', 'foo-nodejs',),);

        restoreXdg();
        restoreHome(handle,);
      },
    },),

    it({
      name: 'joins the project suffix as a dash-suffixed name',
      fn: async () => {
        /**
         Injected home giving the resolution a known config root.
         */
        const handle = createDisposableHome();

        expect(configDirectory({
          projectName: 'foo',
          projectSuffix: 'nodejs',
        },),).toBe(path.join(handle.home, '.config', 'foo-nodejs',),);

        restoreHome(handle,);
      },
    },),

    it({
      name: 'appends nothing for an empty project suffix',
      fn: async () => {
        /**
         Injected home giving the resolution a known config root.
         */
        const handle = createDisposableHome();
        /**
         Resolved directory for the bare project name.
         */
        const resolved = configDirectory({
          projectName: 'foo',
          projectSuffix: '',
        },);

        expect(path.basename(resolved,),).toBe('foo',);
        expect(resolved,).toBe(path.join(handle.home, '.config', 'foo',),);

        restoreHome(handle,);
      },
    },),

    it({
      name: 'reads %APPDATA% for the Windows layout',
      fn: async () => {
        /**
         Injected home the roaming fallback would resolve beneath;
         the test asserts it is not used.
         */
        const handle = createDisposableHome();
        /**
         Roaming root this test supplies,
         which the resolution must prefer over the home fallback.
         */
        const roaming = path.join(handle.home, 'roaming',);
        /**
         Puts the host platform back.
         */
        const restorePlatform = setProcessPlatform('win32',);
        /**
         Puts back the APPDATA this test supplies.
         */
        const restoreAppData = setEnvironmentVariable({
          name: 'APPDATA',
          value: roaming,
        },);

        expect(configDirectory({
          projectName: 'foo',
          projectSuffix: 'nodejs',
        },),).toBe(path.join(roaming, 'foo-nodejs', 'Config',),);

        restoreAppData();
        restorePlatform();
        restoreHome(handle,);
      },
    },),

    it({
      name: 'falls back to $HOME/AppData/Roaming when APPDATA is empty',
      fn: async () => {
        /**
         Injected home the roaming fallback resolves beneath.
         */
        const handle = createDisposableHome();
        /**
         Puts the host platform back.
         */
        const restorePlatform = setProcessPlatform('win32',);
        /**
         Puts back the APPDATA this test blanks.
         */
        const restoreAppData = setEnvironmentVariable({
          name: 'APPDATA',
          value: '',
        },);

        expect(configDirectory({
          projectName: 'foo',
          projectSuffix: 'nodejs',
        },),).toBe(
          path.join(
            handle.home,
            'AppData',
            'Roaming',
            'foo-nodejs',
            'Config',
          ),
        );

        restoreAppData();
        restorePlatform();
        restoreHome(handle,);
      },
    },),
  ],
},);
