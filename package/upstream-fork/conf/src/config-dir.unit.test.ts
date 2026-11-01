/**
 Per-user config directory resolution tests run under injected disposable
 homes,
 asserting the freedesktop.org layout only (macOS and Windows layouts are
 platform-gated and not exercised here).
 
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
  ],
},);
