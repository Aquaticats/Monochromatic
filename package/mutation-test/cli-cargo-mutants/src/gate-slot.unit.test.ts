import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CargoMutantsToolingError } from './error.ts';
import type { Environment } from './options.ts';
import {
  type GateSlotOptions,
  parseGateSlotArguments,
  slotClaimPrefix,
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
              noticeIntervalMs: 10_000,
              command: ['mise', 'run'],
            });
          },
        }),
        it({
          name: 'reads the options before the command',
          fn: async () => {
            expect(parse({ argv: ['--slot-count', '2', '--lock-dir', '/l', '--poll-interval-ms', '9', '--notice-interval-ms', '5', 'c', 'a'] })).toEqual({
              slotCount: 2,
              lockDirectory: '/l',
              pollIntervalMs: 9,
              noticeIntervalMs: 5,
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
          ['a non-numeric notice interval', ['--notice-interval-ms', 'soon', 'c']],
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
      name: slotClaimPrefix.name,
      children: [
        it({
          name: 'names the claim series of a slot, apart from the lock file the earlier release used',
          fn: async () => {
            expect(slotClaimPrefix(3)).toBe('slot-3.owner');
          },
        }),
      ],
    }),
  ],
});
