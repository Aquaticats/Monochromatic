import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildLimitArguments,
  type ContainerLimits,
  cpuQuotaMicroseconds,
  defaultContainerLimits,
  memoryArguments,
  parallelJobs,
  parseContainerLimitOptions,
  runLimitArguments,
} from './container-limits.ts';

/**
 Limits a test overrides every field of.

 The values differ from the defaults, so a helper that ignored one would show it.
 */
const overridden: ContainerLimits = {
  memory: '512m',
  cpus: 1.5,
  pidsLimit: 64,
};

/**
 Parse options and return the message the parser threw.

 @param parameters - options, and whether the caller accepts a PID cap and stops at its first operand

 @returns the thrown message, or an empty text when nothing was thrown
 */
function rejectionOf(parameters: Readonly<{
  options: readonly string[];
  acceptPidsLimit: boolean;
  leadingOnly: boolean;
}>,): string {
  try {
    parseContainerLimitOptions(parameters,);
  }
  catch (error) {
    return error instanceof Error ? error.message : String(error,);
  }
  return '';
}

await describe({
  name: '',
  children: [
    //region Defaults reproduce the arguments the scripts spelled by hand

    describe({
      name: 'defaults',
      children: [
        it({
          name: 'are 2 GiB, 2 CPUs and 128 PIDs',
          fn: async () => {
            expect(defaultContainerLimits,).toEqual({
              memory: '2g',
              cpus: 2,
              pidsLimit: 128,
            },);
          },
        },),
        it({
          name: 'build arguments are the memory, period and quota the image builds used',
          fn: async () => {
            expect(buildLimitArguments(defaultContainerLimits,),).toEqual([
              '--memory=2g',
              '--cpu-period=100000',
              '--cpu-quota=200000',
            ],);
          },
        },),
        it({
          name: 'run arguments with a PID cap are the gate, mutation and image-build container bounds',
          fn: async () => {
            expect(runLimitArguments({
              limits: defaultContainerLimits,
              withPidsLimit: true,
            },),).toEqual([
              '--memory=2g',
              '--cpus=2',
              '--pids-limit=128',
            ],);
          },
        },),
        it({
          name: 'run arguments without a PID cap are the interop and e2e container bounds',
          fn: async () => {
            expect(runLimitArguments({
              limits: defaultContainerLimits,
              withPidsLimit: false,
            },),).toEqual([
              '--memory=2g',
              '--cpus=2',
            ],);
          },
        },),
        it({
          name: 'memory arguments are the e2e base image build bound',
          fn: async () => {
            expect(memoryArguments(defaultContainerLimits,),).toEqual(['--memory=2g',],);
          },
        },),
        it({
          name: 'parallel jobs are the 2 the Git build used',
          fn: async () => {
            expect(parallelJobs(defaultContainerLimits,),).toBe(2,);
          },
        },),
        it({
          name: 'no options parse to the defaults and leave no remainder',
          fn: async () => {
            expect(parseContainerLimitOptions({
              options: [],
              acceptPidsLimit: true,
              leadingOnly: false,
            },),).toEqual({
              limits: defaultContainerLimits,
              rest: [],
            },);
          },
        },),
      ],
    },),

    //endregion Defaults reproduce the arguments the scripts spelled by hand

    //region Overrides change exactly the memory, CPU and quota arguments

    describe({
      name: 'overrides',
      children: [
        it({
          name: 'build arguments change the memory and the derived quota, never the period',
          fn: async () => {
            expect(buildLimitArguments(overridden,),).toEqual([
              '--memory=512m',
              '--cpu-period=100000',
              '--cpu-quota=150000',
            ],);
          },
        },),
        it({
          name: 'run arguments change the memory, the CPUs and the PID cap',
          fn: async () => {
            expect(runLimitArguments({
              limits: overridden,
              withPidsLimit: true,
            },),).toEqual([
              '--memory=512m',
              '--cpus=1.5',
              '--pids-limit=64',
            ],);
          },
        },),
        it({
          name: 'run arguments without a PID cap ignore the PID override',
          fn: async () => {
            expect(runLimitArguments({
              limits: overridden,
              withPidsLimit: false,
            },),).toEqual([
              '--memory=512m',
              '--cpus=1.5',
            ],);
          },
        },),
        it({
          name: 'quota is the CPUs times the period, rounded to a whole microsecond',
          fn: async () => {
            expect(cpuQuotaMicroseconds(2,),).toBe(200_000,);
            expect(cpuQuotaMicroseconds(1.5,),).toBe(150_000,);
            // 0.1 * 100000 is 10000.000000000002 in floating point.
            expect(cpuQuotaMicroseconds(0.1,),).toBe(10_000,);
            expect(cpuQuotaMicroseconds(0.07,),).toBe(7_000,);
          },
        },),
        it({
          name: 'parallel jobs round the CPUs down and never drop below one',
          fn: async () => {
            expect(parallelJobs({
              ...defaultContainerLimits,
              cpus: 4,
            },),).toBe(4,);
            expect(parallelJobs({
              ...defaultContainerLimits,
              cpus: 2.5,
            },),).toBe(2,);
            expect(parallelJobs({
              ...defaultContainerLimits,
              cpus: 0.5,
            },),).toBe(1,);
          },
        },),
      ],
    },),

    //endregion Overrides change exactly the memory, CPU and quota arguments

    //region Parsing

    describe({
      name: parseContainerLimitOptions.name,
      children: [
        it({
          name: 'reads all three options and returns the other arguments in order',
          fn: async () => {
            expect(parseContainerLimitOptions({
              options: [
                '--timeout',
                '60',
                '--memory',
                '512m',
                '--cpus',
                '1.5',
                '--file',
                'src/native/lock.rs',
                '--pids-limit',
                '64',
              ],
              acceptPidsLimit: true,
              leadingOnly: false,
            },),).toEqual({
              limits: overridden,
              rest: [
                '--timeout',
                '60',
                '--file',
                'src/native/lock.rs',
              ],
            },);
          },
        },),
        it({
          name: 'lets the last repeat of an option win',
          fn: async () => {
            expect(parseContainerLimitOptions({
              options: [
                '--cpus',
                '4',
                '--cpus',
                '3',
              ],
              acceptPidsLimit: false,
              leadingOnly: false,
            },).limits.cpus,).toBe(3,);
          },
        },),
        it({
          name: 'accepts every memory unit and a bare byte count',
          fn: async () => {
            for (const memory of [
              '1048576',
              '64b',
              '512k',
              '512m',
              '2g',
            ]) {
              expect(parseContainerLimitOptions({
                options: [
                  '--memory',
                  memory,
                ],
                acceptPidsLimit: false,
                leadingOnly: false,
              },).limits.memory,).toBe(memory,);
            }
          },
        },),
        it({
          name: 'accepts a CPU count with up to two decimals, down to 0.01',
          fn: async () => {
            for (const cpus of [
              '1',
              '0.5',
              '0.01',
              '2.25',
              '16',
            ]) {
              expect(parseContainerLimitOptions({
                options: [
                  '--cpus',
                  cpus,
                ],
                acceptPidsLimit: false,
                leadingOnly: false,
              },).limits.cpus,).toBe(Number(cpus,),);
            }
          },
        },),
        it({
          name: 'in leading mode stops at the first argument that is not a limit option and keeps the rest verbatim',
          fn: async () => {
            expect(parseContainerLimitOptions({
              options: [
                '--cpus',
                '1',
                'old-format-driver',
                '--memory',
                'driver-value',
              ],
              acceptPidsLimit: false,
              leadingOnly: true,
            },),).toEqual({
              limits: {
                ...defaultContainerLimits,
                cpus: 1,
              },
              rest: [
                'old-format-driver',
                '--memory',
                'driver-value',
              ],
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'rejections',
      children: [
        ...[
          // Each case pairs the options with the option name and the accepted forms the message must carry.
          {
            options: ['--memory', '0',],
            named: ['--memory', '512m', '2g', 'got 0',],
          },
          {
            options: ['--memory', '2gb',],
            named: ['--memory', 'b, k, m or g', 'got 2gb',],
          },
          {
            options: ['--memory', '-1g',],
            named: ['--memory', 'got -1g',],
          },
          {
            options: ['--memory', '01g',],
            named: ['--memory', 'got 01g',],
          },
          {
            options: ['--memory',],
            named: ['--memory', 'got no value',],
          },
          {
            options: ['--cpus', '0',],
            named: ['--cpus', 'fractions', '0.01', 'got 0',],
          },
          {
            options: ['--cpus', '0.001',],
            named: ['--cpus', 'got 0.001',],
          },
          {
            options: ['--cpus', '1.5.2',],
            named: ['--cpus', 'got 1.5.2',],
          },
          {
            options: ['--cpus', '-2',],
            named: ['--cpus', 'got -2',],
          },
          {
            options: ['--cpus', 'two',],
            named: ['--cpus', 'got two',],
          },
          {
            options: ['--cpus', '',],
            named: ['--cpus', 'got no value',],
          },
          {
            options: ['--pids-limit', '0',],
            named: ['--pids-limit', 'whole number', 'got 0',],
          },
          {
            options: ['--pids-limit', '1.5',],
            named: ['--pids-limit', 'got 1.5',],
          },
        ].map(function rejectionCase({
          options,
          named,
        },) {
          return it({
            name: `${options.join(' ',)} is rejected naming ${named.join(' | ',)}`,
            fn: async () => {
              /**
               Message of the error the parser threw for the options.
               */
              const message = rejectionOf({
                options,
                acceptPidsLimit: true,
                leadingOnly: false,
              },);
              expect(message,).not.toBe('',);
              for (const fragment of named)
                expect(message,).toContain(fragment,);
            },
          },);
        },),
        it({
          name: '--pids-limit is rejected where the container has no PID cap',
          fn: async () => {
            const message = rejectionOf({
              options: [
                '--pids-limit',
                '64',
              ],
              acceptPidsLimit: false,
              leadingOnly: false,
            },);
            expect(message,).toContain('--pids-limit',);
            expect(message,).toContain('not accepted',);
          },
        },),
      ],
    },),

    //endregion Parsing
  ],
},);
