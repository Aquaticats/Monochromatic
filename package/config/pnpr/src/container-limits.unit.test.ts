import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import { diskWeightArguments, } from '@monochromatic-dev/container-limit-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Reads one of this package's task files, whose `podman` commands are shell strings and so cannot import the constant.

 @param name - file name beside `package.json`

 @returns File text.

 @example
 ```ts
 const text = await packageFile('mise.toml');
 ```
 */
async function packageFile(name: string,): Promise<string> {
  return await readFile(
    join(
      import.meta.dirname,
      '..',
      name,
    ),
    'utf8',
  );
}

/**
 The task files that both start the local registry container and build its image.
 */
const taskFiles = [
  'mise.toml',
  'task.ts',
] as const;

await describe({
  name: 'the local pnpr container commands',
  children: taskFiles.map((name,) =>
    describe({
      name,
      children: [
        it({
          name: 'start the registry container at the lowest disk weight with CPU, memory and process limits',
          fn: async () => {
            /**
             The `podman run` command of the file.
             */
            const run = (await packageFile(name,))
              .split('\n',)
              .find((line,) => line.includes('podman run',),);
            expect(run,).toBeDefined();
            for (const option of diskWeightArguments()) {
              expect(run,).toContain(option,);
            }
            expect(run,).toContain('--memory=2g',);
            expect(run,).toContain('--cpus=2',);
            expect(run,).toContain('--pids-limit=256',);
          },
        },),
        it({
          name: 'build the image with the memory and the CFS period and quota podman build accepts',
          fn: async () => {
            /**
             The `podman build` command of the file.
             */
            const build = (await packageFile(name,))
              .split('\n',)
              .find((line,) => line.includes('podman build',),);
            expect(build,).toBeDefined();
            expect(build,).toContain('--memory=2g',);
            expect(build,).toContain('--cpu-period=100000',);
            expect(build,).toContain('--cpu-quota=200000',);
            expect(build,).not.toContain('--blkio-weight',);
          },
        },),
      ],
    },),),
},);
