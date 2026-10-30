import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import type { Environment } from './options.ts';
import {
  buildFlockArguments,
  type GateSlotOptions,
  parseGateSlotArguments,
  runInSlot,
  slotLockPath,
} from './gate-slot.ts';

/**
 Parse with a fixed home.

 @param argv - arguments after the script path
 @param environment - process environment
 @returns the parsed settings
 */
function parse({
  argv,
  environment = {},
}: {
  readonly argv: readonly string[];
  readonly environment?: Environment;
}): GateSlotOptions {
  return parseGateSlotArguments({ argv, environment, home: '/h' });
}

/**
 Settings for the run cases: three slots and a one-word command.
 */
const runOptions: GateSlotOptions = {
  slotCount: 3,
  lockDirectory: '/locks',
  pollIntervalMs: 7,
  command: ['cmd'],
};

await describe({
  name: '',
  children: [
    describe({
      name: parseGateSlotArguments.name,
      children: [
        it({
          name: 'defaults to six slots in the shared lock directory',
          fn: async () => {
            expect(parse({ argv: ['mise', 'run'] })).toEqual({
              slotCount: 6,
              lockDirectory: '/h/temp/agent/gate-slots',
              pollIntervalMs: 15_000,
              command: ['mise', 'run'],
            });
          },
        }),
        it({
          name: 'reads the options before the command',
          fn: async () => {
            expect(parse({ argv: ['--slot-count', '2', '--lock-dir', '/l', '--poll-interval-ms', '9', 'c', 'a'] })).toEqual({
              slotCount: 2,
              lockDirectory: '/l',
              pollIntervalMs: 9,
              command: ['c', 'a'],
            });
          },
        }),
        it({
          name: 'leaves flags after the command to the command',
          fn: async () => {
            expect(parse({ argv: ['mise', 'run', '--slot-count', '2'] }).command).toEqual(['mise', 'run', '--slot-count', '2']);
          },
        }),
        it({
          name: 'ends the options at a double hyphen so the command may start with a flag',
          fn: async () => {
            expect(parse({ argv: ['--slot-count', '2', '--', '--weird', 'x'] }).command).toEqual(['--weird', 'x']);
          },
        }),
        it({
          name: 'falls back to the environment',
          fn: async () => {
            const parsed = parse({ argv: ['c'], environment: { GATE_SLOT_COUNT: '4', GATE_SLOT_DIRECTORY: '/env' } });
            expect(parsed.slotCount).toBe(4);
            expect(parsed.lockDirectory).toBe('/env');
          },
        }),
        it({
          name: 'prefers the flag over the environment',
          fn: async () => {
            expect(parse({ argv: ['--slot-count', '2', 'c'], environment: { GATE_SLOT_COUNT: '4' } }).slotCount).toBe(2);
          },
        }),
        ...[
          ['no command', []],
          ['only options', ['--slot-count', '2']],
          ['an unknown option', ['--bogus', 'x', 'c']],
          ['an option without a value', ['--slot-count']],
          ['a zero slot count', ['--slot-count', '0', 'c']],
          ['a non-numeric poll interval', ['--poll-interval-ms', 'soon', 'c']],
        ].map(function rejection([name, argv]) {
          return it({
            name: `rejects ${String(name)}`,
            fn: async () => {
              expect(() => parse({ argv: argv as readonly string[] })).toThrow(CargoMutantsToolingError);
            },
          });
        }),
      ],
    }),
    describe({
      name: slotLockPath.name,
      children: [
        it({
          name: 'names the lock file of a slot',
          fn: async () => {
            expect(slotLockPath({ lockDirectory: '/locks', slot: 3 })).toBe('/locks/slot-3.lock');
          },
        }),
      ],
    }),
    describe({
      name: buildFlockArguments.name,
      children: [
        it({
          name: 'tries the slot without waiting and runs the command niced inside it',
          fn: async () => {
            expect(buildFlockArguments({ lockDirectory: '/locks', slot: 2, command: ['mise', 'run', 'x y'] })).toEqual([
              '--nonblock',
              '--exclusive',
              '--conflict-exit-code',
              '200',
              '/locks/slot-2.lock',
              'nice',
              '--adjustment=19',
              'mise',
              'run',
              'x y',
            ]);
          },
        }),
      ],
    }),
    describe({
      name: runInSlot.name,
      children: [
        it({
          name: 'runs in the first free slot and returns its status',
          fn: async () => {
            const slots: string[] = [];
            const status = await runInSlot({
              options: runOptions,
              runFlock: async (flockArguments) => {
                slots.push(flockArguments[4] ?? '');
                return 5;
              },
              sleep: async () => {},
            });
            expect(status).toBe(5);
            expect(slots).toEqual(['/locks/slot-1.lock']);
          },
        }),
        it({
          name: 'moves to the next slot while one is held',
          fn: async () => {
            const slots: string[] = [];
            const status = await runInSlot({
              options: runOptions,
              runFlock: async (flockArguments) => {
                slots.push(flockArguments[4] ?? '');
                return slots.length < 3 ? 200 : 0;
              },
              sleep: async () => {},
            });
            expect(status).toBe(0);
            expect(slots).toEqual(['/locks/slot-1.lock', '/locks/slot-2.lock', '/locks/slot-3.lock']);
          },
        }),
        it({
          name: 'sleeps the poll interval and retries every slot when all are held',
          fn: async () => {
            const sleeps: number[] = [];
            let calls = 0;
            const status = await runInSlot({
              options: runOptions,
              runFlock: async () => {
                calls += 1;
                return calls <= 3 ? 200 : 0;
              },
              sleep: async (milliseconds) => {
                sleeps.push(milliseconds);
              },
            });
            expect(status).toBe(0);
            expect(sleeps).toEqual([7]);
            expect(calls).toBe(4);
          },
        }),
        it({
          name: 'returns a nonzero command status unchanged',
          fn: async () => {
            const status = await runInSlot({
              options: runOptions,
              runFlock: async () => 3,
              sleep: async () => {},
            });
            expect(status).toBe(3);
          },
        }),
      ],
    }),
  ],
});
