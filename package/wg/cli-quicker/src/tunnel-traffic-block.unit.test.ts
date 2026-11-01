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
  removeTrafficBlock,
  trafficBlockExists,
  trafficBlockScript,
  trafficBlockState,
} from '../dist/final/node/tunnel-traffic-block.mjs';

/**
 Runner of commands that may fail,
 as the removal and the state check take it.
 */
type AllowFailureRunner = NonNullable<Parameters<typeof removeTrafficBlock>[0]['runCommand']>;

/**
 Runner of commands that must succeed,
 as the install takes it.
 */
type StrictRunner = NonNullable<Parameters<typeof installTrafficBlock>[0]['runCommand']>;

/**
 What one scripted `nft` call answers.
 */
type Answer = {
  /**
   Exit code of the call.
   */
  readonly exitCode: number;

  /**
   Standard error of the call.
   */
  readonly stderr: string;
};

/**
 Builds a runner that answers from a script and records the calls it received.

 @param answers - Answers in call order.

 @returns The runner and the recorded invocations,
 each as the arguments after `nft`.

 @example
 ```ts
 scripted({ answers: [{ exitCode: 1, stderr: 'Error: No such file or directory' }] });
 ```
 */
function scripted({ answers, }: { readonly answers: readonly Answer[]; },): { readonly runner: AllowFailureRunner; readonly calls: string[][]; } {
  /**
   Invocations in order.
   */
  const calls: string[][] = [];
  /**
   Position in the script.
   */
  const cursor = { next: 0, };
  /**
   The runner under test.

   @param call - The invocation.

   @returns The scripted answer.
   */
  function runner(call: { readonly command: string; readonly args: readonly string[]; },): ReturnType<AllowFailureRunner> {
    calls.push([...call.args,],);
    /**
     Answer for this call.
     */
    const answer = answers[cursor.next] ?? { exitCode: 0, stderr: '', };
    cursor.next += 1;
    return Promise.resolve({ stdout: '', ...answer, },);
  }
  return { runner, calls, };
}

/**
 Answer of `nft list table` for a table that exists.
 */
const LISTED: Answer = { exitCode: 0, stderr: '', };

/**
 Answer of `nft` for a table that does not exist.
 */
const NO_SUCH_TABLE: Answer = { exitCode: 1, stderr: 'Error: No such file or directory\ndelete table inet x\n', };

/**
 Answer of `nft` when the caller may not read or change the rules.
 */
const NOT_PERMITTED: Answer = { exitCode: 1, stderr: 'Error: Operation not permitted\n', };

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

await describe({
  name: trafficBlockState.name,
  children: [
    it({
      name: 'tells a table that is there, one that is not, and one that cannot be read apart, and only a table known to be there counts as existing',
      fn: async () => {
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [LISTED,], },).runner, },),).toBe('present',);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NO_SUCH_TABLE,], },).runner, },),).toBe('absent',);
        expect(await trafficBlockState({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NOT_PERMITTED,], },).runner, },),).toBe('unknown',);
        expect(await trafficBlockExists({ interfaceName: 'wgtest', runCommand: scripted({ answers: [LISTED,], },).runner, },),).toBe(true,);
        expect(await trafficBlockExists({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NO_SUCH_TABLE,], },).runner, },),).toBe(false,);
        expect(await trafficBlockExists({ interfaceName: 'wgtest', runCommand: scripted({ answers: [NOT_PERMITTED,], },).runner, },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: removeTrafficBlock.name,
  children: [
    it({
      name: 'resolves when the table was never there or the delete worked and the table is gone',
      fn: async () => {
        for (const answers of [[NO_SUCH_TABLE, NO_SUCH_TABLE,], [LISTED, NO_SUCH_TABLE,],]) {
          const { runner, calls, } = scripted({ answers, },);
          // oxlint-disable-next-line eslint/no-await-in-loop -- Each case reads its own recording.
          await removeTrafficBlock({ interfaceName: 'wgtest', runCommand: runner, },);
          expect(calls[0],).toEqual(['delete', 'table', 'inet', 'wg-quicker-wgtest-blocked',],);
          expect(calls[1],).toEqual(['list', 'table', 'inet', 'wg-quicker-wgtest-blocked',],);
        }
      },
    },),
    it({
      name: 'throws a typed error naming the table and the command that removes it when the delete fails and the table is still there',
      fn: async () => {
        /**
         What the removal threw.
         */
        const thrown = await removeTrafficBlock({
          interfaceName: 'wgtest',
          runCommand: scripted({ answers: [NOT_PERMITTED, LISTED,], },).runner,
        },).then(
          (): undefined => undefined,
          (error: unknown) => error,
        );
        expect((thrown as Error).name,).toBe('TrafficBlockError',);
        /**
         Its message.
         */
        const message = String(thrown,);
        expect(message,).toContain('the traffic block (nft table inet wg-quicker-wgtest-blocked) is still in place',);
        expect(message,).toContain('nft delete table exited 1: Error: Operation not permitted',);
        expect(message,).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
      },
    },),
    it({
      name: 'throws a typed error, and does not claim success, when it cannot tell whether the table is gone',
      fn: async () => {
        /**
         What the removal threw.
         */
        const thrown = await removeTrafficBlock({
          interfaceName: 'wgtest',
          runCommand: scripted({ answers: [NOT_PERMITTED, NOT_PERMITTED,], },).runner,
        },).then(
          (): undefined => undefined,
          (error: unknown) => error,
        );
        expect((thrown as Error).name,).toBe('TrafficBlockError',);
        expect(String(thrown,),).toContain('could not tell whether the traffic block (nft table inet wg-quicker-wgtest-blocked) was removed',);
        expect(String(thrown,),).toContain('sudo nft list table inet wg-quicker-wgtest-blocked',);
        expect(String(thrown,),).toContain('sudo nft delete table inet wg-quicker-wgtest-blocked',);
      },
    },),
  ],
},);
