import { spawn, spawnSync } from 'node:child_process';
import { once } from 'node:events';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { createFakePodman } from './fake-podman-fixture.ts';
import { runShellCommand } from './run-fixture-program.ts';

/**
 Longest a test waits for a process to appear or to be gone.
 */
const waitMilliseconds = 10_000;

/**
 Make a sleep duration that no other process on the host has, so that `ps` finds exactly the processes of one test.

 @returns the duration as text, in seconds

 @example
 ```ts
 uniqueSleep();
 ```
 */
function uniqueSleep(): string {
  return `3000.${String(Date.now())}${String(process.pid)}`;
}

/**
 Find the processes that sleep for a duration.

 @param duration - the unique duration

 @returns their process IDs

 @example
 ```ts
 sleepersOf('3000.1');
 ```
 */
function sleepersOf(duration: string): readonly number[] {
  return spawnSync('pgrep', ['--full', `^sleep ${duration}$`], { encoding: 'utf8' }).stdout.split('\n')
    .filter((line) => line !== '')
    .map(Number);
}

/**
 Wait until the number of sleepers is as wanted.

 @param duration - the unique duration

 @param wanted - how many should exist

 @returns the number found when the wait ended

 @example
 ```ts
 await sleepersReach('3000.1', 0);
 ```
 */
async function sleepersReach(duration: string, wanted: number): Promise<number> {
  const deadline = Date.now() + waitMilliseconds;
  while (sleepersOf(duration).length !== wanted && Date.now() < deadline)
    await sleep(50);
  return sleepersOf(duration).length;
}

/**
 Make sure no sleeper of a duration outlives the test, whatever the test found.

 @param duration - the unique duration

 @returns the guard; declare it with `await using`

 @example
 ```ts
 await using guard = sleeperGuard('3000.1');
 ```
 */
function sleeperGuard(duration: string): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      for (const pid of sleepersOf(duration))
        process.kill(pid, 'SIGKILL');
    },
  };
}

/**
 Script of a process that starts a program in a fixture and waits for it.

 @param command - the shell command the program runs

 @returns the module text

 @example
 ```ts
 ownerScript('sleep 3000.1');
 ```
 */
function ownerScript(command: string): string {
  return `import { createFakePodman } from ${JSON.stringify(new URL('fake-podman-fixture.ts', import.meta.url).href)};
import { runShellCommand } from ${JSON.stringify(new URL('run-fixture-program.ts', import.meta.url).href)};
const fake = await createFakePodman();
console.log('ready');
await runShellCommand({ fake, command: ${JSON.stringify(command)}, workingDirectory: fake.workingDirectory, extraEnvironment: {} });
`;
}

await describe({
  name: 'programs run in a fixture',
  children: [
    it({
      name: 'end with the process that started them, even when it is killed without warning, and so do the programs they started',
      fn: async () => {
        const duration = uniqueSleep();
        await using guard = sleeperGuard(duration);
        expect(guard).toBeDefined();
        const owner = spawn(
          process.execPath,
          [
            '--input-type=module',
            '--eval',
            ownerScript(`sleep ${duration} & sleep ${duration}; wait`),
          ],
          { stdio: ['ignore', 'pipe', 'inherit'] },
        );
        await once(owner.stdout, 'data');
        expect(await sleepersReach(duration, 2)).toBe(2);
        owner.kill('SIGKILL');
        await once(owner, 'close');
        expect(await sleepersReach(duration, 0)).toBe(0);
      },
    }),
    it({
      name: 'fail at once, naming the process limit, when a program could not start a process or a thread',
      fn: async () => {
        await using fake = await createFakePodman();
        const signatures = [
          'Error: spawn podman EAGAIN',
          '#  Assertion failed: (0) == (uv_thread_create(t.get(), start_thread, this))',
          'pthread_create: Resource temporarily unavailable',
        ];
        for (const signature of signatures) {
          await expect(runShellCommand({
            fake,
            command: `echo ${JSON.stringify(signature)} >&2; exit 1`,
            workingDirectory: fake.workingDirectory,
            extraEnvironment: {},
          })).rejects.toThrow('process limit');
        }
      },
    }),
  ],
});
