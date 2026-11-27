import {
  chmod,
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { writeFileSync, } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runGateContainer, } from './native-gate-container.ts';
import type { ContainerMemorySample, ContainerMemoryWatch, } from './native-gate-container-memory.ts';
import {
  announceContainerId,
  containerIdWhenWritten,
} from './native-gate-container-id-file.ts';
import { gateContainerIdentityOf, } from './native-gate-container-identity.ts';
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
 A run id in the shape the evidence directory gives one, which every stand-in below is told.
 */
const runId = 'native-HZPg8O';

/**
 The suite's identity, which every run below gives its container.
 */
const suiteIdentity = gateContainerIdentityOf({
  runId,
  role: 'suite',
});

/**
 An ID in the shape podman writes to the ID file, so the test can tell which run it read.
 */
const containerId = '3f2a9c1e'.repeat(8,);

/**
 A memory sample of a container the kernel killed inside its bound, in the shape the watcher answers.

 @returns the sample
 */
function killedMemorySample(): ContainerMemorySample {
  return {
    cgroupPath: `/sys/fs/cgroup/libpod-${containerId}.scope`,
    readAt: '2026-11-27T13:00:00.000Z',
    currentBytes: 2_147_483_648,
    peakBytes: 2_147_483_648,
    maxBytes: 2_147_483_648,
    swapCurrentBytes: 0,
    swapMaxBytes: 0,
    pidsCurrent: 4,
    pidsPeak: 576,
    stat: { anon: 1_073_741_824, },
    events: {
      max: 1_024,
      oom: 1,
      oom_kill: 1,
    },
  };
}

/**
 A watch of a container that met no kill at its bound.

 @returns the watch, which reports a sample of a container that peaked inside its bound
 */
function survivedWatch(): ContainerMemoryWatch {
  return {
    lastRead: function lastRead() {
      return {
        ...killedMemorySample(),
        currentBytes: 629_145_600,
        peakBytes: 1_516_845_056,
        events: {
          max: 0,
          oom: 0,
          oom_kill: 0,
        },
      };
    },
    stop: async function stop(): Promise<void> {},
  };
}

/**
 The bound every container of the runs below is given, in the option form the caller's own limits take
 (`bin/native-gate-container-limits.ts`): memory, CPUs, processes and the shared disk weight.
 */
const gateLimits = ['--memory=2g', '--cpus=4', '--pids-limit=1024', '--blkio-weight=10',];

/**
 The client's arguments that name this run's container, which both stand-ins below write their ID file for.
 */
const identityWriting = `  const cidFile = args[args.indexOf('--cidfile') + 1];
  require('node:fs').writeFileSync(cidFile, process.env.PODMAN_CONTAINER_ID + '\\n');
`;

/**
 A stand-in `podman` that records every argument array it is given and answers as the run this test needs.

 Its `run` writes the container's ID to the ID file its client was given and then never exits, which is the client the
 gate's deadline has to stop; every other subcommand answers as a program that found nothing to do.
 */
const standInPodman = `#!/usr/bin/env node
const { appendFileSync } = require('node:fs');
const args = process.argv.slice(2);
appendFileSync(process.env.PODMAN_LOG, JSON.stringify(args) + '\\n');
if (args[0] === 'run') {
${identityWriting}  setTimeout(() => {}, 60000);
} else {
  process.exit(0);
}
`;

/**
 A stand-in `podman` whose `run` writes the container's ID and answers at once, so the removal is exercised on a
 container that ended by itself.
 */
const standInPodmanThatEnds = `#!/usr/bin/env node
const { appendFileSync } = require('node:fs');
const args = process.argv.slice(2);
appendFileSync(process.env.PODMAN_LOG, JSON.stringify(args) + '\\n');
if (args[0] === 'run') {
${identityWriting}}
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

/**
 A scratch directory holding the ID file one container's client writes, removed when the test's scope ends.
 */
async function identityTestTree(): Promise<{ readonly cidFile: string; readonly [Symbol.asyncDispose]: () => Promise<void>; }> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: 0o700,
    },
  );
  /**
   The directory, removed when the caller's scope ends.
   */
  const tree = await mkdtempDisposable(join(
    scratchParent,
    'gate-container-id-test-',
  ),);
  return {
    cidFile: join(
      tree.path,
      'suite.cid',
    ),
    async [Symbol.asyncDispose](): Promise<void> {
      await tree[Symbol.asyncDispose]();
    },
  };
}

/**
 Collect the lines that would go to stdout, putting the real reporter back when the scope ends.

 @param lines - collector the caller reads afterwards

 @returns the collector, and the restore that disposal runs

 @example
 ```ts
 using printed = collectingLines({ lines: [], },);
 ```
 */
function collectingLines(
  { lines, }: { readonly lines: string[]; },
): { readonly lines: readonly string[]; } & Disposable {
  /**
   The real reporter, put back when this scope ends.
   */
  const reported = console.log;
  console.log = (...parts: readonly unknown[]) => {
    lines.push(parts.map(String,)
      .join(' ',),);
  };
  return {
    lines,
    [Symbol.dispose]: () => {
      console.log = reported;
    },
  };
}

await describe({
  name: runGateContainer.name,
  children: [
    it({
      name: 'runs the container under an ID file and removes that container when the run ends',
      fn: async () => {
        await using tree = await containerTestTree({ delegate: standInPodmanThatEnds, },);
        /**
         Every ID the run reported as it read it, which the evidence records.
         */
        const started: string[] = [];
        /**
         What the client answered.
         */
        const outcome = await runGateContainer({
          cidFile: join(
            tree.path,
            'ended.cid',
          ),
          identity: suiteIdentity,
          limits: gateLimits,
          args: ['image'],
          capture: true,
          deadlineMilliseconds: standInDeadlineMilliseconds,
          environment: {
            ...process.env,
            PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
            PODMAN_LOG: tree.argumentLog,
            PODMAN_CONTAINER_ID: containerId,
          },
          onStarted: async function recorded(id) {
            started.push(id,);
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
        expect(started,).toEqual([containerId,],);
      },
      timeout: 20_000,
    },),
    it({
      name: 'gives the container the name and label of its run, so a person stops it by a name this run printed',
      fn: async () => {
        await using tree = await containerTestTree({ delegate: standInPodmanThatEnds, },);
        await runGateContainer({
          cidFile: join(
            tree.path,
            'named.cid',
          ),
          identity: suiteIdentity,
          limits: gateLimits,
          args: ['image'],
          capture: true,
          deadlineMilliseconds: standInDeadlineMilliseconds,
          environment: {
            ...process.env,
            PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
            PODMAN_LOG: tree.argumentLog,
            PODMAN_CONTAINER_ID: containerId,
          },
        },);
        /**
         The client's own call, which is where the options go.
         */
        const [run] = await recordedCalls(tree.argumentLog,);
        expect(optionValue({
          args: run ?? [],
          name: '--name',
        },),).toBe(suiteIdentity.name,);
        expect(optionValue({
          args: run ?? [],
          name: '--label',
        },),).toBe(suiteIdentity.label,);
      },
      timeout: 20_000,
    },),
    it({
      name: 'gives the container the limits its caller handed it, so no gate container starts unbounded',
      fn: async () => {
        await using tree = await containerTestTree({ delegate: standInPodmanThatEnds, },);
        await runGateContainer({
          cidFile: join(
            tree.path,
            'bounded.cid',
          ),
          identity: suiteIdentity,
          limits: gateLimits,
          args: ['image'],
          capture: true,
          deadlineMilliseconds: standInDeadlineMilliseconds,
          environment: {
            ...process.env,
            PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
            PODMAN_LOG: tree.argumentLog,
            PODMAN_CONTAINER_ID: containerId,
          },
        },);
        /**
         The client's own call, which is where the limit options go.
         */
        const [run] = await recordedCalls(tree.argumentLog,);
        expect((run ?? []).filter(function isALimit(argument,) {
          return gateLimits.includes(argument,);
        },),).toEqual(gateLimits,);
      },
      timeout: 20_000,
    },),
    it({
      name: 'removes the container of a client the deadline stopped, so a stopped check leaves nothing running',
      fn: async () => {
        await using tree = await containerTestTree({ delegate: standInPodman, },);
        /**
         Every ID the run reported before its deadline stopped the client.
         */
        const started: string[] = [];
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
            identity: suiteIdentity,
            limits: gateLimits,
            args: ['image'],
            capture: true,
            deadlineMilliseconds: standInDeadlineMilliseconds,
            environment: {
              ...process.env,
              PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
              PODMAN_LOG: tree.argumentLog,
              PODMAN_CONTAINER_ID: containerId,
            },
            onStarted: async function recorded(id) {
              started.push(id,);
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
        expect(started,).toEqual([containerId,],);
      },
      timeout: 20_000,
    },),
    describe({
      name: 'when the container was killed for memory',
      // Each check below diverts stdout for its own duration, so they run one after the other.
      concurrency: 1,
      children: [
        it({
          name: 'says so when the container it ran was killed at its memory bound, apart from any test failure beside it',
          fn: async () => {
            await using tree = await containerTestTree({ delegate: standInPodmanThatEnds, },);
            using printed = collectingLines({ lines: [], },);
            /**
             Every container the run started a memory watch for.
             */
            const watched: string[] = [];
            /**
             Whether the watch was stopped before the run answered.
             */
            let stopped = false;
            /**
             What the client answered.
             */
            const outcome = await runGateContainer({
              cidFile: join(
                tree.path,
                'killed.cid',
              ),
              identity: suiteIdentity,
              limits: gateLimits,
              args: ['image'],
              capture: true,
              deadlineMilliseconds: standInDeadlineMilliseconds,
              environment: {
                ...process.env,
                PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
                PODMAN_LOG: tree.argumentLog,
                PODMAN_CONTAINER_ID: containerId,
              },
              watchMemory: function watchedMemory({ containerId: id, },) {
                watched.push(id,);
                return {
                  lastRead: killedMemorySample,
                  stop: async function stop(): Promise<void> {
                    stopped = true;
                  },
                };
              },
            },);
            expect(outcome.status,).toBe(0,);
            expect(watched,).toEqual([containerId,],);
            expect(stopped,).toBe(true,);
            expect(printed.lines.join('\n',),).toContain('was killed for memory',);
          },
          timeout: 20_000,
        },),
        it({
          name: 'says nothing about memory when the container it ran met no kill',
          fn: async () => {
            await using tree = await containerTestTree({ delegate: standInPodmanThatEnds, },);
            using printed = collectingLines({ lines: [], },);
            await runGateContainer({
              cidFile: join(
                tree.path,
                'survived.cid',
              ),
              identity: suiteIdentity,
              limits: gateLimits,
              args: ['image'],
              capture: true,
              deadlineMilliseconds: standInDeadlineMilliseconds,
              environment: {
                ...process.env,
                PATH: `${tree.standInDirectory}:${process.env.PATH ?? ''}`,
                PODMAN_LOG: tree.argumentLog,
                PODMAN_CONTAINER_ID: containerId,
              },
              watchMemory: survivedWatch,
            },);
            expect(printed.lines.join('\n',),).not.toContain('killed for memory',);
          },
          timeout: 20_000,
        },),
      ],
    }),
  ],
},);

await describe({
  name: containerIdWhenWritten.name,
  children: [
    it({
      name: 'answers at once with an ID its client wrote before this wait looked',
      fn: async () => {
        await using tree = await identityTestTree();
        await writeFile(
          tree.cidFile,
          `${containerId}\n`,
        );
        /**
         A client that is still running, so only the file can end this wait.
         */
        const ended = Promise.withResolvers<unknown>();
        expect(await containerIdWhenWritten({
          cidFile: tree.cidFile,
          ended: ended.promise,
        },),).toEqual({
          kind: 'written',
          id: containerId,
        },);
      },
      timeout: 20_000,
    },),
    it({
      name: 'reads an ID its client writes while the gate waits, so the line naming it comes before the container speaks',
      fn: async () => {
        await using tree = await identityTestTree();
        /**
         The client, writing its ID while the gate is between two looks.
         */
        const ended = Promise.withResolvers<unknown>();
        expect(await containerIdWhenWritten({
          cidFile: tree.cidFile,
          ended: ended.promise,
          wait: async function writing(): Promise<undefined> {
            await writeFile(
              tree.cidFile,
              `${containerId}\n`,
            );
            return undefined;
          },
        },),).toEqual({
          kind: 'written',
          id: containerId,
        },);
      },
      timeout: 20_000,
    },),
    it({
      name: 'looks once more when the client ended, so an ID written just before it ended is still read',
      fn: async () => {
        await using tree = await identityTestTree();
        expect(await containerIdWhenWritten({
          cidFile: tree.cidFile,
          ended: Promise.resolve(),
          // Written where the client would write it: synchronously, before the race that sees the client end is decided.
          wait: async function writing(): Promise<undefined> {
            writeFileSync(
              tree.cidFile,
              `${containerId}\n`,
            );
            return undefined;
          },
        },),).toEqual({
          kind: 'written',
          id: containerId,
        },);
      },
      timeout: 20_000,
    },),
    it({
      name: 'answers that the client ended when no ID was ever written, which means it created no container',
      fn: async () => {
        await using tree = await identityTestTree();
        expect(await containerIdWhenWritten({
          cidFile: tree.cidFile,
          ended: Promise.resolve(),
          wait: async function waiting(): Promise<undefined> {
            return undefined;
          },
        },),).toEqual({ kind: 'client-ended', },);
      },
      timeout: 20_000,
    },),
    it({
      name: 'answers at its own bound when the client neither ends nor writes an ID',
      fn: async () => {
        await using tree = await identityTestTree();
        /**
         Milliseconds this wait is given, far below the two minutes a real client that stopped answering costs.
         */
        const boundMilliseconds = 20;
        /**
         A client that is still running, so only the bound can end this wait.
         */
        const ended = Promise.withResolvers<unknown>();
        expect(await containerIdWhenWritten({
          cidFile: tree.cidFile,
          ended: ended.promise,
          deadlineMilliseconds: boundMilliseconds,
          wait: async function waiting(): Promise<undefined> {
            return undefined;
          },
        },),).toEqual({ kind: 'deadline', },);
      },
      timeout: 20_000,
    },),
    it({
      name: 'refuses to name a container when its ID file is there and cannot be read',
      fn: async () => {
        await using tree = await identityTestTree();
        // A directory in the ID file's place is there and unreadable as a file, whatever the test runs as.
        await mkdir(tree.cidFile,);
        /**
         The wait that meets an ID file it cannot read.
         */
        const refused = containerIdWhenWritten({
          cidFile: tree.cidFile,
          ended: Promise.withResolvers<unknown>().promise,
        },);
        await expect(refused,).rejects.toThrow(NativeVerificationError,);
        await expect(refused,).rejects.toThrow('could not be read',);
      },
      timeout: 20_000,
    },),
  ],
},);

await describe({
  name: announceContainerId.name,
  // Each check below diverts stdout for its own duration, so they run one after the other.
  concurrency: 1,
  children: [
    it({
      name: 'prints the ID its client wrote and hands it to the run as soon as it is read',
      fn: async () => {
        await using tree = await identityTestTree();
        await writeFile(
          tree.cidFile,
          `${containerId}\n`,
        );
        /**
         Every line this announce printed.
         */
        const lines: string[] = [];
        using printed = collectingLines({ lines, },);
        /**
         Every ID the run recorded.
         */
        const started: string[] = [];
        await announceContainerId({
          identity: suiteIdentity,
          cidFile: tree.cidFile,
          running: Promise.withResolvers<unknown>().promise,
          onStarted: async function recorded(id) {
            started.push(id,);
          },
        },);
        expect(started,).toEqual([containerId,],);
        expect(printed.lines.join('\n',),).toContain(`Gate container suite: ID ${containerId}`,);
      },
      timeout: 20_000,
    },),
    it({
      name: 'prints that a client which ended before writing an ID created no container',
      fn: async () => {
        await using tree = await identityTestTree();
        /**
         Every line this announce printed.
         */
        const lines: string[] = [];
        using printed = collectingLines({ lines, },);
        /**
         Every ID the run recorded, which this ended client never gives it one of.
         */
        const started: string[] = [];
        await announceContainerId({
          identity: suiteIdentity,
          cidFile: tree.cidFile,
          running: Promise.resolve(),
          onStarted: async function recorded(id) {
            started.push(id,);
          },
        },);
        expect(started,).toEqual([],);
        expect(printed.lines.join('\n',),).toContain('created no container',);
        expect(printed.lines.join('\n',),).toContain(tree.cidFile,);
      },
      timeout: 20_000,
    },),
    it({
      name: 'prints the name and label left to stop a container by when no ID arrived in the time it waited',
      fn: async () => {
        await using tree = await identityTestTree();
        /**
         Every line this announce printed.
         */
        const lines: string[] = [];
        using printed = collectingLines({ lines, },);
        /**
         Milliseconds this announce waits, far below the two minutes a real client that stopped answering costs.
         */
        const waitedMilliseconds = 20;
        await announceContainerId({
          identity: suiteIdentity,
          cidFile: tree.cidFile,
          running: Promise.withResolvers<unknown>().promise,
          deadlineMilliseconds: waitedMilliseconds,
        },);
        expect(printed.lines.join('\n',),).toContain('no ID reached',);
        expect(printed.lines.join('\n',),).toContain(`name ${suiteIdentity.name}`,);
        expect(printed.lines.join('\n',),).toContain(`label ${suiteIdentity.label}`,);
        expect(printed.lines.join('\n',),).toContain(`${String(waitedMilliseconds)} ms`,);
      },
      timeout: 20_000,
    },),
  ],
},);
