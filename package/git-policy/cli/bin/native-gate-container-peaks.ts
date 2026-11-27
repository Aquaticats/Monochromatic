#!/usr/bin/env node
/**
 Record one gate run's two containers while they run, as one JSON line per sample: the whole of `memory.current`,
 `memory.peak`, `memory.max`, `memory.stat` and `memory.events`, the process figures, and the tests holding the most
 memory at that moment.

 The gate's containers are `--rm`, so each one's cgroup exists only while the container does, and its figures are gone
 with it. This is the reader a measurement of the gate's own containers runs: it finds the run's evidence directory in
 the gate's log, reads each role's container ID from the file that run wrote, and polls that container's own
 `libpod-*.scope` until the scope goes, which it marks with one last line holding the figures the scope last gave. The
 tests are named from the processes themselves, by `bin/native-gate-container-processes.ts`.

 Usage:
   node bin/native-gate-container-peaks.ts --log <gate log> --out <jsonl> [--interval-ms 100]
     [--process-interval-ms 1000] [--deadline-seconds 5400]
 */
import { appendFile, } from 'node:fs/promises';

import { wait as waitUntil, } from '@monochromatic-dev/async-time-core/ts';
import { isMissing, } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import type { GateContainerRole, } from './native-gate-container-identity.ts';
import {
  type ContainerMemoryRead,
  type ContainerMemorySample,
  CGROUP_NOT_THERE,
  containerCgroupPathOf,
  readContainerMemory,
} from './native-gate-container-cgroup-memory.ts';
import { gateMemoryPollMilliseconds, } from './native-gate-container-memory.ts';
import {
  type ContainerProcess,
  containerProcesses,
  gateProcessPollMilliseconds,
} from './native-gate-container-processes.ts';
import {
  type AppendedLogText,
  NO_CONTAINER_ID,
  NO_EVIDENCE_DIRECTORY,
  appendedLogText,
  evidenceDirectoryOf,
  gatePeakRoles,
  readContainerId,
} from './native-gate-evidence-directory.ts';

/**
 How many of a container's heaviest processes one sample names.
 */
const namedProcessesPerSample = 8;

/**
 Longest partial log line this reader carries from one read to the next, so a line split across two reads is still read
 whole. An evidence line is shorter than this; a longer line is some other output and is not carried.
 */
const carriedLineLimit = 4_096;

/**
 What one run of this reader found.
 */
export type GatePeakOutcome = {
  /**
   How many sample lines it wrote.
   */
  readonly samples: number;
  /**
   The roles whose cgroup gave at least one sample.
   */
  readonly rolesRead: readonly GateContainerRole[];
  /**
   The roles it was still watching when its deadline came, which includes one that had written no ID yet.
   */
  readonly rolesStillRunning: readonly GateContainerRole[];
};

/**
 Everything one run of this reader keeps between two polls.
 */
type PeakState = {
  /**
   Where the log has been read up to.
   */
  logRead: number;
  /**
   The evidence directory the log named, or {@link NO_EVIDENCE_DIRECTORY}.
   */
  evidence: string | typeof NO_EVIDENCE_DIRECTORY;
  /**
   The last line the log held that no newline ended yet.
   */
  carried: string;
  /**
   The ID of each role's container, by role.
   */
  readonly ids: Map<GateContainerRole, string>;
  /**
   The last figures each role's scope gave, which its end line repeats.
   */
  readonly lastSamples: Map<GateContainerRole, ContainerMemorySample>;
  /**
   The roles whose scope was read at least once and has gone since.
   */
  readonly ended: Set<GateContainerRole>;
  /**
   The last moment each role's processes were read, so one container's sample never spends the other's interval.
   */
  readonly processesReadAt: Map<GateContainerRole, number>;
  /**
   How many sample lines were written.
   */
  samples: number;
};

/**
 What one run of this reader is told.
 */
export type GatePeakRecording = {
  /**
   The gate's log, where the run's evidence directory is named.
   */
  readonly logPath: string;
  /**
   The file one JSON line per sample is appended to.
   */
  readonly outPath: string;
  /**
   The cgroup hierarchy's root, which a test points at a fixture tree.
   */
  readonly root?: string;
  /**
   The account whose user manager runs the containers.
   */
  readonly userId?: number;
  /**
   How long between two samples of one container.
   */
  readonly intervalMilliseconds?: number;
  /**
   How long between two reads of a container's processes.
   */
  readonly processIntervalMilliseconds?: number;
  /**
   When this stops even if a container is still running.
   */
  readonly deadlineMilliseconds: number;
  /**
   The containers to watch, which defaults to both of a gate's.
   */
  readonly roles?: readonly GateContainerRole[];
};

/**
 The part of a recording that names the processes of a container.
 */
export type PeakProcessRecording = {
  /**
   The cgroup hierarchy's root, which a test points at a fixture tree.
   */
  readonly root: string;
  /**
   The account whose user manager runs the containers.
   */
  readonly userId: number;
  /**
   How long between two reads of a container's processes.
   */
  readonly processIntervalMilliseconds: number;
};

/**
 The heaviest processes of one running container, read at most once per process interval.

 @param state - what this run has read so far

 @param role - the container to read

 @param recording - the part of the recording that names processes

 @returns its heaviest processes, or none when they were read within the interval already or its scope has gone
 */
async function processesOfRole({
  state,
  role,
  recording,
}: {
  readonly state: PeakState;
  readonly role: GateContainerRole;
  readonly recording: PeakProcessRecording;
},): Promise<readonly ContainerProcess[]> {
  if ((Date.now() - (state.processesReadAt
    .get(role,)
    ?? 0)) < recording.processIntervalMilliseconds)
    return [];
  state.processesReadAt
    .set(
    role,
    Date.now(),
    );
  /**
   The container's own scope.
   */
  const cgroupPath = containerCgroupPathOf({
    containerId: state.ids
      .get(role,)
      ?? '',
    root: recording.root,
    userId: recording.userId,
  },);
  try {
    return await containerProcesses({
      cgroupPath,
      count: namedProcessesPerSample,
    },);
  }
  catch (error) {
    // A scope that went between the sample and this read leaves nothing to name, which the sample itself says.
    if (isMissing(error,))
      return [];
    throw error;
  }
}

/**
 Write one line for one role.

 @param state - what this run has read so far

 @param recording - what this run was told

 @param role - the container the line is about

 @param sample - the figures the line carries

 @param end - whether this is the last line that role's scope gives
 */
async function writeLine({
  state,
  recording,
  role,
  sample,
  end,
}: {
  readonly state: PeakState;
  readonly recording: GatePeakRecording;
  readonly role: GateContainerRole;
  readonly sample: ContainerMemorySample;
  readonly end: boolean;
},): Promise<void> {
  /**
   The processes to name beside the figures, which an end line does not need.
   */
  const processes = end
    ? []
    : await processesOfRole({
      state,
      role,
      recording: {
        root: recording.root ?? '',
        userId: recording.userId ?? 0,
        processIntervalMilliseconds: recording.processIntervalMilliseconds ?? gateProcessPollMilliseconds,
      },
    },);
  await appendFile(
    recording.outPath,
    `${JSON.stringify({
      role,
      end,
      ...sample,
      processes,
    })}\n`,
  );
}

/**
 Read one role's container once: its ID, its figures, and the line this sample becomes.

 @param state - what this run has read so far

 @param recording - what this run was told

 @param role - the container to read
 */
async function readRoleOnce({
  state,
  recording,
  role,
}: {
  readonly state: PeakState;
  readonly recording: GatePeakRecording;
  readonly role: GateContainerRole;
},): Promise<void> {
  if (state.ended
    .has(role,))
    return;
  /**
   The role's container ID, or the statement that it has written none yet.
   */
  const read = await readContainerId({
    evidence: state.evidence,
    role,
  },);
  if (read !== NO_CONTAINER_ID)
    state.ids
      .set(
      role,
      read,
      );
  /**
   The ID this role's container carries now.
   */
  const id = state.ids
    .get(role,);
  if (id === undefined)
    return;
  /**
   The container's own scope.
   */
  const cgroupPath = containerCgroupPathOf({
    containerId: id,
    ...(recording.root === undefined ? {} : { root: recording.root, }),
    ...(recording.userId === undefined ? {} : { userId: recording.userId, }),
  },);
  /**
   What that scope holds now.
   */
  const sample: ContainerMemoryRead = await readContainerMemory({ cgroupPath, },);
  if (sample !== CGROUP_NOT_THERE) {
    state.lastSamples
      .set(
      role,
      sample,
      );
    await writeLine({
      state,
      recording,
      role,
      sample,
      end: false,
    },);
    state.samples += 1;
    return;
  }
  /**
   The last figures this role's scope gave, if it ever did.
   */
  const last = state.lastSamples
    .get(role,);
  if (last === undefined)
    return;
  // The scope is gone, so the sample above is the last moment the kernel could be asked; the end line repeats it, so a
  // reader needs no search for a role's final figures.
  await writeLine({
    state,
    recording,
    role,
    sample: last,
    end: true,
  },);
  state.ended
    .add(role,);
}

/**
 Read every role's container once, and wait one interval before the next poll.

 @param state - what this run has read so far

 @param recording - what this run was told

 @param roles - the containers to read
 */
async function pollOnce({
  state,
  recording,
  roles,
}: {
  readonly state: PeakState;
  readonly recording: GatePeakRecording;
  readonly roles: readonly GateContainerRole[];
},): Promise<void> {
  if (state.evidence === NO_EVIDENCE_DIRECTORY) {
    /**
     What the gate appended to its log since the last look.
     */
    const appended: AppendedLogText = await appendedLogText({
      logPath: recording.logPath,
      fromByte: state.logRead,
    },);
    state.logRead = appended.fromByte;
    /**
     The text searched for the evidence line: what was carried from the last read, and what the gate appended.
     */
    const searched = `${state.carried}${appended.text}`;
    state.evidence = evidenceDirectoryOf({ logText: searched, },);
    state.carried = searched.slice(searched.lastIndexOf('\n',) + 1,);
    if (state.carried
      .length
      > carriedLineLimit)
      state.carried = '';
  }
  await Promise.all(roles.map(function each(role,) {
    return readRoleOnce({
      state,
      recording,
      role,
    },);
  },),);
  await waitUntil(recording.intervalMilliseconds ?? gateMemoryPollMilliseconds,);
}

/**
 Record one gate run's containers while they run.

 @param recording - the log to follow, the file to write and how often to look

 @returns how many samples were written, which roles answered, and which were still running at the deadline

 @example
 ```ts
 const outcome = await recordGateContainerPeaks({ logPath, outPath, deadlineMilliseconds: 5_400_000 });
 ```
 */
export async function recordGateContainerPeaks(recording: GatePeakRecording,): Promise<GatePeakOutcome> {
  /**
   When this stops even if a container still runs.
   */
  const deadline = Date.now() + recording.deadlineMilliseconds;
  /**
   The containers to watch.
   */
  const roles = recording.roles ?? gatePeakRoles;
  /**
   What this recording has read so far.
   */
  const state: PeakState = {
    logRead: 0,
    evidence: NO_EVIDENCE_DIRECTORY,
    carried: '',
    ids: new Map(),
    lastSamples: new Map(),
    ended: new Set(),
    processesReadAt: new Map(),
    samples: 0,
  };
  for (;;) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- the recording polls its containers once per interval
    await pollOnce({
      state,
      recording,
      roles,
    },);
    if (state.ended
      .size
      >= roles.length)
      break;
    if (Date.now() > deadline)
      break;
  }
  return {
    samples: state.samples,
    rolesRead: roles.filter(function read(role,) {
      return state.lastSamples
        .has(role,);
    },),
    rolesStillRunning: roles.filter(function stillRunning(role,) {
      return !state.ended
        .has(role,);
    },),
  };
}

//region The command line

/**
 How the command is called, printed by a refusal that reads the command line.
 */
const usage = 'usage: native-gate-container-peaks --log <gate log> --out <jsonl> [--interval-ms <count>] [--process-interval-ms <count>] [--deadline-seconds <count>]';

/**
 In numbers an option falls back to when the command line names it not at all.
 */
const defaultDeadlineSeconds = 5_400;

/**
 Milliseconds in one second, which a deadline given in seconds becomes.
 */
const millisecondsPerSecond = 1_000;

/**
 The statement that the command line gave no value for an option, which a sentinel carries because this repository
 models absence without nullish unions.
 */
export const NO_OPTION_VALUE: unique symbol = Symbol('command-line option the caller did not give',);

/**
 Read one option's value from the command line.

 @param name - the option

 @returns its value, or nothing when it was not given

 @example
 ```ts
 optionValue('--log');
 ```
 */
function optionValue(name: string,): string | typeof NO_OPTION_VALUE {
  /**
   Where the option sits in the command line.
   */
  const at = process.argv
    .indexOf(name,);
  return at === (-1) ? NO_OPTION_VALUE : (process.argv[at + 1] ?? NO_OPTION_VALUE);
}

/**
 Read one whole-number option, or the number this command falls back to.

 @param name - the option

 @param fallback - the number this command uses when the option is not given

 @returns the number the command line named, or the fallback

 @example
 ```ts
 numberOption({ name: '--interval-ms', fallback: gateMemoryPollMilliseconds });
 ```
 */
function numberOption({
  name,
  fallback,
}: {
  readonly name: string;
  readonly fallback: number;
},): number {
  /**
   The value the command line gave, when it gave one.
   */
  const given = optionValue(name,);
  return (typeof given) === 'symbol' ? fallback : Number(given,);
}

/**
 Record one gate run's containers and print what it wrote.

 @returns the exit status this command ends with, which is zero once the recording is written
 */
async function main(): Promise<number> {
  /**
   The gate log this follows.
   */
  const logPath = optionValue('--log',);
  /**
   The file one JSON line per sample is written to.
   */
  const outPath = optionValue('--out',);
  if (((typeof logPath) === 'symbol') || ((typeof outPath) === 'symbol'))
    throw new Error(usage,);
  /**
   What the recording found.
   */
  const outcome = await recordGateContainerPeaks({
    logPath,
    outPath,
    intervalMilliseconds: numberOption({
      name: '--interval-ms',
      fallback: gateMemoryPollMilliseconds,
    },),
    processIntervalMilliseconds: numberOption({
      name: '--process-interval-ms',
      fallback: gateProcessPollMilliseconds,
    },),
    deadlineMilliseconds: numberOption({
      name: '--deadline-seconds',
      fallback: defaultDeadlineSeconds,
    },) * millisecondsPerSecond,
  },);
  console.log(`${String(outcome.samples)} samples of ${outcome.rolesRead
    .join(', ')
    || 'no container'} written to ${outPath}; still running at the deadline: ${outcome.rolesStillRunning
      .join(', ')
      || 'none'}`,);
  return 0;
}

if (import.meta.main)
  process.exitCode = await main();

//endregion The command line
