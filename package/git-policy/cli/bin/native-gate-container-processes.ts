/**
 The processes of one running gate container, named by what they are and charged by their resident set size.

 A sample of a container's memory says how much is charged and what it is made of; this module says who holds it. nextest
 runs every test as one process of the test binary started with `--exact <test name>`, so the heaviest processes of a
 sample name the tests that hold that sample's peak (measured 2026-11-27 in the gate's own suite container, where a
 sample's processes read
 `/work/.../out/git_policy_cli-85367b807f819bb5 --exact repo_read::repo_read_config_typed::tests::an_empty_value_over_every_documented_key_declines_or_equals_git --nocapture`).
 A fixture's own programs, such as its Git, are named by their program and first argument instead.
 */
import { readFile, } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import { isMissing, } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts/fs-guards.ts';

import { isWholeNumberText, } from './native-gate-arguments.ts';
import {
  FIGURE_NOT_WRITTEN,
  fieldsOf,
} from './native-gate-container-cgroup-memory.ts';

/**
 Milliseconds between two reads of a container's processes, which costs two file reads per process and so is slower than
 the memory figures beside it.
 */
export const gateProcessPollMilliseconds = 1_000;

/**
 Bytes in one kibibyte, which is the unit a process's status file states its memory in.
 */
const bytesPerKibibyte = 1_024;

/**
 The statement that a process's own files name nothing to call it by, which a sentinel carries because this repository
 models absence without nullish unions.
 */
export const NO_PROCESS_LABEL: unique symbol = Symbol('process whose own files name nothing to call it by',);

/**
 The statement that a process's files are gone, which a process that ended between two reads leaves behind.
 */
export const NO_PROCESS_READ: unique symbol = Symbol('process whose own files are gone to read',);

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
   {@link NO_PROCESS_LABEL} for a process whose argument array is empty

 @example
 ```ts
 processLabelOf({ argv: ['/w/git_policy_cli-abc', '--exact', 'mod::tests::name', '--nocapture'] });
 ```
 */
export function processLabelOf({ argv }: { readonly argv: readonly string[]; },): string | typeof NO_PROCESS_LABEL {
  /**
   Where the exact filter nextest starts a test with sits in the argument array.
   */
  const exact = argv.indexOf('--exact',);
  if (exact !== (-1))
    return argv.at(exact + 1,) ?? NO_PROCESS_LABEL;
  /**
   The program, which names itself, and its first argument, which tells its own subcommand from another.
   */
  const [program, first,] = argv;
  if (program === undefined)
    return NO_PROCESS_LABEL;
  return first === undefined
    ? basename(program,)
    : `${basename(program,)} ${first}`;
}

/**
 One field of a process's status file, which is where its resident set size is.

 @param status - the text of `/proc/<pid>/status`

 @param name - the field's name, as in `VmRSS`

 @returns the number of bytes, or nothing when the file holds no such field
 */
function statusField({
  status,
  name,
}: {
  readonly status: string;
  readonly name: string;
},): number | typeof FIGURE_NOT_WRITTEN {
  for (const line of status.split('\n',)) {
    /**
     The two fields a status line holds, and the unit the kernel writes beside its number.
     */
    const [field, value, unit, ...surplus] = fieldsOf(line,);
    if ((field !== `${name}:`) || (value === undefined)
      || (surplus.length > 0)
      || (!isWholeNumberText(value,)))
      continue;
    return unit === 'kB' ? Number(value,) * bytesPerKibibyte : Number(value,);
  }
  return FIGURE_NOT_WRITTEN;
}

/**
 Read one process's files, and answer nothing when it ended between the cgroup's listing and this read.

 @param pid - the process number

 @returns what this reader calls the process and the memory it holds, or nothing when its files are gone

 @throws the error of a process whose files are there and cannot be read, which is a host this reader cannot account for
 */
async function processOf(pid: string,): Promise<
  | {
    readonly label: string;
    readonly rssBytes: number;
  }
    | typeof NO_PROCESS_READ
  > {
  try {
    /**
     The process's own arguments, which name the test it runs.
     */
    const argv = (await readFile(
      join(
        '/proc',
        pid,
        'cmdline',
      ),
      'utf8',
    ))
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
      status: await readFile(
        join(
          '/proc',
          pid,
          'status',
        ),
        'utf8',
      ),
      name: 'VmRSS',
    },);
    if (((typeof label) === 'symbol') || ((typeof rssBytes) === 'symbol'))
      return NO_PROCESS_READ;
    return {
      label,
      rssBytes,
    };
  }
  catch (error) {
    // A process that ended between the cgroup's listing and this read is one this sample need not name.
    if (isMissing(error,))
      return NO_PROCESS_READ;
    throw error;
  }
}

/**
 The processes of one container, heaviest first.

 @param cgroupPath - the container's own scope

 @param count - how many of the heaviest to answer with

 @returns one entry per process read, the heaviest `count` of them, empty when the scope is gone

 @example
 ```ts
 const heaviest = await containerProcesses({ cgroupPath, count: 8 });
 ```
 */
export async function containerProcesses({
  cgroupPath,
  count,
}: {
  readonly cgroupPath: string;
  readonly count: number;
},): Promise<readonly ContainerProcess[]> {
  /**
   Every process number the container's cgroup lists.
   */
  const pids = (await readFile(
    join(
      cgroupPath,
      'cgroup.procs',
    ),
    'utf8',
  ))
    .split('\n',)
    .filter(function listed(line,) {
      return line.trim() !== '';
    },);
  /**
   The processes read so far.
   */
  const processes: ContainerProcess[] = [];
  for (const pid of pids) {
    /**
     What this reader could read of one process.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- one read per process of the container is the sample itself
    const process = await processOf(pid,);
    if (process === NO_PROCESS_READ)
      continue;
    processes.push({
      pid: Number(pid,),
      rssBytes: process.rssBytes,
      label: process.label,
    },);
  }
  return processes.toSorted(function heaviestFirst(
    left,
    right,
  ) {
    return right.rssBytes - left.rssBytes;
  },)
    .slice(
    0,
    count,
    );
}
