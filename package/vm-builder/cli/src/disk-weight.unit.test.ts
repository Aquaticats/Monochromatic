import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  diskWeightArguments,
  LOWEST_DISK_WEIGHT,
} from '../dist/final/node/disk-weight.mjs';

/**
 Reads this package's `mise.toml`, whose `image:check` commands are shell strings and so cannot import the constant.

 @returns File text.

 @example
 ```ts
 const text = await miseToml();
 ```
 */
async function miseToml(): Promise<string> {
  return await readFile(
    join(
      import.meta.dirname,
      '..',
      'mise.toml',
    ),
    'utf8',
  );
}

await describe({
  name: diskWeightArguments.name,
  children: [
    it({
      name: 'sets both block-layer weight files from the one named weight',
      fn: async () => {
        expect(diskWeightArguments(),).toEqual([
          `--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,
          `--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,
        ],);
      },
    },),
    it({
      name: 'matches the lowest_disk_weight variable of mise.toml',
      fn: async () => {
        /**
         Text before the value in the variable's line.
         */
        const prefix = 'lowest_disk_weight = "';
        /**
         The variable's line as written in the file.
         */
        const line = (await miseToml())
          .split('\n',)
          .find((candidate,) => candidate.startsWith(prefix,),);
        expect(line,).toBe(`${prefix}${String(LOWEST_DISK_WEIGHT,)}"`,);
      },
    },),
    it({
      name: 'gives every podman run of mise.toml both options from that variable',
      fn: async () => {
        /**
         Lines of mise.toml that start a container.
         */
        const runs = (await miseToml())
          .split('\n',)
          .filter((line,) => line.includes('podman run',),);
        expect(runs.length,).toBeGreaterThan(0,);
        for (const line of runs) {
          expect(line,).toContain('--blkio-weight {{vars.lowest_disk_weight}}',);
          expect(line,).toContain('--cgroup-conf io.weight={{vars.lowest_disk_weight}}',);
        }
      },
    },),
  ],
},);
