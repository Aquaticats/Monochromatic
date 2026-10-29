/**
 Tests for the command line: the app directory flag.
 
 @module
 */

import { resolve, } from 'node:path';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseAppDir, } from './cli.ts';

await describe({
  name: parseAppDir.name,
  children: [
    it({
      name: 'defaults to the current directory',
      fn: async ({ expect, }) => {
        expect(parseAppDir([],),).toBe(process.cwd(),);
      },
    },),
    it({
      name: 'resolves the --app-dir value to an absolute path',
      fn: async ({ expect, }) => {
        expect(parseAppDir(['--app-dir', 'package/music-player/desktop-app',],),).toBe(
          resolve('package/music-player/desktop-app',),
        );
        expect(parseAppDir(['--app-dir', '/abs/app',],),).toBe('/abs/app',);
      },
    },),
    it({
      name: 'rejects flags it does not know',
      fn: async ({ expect, }) => {
        expect(() => parseAppDir(['--unknown',],),).toThrow();
        expect(() => parseAppDir(['positional',],),).toThrow();
      },
    },),
  ],
},);
