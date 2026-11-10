/**
 Command-line grammar for the owner-answer command.

 @module
 */

import { parseArgs, } from 'node:util';

import { CliInvocationError, } from './errors.ts';

/**
 Usage text printed for `--help` and for every misuse.
 */
export const USAGE: string = [
  'Usage:',
  '  owner-answer --answers FILE [--apply]',
  '',
  '--answers names the JSON file holding the block to record and the parts it',
  'replaces in the questions file. Without --apply the run prints what it would',
  'remove and where it would insert, and writes nothing.',
].join('\n',);

/**
 Flags this grammar defines.
 */
export type ParsedFlags = {
  readonly answers?: string;
  readonly apply?: boolean;
  readonly help?: boolean;
};

/**
 Flag names, in the order the usage text names them.
 */
const FLAG_NAMES: readonly (keyof ParsedFlags)[] = [
  'answers',
  'apply',
  'help',
];

/**
 One parsed token set.
 */
export type ParsedArguments = {
  readonly values: ParsedFlags;
  readonly positionals: readonly string[];
};

/**
 A run that records one answers file.
 */
export type RecordCommand = {
  readonly kind: 'record';
  readonly answersPath: string;
  readonly apply: boolean;
};

/**
 A run that prints the usage text.
 */
export type HelpCommand = {
  readonly kind: 'help';
};

/**
 One parsed command.
 */
export type Command = RecordCommand | HelpCommand;

/**
 Parses the raw tokens, keeping the parser's own types.
 
 @param tokens - Tokens after the executable and script paths.
 
 @returns Parsed values and positionals.
 
 @throws {@link CliInvocationError} when the tokens break the parser's grammar.
 
 @example
 ```ts
 const parsed = parseRaw(['--help']);
 ```
 */
function parseRaw(tokens: readonly string[],): ParsedArguments {
  try {
    return parseArgs({
      args: [...tokens,],
      strict: true,
      allowPositionals: true,
      options: {
        answers: { type: 'string', },
        apply: { type: 'boolean', },
        help: { type: 'boolean', },
      },
    },);
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new CliInvocationError(`${error.message}\n\n${USAGE}`,);
  }
}

/**
 Parses the command line.
 
 @param tokens - Tokens after the executable and script paths.
 
 @returns The parsed command.
 
 @throws {@link CliInvocationError} when the tokens do not form one command.
 
 @example
 ```ts
 const command = parseArguments({ tokens: ['--answers', 'answers.json'] });
 ```
 */
export function parseArguments({ tokens, }: { readonly tokens: readonly string[]; },): Command {
  /**
   Parsed tokens, before they are required to form one command.
   */
  const parsed = parseRaw(tokens,);
  if (parsed.values
    .help
    === true) {
    return { kind: 'help', };
  }
  if (parsed.positionals
    .length
    > 0) {
    throw new CliInvocationError(`The run takes no positional argument, and ${JSON.stringify(parsed.positionals[0],)} is one.\n\n${USAGE}`,);
  }
  /**
   Answers file path as typed.
   */
  const answersPath = parsed.values
    .answers;
  if ((answersPath === undefined) || (answersPath === '')) {
    throw new CliInvocationError(`--answers FILE is required, and names the answers file.\n\n${USAGE}`,);
  }
  return {
    kind: 'record',
    answersPath,
    apply: parsed.values
      .apply
      === true,
  };
}
