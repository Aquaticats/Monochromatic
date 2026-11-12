import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  containerPidsLimitOption,
  cpuQuotaOption,
  deadlineMillisecondsOption,
  gateCapsFromOptions,
  gateCapsOptionNames,
  gateOptions,
  ioWeightOption,
  memoryMaxOption,
  numberOption,
  tasksMaxOption,
  textOption,
} from './native-gate-arguments.ts';
import { defaultGateCaps } from './native-gate-caps.ts';
import { NativeGateError } from './native-gate-error.ts';

/**
 The options a test command takes.

 @returns two flags and the caps options
 */
function knownOptions(): readonly string[] {
  return [
    '--repository',
    '--poll-milliseconds',
    deadlineOptionName,
    ...gateCapsOptionNames,
  ];
}

/**
 The deadline option the tests read, which the commands that bound a step take.
 */
const deadlineOptionName = '--deadline-seconds';

await describe({
  name: 'the command line of a gate command',
  children: [
    it({
      name: 'splits options from the positionals a person typed first',
      fn: async () => {
        /**
         The command line.
         */
        const options = gateOptions({
          argv: [
            '--repository',
            '/repo',
            'gate-wrapper-next-alpha',
            'wrapper-next-alpha',
          ],
          knownOptions: knownOptions(),
        });
        expect(options.positionals,).toEqual([
          'gate-wrapper-next-alpha',
          'wrapper-next-alpha',
        ],);
        expect(textOption({
          options,
          name: '--repository',
          fallback: '/fallback',
        }),).toBe('/repo',);
      },
    }),
    it({
      name: 'reads a value attached with an equals sign, which keeps a flag and its value in one argument',
      fn: async () => {
        expect(gateOptions({
          argv: ['--repository=/repo',],
          knownOptions: knownOptions(),
        }).values
          .get('--repository'),).toBe('/repo',);
      },
    }),
    it({
      name: 'keeps a value whole even when it holds an equals sign of its own',
      fn: async () => {
        expect(gateOptions({
          argv: ['--repository=/a=b',],
          knownOptions: knownOptions(),
        }).values
          .get('--repository'),).toBe('/a=b',);
      },
    }),
    it({
      name: 'takes everything after a `--` as a positional, even when it looks like a flag',
      fn: async () => {
        expect(gateOptions({
          argv: [
            '--',
            '--not-an-option',
          ],
          knownOptions: knownOptions(),
        }).positionals,).toEqual(['--not-an-option',],);
      },
    }),
    it({
      name: 'refuses an option it does not know, instead of reading a mistyped flag as a worktree',
      fn: async () => {
        expect(() => gateOptions({
          argv: ['--worktre', 'gate-a',],
          knownOptions: knownOptions(),
        }),)
          .toThrow('Unknown option',);
        expect(() => gateOptions({
          argv: ['--worktre',],
          knownOptions: knownOptions(),
        }),)
          .toThrow(NativeGateError,);
      },
    }),
    it({
      name: 'names the options it does take in that refusal, so a person fixes the typo without a doc',
      fn: async () => {
        expect(() => gateOptions({
          argv: ['--nope',],
          knownOptions: knownOptions(),
        }),)
          .toThrow('--unit-cpu-quota',);
      },
    }),
    it({
      name: 'refuses a flag given twice, because which value names the gate would be unknown',
      fn: async () => {
        expect(() => gateOptions({
          argv: [
            '--repository',
            '/a',
            '--repository',
            '/b',
          ],
          knownOptions: knownOptions(),
        }),)
          .toThrow('--repository is given twice',);
      },
    }),
    it({
      name: 'refuses an option with no value after it',
      fn: async () => {
        expect(() => gateOptions({
          argv: ['--repository',],
          knownOptions: knownOptions(),
        }),)
          .toThrow('argument missing',);
      },
    }),
    it({
      name: 'reads a whole number and keeps the fallback when the command line is silent',
      fn: async () => {
        /**
         A command line with one number on it.
         */
        const options = gateOptions({
          argv: ['--poll-milliseconds', '5000',],
          knownOptions: knownOptions(),
        });
        expect(numberOption({
          options,
          name: '--poll-milliseconds',
          fallback: 20_000,
        }),).toBe(5_000,);
        expect(numberOption({
          options,
          name: '--unit-tasks-max',
          fallback: defaultGateCaps.unitTasksMax,
        }),).toBe(defaultGateCaps.unitTasksMax,);
      },
    }),
    it({
      name: 'refuses a value that is no whole number, naming the option',
      fn: async () => {
        for (const given of [
          '--poll-milliseconds=many',
          '--poll-milliseconds=1.5',
          '--poll-milliseconds=-1',
        ])
          expect(() => numberOption({
            options: gateOptions({
              argv: [given,],
              knownOptions: knownOptions(),
            }),
            name: '--poll-milliseconds',
            fallback: 1,
          }),).toThrow('is not a whole number from zero',);
      },
    }),
    it({
      name: 'reads a deadline in milliseconds, and refuses one that gives a command no bound at all',
      fn: async () => {
        /**
         A command line naming a half-minute deadline.
         */
        const halfMinute = deadlineMillisecondsOption({
          options: gateOptions({
            argv: [
              deadlineOptionName,
              '30',
            ],
            knownOptions: knownOptions(),
          }),
          name: deadlineOptionName,
          fallbackSeconds: 900,
        });
        expect(halfMinute,).toBe(30_000,);
        // A silent command line keeps the bound the command was written with.
        expect(deadlineMillisecondsOption({
          options: gateOptions({
            argv: [],
            knownOptions: knownOptions(),
          }),
          name: deadlineOptionName,
          fallbackSeconds: 900,
        }),).toBe(900_000,);
        // Node reads `timeout: 0` as no timeout at all, so a zero deadline would wait forever.
        expect(() => deadlineMillisecondsOption({
          options: gateOptions({
            argv: [
              deadlineOptionName,
              '0',
            ],
            knownOptions: knownOptions(),
          }),
          name: deadlineOptionName,
          fallbackSeconds: 900,
        }),).toThrow('leaves a command with no deadline at all',);
        // A delay past a 32-bit signed integer is cut to one millisecond, which stops the first step at once.
        expect(() => deadlineMillisecondsOption({
          options: gateOptions({
            argv: [
              deadlineOptionName,
              '2147484',
            ],
            knownOptions: knownOptions(),
          }),
          name: deadlineOptionName,
          fallbackSeconds: 900,
        }),).toThrow('2147483 seconds',);
      },
    }),
    it({
      name: 'starts a gate with every cap the recorded runs used when the command line names none',
      fn: async () => {
        expect(gateCapsFromOptions({
          options: gateOptions({
            argv: [],
            knownOptions: knownOptions(),
          }),
        }),).toEqual(defaultGateCaps,);
      },
    }),
    it({
      name: 'overrides one cap at a time and keeps the rest of the recorded values',
      fn: async () => {
        expect(gateCapsFromOptions({
          options: gateOptions({
            argv: [
              cpuQuotaOption, '400%',
              memoryMaxOption, '8G',
              tasksMaxOption, '900',
              ioWeightOption, '5',
              containerPidsLimitOption, '256',
            ],
            knownOptions: knownOptions(),
          }),
        }),).toEqual({
          unitCpuQuota: '400%',
          unitMemoryMax: '8G',
          unitTasksMax: 900,
          unitIoWeight: 5,
          containerPidsLimit: 256,
        },);
      },
    }),
    it({
      name: 'refuses caps systemd cannot read before anything starts',
      fn: async () => {
        expect(() => gateCapsFromOptions({
          options: gateOptions({
            argv: [ioWeightOption, '11',],
            knownOptions: knownOptions(),
          }),
        }),)
          .toThrow('--unit-io-weight',);
      },
    }),
  ],
});
