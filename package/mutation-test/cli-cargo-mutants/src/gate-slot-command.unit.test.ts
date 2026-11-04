import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runNicedCommand } from './gate-slot-command.ts';

/**
 The signal number of `SIGTERM`, which a shell reports as 128 plus this.
 */
const terminationSignal = 15;

/**
 Exit status a shell reports for a command ended by a signal, as `flock` did for the command it ran.
 */
const signalStatusBase = 128;

/**
 Run a command through the real runner in a child Node process and read its first line.

 @param command - the command to run

 @returns the child, its first line, and a promise of its exit code

 @example
 ```ts
 const run = await runInChild(['sh', '-c', 'echo up; exec sleep 30']);
 ```
 */
async function runInChild(command: readonly string[]) {
  /**
   The child that runs the runner.
   */
  const child = spawn(
    process.execPath,
    [
      '--input-type=module',
      '--eval',
      `import { runNicedCommand } from ${JSON.stringify(join(
        import.meta.dirname,
        'gate-slot-command.ts',
      ))};
process.exitCode = await runNicedCommand({ command: ${JSON.stringify(command)} });`,
    ],
    {
      stdio: [
        'ignore',
        'pipe',
        'inherit',
      ],
    },
  );
  if (child.stdout === null)
    throw new Error('The child has no standard output.');
  /**
   Chunks of the first write.
   */
  const first: readonly unknown[] = await once(
    child.stdout,
    'data',
  );
  /**
   How the child ended: its exit code.
   */
  const ended: Promise<readonly unknown[]> = once(
    child,
    'close',
  );
  return {
    child,
    line: String(first[0])
      .trim(),
    ended,
  };
}

/**
 Make a value that kills a leftover process when disposed.

 @param pid - the process id

 @returns the value; declare it with `using`

 @example
 ```ts
 using reaper = killOnDispose(4242);
 ```
 */
function killOnDispose(pid: number): Disposable {
  return {
    [Symbol.dispose](): void {
      if (isAlive(pid))
        process.kill(
          pid,
          'SIGKILL',
        );
    },
  };
}

/**
 Tell whether a process with this id exists.

 @param pid - the process id

 @returns false when the kernel reports no such process

 @example
 ```ts
 isAlive(process.pid);
 ```
 */
function isAlive(pid: number): boolean {
  try {
    process.kill(
      pid,
      0,
    );
    return true;
  }
  catch (error) {
    return !(Error.isError(error) && ('code' in error) && (error.code === 'ESRCH'));
  }
}

await describe({
  name: runNicedCommand.name,
  children: [
    it({
      name: 'returns the exit status of the command',
      fn: async () => {
        expect(await runNicedCommand({
          command: ['sh', '-c', 'exit 7'],
        })).toBe(7);
        expect(await runNicedCommand({ command: ['true'] })).toBe(0);
      },
    }),
    it({
      name: 'returns 128 plus the signal number for a command a signal ended',
      fn: async () => {
        expect(await runNicedCommand({
          command: ['sh', '-c', 'kill -TERM $$'],
        })).toBe(signalStatusBase + terminationSignal);
      },
    }),
    it({
      name: 'returns 127 for a command that cannot be started',
      fn: async () => {
        expect(await runNicedCommand({
          command: ['/no/such/program'],
        })).toBe(127);
      },
    }),
    it({
      name: 'runs the command at the lowest priority',
      fn: async () => {
        const run = await runInChild(['sh', '-c', 'echo $(ps -o ni= -p $$)']);
        const [code] = await run.ended;
        expect(run.line).toBe('19');
        expect(code).toBe(0);
      },
    }),
    it({
      name: 'passes a termination signal on to the command and ends only after it, so the slot is not given back while the command runs',
      fn: async () => {
        const run = await runInChild(['sh', '-c', 'echo $$; exec sleep 30']);
        const commandPid = Number(run.line);
        run.child.kill('SIGTERM');
        const [code] = await run.ended;
        expect(code).toBe(signalStatusBase + terminationSignal);
        expect(isAlive(commandPid)).toBe(false);
      },
    }),
    it({
      name: 'passes a termination signal on to every process of the command and ends only after all of them have ended, so no worker runs without its slot',
      fn: async () => {
        const run = await runInChild(['sh', '-c', 'sleep 30 >/dev/null & echo $!; wait']);
        const workerPid = Number(run.line);
        using reaper = killOnDispose(workerPid);
        run.child.kill('SIGTERM');
        const [code] = await run.ended;
        expect(code).toBe(signalStatusBase + terminationSignal);
        expect(isAlive(workerPid)).toBe(false);
      },
    }),
    it({
      name: 'ends when the command ends on its own even if it left a background process running, because only a forwarded termination waits for the workers',
      fn: async () => {
        const run = await runInChild(['sh', '-c', 'sleep 30 >/dev/null & echo $!; exit 0']);
        const workerPid = Number(run.line);
        using reaper = killOnDispose(workerPid);
        const [code] = await run.ended;
        expect(code).toBe(0);
        expect(isAlive(workerPid)).toBe(true);
      },
    }),
  ],
});
