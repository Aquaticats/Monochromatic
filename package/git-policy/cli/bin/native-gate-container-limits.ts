/**
 The container limits of the wrapper's release gate, which differ from the repository's defaults in two bounds that
 follow each other: the CPUs, and the processes the suite needs while running that many tests at once.

 A release waits for the gate, so the gate's container is sized for the release path rather than for the least a
 container may take. The suite is the gate's longest phase and it saturates whatever it is given: on 2026-11-10 the
 7,192 tests of gate `cd9b8132f8` summed to 2,000 s of test time inside a 1,000 s run at two CPUs, which is exactly two
 test threads busy for the whole run. Every test in that run either worked or waited for something it had itself
 started, so the wall time is the summed test time divided by the threads, and the only way to shorten it without
 running fewer tests is to run more of them at once.

 More threads need more processes. The fault sweeps of the binary suite start six cases at a time inside one test, and
 the services they start carry threads of their own, so the container's process count follows the test-thread count. At
 four threads the suite peaked at 401 processes in one run and reached the 512 the repository's default allows in
 another, which failed three tests on `fork` with `Resource temporarily unavailable`; at 1024 processes the same four
 threads peaked at 401 and passed all 7,192.

 `doc/planning/git-policy-cli-cachyos-test-containers.md` holds the paired measurement that sized both bounds and the
 room the gate's own systemd unit leaves above them.
 */
import {
  type ContainerLimits,
  defaultContainerLimits,
} from './container-limits.ts';


//region The bounds the gate sizes itself

/**
 CPUs the gate's container may use, which is also the number of tests nextest runs at once and the number of compile
 jobs the image build starts: nextest takes its thread count from the container's CPU quota, and the build's job count
 follows the same limit.

 Four is twice the repository's default for a container and well inside the ten CPUs the gate's own systemd unit
 allows, so the gate keeps its measured share while the host's other agents build beside it.
 */
export const gateContainerCpus = 4;

/**
 Processes one test thread of the suite was measured to need, which sizes the process bound for any thread count.

 At four threads two runs peaked at 401 and at the 512 the repository default allows, so 256 for each thread leaves the
 measured peak about half the bound and still stops a runaway fork loop long before it hurts the host.
 */
export const processesPerTestThread = 256;

/**
 Processes the gate's containers may run at a given CPU count: {@link processesPerTestThread} for each test thread.

 @param cpus - CPUs of the container, which is also the number of tests it runs at once

 @returns the process bound for that thread count

 @example
 ```ts
 processesForTestThreads(4);
 ```
 */
export function processesForTestThreads(cpus: number,): number {
  return Math.ceil(cpus,)
    * processesPerTestThread;
}

/**
 The gate's own limits as the option text its parser reads before the person's arguments, so every bound the gate does
 not size itself stays the repository default and every option a person passes wins.

 Only the CPU count is here: it decides the test-thread count, and the process bound follows from it in
 {@link gateLimitsFrom}, which has to see the person's own bounds to keep them.

 @example
 ```ts
 const { limits } = parseContainerLimitOptions({
   options: [...gateContainerLimitOptions, ...process.argv.slice(2)],
   acceptPidsLimit: true,
 });
 ```
 */
export const gateContainerLimitOptions: readonly string[] = [
  '--cpus',
  String(gateContainerCpus),
];

/**
 The limits a gate whose person passes nothing runs under: the repository defaults, with the gate's CPU count and the
 process bound those threads need.

 The mutation runner's tests and the proof a campaign reads need one value to name, which is why this is a constant
 rather than an expression at each use.
 */
export const gateContainerLimits: ContainerLimits = gateLimitsFrom({
  given: {
    ...defaultContainerLimits,
    cpus: gateContainerCpus,
  },
})
  .limits;

//endregion The bounds the gate sizes itself

//region What the gate runs with

/**
 The limit a run's containers actually use, and the line that says so when it is not what the person asked for.

 @example
 ```ts
 const { limits, raised } = gateLimitsFrom({ given: defaultContainerLimits });
 ```
 */
export type GateLimits = {
  /**
   The limits every container of this gate runs under.
   */
  readonly limits: ContainerLimits;
  /**
   What the gate says about a process bound it had to raise, absent when it kept the one it was given.
   */
  readonly raised?: string;
};

/**
 The gate's limits from the ones its parser read, with the process bound raised to what the test threads need.

 A process cap below the thread count's need is not a smaller bound but a coin flip: the same four threads passed every
 test under 1024 processes and failed three at 512. So the cap is a floor here and any larger value, the person's
 included, is kept; the memory and CPU bounds are the person's, unchanged.

 @param given - the limits the parser read from the gate's own options and the person's

 @returns the limits to run with, and the line to print when the process bound was raised

 @example
 ```ts
 const { limits, raised } = gateLimitsFrom({ given: { memory: '2g', cpus: 4, pidsLimit: 512 } });
 ```
 */
export function gateLimitsFrom({ given, }: {
  readonly given: ContainerLimits;
},): GateLimits {
  /**
   Processes the test threads of these limits need.
   */
  const needed = processesForTestThreads(given.cpus,);
  if (given.pidsLimit >= needed)
    return { limits: given, };
  return {
    limits: {
      ...given,
      pidsLimit: needed,
    },
    raised: `The gate runs ${String(given.cpus)} test threads at once, which need ${String(needed)} processes, so its containers get that instead of the --pids-limit of ${String(given.pidsLimit)} it was given: this gate's process bound follows its test-thread count.`,
  };
}

//endregion What the gate runs with
