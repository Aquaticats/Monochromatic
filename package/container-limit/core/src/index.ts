/**
 The one lowest disk weight, and the Podman options that bound a started container or an image build.

 Repository rule HXR (`AGENTS.md`) starts every container that can exhaust the host with CPU, memory, process and
 disk-weight limits, because a container sits in its own cgroup scope outside the unit of whatever launched it and
 ignores that unit's `IOWeight`.
 The tasks, runners and workflows that start containers used to spell the weight themselves (several copies, one per
 package); they take it from here.

 This file imports nothing, so a `mise.toml` script of a package without a `package.json` can import it by relative path
 and node strips its types, as those scripts already import `package/test-harness/core-matrix/src/bin/*.ts`.
 */

/**
 Lowest weight Podman accepts for a container's disk I/O (`podman run --blkio-weight` takes 10 to 1000).

 The container-wide default is 100, so at this weight a started container gets a tenth of the disk time of a default
 sibling when both want it.
 `.github/workflows/pnpr-publish.yml` spells it as a literal for `docker run`, which has no second option, and a test
 in this package reads the workflow and fails when the two differ.
 */
export const LOWEST_DISK_WEIGHT = 10;

/**
 Name of the environment variable that turns the disk weight off.

 GitHub-hosted runners do not delegate the cgroup `io` controller to the user that runs `podman`, so a container started
 with the weight is refused there: run 42447009237 of `.github/workflows/music-player-release.yml` (2026-11-08) failed in
 the jobs `test-alpine-x86_64` and `test-alpine-aarch64` with `crun: opening file 'io.weight' for writing: No such file
 or directory`.
 A workflow whose jobs start containers on such a runner sets this variable to {@link DISK_WEIGHT_OFF}; nothing else
 reads it, and a developer machine never sets it.
 */
export const DISK_WEIGHT_SWITCH = 'CONTAINER_LIMIT_DISK_WEIGHT';

/**
 The only value of {@link DISK_WEIGHT_SWITCH} that turns the weight off; any other value, an empty one included, keeps it.
 */
export const DISK_WEIGHT_OFF = 'off';

/**
 Length of one CFS scheduling period in microseconds, which is also Podman's own default period.
 */
export const CFS_PERIOD_MICROSECONDS = 100_000;

/**
 Bounds of one container.
 */
export type ContainerLimits = {
  /**
   Podman memory size: digits and an optional unit, such as `2g` or `512m`.
   */
  readonly memory: string;
  /**
   CPUs the container may use; fractions allowed.
   */
  readonly cpus: number;
  /**
   Processes (threads count) the container may run.
   */
  readonly pidsLimit: number;
};

/**
 Bounds of one image build: the part of {@link ContainerLimits} that `podman build` accepts.
 */
export type ImageBuildLimits = Pick<ContainerLimits, 'cpus' | 'memory'>;

/**
 Disk-weight options of a started container, in the two forms the two block-layer weight files need.

 `--blkio-weight` sets the cgroup file `io.bfq.weight`, which the BFQ I/O scheduler reads, and leaves `io.weight`
 (which the iocost controller reads) at the default; `--cgroup-conf=io.weight=` sets `io.weight` and leaves
 `io.bfq.weight` at the default.
 Passing both puts whichever file the host's scheduler reads at {@link LOWEST_DISK_WEIGHT}.
 `podman build` has neither option, so image builds are not covered.

 @param environment - where {@link DISK_WEIGHT_SWITCH} is read; the process environment unless a test gives its own

 @returns `['--blkio-weight=10', '--cgroup-conf=io.weight=10']`, or no argument when the switch says off

 @example
 ```ts
 const options = diskWeightArguments();
 ```
 */
export function diskWeightArguments({
  environment = process.env,
}: {
  readonly environment?: Readonly<NodeJS.ProcessEnv>;
} = {},): readonly string[] {
  if (environment[DISK_WEIGHT_SWITCH] === DISK_WEIGHT_OFF)
    return [];
  return [
    `--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,
    `--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,
  ];
}

/**
 Every bound of a started container: memory, CPUs, processes and the lowest disk weight.

 @param limits - memory, CPUs and processes sized to the container's job

 @returns the `podman run` or `podman create` options; the weight is left out when {@link DISK_WEIGHT_SWITCH} says off

 @example
 ```ts
 const options = containerLimitArguments({ memory: '2g', cpus: 2, pidsLimit: 512 });
 ```
 */
export function containerLimitArguments(limits: ContainerLimits,): readonly string[] {
  return [
    `--memory=${limits.memory}`,
    `--cpus=${String(limits.cpus,)}`,
    `--pids-limit=${String(limits.pidsLimit,)}`,
    ...diskWeightArguments(),
  ];
}

/**
 Bounds of `podman build`, which has no `--cpus`, `--pids-limit` or disk-weight option.

 @param limits - memory and CPUs sized to the build

 @returns memory, the CFS period and the quota derived from the CPUs (rounded, because floating point turns
   `0.1 * 100000` into `10000.000000000002`)

 @example
 ```ts
 const options = imageBuildLimitArguments({ memory: '4g', cpus: 4 });
 ```
 */
export function imageBuildLimitArguments(limits: ImageBuildLimits,): readonly string[] {
  return [
    `--memory=${limits.memory}`,
    `--cpu-period=${String(CFS_PERIOD_MICROSECONDS,)}`,
    `--cpu-quota=${String(Math.round(limits.cpus * CFS_PERIOD_MICROSECONDS,),)}`,
  ];
}
