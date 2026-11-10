/**
 The container limits of the wrapper's release gate, which differ from the repository's defaults in one bound: the
 number of CPUs, and with it the number of tests the gate runs at once.

 A release waits for the gate, so the gate's container is sized for the release path rather than for the least a
 container may take. The suite is the gate's longest phase and it saturates whatever it is given: on 2026-11-10 the
 7,192 tests of gate `cd9b8132f8` summed to 2,000 s of test time inside a 1,000 s run at two CPUs, which is exactly two
 test threads busy for the whole run. Every test in that run either worked or waited for something it had itself
 started, so the wall time is the summed test time divided by the threads, and the only way to shorten it without
 running fewer tests is to run more of them at once.

 `doc/planning/git-policy-cli-cachyos-test-containers.md` holds the paired measurement that sized this bound and the
 room the gate's own systemd unit leaves above it. A person who needs another count passes `--cpus <count>` after the
 `--` of the task, which wins over the option here because the parser keeps the last repeat of an option.
 */
import {
  type ContainerLimits,
  defaultContainerLimits,
} from './container-limits.ts';

//region The bound the gate sizes itself

/**
 CPUs the gate's container may use, which is also the number of tests nextest runs at once and the number of compile
 jobs the image build starts: nextest takes its thread count from the container's CPU quota, and the build's job count
 follows the same limit.

 Four is twice the repository's default for a container and well inside the ten CPUs the gate's own systemd unit
 allows, so the gate keeps its measured share while the host's other agents build beside it.
 */
export const gateContainerCpus = 4;

/**
 The limits every container of this gate runs under: the repository defaults, with the gate's own CPU count.

 The memory and process bounds stay the repository's, because neither was what held the gate's suite back, and because
 a campaign's proof of a green gate compares the two.
 */
export const gateContainerLimits: ContainerLimits = {
  ...defaultContainerLimits,
  cpus: gateContainerCpus,
};

/**
 The gate's own limits as the option text its parser reads before the person's arguments, so every limit it does not
 name stays the repository default and every option a person passes wins.

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
  String(gateContainerLimits.cpus),
];

//endregion The bound the gate sizes itself
