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
  type ContainerLimitFlag,
  type ContainerLimits,
  defaultContainerLimits,
  runLimitArguments,
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
 Memory the gate's image build may use: 8 GiB.

 Not the repository default, which stays the suite's and Clippy's bound. The build compiles the crate's three layers at
 four jobs, and the arms of 2026-11-24 (`podman build --no-cache` at the gate's own CPU count, the build containers'
 cgroups read every 250 ms; `doc/planning/git-policy-cli-cachyos-test-containers.md`, "The image build's own memory
 bound, measured 2026-11-24") peaked at 5,363.4 MiB in the compile of the wrapper's own sources under a 10 GiB cap no
 step of it met. The same step under the shared 2 GiB reached that cap and stayed there: `memory.peak` 2,048 MiB,
 `memory.events` `max` 149,888, its own `memory.pressure` some avg10 22.33, 29.8 s of stall over 196 s, and 197.1 s
 against 87.2 s without the cap. That reclaim is what the issue measured at 2026-11-24T19:20-05:00
 (`doc/troubleshooting/desktop-freezes-while-many-agent-runs-thrash-zram-swap.md`, the catalogue of that evening), and
 it lifted the machine-wide stall that decides whether new work may start.

 8 GiB is 8,192 MiB, 1.53 times that measured peak, so a source change that adds work still fits, and it is well inside
 the 20 GiB the gate's own systemd unit allows (`bin/native-gate-caps.ts`).

 The bound is sized at the gate's own four compile jobs. A gate given more CPUs compiles more jobs at once and wants more
 memory, and a host that cannot spare 8 GiB wants less; `--memory <size>` after the task name is the one option that does
 both, because it bounds the build and the suite together. A build held below what it compiles needs is not a smaller
 build but a slower one: its pages are reclaimed and swapped instead of used, and the capped arm here spent 29.8 s of its
 196 s compile held at the cap.
 */
export const gateBuildMemory = '8g';

/**
 Memory the gate's Clippy container may use: 8 GiB.

 Not the repository default, which is the suite's bound. Clippy compiles every target of the crate, the test build
 included, in one `cargo clippy --all-targets` run at the gate's four compile jobs, and on 2026-11-27 a gate that had
 passed every test of the suite refused because Clippy was killed at the 2 GiB it shared with that suite (issue 6562).
 Measured the same day in the gate's own image and settings: an arm whose 8 GiB bound no step met peaked at 4,574.4 MiB
 with 647.1 MiB of that charge in host-driven swap, and five gates at this bound read 4,790.0, 4,835.4, 5,062.8, 5,273.4
 and 5,278.7 MiB, every one with `memory.events` `max` 0 and `oom_kill` 0. A compile with room to keep its pages peaks
 higher than one the host is reclaiming, so the bound is sized from the highest of those, 5,278.7 MiB: 8 GiB is
 8,192 MiB, 1.55 times it and 2,913 MiB above it. The suite's container read 1,149.3 MiB in the arm and up to 1,516.5 MiB
 in the gates, of the 2 GiB its own measurement supports
 (`doc/planning/git-policy-cli-cachyos-test-containers.md`, "The Clippy container's own memory bound, measured
 2026-11-27").

 A container held at its cap is not a smaller container but a stalled one, and a compile held there is killed outright:
 the same crate's test build died with `SIGKILL` under 2 GiB where 4 GiB let it finish (the comment on issue 6455 of
 2026-11-24). `--memory <size>` after the task name still bounds this container, so a host that cannot spare the bound
 lowers it.
 */
export const gateClippyMemory = '8g';

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
  givenFlags: [],
},)
  .limits;

//endregion The bounds the gate sizes itself

//region What the gate runs with

/**
 The limit a run's containers actually use, the bound the image build uses, and the line that says so when a process
 bound is not what the person asked for.

 @example
 ```ts
 const { limits, buildLimits, raised } = gateLimitsFrom({ given: defaultContainerLimits, givenFlags: [] });
 ```
 */
export type GateLimits = {
  /**
   The limits every container of this gate runs under.
   */
  readonly limits: ContainerLimits;
  /**
   The limits Clippy's container runs under: {@link limits} with Clippy's own memory bound, or the memory the person
   gave, which bounds Clippy as it bounds every other container.
   */
  readonly clippyLimits: ContainerLimits;
  /**
   The bound the gate's image build runs under: {@link limits} with the build's own memory default, or the memory the
   person gave, which bounds the build as it bounds every other container.
   */
  readonly buildLimits: ContainerLimits;
  /**
   What the gate says about a process bound it had to raise, absent when it kept the one it was given.
   */
  readonly raised?: string;
};

/**
 The line a gate prints to name which memory bound each of its containers runs under.

 The build's bound, the suite's and Clippy's differ unless the person passed `--memory`, and a log that names all three
 is what lets a reader tell a build, a suite or a Clippy compile held at its cap from one that had room.

 @param limits - the limits the suite's container runs under

 @param clippyLimits - the limits Clippy's container runs under

 @param buildLimits - the limits the image build runs under

 @returns one line, ending in a sentence

 @example
 ```ts
 console.log(buildMemoryLine({ limits: gateContainerLimits, clippyLimits, buildLimits }));
 ```
 */
export function buildMemoryLine({
  limits,
  clippyLimits,
  buildLimits,
}: {
  readonly limits: ContainerLimits;
  readonly clippyLimits: ContainerLimits;
  readonly buildLimits: ContainerLimits;
},): string {
  if ((buildLimits.memory === limits.memory) && (clippyLimits.memory === limits.memory))
    return `The image build, the suite's container and Clippy's are bounded at ${limits.memory}, the memory this gate was given.`;
  return `The image build is bounded at ${buildLimits.memory}, its own measured bound; the suite's container at ${limits.memory}; Clippy's at ${clippyLimits.memory}, its own measured bound. A --memory after the task name bounds all three.`;
}

/**
 The gate's limits from the ones its parser read, with the process bound raised to what the test threads need and the
 image build given its own memory bound.

 A process cap below the thread count's need is not a smaller bound but a coin flip: the same four threads passed every
 test under 1024 processes and failed three at 512. So the cap is a floor here and any larger value, the person's
 included, is kept.

 The image build compiles the crate rather than running it, so it gets {@link gateBuildMemory} instead of the repository's
 2 GiB. Clippy compiles the crate's test targets, so it gets {@link gateClippyMemory}. A `--memory` the person gave still
 bounds both, because the person asking every container of the gate for a smaller bound means the build and Clippy too;
 `givenFlags` is what tells a person's own `--memory 2g` from the repository default they never named.

 @param given - the limits the parser read from the gate's own options and the person's

 @param givenFlags - the limit options the parser read, so a memory the person named is told from the default

 @returns the limits to run with, Clippy's limits, the bound to build with, and the line to print when the process bound
   was raised

 @example
 ```ts
 const { limits, clippyLimits, buildLimits, raised } = gateLimitsFrom({
   given: { memory: '2g', cpus: 4, pidsLimit: 512 },
   givenFlags: [],
 });
 ```
 */
export function gateLimitsFrom({
  given,
  givenFlags,
}: {
  readonly given: ContainerLimits;
  readonly givenFlags: readonly ContainerLimitFlag[];
},): GateLimits {
  /**
   Processes the test threads of these limits need.
   */
  const needed = processesForTestThreads(given.cpus,);
  /**
   The limits every container of the gate runs under, which is the person's own when its process bound is enough.
   */
  const limits = given.pidsLimit >= needed
    ? given
    : {
      ...given,
      pidsLimit: needed,
    };
  return {
    limits,
    clippyLimits: {
      ...limits,
      memory: givenFlags.includes('--memory',) ? given.memory : gateClippyMemory,
    },
    buildLimits: {
      ...limits,
      memory: givenFlags.includes('--memory',) ? given.memory : gateBuildMemory,
    },
    ...(given.pidsLimit >= needed
      ? {}
      : { raised: `The gate runs ${String(given.cpus)} test threads at once, which need ${String(needed)} processes, so its containers get that: this gate's process bound follows its test-thread count, and the --pids-limit it was given was ${String(given.pidsLimit)}.` }),
  };
}

/**
 Both gate containers' complete `podman run` bounds: the flags they share, then each container's own memory, CPUs,
 processes and disk weight.

 The two containers do not share a memory bound: the suite keeps the one its own measurement supports, and Clippy gets
 {@link gateClippyMemory}, because Clippy compiles the crate's test targets and the suite runs them. Everything else is
 the same for both, and both arrays come from here so neither container can be started with the other's memory bound.

 @param shared - the flags both containers take before their own bounds, such as `--rm`, `--init` and `--network=none`

 @param limits - the limits the suite's container runs under

 @param clippyLimits - the limits Clippy's container runs under

 @returns one complete argument array per container, in the order its role names them

 @example
 ```ts
 const bounds = gateContainerBounds({ shared: ['--rm'], limits: gateContainerLimits, clippyLimits });
 ```
 */
export function gateContainerBounds({
  shared,
  limits,
  clippyLimits,
}: {
  readonly shared: readonly string[];
  readonly limits: ContainerLimits;
  readonly clippyLimits: ContainerLimits;
},): {
  readonly suite: readonly string[];
  readonly clippy: readonly string[];
} {
  return {
    suite: [
      ...shared,
      ...runLimitArguments({
        limits,
        withPidsLimit: true,
      }),
    ],
    clippy: [
      ...shared,
      ...runLimitArguments({
        limits: clippyLimits,
        withPidsLimit: true,
      }),
    ],
  };
}

//endregion What the gate runs with
