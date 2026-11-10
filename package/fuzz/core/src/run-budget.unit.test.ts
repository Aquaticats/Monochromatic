import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  DEFAULT_FUZZ_RUNS,
  fuzzRunsFromEnvironment,
} from './run-budget.ts';

/**
 A variable name no other test or task sets, so a runaway environment cannot change the result.
 */
const ENV_NAME = 'FUZZ_CORE_UNIT_TEST_RUNS';

/**
 One sidecar's budget over a stated environment, which keeps the process's own out of every case.

 @param raw - variable value to state, or nothing for an unset variable

 @returns the count the budget resolves to

 @example
 ```ts
 const runs = runsOver('5000');
 ```
 */
function runsOver(raw?: string,): number {
  return fuzzRunsFromEnvironment({
    envName: ENV_NAME,
    environment: raw === undefined ? {} : { [ENV_NAME]: raw, },
  },);
}

await describe({
  name: fuzzRunsFromEnvironment.name,
  children: [
    it({
      name: 'uses the shared default for an unset variable',
      fn: async () => {
        expect(runsOver(),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'takes a positive integer as the campaign count',
      fn: async () => {
        /**
         Count a campaign asks for, well above the default so the two cannot be confused.
         */
        const campaignCount = DEFAULT_FUZZ_RUNS * 10;
        expect(runsOver(String(campaignCount,),),).toBe(campaignCount,);
      },
    },),
    it({
      name: 'uses the sidecar own default for an unset variable',
      fn: async () => {
        /**
         Count a sidecar whose properties cost milliseconds per run states.
         */
        const sidecarDefault = DEFAULT_FUZZ_RUNS / 4;
        expect(
          fuzzRunsFromEnvironment({
            envName: ENV_NAME,
            defaultRuns: sidecarDefault,
            environment: {},
          },),
        ).toBe(sidecarDefault,);
      },
    },),
    it({
      name: 'keeps the campaign count over the sidecar own default',
      fn: async () => {
        /**
         Count a campaign asks for.
         */
        const campaignCount = DEFAULT_FUZZ_RUNS * 2;
        expect(
          fuzzRunsFromEnvironment({
            envName: ENV_NAME,
            defaultRuns: DEFAULT_FUZZ_RUNS / 4,
            environment: { [ENV_NAME]: String(campaignCount,), },
          },),
        ).toBe(campaignCount,);
      },
    },),
    it({
      name: 'falls back to the default for a variable holding text',
      fn: async () => {
        expect(runsOver('many',),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'falls back to the default for an empty variable, which would otherwise run every property zero times',
      fn: async () => {
        expect(runsOver('',),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'falls back to the default for a blank variable',
      fn: async () => {
        expect(runsOver('  ',),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'falls back to the default for zero',
      fn: async () => {
        expect(runsOver('0',),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'falls back to the default for a negative count',
      fn: async () => {
        expect(runsOver('-5',),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'falls back to the default for a fractional count',
      fn: async () => {
        expect(runsOver('2.5',),).toBe(DEFAULT_FUZZ_RUNS,);
      },
    },),
    it({
      name: 'reads the process environment when none is given',
      fn: async () => {
        /**
         Variable the process's own environment carries for this case, restored after it.
         */
        /**
         Count the process's own environment states for this case.
         */
        const campaignCount = DEFAULT_FUZZ_RUNS * 3;
        const prior = process.env[ENV_NAME];
        process.env[ENV_NAME] = String(campaignCount,);
        try {
          expect(fuzzRunsFromEnvironment({ envName: ENV_NAME, },),).toBe(campaignCount,);
        } finally {
          if (prior === undefined)
            delete process.env[ENV_NAME];
          else
            process.env[ENV_NAME] = prior;
        }
      },
    },),
  ],
},);
