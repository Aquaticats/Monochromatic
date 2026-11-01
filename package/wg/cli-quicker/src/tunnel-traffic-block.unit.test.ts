import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  installTrafficBlock,
  trafficBlockScript,
} from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Runner of commands that must succeed,
 as the install takes it.
 */
type StrictRunner = NonNullable<Parameters<typeof installTrafficBlock>[0]['runCommand']>;

/**
 Makes a scratch directory in the test scratch root,
 removed when the scope exits,
 where the real one may belong to another user.

 @returns A disposable directory.

 @example
 ```ts
 await using dir = await scratchDirectory();
 ```
 */
async function scratchDirectory(): ReturnType<NonNullable<Parameters<typeof installTrafficBlock>[0]['makeDirectory']>> {
  /**
   Path of the new directory.
   */
  const path = await mkdtemp(join(scratchRoot(), 'wg-quicker-block-',),);
  return {
    path,
    [Symbol.asyncDispose]: async (): Promise<void> => {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: installTrafficBlock.name,
  children: [
    it({
      name: 'installs and replaces the block in one nft call that creates, empties and fills the table, with no delete',
      fn: async () => {
        /**
         Invocations and the script each one read.
         */
        const seen: { readonly args: readonly string[]; readonly script: string; }[] = [];
        /**
         Runner that reads the script file while the call runs,
         because the file is removed when the install ends.

         @param call - The invocation.

         @returns An empty result.
         */
        const runner: StrictRunner = async (call,) => {
          seen.push({ args: [...call.args,], script: await readFile(call.args[1] ?? '', 'utf8',), },);
          return { stdout: '', stderr: '', };
        };
        await installTrafficBlock({ interfaceName: 'wgtest', runCommand: runner, makeDirectory: scratchDirectory, },);
        expect(seen.length,).toBe(1,);
        expect(seen[0]?.args[0],).toBe('-f',);
        /**
         Lines of the script.
         */
        const lines = (seen[0]?.script ?? '').split('\n',);
        expect(lines[0],).toBe('add table inet wg-quicker-wgtest-blocked',);
        expect(lines[1],).toBe('flush table inet wg-quicker-wgtest-blocked',);
        expect(lines.some((line,) => line.startsWith('delete',) || line.includes('delete table',),),).toBe(false,);
        expect(seen[0]?.script,).toBe(trafficBlockScript({ interfaceName: 'wgtest', },),);
      },
    },),
  ],
},);
