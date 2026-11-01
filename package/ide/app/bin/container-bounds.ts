// The resource bound of the IDE's build and test containers, in one place.
// What: the podman limit arguments, the build environment, and the image build arguments, all derived from one
// CPU count and one memory size.
// Why: the bound used to be copied into mise.toml and seventeen inspection scripts, so changing it meant
// editing every copy, and the thread count of the test runner never followed the CPU limit of the container.
// The tasks and scripts import this file (node strips the types); the pure functions are tested in
// container-bounds.test.ts, and no number below is a host measurement.

import {
  availableParallelism,
  totalmem
} from 'node:os';

/**
 Share of the host's maximum that a container may use unless the caller says otherwise.
 
 Owner decisions (recorded in doc/handover/slint-ide-0x.md): on 2026-10-18 the containers used 75 percent of the
 host's CPUs and memory; on 2026-10-22, after "Use my system (almost) fully instead of limiting to only a very limited
 subset of it", the owner chose 90 percent. The share is computed from the host at run time; a loaded host lowers it
 with the variables below.
 */
export const hostShare = 0.9;

/**
 Smallest memory in GiB that the toolchain image and a cargo build work in.
 */
export const minimumMemoryGiB = 2;

/**
 Smallest process limit; a larger CPU count raises it by {@link pidsPerCpu}.
 */
export const minimumPids = 512;

/**
 Processes per CPU when that exceeds {@link minimumPids}: test threads each spawn helpers.
 */
export const pidsPerCpu = 256;

/**
 Open-file limit of every container; it does not scale with the CPU count.
 */
export const defaultOpenFiles = 4_096;

/**
 CFS period in microseconds that `podman build --cpu-quota` is measured against.
 */
export const cpuPeriod = 100_000;

/**
 The variables a caller may set; an unset or empty one leaves the computed value.
 */
export const variableNames = {
  cpus: 'IDE_CONTAINER_CPUS',
  memory: 'IDE_CONTAINER_MEMORY',
  pids: 'IDE_CONTAINER_PIDS',
  openFiles: 'IDE_CONTAINER_NOFILE',
  jobs: 'IDE_CONTAINER_JOBS',
  testThreads: 'IDE_CONTAINER_TEST_THREADS',
} as const;

/**
 What a caller sees when a variable is malformed.
 */
export class ContainerBoundError extends Error {
  override name = 'ContainerBoundError';
}

/**
 Host maximum the default bound is a share of.
 */
export type Host = {
  readonly cpus: number;
  readonly memoryBytes: number
};

/**
 The default CPU count and memory size computed from a host.
 */
export type HostBound = {
  readonly cpus: number;
  readonly memoryGiB: number
};

/**
 Everything a container run needs, as the strings podman and cargo take.
 */
export type ContainerBounds = {
  readonly cpus: string;
  readonly memory: string;
  readonly pids: string;
  readonly openFiles: string;
  readonly jobs: string;
  readonly testThreads: string;
  /**
   Arguments of `podman run` that bound the container, in the order the tasks always passed them.
   */
  readonly limitArguments: readonly string[];
  /**
   `--env` pairs for cargo and the test runner, sized to the CPU limit.
   */
  readonly environmentArguments: readonly string[];
  /**
   Arguments of `podman build` that bound the image build to the same CPU and memory.
   */
  readonly imageBuildArguments: readonly string[];
  /**
   One line naming the values, for task output.
   */
  readonly summary: string;
};

/**
 The host this process runs on.
 */
export function currentHost(): Host { return {
  cpus: availableParallelism(),
  memoryBytes: totalmem()
}; }

/**
 CPU count and memory size that are {@link hostShare} of a host, rounded down.
 
 @param host - Maximum CPU count and memory of the machine.
 
 @returns At least one CPU and {@link minimumMemoryGiB} GiB, so a small host still gets a working container.
 */
export function hostBound(host: Host): HostBound { return {
  cpus: Math.max(
    1,
    Math.floor(hostShare * host.cpus)
  ),
  memoryGiB: Math.max(
    minimumMemoryGiB,
    Math.floor((hostShare * host.memoryBytes) / (2
      ** 30))
  ),
}; }

/**
 Process limit for a CPU count: the floor, or {@link pidsPerCpu} per CPU when that is more.
 */
export function pidsFor(cpus: number): number { return Math.max(
  minimumPids,
  Math.floor(cpus) * pidsPerCpu
); }

const wholeNumber = /^[1-9][0-9]{0,8}$/;
const cpuForm = /^(?:[1-9][0-9]{0,3}|0)(?:\.[0-9]{1,2})?$/;
const memoryForm = /^[1-9][0-9]{0,8}[kmgKMG]$/;

/**
 Value of a variable, or undefined when it is unset or empty.
 
 @throws {ContainerBoundError} When the value does not match the accepted form.
 */
function readVariable({
  environment,
  name,
  form,
  accepted
}: {
  environment: Readonly<Record<string, string | undefined>>;
  name: string;
  form: RegExp;
  accepted: string;
}): string | undefined {
  const value = environment[name];
  if ((value === undefined) || (value === '')) {
    return undefined;
  }
  if (!form.test(value)) {
    throw new ContainerBoundError(`${name} is "${value}", which is not accepted. Use ${accepted}, or leave it unset.`);
  }
  return value;
}

/**
 Bound of one container run.
 
 Unset variables take the computed share of the host. CARGO_BUILD_JOBS and NEXTEST_TEST_THREADS follow the CPU
 limit (whole CPUs, at least one), because inside the container `nproc` and the runner's default report the host's
 CPUs, not the quota the cgroup really gives.
 
 @param options - `environment` holds the caller's variables, `host` the machine the default is computed from.
 
 @returns The strings to pass on, plus a one-line summary.
 
 @throws {ContainerBoundError} When a variable is set to something malformed; the message names the variable and the
 accepted form.
 */
export function containerBounds(
  options: {
    environment?: Readonly<Record<string, string | undefined>>;
    host?: Host
  } = {},
): ContainerBounds {
  const {
    environment = process.env,
    host = currentHost()
  } = options;
  const share = hostBound(host);
  const cpus = readVariable({
    environment,
    name: variableNames.cpus,
    form: cpuForm,
    accepted: 'a positive number of CPUs with at most two decimals, such as 4 or 1.5',
  }) ?? String(share.cpus);
  if (!(Number(cpus) > 0)) {
    throw new ContainerBoundError(`${variableNames.cpus} is "${cpus}", which is not accepted. Use a number above zero, or leave it unset.`);
  }
  const memory = readVariable({
    environment,
    name: variableNames.memory,
    form: memoryForm,
    accepted: 'a whole number with the unit k, m, or g, such as 512m or 8g',
  }) ?? `${share.memoryGiB}g`;
  const wholeCpus = Math.max(
    1,
    Math.floor(Number(cpus))
  );
  function count({
    name,
    fallback
  }: {
    name: string;
    fallback: number
  }): string { return readVariable({
    environment,
    name,
    form: wholeNumber,
    accepted: 'a positive whole number',
  }) ?? String(fallback); }
  const pids = count({
    name: variableNames.pids,
    fallback: pidsFor(wholeCpus)
  });
  const openFiles = count({
    name: variableNames.openFiles,
    fallback: defaultOpenFiles
  });
  const jobs = count({
    name: variableNames.jobs,
    fallback: wholeCpus
  });
  const testThreads = count({
    name: variableNames.testThreads,
    fallback: wholeCpus
  });
  return {
    cpus,
    memory,
    pids,
    openFiles,
    jobs,
    testThreads,
    limitArguments: [
      `--memory=${memory}`,
      `--cpus=${cpus}`,
      `--pids-limit=${pids}`,
      '--ulimit',
      `nofile=${openFiles}:${openFiles}`
    ],
    environmentArguments: [
      '--env',
      `CARGO_BUILD_JOBS=${jobs}`,
      '--env',
      `NEXTEST_TEST_THREADS=${testThreads}`
    ],
    imageBuildArguments: [
      `--memory=${memory}`,
      `--cpu-period=${cpuPeriod}`,
      `--cpu-quota=${Math.round(Number(cpus) * cpuPeriod)}`
    ],
    summary: `Container bound: ${cpus} CPUs, ${memory} memory, ${pids} processes, ${openFiles} open files, ${jobs} cargo jobs, ${testThreads} test threads`,
  };
}
