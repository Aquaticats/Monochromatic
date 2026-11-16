/**
 What a start asks this machine's user manager, and what it decides: the very call a start makes, asked twice, bounded, and the
 folder its socket lives in, named in a unit only when the socket is really there.

 Why these are tested without a real manager: the subject is the decision, not the manager. A stand-in command runner answers in
 each way a manager can, so every state of the decision is exercised on any machine, and the one real unit these need is the socket
 file the folder rule reads.

 @module
 */

import { createServer, } from 'node:net';
import {
  mkdir,
  mkdtemp,
  rm,
  stat,
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
  type CommandOutput,
  MANAGER_ANSWER_MILLISECONDS,
  type RunCommand,
  managerAnswerOf,
  managerProblemIn,
  runtimeDirectoryTheManagerKeepsItsSocketIn,
} from '../dist/final/node/index.mjs';

/**
 The longest path a Unix socket address may name: the kernel's fixed 108-byte field less the NUL that ends the text.
 */
const LONGEST_SOCKET_PATH = 107;

/**
 The folders a manager fixture adds under its base, up to the socket itself.
 */
const SOCKET_SUFFIX = '/pi-manager-XXXXXX/runtime/systemd/private';

/**
 A folder a test may write in, removed when the test ends.
 */
type AThrowawayFolder = {
  /**
   The folder's path.
   */
  readonly path: string;

  /**
   Removes the folder and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a folder whose manager socket the fixture may bind, which must be short enough for the kernel.

 Why the length is measured here: a Unix socket address is a fixed field of 108 bytes and `listen` answers `EINVAL` past it, so a
 fixture under a long scratch root (a review's own folder, for example) would fail for its folder's length rather than for the code it
 tests. The user's runtime directory is short on any session that has a manager.

 @returns the folder, removed when the test ends

 @example
 ```ts
 await using base = await makeSocketBase();
 ```
 */
async function makeSocketBase(): Promise<AThrowawayFolder> {
  /**
   The harness's scratch root, which is where fixtures go when it is short enough.
   */
  const scratch = scratchRoot();
  /**
   The folder the fixture's own folder is made in.
   */
  const parent = ((scratch.length + SOCKET_SUFFIX.length) <= LONGEST_SOCKET_PATH)
    ? scratch
    : join('/run/user', String(process.getuid?.() ?? 1_000,),);
  /**
   The fixture's own folder.
   */
  const path = await mkdtemp(join(parent, 'pi-manager-',),);
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { recursive: true, force: true, maxRetries: 5, retryDelay: 200, },);
    },
  };
}

/**
 A stand-in command runner: answers each question from the script it is given, and records what was asked.

 @param answers - what each question answers, one per asking, the last repeated

 @returns the runner and the words it was asked

 @example
 ```ts
 const { run, asked } = standInManager([{ status: 1, stdout: '', stderr: 'no' }]);
 ```
 */
function standInManager(answers: readonly CommandOutput[],): {
  readonly run: RunCommand;
  readonly asked: readonly string[];
} {
  /**
   What was asked so far, which is also how many askings there have been.
   */
  const asked: string[] = [];
  return {
    asked,
    run: async function answered(request,) {
      asked.push(`${request.command} ${request.words.join(' ',)}`,);
      /**
       The answer this asking gets: each of the scripted ones in turn, and the last one for any asking after them.
       */
      const index = Math.min(asked.length - 1, answers.length - 1);
      return answers[index] ?? { status: 1, stdout: '', stderr: 'no answer was scripted', };
    },
  };
}

/**
 A socket listening at one path, closed when the test ends.

 @param path - where the socket listens, which must be short enough for the kernel's fixed address

 @returns the server, and the way to close it

 @example
 ```ts
 await using socket = await aListeningSocket({ path: '/run/user/1000/pi-manager-x/systemd/private', });
 ```
 */
async function aListeningSocket({ path }: { readonly path: string },): Promise<{
  readonly [Symbol.asyncDispose]: () => Promise<void>;
}> {
  const server = createServer(function accept() {},);
  await new Promise<void>(function waitForListening(resolve,) {
    server.listen(path, resolve,);
  },);
  return {
    async [Symbol.asyncDispose]() {
      await new Promise<void>(function waitForClosing(resolve,) {
        server.close(function closed() {
          resolve();
        },);
      },);
    },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: managerAnswerOf.name,
      children: [
        it({
          name: 'asks the manager for a unit of its own, which is the call a start makes, and names the run in that unit',
          fn: async () => {
            const { run, asked, } = standInManager([{ status: 0, stdout: '', stderr: '', },],);
            expect(await managerAnswerOf({
              run,
              folder: '/repo',
              runName: 'copy-route',
            },),).toEqual({ answered: true, answer: 'a unit of its own started', },);
            expect(asked.length,).toBe(1,);
            expect(asked[0],).toContain('systemd-run --user --wait --collect --quiet');
            expect(asked[0],).toContain('--unit=copy-route-manager-check');
            // Never a run's own unit name with the `pi-` prefix, which the slot count on this machine reads as a run.
            expect(asked[0],).not.toContain('--unit=pi-copy-route');
          },
        },),
        it({
          name: 'answers when the first asking fails and the second succeeds, which a manager that missed one answer does',
          fn: async () => {
            const { run, asked, } = standInManager([
              { status: 1, stdout: '', stderr: 'a manager that missed one answer', },
              { status: 0, stdout: '', stderr: '', },
            ],);
            expect((await managerAnswerOf({ run, folder: '/repo', runName: 'a', },)).answered,).toBe(true,);
            expect(asked.length,).toBe(2,);
          },
        },),
        it({
          name: 'refuses in words that name each asking when the manager starts no unit twice',
          fn: async () => {
            const { run, asked, } = standInManager([{ status: 1, stdout: '', stderr: 'no unit for you', },],);
            expect(await managerAnswerOf({ run, folder: '/repo', runName: 'a', },),).toEqual({
              answered: false,
              answer: 'asked 1: it refused: status 1 "no unit for you"; asked 2: it refused: status 1 "no unit for you"',
            },);
            expect(asked.length,).toBe(2,);
          },
        },),
        it({
          name: 'says a question its own bound ended was killed by that bound, rather than only that it failed',
          fn: async () => {
            const { run, } = standInManager([{ status: -1, stdout: '', stderr: '', killedByItsOwnBound: true, },],);
            const answer = await managerAnswerOf({ run, folder: '/repo', runName: 'a', },);
            expect(answer.answered,).toBe(false,);
            expect(answer.answer,).toContain(`did not answer inside its own ${String(MANAGER_ANSWER_MILLISECONDS,)} ms bound`,);
          },
        },),
      ],
    },),
    describe({
      name: managerProblemIn.name,
      children: [
        it({
          name: 'says nothing when the manager started a unit, and names the question and what was made when it did not',
          fn: async () => {
            const answering = standInManager([{ status: 0, stdout: '', stderr: '', },],);
            expect(await managerProblemIn({
              run: answering.run,
              folder: '/repo',
              runName: 'a',
            },),).toEqual({},);
            const refusing = standInManager([{ status: 1, stdout: '', stderr: 'no', },],);
            /**
             The words of the refusal.
             */
            const problem = await managerProblemIn({
              run: refusing.run,
              folder: '/repo',
              runName: 'a',
            },);
            expect(problem.problem,).toContain('no user manager started a unit of its own');
            expect(problem.problem,).toContain('Nothing was made and no run happened');
            expect(problem.problem,).toContain('start the run on a machine that has one',);
          },
        },),
      ],
    },),
    describe({
      name: runtimeDirectoryTheManagerKeepsItsSocketIn.name,
      children: [
        it({
          name: 'names the folder whose systemd/private is a socket, and so is where a scope inside a unit can reach the manager',
          fn: async () => {
            await using base = await makeSocketBase();
            /**
             The folder a unit would be given.
             */
            const runtimeDirectory = join(base.path, 'runtime',);
            await mkdir(join(runtimeDirectory, 'systemd',), { recursive: true, mode: 0o700, },);
            await using socket = await aListeningSocket({ path: join(runtimeDirectory, 'systemd', 'private',), },);
            expect(await runtimeDirectoryTheManagerKeepsItsSocketIn({ runtimeDirectory, },),).toEqual({ runtimeDirectory, },);
            // The path really is a socket, which is what the rule reads.
            expect((await stat(join(runtimeDirectory, 'systemd', 'private',),)).isSocket(),).toBe(true,);
            void socket;
          },
        },),
        it({
          name: 'names no folder when systemd/private is a regular file, when it is absent, and when the launcher had no folder',
          fn: async () => {
            await using base = await makeSocketBase();
            /**
             A folder whose `systemd/private` is a regular file, which no manager listens on.
             */
            const withRegularFile = join(base.path, 'regular',);
            await mkdir(join(withRegularFile, 'systemd',), { recursive: true, },);
            await writeFile(join(withRegularFile, 'systemd', 'private',), 'not a socket\n',);
            expect(await runtimeDirectoryTheManagerKeepsItsSocketIn({ runtimeDirectory: withRegularFile, },),).toEqual({},);
            /**
             A folder holding no manager's socket at all.
             */
            const withoutAnything = join(base.path, 'absent',);
            await mkdir(withoutAnything, { recursive: true, },);
            expect(await runtimeDirectoryTheManagerKeepsItsSocketIn({ runtimeDirectory: withoutAnything, },),).toEqual({},);
            expect(await runtimeDirectoryTheManagerKeepsItsSocketIn({},),).toEqual({},);
          },
        },),
      ],
    },),
  ],
},);
