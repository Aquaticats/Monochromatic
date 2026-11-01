import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  shutdownTargetMeansStopping,
  systemIsStopping,
} from '../dist/final/node/system-stop.mjs';

await describe({
  name: shutdownTargetMeansStopping.name,
  children: [
    it({
      name: 'means stopping only for an active or activating shutdown target',
      fn: async () => {
        expect(shutdownTargetMeansStopping({ printed: 'active\n', },),).toBe(true,);
        expect(shutdownTargetMeansStopping({ printed: 'activating\n', },),).toBe(true,);
        expect(shutdownTargetMeansStopping({ printed: 'inactive\n', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'deactivating\n', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'failed\n', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: '', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'inactive active', },),).toBe(false,);
        expect(shutdownTargetMeansStopping({ printed: 'reactivating', },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: systemIsStopping.name,
  children: [
    it({
      name: 'says stopping when the no-login file exists, without asking systemctl',
      fn: async () => {
        /**
         Scratch directory for the no-login file.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-nologin-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         The no-login file of the test.
         */
        const nologinPath = join(directory, 'nologin',);
        await writeFile(nologinPath, 'System is going down.\n',);
        /**
         Whether systemctl was asked.
         */
        const asked = { count: 0, };
        expect(await systemIsStopping({
          nologinPath,
          readState: () => {
            asked.count += 1;
            return Promise.resolve('inactive\n',);
          },
        },),).toBe(true,);
        expect(asked.count,).toBe(0,);
      },
    },),
    it({
      name: 'follows the shutdown target when there is no no-login file, and judges a probe that cannot tell as a system that keeps running',
      fn: async () => {
        /**
         A path that does not exist.
         */
        const nologinPath = '/nonexistent-directory/nologin';
        expect(await systemIsStopping({ nologinPath, readState: () => Promise.resolve('active\n',), },),).toBe(true,);
        expect(await systemIsStopping({ nologinPath, readState: () => Promise.resolve('activating\n',), },),).toBe(true,);
        expect(await systemIsStopping({ nologinPath, readState: () => Promise.resolve('inactive\n',), },),).toBe(false,);
        expect(await systemIsStopping({ nologinPath, readState: () => Promise.reject(new Error('systemctl exited 1',),), },),).toBe(false,);
      },
    },),
  ],
},);
