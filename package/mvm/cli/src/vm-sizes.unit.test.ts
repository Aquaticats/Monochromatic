/**
 Unit tests for reading a VM's size back out of the metadata mvm wrote for it.

 The metadata here is written by `writeVmMeta`, the producer this reader reads,
 so these cases cover mvm's own record rather than a captured file.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readVmSizes,
  writeVmMeta,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Guest a metadata record describes; the reader under test never consults it.
 */
const GUEST = {
  defaultUser: 'ubuntu',
  initSystem: 'systemd',
  osFamily: 'linux',
  shell: '/bin/bash',
} as const;

/**
 Virtual CPUs the record under test carries.
 */
const TWO_VCPUS = 2;

/**
 Memory the record under test carries, in gibibytes.
 */
const FOUR_GIB = 4;

/**
 Makes a throwaway directory for one VM's metadata.
 
 @returns the directory's absolute path
 
 @example
 ```ts
 const dir = await makeVmDir();
 ```
 */
async function makeVmDir(): Promise<string> {
  return mkdtemp(join(
    scratchRoot(),
    'vm-sizes-',
  ),);
}

//endregion Fixtures

await describe({
  name: '',
  concurrency: 1,
  children: [
    //region A record that carries the numbers

    describe({
      name: readVmSizes.name,
      children: [
        it({
          name: 'reads back the two numbers the metadata was written with',
          fn: async () => {
            const vmDir = await makeVmDir();
            try {
              await writeVmMeta({
                guest: GUEST,
                image: 'ubuntu',
                memoryGiB: FOUR_GIB,
                vmDir,
                vcpus: TWO_VCPUS,
              },);
              expect(await readVmSizes({ vmDir, },),).toEqual({
                memoryGiB: FOUR_GIB,
                vcpus: TWO_VCPUS,
              },);
            }
            finally {
              await rm(
                vmDir,
                {
                  force: true,
                  recursive: true,
                },
              );
            }
          },
        },),
        it({
          name: 'reports only the one number a record carries',
          fn: async () => {
            const vmDir = await makeVmDir();
            try {
              await writeVmMeta({
                guest: GUEST,
                image: 'ubuntu',
                vmDir,
                vcpus: TWO_VCPUS,
              },);
              expect(await readVmSizes({ vmDir, },),).toEqual({ vcpus: TWO_VCPUS, },);
            }
            finally {
              await rm(
                vmDir,
                {
                  force: true,
                  recursive: true,
                },
              );
            }
          },
        },),

        //endregion A record that carries the numbers

        //region A record that cannot tell

        it({
          name: 'reports nothing for a VM made before the numbers were recorded',
          fn: async () => {
            const vmDir = await makeVmDir();
            try {
              await mkdir(
                vmDir,
                { recursive: true, },
              );
              await writeVmMeta({
                guest: GUEST,
                image: 'ubuntu',
                vmDir,
              },);
              expect(await readVmSizes({ vmDir, },),).toEqual({},);
            }
            finally {
              await rm(
                vmDir,
                {
                  force: true,
                  recursive: true,
                },
              );
            }
          },
        },),
        it({
          name: 'reports nothing for a directory with no metadata at all',
          fn: async () => {
            const vmDir = await makeVmDir();
            try {
              expect(await readVmSizes({ vmDir, },),).toEqual({},);
            }
            finally {
              await rm(
                vmDir,
                {
                  force: true,
                  recursive: true,
                },
              );
            }
          },
        },),
        it({
          name: 'reports nothing for a metadata record whose size fields are not whole positive numbers',
          fn: async () => {
            const vmDir = await makeVmDir();
            try {
              await writeFile(
                join(
                  vmDir,
                  'meta.json',
                ),
                JSON.stringify({
                  createdAt: '2026-11-28T00:00:00.000Z',
                  defaultUser: 'ubuntu',
                  image: 'ubuntu',
                  memoryGiB: 'four',
                  osFamily: 'linux',
                  shell: '/bin/bash',
                  vcpus: 2.5,
                },),
              );
              expect(await readVmSizes({ vmDir, },),).toEqual({},);
            }
            finally {
              await rm(
                vmDir,
                {
                  force: true,
                  recursive: true,
                },
              );
            }
          },
        },),
      ],
    },),

    //endregion A record that cannot tell
  ],
},);
