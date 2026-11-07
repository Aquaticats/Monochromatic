/**
 Command-line options of the copy tool.

 @module
 */

import {
  DEFAULT_COMMAND_DEADLINE_SECONDS,
  DEFAULT_EXTERNAL_DEADLINE_SECONDS,
  DEFAULT_MIN_FREE_GIB,
  DEFAULT_TOTAL_DEADLINE_SECONDS,
  DEFAULT_TRANSFER_DEADLINE_SECONDS,
} from './constants.ts';
import { MoveStopError, } from './errors.ts';

/**
 The passes the tool runs: two copies, the freeze and its thaw, and the two external-folder passes.
 */
export type MovePass = 'external-final' | 'external-first' | 'final' | 'first' | 'freeze' | 'thaw';

/**
 The options of one invocation. A string option that was not given is the empty string, and an absent limit is
 positive infinity, so no field is a union with undefined.
 */
export type MoveOptions = {
  readonly pass: MovePass;
  readonly mainPath: string;
  readonly snapshotFolder: string;
  readonly receivedFolder: string;
  readonly finalPath: string;
  readonly stateFolder: string;
  readonly inventoryFile: string;
  readonly inventoryMain: string;
  readonly inventoryPrefixFrom: string;
  readonly inventoryPrefixTo: string;
  readonly transferDeadlineSeconds: number;
  readonly totalDeadlineSeconds: number;
  readonly commandDeadlineSeconds: number;
  readonly externalDeadlineSeconds: number;
  readonly minFreeGiB: number;
  readonly cloneSources: 'all' | 'main';
  readonly limit: number;
  readonly run: boolean;
  readonly noScope: boolean;
  readonly subvolumeMarker: string;
  readonly externalSource: string;
  readonly externalFinal: string;
};

/**
 Every pass name, for validating `--pass`.
 */
const PASSES: readonly MovePass[] = [
  'first',
  'final',
  'freeze',
  'thaw',
  'external-first',
  'external-final',
];

/**
 Inventory prefix mapping when none is given: the top-level path `home/` is mounted at `/var/home/`.
 */
const DEFAULT_PREFIX = 'home/=/var/home/';

/**
 Reads `--name=value` from an argument list.

 @param argv - the arguments
 
 @param name - the flag name without dashes
 
 @returns the value, or the empty string when the flag is absent
 */
function flagValue(
  {
    argv,
    name,
  }: {
    readonly argv: readonly string[];
    readonly name: string;
  },
): string {
  /**
   The flag prefix searched for.
   */
  const lead = `--${name}=`;
  /**
   The found of this step.
   */
  const found = argv.find(function startsWithLead(argument,) {
    return argument.startsWith(lead,);
  },);
  return found === undefined ? '' : found.slice(lead.length,);
}

/**
 Reads a required `--name=value`.

 @param argv - the arguments
 
 @param name - the flag name without dashes
 
 @returns the value
 
 @throws MoveStopError when the flag is absent
 */
function requiredFlag(
  {
    argv,
    name,
  }: {
    readonly argv: readonly string[];
    readonly name: string;
  },
): string {
  /**
   The value this step works on.
   */
  const value = flagValue({
    argv,
    name,
  },);
  if (value === '') {
    throw new MoveStopError(`missing --${name}=`,);
  }
  return value;
}

/**
 Reads a non-negative numeric flag.

 @param argv - the arguments
 
 @param name - the flag name without dashes
 
 @param fallback - value when the flag is absent
 
 @returns the number
 
 @throws MoveStopError when the value is not a non-negative number
 */
function numberFlag(
  {
    argv,
    name,
    fallback,
  }: {
    readonly argv: readonly string[];
    readonly name: string;
    readonly fallback: number;
  },
): number {
  /**
   The text this step works on.
   */
  const text = flagValue({
    argv,
    name,
  },);
  if (text === '') {
    return fallback;
  }
  /**
   The value this step works on.
   */
  const value = Number(text,);
  if ((!Number.isFinite(value,)) || (value < 0)) {
    throw new MoveStopError(`--${name}= must be a non-negative number, got ${text}`,);
  }
  return value;
}

/**
 Narrows a string to a pass name.

 @param value - the text
 
 @returns true when it names a pass
 */
function isPass(value: string,): value is MovePass {
  return PASSES.some(function same(pass,) {
    return pass === value;
  },);
}

/**
 Reads the options from an argument list.

 @param argv - arguments after the program name
 
 @returns the options; `--print` after `--run` wins, so print mode can always be forced
 
 @throws MoveStopError when a required flag is missing or a value is malformed
 */
export function parseMoveOptions(argv: readonly string[],): MoveOptions {
  /**
   The pass of this step.
   */
  const pass = requiredFlag({
    argv,
    name: 'pass',
  },);
  if (!isPass(pass,)) {
    throw new MoveStopError(`--pass must be one of ${PASSES.join(', ',)}`,);
  }
  /**
   The mapping of this step.
   */
  const mapping = flagValue({
    argv,
    name: 'inventory-prefix',
  },) || DEFAULT_PREFIX;
  /**
   The parts of the value this function uses, taken out of it.
   */
  const [prefixFrom = '', prefixTo = '',] = mapping.split('=',);
  /**
   The clone sources of this step.
   */
  const cloneSources = flagValue({
    argv,
    name: 'clone-sources',
  },) || 'all';
  if ((cloneSources !== 'all') && (cloneSources !== 'main')) {
    throw new MoveStopError('--clone-sources must be all or main',);
  }
  return {
    pass,
    mainPath: requiredFlag({
      argv,
      name: 'main',
    },),
    snapshotFolder: requiredFlag({
      argv,
      name: 'snapshots',
    },),
    receivedFolder: requiredFlag({
      argv,
      name: 'received',
    },),
    finalPath: requiredFlag({
      argv,
      name: 'final',
    },),
    stateFolder: requiredFlag({
      argv,
      name: 'state',
    },),
    inventoryFile: flagValue({
      argv,
      name: 'inventory',
    },),
    inventoryMain: flagValue({
      argv,
      name: 'inventory-main',
    },),
    inventoryPrefixFrom: prefixFrom,
    inventoryPrefixTo: prefixTo,
    transferDeadlineSeconds: numberFlag({
      argv,
      name: 'deadline-seconds',
      fallback: DEFAULT_TRANSFER_DEADLINE_SECONDS,
    },),
    totalDeadlineSeconds: numberFlag({
      argv,
      name: 'total-deadline-seconds',
      fallback: DEFAULT_TOTAL_DEADLINE_SECONDS,
    },),
    commandDeadlineSeconds: numberFlag({
      argv,
      name: 'command-deadline-seconds',
      fallback: DEFAULT_COMMAND_DEADLINE_SECONDS,
    },),
    externalDeadlineSeconds: numberFlag({
      argv,
      name: 'external-deadline-seconds',
      fallback: DEFAULT_EXTERNAL_DEADLINE_SECONDS,
    },),
    minFreeGiB: numberFlag({
      argv,
      name: 'min-free-gib',
      fallback: DEFAULT_MIN_FREE_GIB,
    },),
    cloneSources,
    limit: numberFlag({
      argv,
      name: 'limit',
      fallback: Number.POSITIVE_INFINITY,
    },),
    run: argv.includes('--run',) && (!argv.includes('--print',)),
    noScope: argv.includes('--no-scope',),
    subvolumeMarker: flagValue({
      argv,
      name: 'subvolume-marker',
    },),
    externalSource: flagValue({
      argv,
      name: 'external-source',
    },),
    externalFinal: flagValue({
      argv,
      name: 'external-final',
    },),
  };
}
