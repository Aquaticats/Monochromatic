import {
  join,
  resolve,
} from 'node:path';

import {
  defaultPollIntervalMs,
  niceProgram,
  defaultSlotCount,
  lockDirectoryEnvironmentVariable,
  lockDirectorySegment,
  scratchSegments,
  slotConflictExitCode,
  slotCountEnvironmentVariable,
  slotNiceAdjustment,
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
  readonly command: readonly string[];
};

/**
 Usage text printed with every option error.
 */
export const gateSlotUsage: string = [
  'Usage: gate-slot.ts [--slot-count <count>] [--lock-dir <dir>] [--poll-interval-ms <ms>] [--] <command> [arguments...]',
  '',
  'Waits until one of the host-wide slots is free, runs the command in it at the lowest nice priority with inherited',
  'stdio, and exits with the command status.',
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
    command,
  };
}

/**
 Name the lock file of one slot.

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

/**
 Build the `flock` arguments that try one slot without waiting and, when it is free, run the command inside it.

 @param lockDirectory - directory that holds the lock files
 
 @param slot - one-based slot number
 
 @param command - command and arguments to run inside the slot
 
 @returns the argument list after the `flock` program name
 
 @example
 ```ts
 buildFlockArguments({ lockDirectory: '/locks', slot: 1, command: ['mise', 'run'] });
 ```
 */
export function buildFlockArguments({
  lockDirectory,
  slot,
  command,
}: {
  readonly lockDirectory: string;
  readonly slot: number;
  readonly command: readonly string[];
}): readonly string[] {
  return [
    '--nonblock',
    '--exclusive',
    '--conflict-exit-code',
    String(slotConflictExitCode),
    slotLockPath({
      lockDirectory,
      slot,
    }),
    niceProgram,
    `--adjustment=${slotNiceAdjustment}`,
    ...command,
  ];
}

/**
 Wait for a free slot and run the command in it.

 Slots are tried in order; when every one is held, the limiter waits the poll interval and tries them all again.
 A command that itself exits with the conflict status is indistinguishable from a held slot and is tried again.

 @param options - slot settings and command
 
 @param runFlock - starts `flock` with the given arguments and returns its exit status
 
 @param sleep - waits the given number of milliseconds
 
 @returns the command's exit status
 
 @example
 ```ts
 await runInSlot({ options, runFlock, sleep });
 ```
 */
export async function runInSlot({
  options,
  runFlock,
  sleep,
}: {
  readonly options: GateSlotOptions;
  readonly runFlock: (flockArguments: readonly string[]) => Promise<number>;
  readonly sleep: (milliseconds: number) => Promise<void>;
}): Promise<number> {
  for (;;) {
    for (let slot = 1; slot <= options.slotCount; slot += 1) {
      // Slots must be tried one at a time, in order, so the lowest free slot is the one taken.
      /**
       Exit status of the attempt in this slot.
       */
      // oxlint-disable-next-line eslint/no-await-in-loop -- sequential by design, one flock attempt per slot
      const status = await runFlock(buildFlockArguments({
        lockDirectory: options.lockDirectory,
        slot,
        command: options.command,
      }));
      if (status !== slotConflictExitCode)
        return status;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the poll wait is the loop's pacing
    await sleep(options.pollIntervalMs);
  }
}
