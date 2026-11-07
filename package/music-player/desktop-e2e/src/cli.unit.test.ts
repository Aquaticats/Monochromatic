/**
 Tests for the command line: the app directory flag.
 
 @module
 */

import { resolve, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseCommandLine, } from './cli.ts';

await describe({
  name: parseCommandLine.name,
  children: [
    it({
      name: 'defaults to the current directory',
      fn: async ({ expect, }) => {
        expect(parseCommandLine([],).appDir,).toBe(process.cwd(),);
      },
    },),
    it({
      name: 'resolves the --app-dir value to an absolute path',
      fn: async ({ expect, }) => {
        expect(parseCommandLine(['--app-dir', 'package/music-player/desktop-app',],).appDir,).toBe(
          resolve('package/music-player/desktop-app',),
        );
        expect(parseCommandLine(['--app-dir', '/abs/app',],).appDir,).toBe('/abs/app',);
      },
    },),
    it({
      name: 'asks for a visible window only with --visible',
      fn: async ({ expect, }) => {
        expect(parseCommandLine([],).visible,).toBe(false,);
        expect(parseCommandLine(['--app-dir', '/abs/app',],).visible,).toBe(false,);
        expect(parseCommandLine(['--visible',],).visible,).toBe(true,);
        expect(parseCommandLine(['--visible', '--app-dir', '/abs/app',],).appDir,).toBe('/abs/app',);
      },
    },),
    it({
      name: 'rejects flags it does not know',
      fn: async ({ expect, }) => {
        expect(() => parseCommandLine(['--unknown',],),).toThrow();
        expect(() => parseCommandLine(['positional',],),).toThrow();
      },
    },),
  ],
},);
