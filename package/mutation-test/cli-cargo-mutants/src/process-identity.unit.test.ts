import { spawn } from 'node:child_process';
import { once } from 'node:events';
import {
  readFile,
  readlink,
} from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import {
  currentProcessIdentity,
  namespaceUnknown,
  processGone,
  readBootId,
  readPidNamespace,
  readProcessBirth,
  startTicksFromStat,
} from './process-identity.ts';

/**
 Number of fields in a Linux `/proc/<pid>/stat` line the cases build; the start time is field twenty-two.
 */
const statFieldCount = 52;

/**
 Position, counted from one, of the start time in a `/proc/<pid>/stat` line.
 */
const startTimeField = 22;

/**
 Build a `/proc/<pid>/stat` text.

 @param command - command name inside the parentheses, which may hold spaces and parentheses

 @param state - process state letter

 @param startTicks - start time in clock ticks

 @returns the text, with every other field a filler number

 @example
 ```ts
 statText({ command: 'a b)', state: 'S', startTicks: '77' });
 ```
 */
function statText({
  command,
  state,
  startTicks,
}: {
  readonly command: string;
  readonly state: string;
  readonly startTicks: string;
}): string {
  /**
   Fields after the command name: state is field three, so the list starts there.
   */
  const rest = Array.from(
    { length: statFieldCount - 2 },
    function filler(
      _,
      index,
    ) {
      return index === 0 ? state : String(index);
    },
  );
  rest[startTimeField - 3] = startTicks;
  return `4321 (${command}) ${rest.join(' ')}\n`;
}

/**
 Start a child that does nothing until it is killed.

 @returns the child and its process ID

 @example
 ```ts
 const { child, pid } = startIdleChild();
 ```
 */
function startIdleChild() {
  /**
   The idle child.
   */
  const child = spawn(
    process.execPath,
    ['--eval', 'setTimeout(() => {}, 60000)'],
    { stdio: 'ignore' },
  );
  if (child.pid === undefined)
    throw new Error('The idle child has no process ID.');
  return {
    child,
    pid: child.pid,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: startTicksFromStat.name,
      children: [
        it({
          name: 'reads field twenty-two after the last closing parenthesis, whatever the command name holds',
          fn: async () => {
            const stat = statText({
              command: 'node a) b',
              state: 'S',
              startTicks: '275883698',
            });
            expect(startTicksFromStat(stat)).toBe('275883698');
          },
        }),
        ...['Z', 'X'].map(function exitedState(state) {
          return it({
            name: `reports no start time for a process in the exited state ${state}`,
            fn: async () => {
              const stat = statText({
                command: 'node',
                state,
                startTicks: '5',
              });
              expect(startTicksFromStat(stat)).toBe(processGone);
            },
          });
        }),
        ...[
          ['text without a closing parenthesis', '4321 node S 1\n'],
          ['a line that ends before the start time', '4321 (node) S 1 2 3\n'],
          ['an empty start time', statText({
            command: 'node',
            state: 'S',
            startTicks: '',
          })],
        ].map(function malformed([name, text]) {
          return it({
            name: `refuses ${String(name)}`,
            fn: async () => {
              expect(() => startTicksFromStat(String(text))).toThrow(CargoMutantsToolingError);
            },
          });
        }),
      ],
    }),
    describe({
      name: readProcessBirth.name,
      children: [
        it({
          name: 'names this process by its start time and the boot, the same on every read',
          fn: async () => {
            const first = await readProcessBirth({ pid: process.pid });
            const second = await readProcessBirth({ pid: process.pid });
            const bootId = await readBootId();
            expect(first).toBe(second);
            expect(String(first).startsWith('linux:')).toBe(true);
            expect(String(first).endsWith(`@${bootId}`)).toBe(true);
          },
        }),
        it({
          name: 'tells a child from this process, and reports the child gone once it was killed and reaped',
          fn: async () => {
            const {
              child,
              pid,
            } = startIdleChild();
            const own = await readProcessBirth({ pid: process.pid });
            const alive = await readProcessBirth({ pid });
            expect(String(alive).startsWith('linux:')).toBe(true);
            expect(alive).not.toBe(own);
            child.kill('SIGKILL');
            await once(
              child,
              'close',
            );
            expect(await readProcessBirth({ pid })).toBe(processGone);
          },
        }),
        it({
          name: 'reports a process id that names no process as gone',
          fn: async () => {
            /**
             The first number the kernel never hands out.
             */
            const unused = Number((await readFile(
              '/proc/sys/kernel/pid_max',
              'utf8',
            )).trim());
            expect(await readProcessBirth({ pid: unused })).toBe(processGone);
          },
        }),
        it({
          name: 'treats a read that fails for any reason but a missing process as an error, never as a dead process',
          fn: async () => {
            /**
             A read refused as a permission failure would be.
             */
            const refused = Object.assign(
              new Error('denied'),
              { code: 'EACCES' },
            );
            let caught: unknown;
            try {
              await readProcessBirth({
                pid: process.pid,
                readText: async () => {
                  throw refused;
                },
              });
            }
            catch (error) {
              caught = error;
            }
            expect(caught).toBeInstanceOf(CargoMutantsToolingError);
          },
        }),
        it({
          name: 'treats a process that exited during the read (ESRCH) as gone',
          fn: async () => {
            const exited = Object.assign(
              new Error('gone'),
              { code: 'ESRCH' },
            );
            expect(await readProcessBirth({
              pid: process.pid,
              readText: async () => {
                throw exited;
              },
            })).toBe(processGone);
          },
        }),
      ],
    }),
    describe({
      name: readPidNamespace.name,
      children: [
        it({
          name: 'reads the namespace link of this process and of a child, which share it',
          fn: async () => {
            const {
              child,
              pid,
            } = startIdleChild();
            const own = await readPidNamespace({ pid: process.pid });
            const ofChild = await readPidNamespace({ pid });
            child.kill('SIGKILL');
            await once(
              child,
              'close',
            );
            expect(own).toBe(await readlink('/proc/self/ns/pid'));
            expect(ofChild).toBe(own);
          },
        }),
        it({
          name: 'reports no namespace for a process that is gone',
          fn: async () => {
            const {
              child,
              pid,
            } = startIdleChild();
            child.kill('SIGKILL');
            await once(
              child,
              'close',
            );
            expect(await readPidNamespace({ pid })).toBe(namespaceUnknown);
          },
        }),
      ],
    }),
    describe({
      name: currentProcessIdentity.name,
      children: [
        it({
          name: 'gathers the process id, birth and namespace of this process',
          fn: async () => {
            const identity = await currentProcessIdentity();
            expect(identity).toEqual({
              pid: process.pid,
              birth: await readProcessBirth({ pid: process.pid }),
              namespace: await readlink('/proc/self/ns/pid'),
            });
          },
        }),
      ],
    }),
  ],
});
