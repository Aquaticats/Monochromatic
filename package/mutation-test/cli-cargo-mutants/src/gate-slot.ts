/*
 One lock stays in this file's gate, kept only for compatibility (the repository rule is lock-free first; a lock that must stay
 covers the smallest unit and says why). A slot is one of a fixed number of host-wide permits, and the limit exists to bound
 the CPU all campaign launchers use together. Each slot is its own numbered claim series (`slot-<n>.owner.<generation>`, see
 `src/owner-claim.ts`), so a gate takes a free slot with one exclusive `link` and never waits for a file; a gate waits only
 while every slot is held by a live process, which is the purpose of the limiter, and it wakes when a slot is released
 (`src/directory-watch.ts`). The unit of the one lock is a slot's `slot-<n>.lock` file. The earlier release held a kernel lock
 on it, and a gate of that release that is still waiting polls `flock --nonblock` on it without reading a claim, so a gate of
 this release keeps that lock for as long as its claim (taken once, without waiting, after the claim), and respects a lock a
 gate of the earlier release holds (`src/legacy-flock.ts`). Nothing waits on the lock; it goes away with the compatibility
 once no gate of the earlier release, nor a checkout of its source, can run.
 */
import {
  join,
  resolve,
} from 'node:path';

import {
  defaultNoticeIntervalMs,
  defaultPollIntervalMs,
  defaultSlotCount,
  lockDirectoryEnvironmentVariable,
  lockDirectorySegment,
  scratchSegments,
  slotCountEnvironmentVariable,
} from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import type { ForeignBorrowed } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { Environment } from './options.ts';
import { positiveWholeNumber } from './validate.ts';

/**
 Settings of one slot-limited run.
 */
export type GateSlotOptions = {
  readonly slotCount: number;
  readonly lockDirectory: string;
  readonly pollIntervalMs: number;
  readonly noticeIntervalMs: number;
  readonly command: readonly string[];
};

/**
 Usage text printed with every option error.
 */
export const gateSlotUsage: string = [
  'Usage: gate-slot.ts [--slot-count <count>] [--lock-dir <dir>] [--poll-interval-ms <ms>] [--notice-interval-ms <ms>] [--] <command> [arguments...]',
  '',
  'Takes one of the host-wide slots at once when one is free, otherwise waits until a holder releases one (or dies),',
  'runs the command in it at the lowest nice priority with inherited stdio, and exits with the command status.',
  'While every slot is held it writes a notice naming the holders, and repeats it at the notice interval.',
  'The poll interval only bounds how long the wait goes without looking again; a release wakes it at once.',
  `The slot count defaults to $${slotCountEnvironmentVariable}, then ${defaultSlotCount}; the lock directory to`,
  `$${lockDirectoryEnvironmentVariable}, then <home>/${[
    ...scratchSegments,
    lockDirectorySegment
  ].join('/')}.`,
].join('\n');

/**
 Options the limiter accepts before the command; each takes one value.
 */
const knownOptions: ReadonlySet<string> = new Set([
  '--slot-count',
  '--lock-dir',
  '--poll-interval-ms',
  '--notice-interval-ms',
]);

/**
 Split the limiter's command line into its options and the command.

 Options are accepted only before the command, so a flag the command itself takes is never read here; `--` ends the
 options explicitly.

 @param argv - arguments after the script path
 
 @returns option values by flag, and the command with its arguments
 
 @throws CargoMutantsToolingError when an option is unknown or lacks its value
 
 @example
 ```ts
 splitGateSlotArguments(['--slot-count', '2', 'mise', 'run']);
 ```
 */
export function splitGateSlotArguments(argv: readonly string[]): {
  readonly options: ReadonlyMap<string, string>;
  readonly command: readonly string[];
} {
  /**
   Values of the options read so far, by flag.
   */
  const options = new Map<string, string>();
  for (let index = 0; index < argv.length; index += 2) {
    /**
     Argument at this position, which is an option or the start of the command.
     */
    const argument = argv[index] ?? '';
    if (argument === '--')
      return {
        options,
        command: argv.slice(index + 1),
      };
    if (!argument.startsWith('--'))
      return {
        options,
        command: argv.slice(index),
      };
    if (!knownOptions.has(argument))
      throw new CargoMutantsToolingError(`Unknown option ${argument}.\n\n${gateSlotUsage}`);
    /**
     Value after the option, absent when the option ends the arguments.
     */
    const value = argv[index + 1];
    if (value === undefined)
      throw new CargoMutantsToolingError(`${argument} needs a value.\n\n${gateSlotUsage}`);
    options.set(
      argument,
      value,
    );
  }
  return {
    options,
    command: [],
  };
}

/**
 Read the limiter's command line.

 @param argv - arguments after the script path
 
 @param environment - process environment
 
 @param home - current user's home directory, the base of the default lock directory
 
 @returns the slot settings and the command to run
 
 @throws CargoMutantsToolingError when an option is unknown or malformed or no command is given
 
 @example
 ```ts
 parseGateSlotArguments({ argv: ['mise', 'run'], environment: {}, home: '/h' });
 ```
 */
export function parseGateSlotArguments({
  argv,
  environment,
  home,
}: ForeignBorrowed<{
  readonly argv: readonly string[];
  readonly environment: Environment;
  readonly home: string;
}>): GateSlotOptions {
  /**
   Option values and the command.
   */
  const {
    options,
    command
  } = splitGateSlotArguments(argv);
  if (command.length === 0)
    throw new CargoMutantsToolingError(`A command is required.\n\n${gateSlotUsage}`);
  /**
   Slot count text from the flag, then the environment.
   */
  const slotCount = options.get('--slot-count') ?? environment[slotCountEnvironmentVariable];
  /**
   Poll interval text from the flag.
   */
  const pollInterval = options.get('--poll-interval-ms');
  /**
   Notice interval text from the flag.
   */
  const noticeInterval = options.get('--notice-interval-ms');
  return {
    slotCount: slotCount === undefined
      ? defaultSlotCount
      : positiveWholeNumber({
        option: '--slot-count',
        value: slotCount,
      }),
    lockDirectory: resolve(options.get('--lock-dir') ?? environment[lockDirectoryEnvironmentVariable]
      ?? join(
        home,
        ...scratchSegments,
        lockDirectorySegment,
      )),
    pollIntervalMs: pollInterval === undefined
      ? defaultPollIntervalMs
      : positiveWholeNumber({
        option: '--poll-interval-ms',
        value: pollInterval,
      }),
    noticeIntervalMs: noticeInterval === undefined
      ? defaultNoticeIntervalMs
      : positiveWholeNumber({
        option: '--notice-interval-ms',
        value: noticeInterval,
      }),
    command,
  };
}

/**
 Name the claim series of one slot: its files are `<prefix>.<generation>` in the lock directory.

 @param slot - one-based slot number

 @returns the series' name

 @example
 ```ts
 slotClaimPrefix(3);
 ```
 */
export function slotClaimPrefix(slot: number): string {
  return `slot-${String(slot)}.owner`;
}

/**
 Name the lock file the earlier release of this package took on one slot, which is only probed now.

 @param lockDirectory - directory that holds the lock files
 
 @param slot - one-based slot number
 
 @returns the lock file path
 
 @example
 ```ts
 slotLockPath({ lockDirectory: '/locks', slot: 3 });
 ```
 */
export function slotLockPath({
  lockDirectory,
  slot,
}: {
  readonly lockDirectory: string;
  readonly slot: number;
}): string {
  return join(
    lockDirectory,
    `slot-${slot}.lock`,
  );
}
