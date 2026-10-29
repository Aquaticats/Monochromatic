/**
 Registered fuzz targets and the command-line options of the container campaign.

 Every option is validated before any container starts, so a typo fails in milliseconds instead of after a build.
 */
import { NativeVerificationError } from '../../cli/bin/native-verification-process.mjs';

/**
 One registered target and the dictionary it fuzzes with.

 @typedef {{ name: string, dictionary: string }} FuzzTarget
 */

/**
 Registered targets, in the order the campaign runs them.

 @type {readonly FuzzTarget[]}
 */
export const targets = [
  {
    name: 'global_arguments',
    dictionary: 'arguments.dict',
  },
  {
    name: 'config_loading',
    dictionary: 'arguments.dict',
  },
  {
    name: 'config_schema',
    dictionary: 'config_schema.dict',
  },
  {
    name: 'wrapper_controls',
    dictionary: 'controls.dict',
  },
  {
    name: 'batch_reply',
    dictionary: 'batch_reply.dict',
  },
  {
    name: 'stage_listing',
    dictionary: 'stage_listing.dict',
  },
  {
    name: 'rules_file',
    dictionary: 'rules_file.dict',
  },
  {
    name: 'final_newline',
    dictionary: 'final_newline.dict',
  },
  {
    name: 'owner_records',
    dictionary: 'owner_records.dict',
  },
  {
    name: 'transaction_records',
    dictionary: 'transaction_records.dict',
  },
  {
    name: 'index_lock_records',
    dictionary: 'index_lock_records.dict',
  },
  {
    name: 'dependent_version',
    dictionary: 'dependent_version.dict',
  },
  {
    name: 'linter_output',
    dictionary: 'linter_output.dict',
  },
  {
    name: 'event_path',
    dictionary: 'event_path.dict',
  },
  {
    name: 'linter_config',
    dictionary: 'linter_config.dict',
  },
];

/** Seconds each target fuzzes when no `--seconds-per-target` option is given. */
const defaultSecondsPerTarget = 30;
/** Largest `--seconds-per-target` value: one day. */
const maxSecondsPerTarget = 86_400;
const buildOnlyOption = '--build-only';
const secondsOption = '--seconds-per-target=';
const targetsOption = '--targets=';

/**
 Whether an argument is one of the three accepted options.

 @param {string} argument - one command-line argument
 @returns {boolean} true for `--build-only`, `--seconds-per-target=...` and `--targets=...`
 */
function isAcceptedOption(argument) {
  return (argument === buildOnlyOption)
    || argument.startsWith(secondsOption)
    || argument.startsWith(targetsOption);
}

/**
 The value after an option prefix, or nothing when the option is absent.

 @param {{ options: readonly string[], prefix: string }} request - the arguments and the `--name=` prefix to find
 @returns {string | undefined} the text after the prefix of the first matching argument
 */
function valueOf({
  options,
  prefix,
}) {
  return options.find(function hasPrefix(argument) {
    return argument.startsWith(prefix);
  })
    ?.slice(prefix.length);
}

/**
 Read the whole-number seconds each target fuzzes.

 @param {string | undefined} text - the `--seconds-per-target` value, absent for the default
 @returns {number} seconds from 1 to the maximum
 */
function secondsOf(text) {
  const seconds = text === undefined
    ? defaultSecondsPerTarget
    : Number(text);
  if ((!Number.isSafeInteger(seconds)) || (seconds < 1)
    || (seconds > maxSecondsPerTarget))
    throw new NativeVerificationError(`${secondsOption}<seconds> must be a whole number from 1 to ${maxSecondsPerTarget}.`);
  return seconds;
}

/**
 Pick the targets a `--targets` list names, or every target when none is given.

 @param {string | undefined} text - the comma-separated names after `--targets=`
 @returns {readonly FuzzTarget[]} the chosen targets in registration order
 */
function targetsOf(text) {
  if (text === undefined)
    return targets;
  const names = text.split(',');
  const unknown = names.filter(function isUnknown(name) {
    return !targets.some(function isNamed(target) {
      return target.name === name;
    });
  });
  if (unknown.length > 0)
    throw new NativeVerificationError(`Unknown fuzz targets: ${unknown.join(', ')}.`);
  return targets.filter(function isChosen(target) {
    return names.includes(target.name);
  });
}

/**
 Validate the command line of the campaign.

 @param {readonly string[]} options - arguments after the script path
 @returns {{ buildOnly: boolean, secondsPerTarget: number, selectedTargets: readonly FuzzTarget[] }} the build-only
   flag, the seconds each target fuzzes, and the targets to fuzz
 @throws {NativeVerificationError} for an unknown option, an unknown target or seconds outside 1 to 86400
 */
export function parseOptions(options) {
  if (!options.every(isAcceptedOption))
    throw new NativeVerificationError(`Only ${buildOnlyOption}, ${secondsOption}<seconds> and ${targetsOption}<name,...> are accepted.`);
  return {
    buildOnly: options.includes(buildOnlyOption),
    secondsPerTarget: secondsOf(valueOf({
      options,
      prefix: secondsOption,
    })),
    selectedTargets: targetsOf(valueOf({
      options,
      prefix: targetsOption,
    })),
  };
}
