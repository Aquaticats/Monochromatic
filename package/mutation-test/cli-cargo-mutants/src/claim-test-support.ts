/**
 Helpers the claim tests share: disposable directories under the agent scratch area, real child processes that hold a claim
 until they are killed, and planted claim files as a dead process would have left them.
 */
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  mkdir,
  mkdtempDisposable,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import {
  claimPath,
  encodeClaimRecord,
} from './claim-record.ts';

/**
 Longest a holder child lives when a test forgets to kill it, in milliseconds.
 */
const holderLifetimeMs = 60_000;

/**
 Make a disposable directory under the agent scratch area, never the system temporary directory.

 @param name - what the directory is for, the start of its name

 @returns the directory, removed when disposed

 @example
 ```ts
 await using directory = await scratchDirectory('claims-');
 ```
 */
export async function scratchDirectory(name: string): Promise<AsyncDisposable & { readonly path: string }> {
  /**
   Parent of every scratch directory.
   */
  const parent = join(
    homedir(),
    'temp',
    'agent',
  );
  await mkdir(
    parent,
    { recursive: true },
  );
  return mkdtempDisposable(join(
    parent,
    name,
  ));
}

/**
 Read the first process id the kernel never hands out, which therefore names no process.

 @returns that number

 @example
 ```ts
 await unusedProcessId();
 ```
 */
export async function unusedProcessId(): Promise<number> {
  return Number((await readFile(
    '/proc/sys/kernel/pid_max',
    'utf8',
  )).trim());
}

/**
 Write a claim file as another process would have left it.

 @param directory - the directory of the claim files

 @param prefix - the claim series' name

 @param generation - the number to write at

 @param pid - the owner's process id

 @param birth - the owner's birth text

 @param namespace - the owner's PID namespace, absent to record none

 @returns the path written

 @example
 ```ts
 await plantClaim({ directory: '/d', prefix: 'item.owner', generation: 1, pid: 99, birth: 'linux:1@b' });
 ```
 */
export async function plantClaim({
  directory,
  prefix,
  generation,
  pid,
  birth,
  namespace,
}: {
  readonly directory: string;
  readonly prefix: string;
  readonly generation: number;
  readonly pid: number;
  readonly birth: string;
  readonly namespace?: string;
}): Promise<string> {
  /**
   Where the file goes.
   */
  const path = claimPath({
    directory,
    prefix,
    generation,
  });
  await writeFile(
    path,
    encodeClaimRecord({
      kind: 'owned',
      pid,
      birth,
      ...(namespace === undefined ? {} : { namespace }),
    }),
  );
  return path;
}

/**
 A child process that runs a script and is killed and awaited when disposed.
 */
export type Child = AsyncDisposable & {
  /**
   The child's process ID.
   */
  readonly pid: number;
  /**
   The first line the child printed.
   */
  readonly line: string;
  /**
   Kill the child with `SIGKILL` and wait until it has ended and been reaped.
   */
  readonly kill: () => Promise<void>;
};

/**
 Run a script in a child Node process and read its first line.

 @param script - module text the child runs

 @returns the child; declare it with `await using` so it is killed at the end

 @example
 ```ts
 await using child = await runChild('console.log("up"); setTimeout(() => {}, 60000);');
 ```
 */
export async function runChild(script: string): Promise<Child> {
  /**
   The child.
   */
  const child = spawn(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      script,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'inherit',
      ],
    },
  );
  if ((child.stdout === null) || (child.pid === undefined))
    throw new Error('The child has no standard output or process ID.');
  /**
   Chunks of the first write.
   */
  const chunks: readonly unknown[] = await once(
    child.stdout,
    'data',
  );
  /**
   Kill the child and wait for it.
   */
  async function kill(): Promise<void> {
    if ((child.exitCode !== null) || (child.signalCode !== null))
      return;
    child.kill('SIGKILL');
    await once(
      child,
      'close',
    );
  }
  return {
    pid: child.pid,
    line: String(chunks[0])
      .trim(),
    kill,
    async [Symbol.asyncDispose](): Promise<void> {
      await kill();
    },
  };
}

/**
 Run a child that takes a claim and keeps it until it is killed.

 @param claimModule - path of the module that exports `takeClaim`

 @param directory - the claim files' directory

 @param prefix - the claim series' name

 @returns the child, whose first line is the kind of outcome (`held` or `busy`)

 @example
 ```ts
 await using holder = await runClaimHolder({ claimModule: '/m/owner-claim.ts', directory: '/d', prefix: 'item.owner' });
 ```
 */
export async function runClaimHolder({
  claimModule,
  directory,
  prefix,
}: {
  readonly claimModule: string;
  readonly directory: string;
  readonly prefix: string;
}): Promise<Child> {
  return await runChild(`import { takeClaim } from ${JSON.stringify(claimModule)};
const outcome = await takeClaim({ directory: ${JSON.stringify(directory)}, prefix: ${JSON.stringify(prefix)} });
console.log(outcome.kind);
setTimeout(() => process.exit(0), ${String(holderLifetimeMs)});`);
}

/**
 Longest an old-style holder lives when a test forgets to kill it, in seconds.
 */
const holderSeconds = 60;

/**
 An old-style holder: the `flock` command of the earlier release, which takes the lock on a path and runs a command in it.
 */
export type OldHolder = AsyncDisposable & {
  /**
   Kill the holder and wait until it has ended.
   */
  readonly kill: () => Promise<void>;
};

/**
 Start an old-style holder as the earlier release did (`flock --nonblock --exclusive <path> <command>`), and wait until it
 holds the lock.

 @param path - the lock file or directory

 @returns the holder; declare it with `await using` so it is killed at the end

 @example
 ```ts
 await using holder = await startOldHolder('/l/slot-1.lock');
 ```
 */
export async function startOldHolder(path: string): Promise<OldHolder> {
  /**
   The holder: `flock` runs a shell that reports the lock and then becomes a sleeping process, so one process holds it.
   */
  const child = spawn(
    'flock',
    [
      '--nonblock',
      '--exclusive',
      '--conflict-exit-code',
      '200',
      path,
      'sh',
      '-c',
      `echo held; exec sleep ${String(holderSeconds)}`,
    ],
    {
      // A group of its own: `flock` runs the command as a child and the child holds the lock, so both must be killed.
      detached: true,
      stdio: [
        'ignore',
        'pipe',
        'inherit',
      ],
    },
  );
  /**
   The holder's process ID, which is also its group.
   */
  const { pid } = child;
  if ((child.stdout === null) || (pid === undefined))
    throw new Error('The holder has no standard output or process ID.');
  /**
   The group to kill, known now.
   */
  const group: number = pid;
  await once(
    child.stdout,
    'data',
  );
  /**
   Kill the holder and wait for it.
   */
  async function kill(): Promise<void> {
    if ((child.exitCode !== null) || (child.signalCode !== null))
      return;
    process.kill(
      -group,
      'SIGKILL',
    );
    await once(
      child,
      'close',
    );
  }
  return {
    kill,
    async [Symbol.asyncDispose](): Promise<void> {
      await kill();
    },
  };
}
