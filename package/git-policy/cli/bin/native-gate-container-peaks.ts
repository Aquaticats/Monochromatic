#!/usr/bin/env node
/**
 Read one gate run's two containers while they run, and write what the kernel says about each as one JSON line per
 sample: the whole of `memory.current`, `memory.peak`, `memory.max`, `memory.stat` and `memory.events`, the process
 figures, and the tests holding the most memory at that moment.

 The gate's containers are `--rm`, so each one's cgroup exists only while the container does, and its figures are gone
 with it. This reader is what a measurement of a gate's own containers needs: it finds the run's evidence directory in
 the gate's log, reads each role's container ID from the file that run wrote, and polls that container's own
 `libpod-*.scope` until the scope goes, which it marks with one last line holding the figures the scope last gave.

 The tests are named from the processes themselves. nextest runs every test as one process of the test binary started
 with `--exact <test name>`, so the process with the largest charge at the peak is the test that held it.

 Usage:
   node bin/native-gate-container-peaks.ts --log <gate log> --out <jsonl> [--interval-ms 100]
     [--process-interval-ms 1000] [--deadline-seconds 5400]
 */
import {
  closeSync,
  existsSync,
  openSync,
  readFileSync,
  readSync,
  statSync,
} from 'node:fs';
import { appendFile } from 'node:fs/promises';
import { basename, join } from 'node:path';

import { wait as waitUntil } from '@monochromatic-dev/async-time-core/ts';

import type { GateContainerRole } from './native-gate-container-identity.ts';
import {
  type ContainerMemorySample,
  cgroupRoot,
  containerCgroupPathOf,
  gateMemoryPollMilliseconds,
  readContainerMemory,
} from './native-gate-container-memory.ts';

//region Finding the run a log belongs to

/**
 The line `bin/test-native-container.ts` prints to name the evidence directory of its run, whose files name its
 containers.
 */
export const gateEvidenceLinePrefix = 'Native wrapper verification evidence: ';

/**
 The containers every gate run starts, in the order its evidence lists them.
 */
export const gatePeakRoles: readonly GateContainerRole[] = [
  'suite',
  'clippy',
];

/**
 The evidence directory a gate log names.

 @param logText - the text the gate printed so far

 @returns the absolute path of the run's evidence directory, or nothing while the log holds no such line yet

 @example
 ```ts
 evidenceDirectoryOf({ logText: 'Native wrapper verification evidence: /w/target/verification/native-HZPg8O' });
 ```
 */
export function evidenceDirectoryOf({ logText }: { readonly logText: string; },): string | undefined {
  for (const line of logText.split('\n',))
    if (line.startsWith(gateEvidenceLinePrefix,))
      return line.slice(gateEvidenceLinePrefix.length,)
        .trim();
  return undefined;
}

/**
 The text a log gained since it was last read, and where the next read starts.

 @param logPath - the log

 @param fromByte - the offset the last read ended at

 @returns the appended text and the new offset, both empty and unchanged when the log is not there or holds nothing new
 */
function appendedText({
  logPath,
  fromByte,
}: {
  readonly logPath: string;
  readonly fromByte: number;
},): { readonly text: string; readonly fromByte: number; } {
  try {
    /**
     How large the log is now, which is where a read of it ends.
     */
    const size = statSync(logPath,)
      .size;
    if (size <= fromByte)
      return {
        text: '',
        fromByte: Math.min(
          fromByte,
          size,
        ),
      };
    /**
     The bytes appended since the last read.
     */
    const buffer = Buffer.alloc(size - fromByte,);
    /**
     The log, held open only for this read.
     */
    const handle = openSync(
      logPath,
      'r',
    );
    try {
      readSync(
        handle,
        buffer,
        0,
        buffer.length,
        fromByte,
      );
    }
    finally {
      closeSync(handle,);
    }
    return {
      text: buffer.toString('utf8',),
      fromByte: size,
    };
  }
  catch {
    // A log not written yet, or one this account cannot read, is no reading at all and not a failure of the gate.
    return {
      text: '',
      fromByte,
    };
  }
}

//endregion Finding the run a log belongs to

//region What one container's processes say

/**
 One process of a container, as this reader names it and charges it.
 */
export type ContainerProcess = {
  /**
   The process number on this host, as the container's own cgroup lists it.
   */
  readonly pid: number;
  /**
   Its resident set size.
   */
  readonly rssBytes: number;
  /**
   What it is, which for a test process is the test it runs.
   */
  readonly label: string;
};

/**
 The label of one process, from its own argument array.

 @param argv - the process's arguments, the program first

 @returns the test name a nextest process was started for, otherwise the program's name and its first argument, and
   nothing for a process whose argument array is empty

 @example
 ```ts
 processLabelOf({ argv: ['/w/git_policy_cli-abc', '--exact', 'mod::tests::name', '--nocapture'] });
 ```
 */
export function processLabelOf({ argv }: { readonly argv: readonly string[]; },): string | undefined {
  /**
   Where the exact filter nextest starts a test with sits in the argument array.
   */
  const exact = argv.indexOf('--exact',);
  if (exact !== (-1))
    return argv[exact + 1];
  /**
   The program, which names itself.
   */
  const program = argv[0];
  if (program === undefined)
    return undefined;
  /**
   The program's first argument, which tells its own subcommand from another.
   */
  const first = argv[1];
  return first === undefined
    ? basename(program,)
    : `${basename(program,)} ${first}`;
}

/**
 One number of a process's status file, which is where its resident set size is.

 @param status - the text of `/proc/<pid>/status`

 @param name - the field's name, as in `VmRSS`

 @returns the number of bytes, or nothing when the file holds no such field
 */
function statusField({ status, name }: {
  readonly status: string;
  readonly name: string;
},): number | undefined {
  for (const line of status.split('\n',)) {
    /**
     The field's name and its value, with the unit the kernel writes beside it.
     */
    const [field, value, unit, ...surplus] = line.trim()
      .split(/\s+/,);
    if ((field !== `${name}:`) || (value === undefined) || (surplus.length > 0))
      continue;
    if (![...value,].every(function isDigit(character,) {
      return (character >= '0') && (character <= '9');
    },))
      return undefined;
    // The status file states its memory in kibibytes; every figure this reader writes is in bytes.
    return unit === 'kB' ? Number(value,) * 1_024 : Number(value,);
  }
  return undefined;
}

/**
 The processes of one container, heaviest first.

 @param cgroupPath - the container's own scope

 @param count - how many of the heaviest to answer with

 @returns one entry per process read, at most `count` of them, empty when the scope is gone
 */
export function containerProcesses({
  cgroupPath,
  count,
}: {
  readonly cgroupPath: string;
  readonly count: number;
},): readonly ContainerProcess[] {
  /**
   Every process number the container's cgroup lists.
   */
  const pids: string[] = readFileSync(
    join(
      cgroupPath,
      'cgroup.procs',
    ),
    'utf8',
  )
    .split('\n',)
    .filter(function listed(line,) {
      return line.trim() !== '';
    },);
  /**
   The processes read so far.
   */
  const processes: ContainerProcess[] = [];
  for (const pid of pids) {
    try {
      /**
       What the process's own files say about it.
       */
      const argv = readFileSync(
        join(
          '/proc',
          pid,
          'cmdline',
        ),
        'utf8',
      )
        .split('\0',)
        .filter(function named(argument,) {
          return argument !== '';
        },);
      /**
       What this reader calls the process.
       */
      const label = processLabelOf({ argv, },);
      /**
       The memory it holds now.
       */
      const rssBytes = statusField({
        status: readFileSync(
          join(
            '/proc',
            pid,
            'status',
          ),
          'utf8',
        ),
        name: 'VmRSS',
      },);
      if ((label === undefined) || (rssBytes === undefined))
        continue;
      processes.push({
        pid: Number(pid,),
        rssBytes,
        label,
      },);
    }
    catch {
      // A process that ended between the cgroup's listing and this read is one this sample need not name.
      continue;
    }
  }
  return processes.sort(function heaviestFirst(left, right,) {
    return right.rssBytes - left.rssBytes;
  },)
    .slice(
      0,
      count,
    );
}

//endregion What one container's processes say

//region Recording one run's peaks

/**
 Milliseconds between two reads of a container's processes, which costs one read of two files per process and so is
 slower than the memory figures beside it.
 */
export const gateProcessPollMilliseconds = 1_000;

/**
 How many of a container's heaviest processes one sample names.
 */
const namedProcessesPerSample = 8;

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
   The roles it was still watching when its deadline came.
   */
  readonly rolesStillRunning: readonly GateContainerRole[];
};

/**
 The ID one role's container wrote to its own evidence file.

 @param evidence - the run's evidence directory, absent while the log names none

 @param role - the container whose ID file to read

 @returns the ID without the newline podman writes, or nothing when there is no file to read yet
 */
function readContainerId({
  evidence,
  role,
}: {
  readonly evidence: string | undefined;
  readonly role: GateContainerRole;
},): string | undefined {
  if (evidence === undefined)
    return undefined;
  /**
   The file the container's client wrote its ID to.
   */
  const path = join(
    evidence,
    `${role}.cid`,
  );
  if (!existsSync(path,))
    return undefined;
  try {
    /**
     The ID as the file holds it.
     */
    const id = readFileSync(
      path,
      'utf8',
    ).trim();
    return id === '' ? undefined : id;
  }
  catch {
    // An ID file that is there and cannot be read is no reading of this run, which the log's own lines still name.
    return undefined;
  }
}

/**
 Record one gate run's containers while they run.

 @param logPath - the gate's log, where the run's evidence directory is named

 @param outPath - the file one JSON line per sample is appended to

 @param root - the cgroup hierarchy's root, which a test points at a fixture tree

 @param userId - the account whose user manager runs the containers

 @param intervalMilliseconds - how long between two samples of one container

 @param processIntervalMilliseconds - how long between two reads of a container's processes

 @param deadlineMilliseconds - when this stops even if a container is still running

 @param roles - the containers to watch, which defaults to both of a gate's

 @returns how many samples were written, which roles answered, and which were still running at the deadline

 @example
 ```ts
 const outcome = await recordGateContainerPeaks({ logPath, outPath, deadlineMilliseconds: 5_400_000 });
 ```
 */
export async function recordGateContainerPeaks({
  logPath,
  outPath,
  root = cgroupRoot,
  userId = process.getuid?.() ?? 0,
  intervalMilliseconds = gateMemoryPollMilliseconds,
  processIntervalMilliseconds = gateProcessPollMilliseconds,
  deadlineMilliseconds,
  roles = gatePeakRoles,
}: {
  readonly logPath: string;
  readonly outPath: string;
  readonly root?: string;
  readonly userId?: number;
  readonly intervalMilliseconds?: number;
  readonly processIntervalMilliseconds?: number;
  readonly deadlineMilliseconds: number;
  readonly roles?: readonly GateContainerRole[];
},): Promise<GatePeakOutcome> {
  /**
   When this stops even if a container still runs.
   */
  const deadline = Date.now() + deadlineMilliseconds;
  /**
   Where the log has been read up to, so every poll reads only what the gate appended since.
   */
  let logRead = 0;
  /**
   The evidence directory the log named, once it names one.
   */
  let evidence: string | undefined;
  /**
   The last line the log held that no newline ended yet, so a line split across two reads is still read whole.
   */
  let carried = '';
  /**
   The ID of each role's container, which its ID file names.
   */
  const ids = new Map<GateContainerRole, string>();
  /**
   The last figures each role's scope gave, which is what its end line repeats.
   */
  const lastSamples = new Map<GateContainerRole, ContainerMemorySample>();
  /**
   The roles whose scope was read at least once and has gone since.
   */
  const ended = new Set<GateContainerRole>();
  /**
   The last moment a container's processes were read.
   */
  let processesReadAt = 0;
  /**
   How many sample lines were written.
   */
  let samples = 0;

  /**
   The heaviest processes of one running container, read at most once per process interval.

   @param role - the container to read

   @returns its heaviest processes, or none when they were read within the interval already
   */
  function containerProcessesOf(role: GateContainerRole,): readonly ContainerProcess[] {
    if ((Date.now() - processesReadAt) < processIntervalMilliseconds)
      return [];
    processesReadAt = Date.now();
    /**
     The container's own scope.
     */
    const cgroupPath = containerCgroupPathOf({
      containerId: ids.get(role,) ?? '',
      root,
      userId,
    },);
    try {
      return containerProcesses({
        cgroupPath,
        count: namedProcessesPerSample,
      },);
    }
    catch {
      // A scope that went between the sample and this read leaves nothing to name, which the sample itself says.
      return [];
    }
  }

  /**
   Write one line for one role, as the sample it is of.

   @param role - the container the line is about

   @param sample - the figures the line carries

   @param end - whether this is the last line that role's scope gives

   @returns nothing, once the line is on disk
   */
  async function writeLine(role: GateContainerRole, sample: ContainerMemorySample, end: boolean,): Promise<void> {
    await appendFile(
      outPath,
      `${JSON.stringify({
        role,
        end,
        ...sample,
        processes: end
          ? []
          : containerProcessesOf(role,),
      })}\n`,
    );
  }

  for (;;) {
    if (evidence === undefined) {
      /**
       What the gate appended to its log since the last look.
       */
      const appended = appendedText({
        logPath,
        fromByte: logRead,
      },);
      logRead = appended.fromByte;
      /**
       The text searched for the evidence line: what was carried from the last read, and what the gate appended.
       */
      const searched = `${carried}${appended.text}`;
      evidence = evidenceDirectoryOf({ logText: searched, },);
      // Only the trailing partial line is carried, and a line longer than a few kilobytes is not an evidence line.
      carried = searched.slice(searched.lastIndexOf('\n',) + 1,);
      if (carried.length > 4_096)
        carried = '';
    }
    for (const role of roles) {
      if (ended.has(role,))
        continue;
      /**
       The role's container ID, read from the file the gate's run wrote it to.
       */
      const id = ids.get(role,) ?? readContainerId({ evidence, role },);
      if (id === undefined)
        continue;
      ids.set(
        role,
        id,
      );
      /**
       The container's own scope.
       */
      const cgroupPath = containerCgroupPathOf({
        containerId: id,
        root,
        userId,
      },);
      /**
       What that scope holds now.
       */
      const sample = readContainerMemory({ cgroupPath, },);
      if (sample !== undefined) {
        lastSamples.set(
          role,
          sample,
        );
        // oxlint-disable-next-line eslint/no-await-in-loop -- one sample per interval is the recording itself
        await writeLine(
          role,
          sample,
          false,
        );
        samples += 1;
        continue;
      }
      /**
       The last figures this role's scope gave, if it ever did.
       */
      const last = lastSamples.get(role,);
      if (last === undefined)
        continue;
      // The scope is gone, so the sample above is the last moment the kernel could be asked; the end line repeats it
      // so a reader needs no search for a role's final figures.
      // oxlint-disable-next-line eslint/no-await-in-loop -- the end line is written once, at the moment the scope went
      await writeLine(
        role,
        last,
        true,
      );
      ended.add(role,);
    }
    if (ended.size >= roles.length)
      break;
    if (Date.now() > deadline)
      break;
    // oxlint-disable-next-line eslint/no-await-in-loop -- the poll interval is the recording itself
    await waitUntil(intervalMilliseconds,);
  }
  return {
    samples,
    rolesRead: roles.filter(function read(role,) {
      return lastSamples.has(role,);
    },),
    rolesStillRunning: roles.filter(function stillRunning(role,) {
      return !ended.has(role,);
    },),
  };
}

//endregion Recording one run's peaks

//region The command line

/**
 How the command is called, printed by a refusal that reads the command line.
 */
const usage = 'usage: native-gate-container-peaks --log <gate log> --out <jsonl> [--interval-ms <count>] [--process-interval-ms <count>] [--deadline-seconds <count>]';

/**
 Read one option's value from the command line.

 @param name - the option

 @returns its value, or nothing when it was not given
 */
function optionValue(name: string,): string | undefined {
  /**
   Where the option sits in the command line.
   */
  const at = process.argv.indexOf(name,);
  return at === (-1) ? undefined : process.argv[at + 1];
}

/**
 Record one gate run's containers and print what it wrote.

 @returns nothing; the JSON lines in the file are the output
 */
async function main(): Promise<void> {
  /**
   The files this reads and writes.
   */
  const logPath = optionValue('--log',);
  const outPath = optionValue('--out',);
  if ((logPath === undefined) || (outPath === undefined))
    throw new Error(usage,);
  /**
   How often each figure is read and how long this may run, in milliseconds.
   */
  const seconds = Number(optionValue('--deadline-seconds',) ?? 5_400,);
  /**
   What the recording found.
   */
  const outcome = await recordGateContainerPeaks({
    logPath,
    outPath,
    intervalMilliseconds: Number(optionValue('--interval-ms',) ?? gateMemoryPollMilliseconds,),
    processIntervalMilliseconds: Number(optionValue('--process-interval-ms',) ?? gateProcessPollMilliseconds,),
    deadlineMilliseconds: seconds * 1_000,
  });
  console.log(`${String(outcome.samples)} samples of ${outcome.rolesRead.join(', ') || 'no container'} written to ${outPath}; still running at the deadline: ${outcome.rolesStillRunning.join(', ') || 'none'}`,);
}

if (import.meta.main)
  await main();

//endregion The command line
