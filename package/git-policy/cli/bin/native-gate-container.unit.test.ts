import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runGateContainer, } from './native-gate-container.ts';
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 Private scratch parent outside the repository, whose backing device is known.
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Milliseconds the container's client is given before its deadline stops it, which is long enough for a stand-in node
 program to start on a host running other agents' builds and far below the minute a stalling one would run.
 */
const standInDeadlineMilliseconds = 3_000;

/**
 A stand-in `podman` that records every argument array it is given and answers as the run this test needs.

 Its `run` is a client whose container never exits, which is the client the gate's deadline has to stop; every other
 subcommand answers as a program that found nothing to do.
 */
const standInPodman = `#!/usr/bin/env node
const { appendFileSync } = require('node:fs');
const args = process.argv.slice(2);
appendFileSync(process.env.PODMAN_LOG, JSON.stringify(args) + '\\n');
if (args[0] === 'run') {
  setTimeout(() => {}, 60000);
} else {
  process.exit(0);
}
`;

/**
 A stand-in `podman` whose `run` answers at once, so the removal is exercised on a container that ended by itself.
 */
const standInPodmanThatEnds = `#!/usr/bin/env node
const { appendFileSync } = require('node:fs');
const args = process.argv.slice(2);
appendFileSync(process.env.PODMAN_LOG, JSON.stringify(args) + '\\n');
process.exit(0);
`;

/**
 One scratch tree holding a stand-in `podman` and the file it records its arguments in.
 */
type ContainerTestTree = {
  /**
   Root of the tree, where a test writes the ID file it names.
   */
  readonly path: string;
  /**
   File every stand-in writes its own argument array to, one JSON array per line.
   */
  readonly argumentLog: string;
  /**
   Directory holding the stand-in, which comes first on the `PATH` the clients run with.
   */
  readonly standInDirectory: string;
};

/**
 Create a scratch tree holding a stand-in `podman` with the given text.

 @param delegate - the `podman` the clients run, written as the program that names this node executable

 @returns the tree and the paths the test reads it through
 */
async function containerTestTree({ delegate, }: {
  readonly delegate: string;
},): Promise<ContainerTestTree & { readonly [Symbol.asyncDispose]: () => Promise<void>; }> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  /**
   The tree, removed when the caller's scope ends.
   */
  const tree = await mkdtempDisposable(join(
    scratchParent,
    'gate-container-test-',
  ),);
  /**
   Directory the stand-in `podman` sits in, first on the clients' `PATH`.
   */
  const standInDirectory = join(
    tree.path,
    'bin',
  );
  await mkdir(standInDirectory,);
  /**
   File the stand-in records its arguments in.
   */
  const argumentLog = join(
    tree.path,
    'podman-arguments.jsonl',
  );
  /**
   The stand-in itself, which has to be executable to be found on `PATH`.
   */
  const standInPath = join(
    standInDirectory,
    'podman',
  );
  await writeFile(
    standInPath,
    delegate,
  );
  await chmod(
    standInPath,
    0o755,
  );
  return {
    path: tree.path,
    argumentLog,
    standInDirectory,
    async [Symbol.asyncDispose](): Promise<void> {
      await tree[Symbol.asyncDispose]();
    },
  };
}

/**
 The argument arrays the stand-in `podman` was given, in the order it was given them.

 @param argumentLog - file the stand-in appended each one to

 @returns one array per call, empty when it was never called
 */
async function recordedCalls(argumentLog: string,): Promise<readonly (readonly string[])[]> {
  /**
   Every line the stand-in wrote.
   */
  const text = await readFile(
    argumentLog,
    'utf8',
  );
  return text.split('\n',)
    .filter(function hasText(line,) {
      return line !== '';
    },)
    .map(function parsed(line,) {
      return JSON.parse(line,) as readonly string[];
    },);
}

/**
 The value of one option in an argument array.

 @param args - the argument array

 @param name - the option whose value to read

 @returns the value, or empty text when the array does not carry that option
 */
function optionValue({ args, name, }: {
  readonly args: readonly string[];
  readonly name: string;
},): string {
  /**
   Where the option sits in the array.
   */
  const at = args.indexOf(name,);
  return at === (-1)
    ? ''
    : args[at + 1] ?? '';
}

await describe({
  name: runGateContainer.name,
  children: [
    it({
      name: 'runs the container under an ID file and removes that container when the run ends',
      fn: async () => {
        await using tree = await containerTestTree({ delegate: standInPodmanThatEnds, },);
        /**
         What the client answered.
         */
        const outcome = await runGateContainer({
          cidFile: join(
            tree.path,
            'ended.cid',
          ),
          args: ['image'],
          capture: true,
          deadlineMilliseconds: standInDeadlineMilliseconds,
          environment: {
            ...process.env,
            PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
            PODMAN_LOG: tree.argumentLog,
          },
        },);
        expect(outcome.status,).toBe(0,);
        /**
         Every call the stand-in was given.
         */
        const calls = await recordedCalls(tree.argumentLog,);
        expect(calls.map(function first(args,) {
          return args[0];
        },),).toEqual([
          'run',
          'rm',
        ],);
        expect(optionValue({
          args: calls[0] ?? [],
          name: '--cidfile',
        },),).toContain('ended.cid',);
        expect(optionValue({
          args: calls[1] ?? [],
          name: '--cidfile',
        },),).toContain('ended.cid',);
      },
      timeout: 20_000,
    },),
    it({
      name: 'removes the container of a client the deadline stopped, so a stopped check leaves nothing running',
      fn: async () => {
        await using tree = await containerTestTree({ delegate: standInPodman, },);
        /**
         The failure of a client whose container never exits.
         */
        let caught: unknown;
        try {
          await runGateContainer({
            cidFile: join(
              tree.path,
              'stopped.cid',
            ),
            args: ['image'],
            capture: true,
            deadlineMilliseconds: standInDeadlineMilliseconds,
            environment: {
              ...process.env,
              PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
              PODMAN_LOG: tree.argumentLog,
            },
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(NativeVerificationError,);
        expect((caught as Error).message,).toContain('was still running after 3000 ms',);
        /**
         The calls the stand-in recorded, the removal among them.
         */
        const calls = await recordedCalls(tree.argumentLog,);
        expect(calls.map(function first(args,) {
          return args[0];
        },),).toEqual([
          'run',
          'rm',
        ],);
        expect(optionValue({
          args: calls[1] ?? [],
          name: '--cidfile',
        },),).toContain('stopped.cid',);
      },
      timeout: 20_000,
    },),
  ],
},);
