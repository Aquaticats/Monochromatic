/**
 The command line of the guided runner: the command, the flags it takes, and the refusal of everything else. A flag
 that is ignored when mistyped (`--redo=` or `--assert=` naming no stage) would skip the redo or the assertion it was
 meant to give, so every argument must be one the runner knows.

 @module
 */

import {
  EXIT_PRECONDITION,
  GuidedStopError,
} from './guided-stop.ts';
import type {
  GuidedCommand,
  GuidedFlags,
  Stage,
} from './guided-types.ts';

/**
 The commands, for validating the first argument.
 */
const COMMANDS: readonly GuidedCommand[] = [
  'prepare',
  'cutover',
  'thaw',
  'rollback',
  'status',
];

/**
 The usage line.
 */
const USAGE =
  'Usage: run-repository-move prepare | cutover | thaw | rollback | status [--dry-run] [--redo=<stage id>] [--assert=<stage id>]';

/**
 Collects every `--name=value` among the arguments.

 @param argv - the arguments
 
 @param name - the flag name
 
 @returns the values
 */
function valuesOf(
  {
    argv,
    name,
  }: {
    readonly argv: readonly string[];
    readonly name: string;
  },
): readonly string[] {
  /**
   The flag prefix searched for.
   */
  const lead = `--${name}=`;
  return argv.filter(function startsWithLead(argument,) {
    return argument.startsWith(lead,);
  },)
    .map(function valueOf(argument,) {
      return argument.slice(lead.length,);
    },);
}

/**
 Reads the command and flags from the arguments.

 @param argv - arguments after the program name
 
 @returns the flags
 
 @throws GuidedStopError with the precondition status when the command is unknown

 @example
 ```ts
 import { parseGuidedFlags, } from '\@monochromatic-dev/repository-move-cli';
 
 const result = parseGuidedFlags(argv,);
 ```
 */
export function parseGuidedFlags(argv: readonly string[],): GuidedFlags {
  /**
   The command text.
   */
  const command = COMMANDS.find(function isNamed(name,) {
    return name === argv[0];
  },);
  if (command === undefined) {
    throw new GuidedStopError({
      status: EXIT_PRECONDITION,
      message: USAGE,
    },);
  }
  /**
   The arguments after the command that the runner does not know.
   */
  const unknown = argv.slice(1,)
    .filter(function isUnknown(argument,) {
      return (argument !== '--dry-run') && (!(argument.startsWith('--redo=',) && (argument.length > '--redo='.length)))
        && (!(argument.startsWith('--assert=',) && (argument.length > '--assert='.length)));
    },);
  if (unknown.length > 0) {
    throw new GuidedStopError({
      status: EXIT_PRECONDITION,
      message: `Not understood: ${unknown.join(' ',)}. Nothing ran. The runner takes exactly these after the command: `
        + '--dry-run, --redo=<stage id> and --assert=<stage id> (each --redo and --assert needs a value).',
    },);
  }
  return {
    command,
    dryRun: argv.includes('--dry-run',),
    redo: valuesOf({
      argv,
      name: 'redo',
    },),
    asserted: valuesOf({
      argv,
      name: 'assert',
    },),
  };
}

/**
 Refuses a `--redo` or `--assert` that names no stage of the command, because a mistyped one that is ignored would
 skip the redo or the assertion it was meant to give.

 @param stages - the stages of the command

 @param flags - the flags

 @throws GuidedStopError with the precondition status, naming the flag and every valid stage id

 @example
 ```ts
 import { requireKnownStages, } from './guided-flags.ts';
 
 requireKnownStages({ stages, flags, },);
 ```
 */
export function requireKnownStages(
  {
    stages,
    flags,
  }: {
    readonly stages: readonly Stage[];
    readonly flags: GuidedFlags;
  },
): void {
  /**
   The stage ids of the command.
   */
  const ids = stages.map(function idOf(stage,) {
    return stage.id;
  },);
  /**
   The flag values that name no stage.
   */
  const unknown = [
    ...flags.redo
      .map(function redoFlag(id,) {
        return `--redo=${id}`;
      },),
    ...flags.asserted
      .map(function assertFlag(id,) {
        return `--assert=${id}`;
      },),
  ].filter(function namesNothing(flag,) {
    return !ids.includes(flag.slice(flag.indexOf('=',) + 1,),);
  },);
  if (unknown.length > 0) {
    throw new GuidedStopError({
      status: EXIT_PRECONDITION,
      message: `${unknown.join(' ',)} names no stage of the ${flags.command} command. Nothing ran. Its stages: ${ids.join(', ',)}.`,
    },);
  }
}
