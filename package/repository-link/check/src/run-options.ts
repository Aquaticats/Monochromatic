/**
 Command-line options shared by the check bins.

 @module
 */

import { EXIT_GIT_FATAL, EXIT_TEMPFAIL, DEFAULT_DEADLINE_SECONDS, } from './constants.ts';
import { identityPathFor, } from './identity-file.ts';

/**
 What the leading `--name=value` options of a bin say, and the rest of the arguments.
 */
export type BinOptions = {
  readonly identityFile: string;
  readonly failureStatus: number;
  readonly waitSeconds: number;
  readonly deadlineSeconds: number;
  readonly rest: readonly string[];
};

/**
 Reads the leading options of a bin: `--identity=<file>`, `--for=git`, `--wait-seconds=<n>`, `--deadline-seconds=<n>`.
 Reading stops at the first argument that is not an option or is `--`, and the rest is the command to run.

 @param argv - arguments after the program name
 @param home - the home folder for the default identity file
 @returns the options and the rest
 */
export function parseBinOptions(
  { argv, home, }: {
    readonly argv: readonly string[];
    readonly home: string;
  },
): BinOptions {
  const leading: string[] = [];
  for (const argument of argv) {
    if (!argument.startsWith('--',) || argument === '--') {
      break;
    }
    leading.push(argument,);
  }
  const rest = argv.slice(leading.length,).filter(function dropSeparator(argument, index,) {
    return !(index === 0 && argument === '--');
  },);

  /**
   Reads the value of one leading option.

   @param name - the option name without dashes
   @returns the value, or an empty string when absent
   */
  function value(name: string,): string {
    const lead = `--${name}=`;
    const found = leading.find(function startsWithLead(argument,) {
      return argument.startsWith(lead,);
    },);
    return found === undefined ? '' : found.slice(lead.length,);
  }

  return {
    identityFile: value('identity',) || identityPathFor(home,),
    failureStatus: value('for',) === 'git' ? EXIT_GIT_FATAL : EXIT_TEMPFAIL,
    waitSeconds: Number(value('wait-seconds',) || '0',),
    deadlineSeconds: Number(value('deadline-seconds',) || String(DEFAULT_DEADLINE_SECONDS,),),
    rest,
  };
}
