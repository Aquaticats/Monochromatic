/**
 Command-line grammar for the issue-acceptance command.

 Every misuse throws {@link CliInvocationError},
 which the entry point maps to exit status two and prints the usage text.

 @module
 */

import { parseArgs, } from 'node:util';

import { CliInvocationError, } from './errors.ts';
import type { ShowMode } from './show.ts';

/**
 Usage text printed for `--help` and for every misuse.
 */
export const USAGE: string = [
  'Usage:',
  '  issue-acceptance settle --plan FILE [--apply]',
  '  issue-acceptance show --repo OWNER/NAME NUMBER... [--acceptance]',
  '  issue-acceptance ready-bugs --repo OWNER/NAME [--out FILE]',
  '',
  'settle checks every issue of the plan against the plan\'s branch on a checkout.',
  'Without --apply it only prints each verdict; with --apply it closes the issues',
  'whose checks hold, or comments on them and leaves them open, as the plan says.',
  'show prints each issue in full, or only the sections naming what it asked for.',
  'ready-bugs groups open bug and ready-for-agent issues by the first package each names.',
].join('\n',);

/**
 Flag names in the order the usage text names them.
 */
const FLAG_NAMES: readonly (keyof ParsedFlags)[] = [
  'plan',
  'apply',
  'repo',
  'acceptance',
  'out',
  'help',
];

/**
 Flags this grammar defines.
 */
export type ParsedFlags = {
  readonly plan?: string;
  readonly apply?: boolean;
  readonly repo?: string;
  readonly acceptance?: boolean;
  readonly out?: string;
  readonly help?: boolean;
};

/**
 One parsed token set.
 */
export type ParsedArguments = {
  readonly values: ParsedFlags;
  readonly positionals: readonly string[];
};

/**
 Asks the plan's checks on the branch and settles the issues.
 */
export type SettleCommand = {
  readonly kind: 'settle';
  readonly planPath: string;
  readonly apply: boolean;
};

/**
 Prints issues for reading.
 */
export type ShowCommand = {
  readonly kind: 'show';
  readonly repository: string;
  readonly numbers: readonly number[];
  readonly mode: ShowMode;
};

/**
 Groups the open issues a batch agent may take.
 */
export type ReadyBugsCommand = {
  readonly kind: 'ready-bugs';
  readonly repository: string;
  readonly outPath?: string;
};

/**
 Prints the usage text.
 */
export type HelpCommand = {
  readonly kind: 'help';
};

/**
 One parsed command.
 */
export type Command = SettleCommand | ShowCommand | ReadyBugsCommand | HelpCommand;

/**
 Adds the usage text to one misuse message.
 
 @param message - What the parser refused.
 
 @returns The message with the usage text under it.
 
 @example
 ```ts
 const text = withUsage('unknown option');
 ```
 */
function withUsage(message: string,): string {
  return `${message}\n\n${USAGE}`;
}

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
        plan: { type: 'string', },
        apply: { type: 'boolean', },
        repo: { type: 'string', },
        acceptance: { type: 'boolean', },
        out: { type: 'string', },
        help: { type: 'boolean', },
      },
    },);
  }
  catch (error) {
    if (!(Error.isError(error,))) {
      throw error;
    }
    throw new CliInvocationError(withUsage(error.message,),);
  }
}

/**
 Reads a flag that this subcommand does not define,
 so `--aply` or a flag on the wrong subcommand fails instead of being ignored.
 
 @param flags - Flags the parser saw.
 
 @param allowed - Flags the subcommand reads.
 
 @param subcommand - Subcommand being read.
 
 @throws {@link CliInvocationError} naming the first flag that does not belong.
 
 @example
 ```ts
 refuseUnexpectedFlags({ flags: ['apply'], allowed: ['plan'], subcommand: 'show' });
 ```
 */
function refuseUnexpectedFlags({
  flags,
  allowed,
  subcommand,
}: {
  readonly flags: readonly string[];
  readonly allowed: readonly string[];
  readonly subcommand: string;
},): void {
  for (const flag of flags) {
    if (!allowed.includes(flag,)) {
      throw new CliInvocationError(`${subcommand} does not take --${flag}.`,);
    }
  }
}

/**
 Reads one issue number from a positional argument.
 
 @param value - Positional argument as typed.
 
 @returns The issue number.
 
 @throws {@link CliInvocationError} when the argument is not a positive integer.
 
 @example
 ```ts
 const number = readNumber('1234');
 ```
 */
function readNumber(value: string,): number {
  /**
   Argument read as a number.
   */
  const number = Number(value,);
  if ((!Number.isSafeInteger(number,)) || (number < 1)) {
    throw new CliInvocationError(`show takes issue numbers, and ${JSON.stringify(value,)} is not one.`,);
  }
  return number;
}

/**
 Reads the repository a read-only subcommand works on.
 
 @param value - Value of `--repo`.
 
 @param subcommand - Subcommand that needs it.
 
 @returns The repository slug.
 
 @throws {@link CliInvocationError} when the flag is missing or empty.
 
 @example
 ```ts
 const repository = readRepository({ value: 'OWNER/NAME', subcommand: 'show' });
 ```
 */
function readRepository({
  value,
  subcommand,
}: {
  readonly value?: string;
  readonly subcommand: string;
},): string {
  if ((value === undefined) || (value === '')) {
    throw new CliInvocationError(`${subcommand} needs --repo OWNER/NAME, which names the repository it reads.`,);
  }
  return value;
}

/**
 Reads the settle subcommand.
 
 @param parsed - Parsed tokens.
 
 @param flags - Flags the parser saw.
 
 @returns The settle command.
 
 @throws {@link CliInvocationError} when a flag or argument does not belong to settle.
 
 @example
 ```ts
 const command = readSettle({ parsed, flags: ['plan'] });
 ```
 */
function readSettle({
  parsed,
  flags,
}: {
  readonly parsed: ParsedArguments;
  readonly flags: readonly string[];
},): SettleCommand {
  refuseUnexpectedFlags({
    flags,
    allowed: [
      'plan',
      'apply',
    ],
    subcommand: 'settle',
  },);
  if (parsed.positionals
    .length
    > 0) {
    throw new CliInvocationError(`settle takes no positional argument, and ${JSON.stringify(parsed.positionals[0],)} is one.`,);
  }
  /**
   Plan file path as typed.
   */
  const planPath = parsed.values
    .plan;
  if ((planPath === undefined) || (planPath === '')) {
    throw new CliInvocationError('settle needs --plan FILE, which names the settlement plan.',);
  }
  return {
    kind: 'settle',
    planPath,
    apply: parsed.values
      .apply
      === true,
  };
}

/**
 Reads the show subcommand.
 
 @param parsed - Parsed tokens.
 
 @param flags - Flags the parser saw.
 
 @returns The show command.
 
 @throws {@link CliInvocationError} when a flag or argument does not belong to show.
 
 @example
 ```ts
 const command = readShow({ parsed, flags: ['repo'] });
 ```
 */
function readShow({
  parsed,
  flags,
}: {
  readonly parsed: ParsedArguments;
  readonly flags: readonly string[];
},): ShowCommand {
  refuseUnexpectedFlags({
    flags,
    allowed: [
      'repo',
      'acceptance',
    ],
    subcommand: 'show',
  },);
  if (parsed.positionals
    .length
    === 0) {
    throw new CliInvocationError('show needs at least one issue number.',);
  }
  return {
    kind: 'show',
    repository: readRepository({
      ...(parsed.values
        .repo
        === undefined ? {} : { value: parsed.values
          .repo, }),
      subcommand: 'show',
    },),
    numbers: parsed.positionals
      .map(readNumber,),
    mode: parsed.values
      .acceptance
      === true ? 'acceptance' : 'full',
  };
}

/**
 Reads the ready-bugs subcommand.
 
 @param parsed - Parsed tokens.
 
 @param flags - Flags the parser saw.
 
 @returns The ready-bugs command.
 
 @throws {@link CliInvocationError} when a flag or argument does not belong to ready-bugs.
 
 @example
 ```ts
 const command = readReadyBugs({ parsed, flags: ['repo'] });
 ```
 */
function readReadyBugs({
  parsed,
  flags,
}: {
  readonly parsed: ParsedArguments;
  readonly flags: readonly string[];
},): ReadyBugsCommand {
  refuseUnexpectedFlags({
    flags,
    allowed: [
      'repo',
      'out',
    ],
    subcommand: 'ready-bugs',
  },);
  if (parsed.positionals
    .length
    > 0) {
    throw new CliInvocationError(`ready-bugs takes no positional argument, and ${JSON.stringify(parsed.positionals[0],)} is one.`,);
  }
  /**
   File the grouping is also written to as JSON.
   */
  const outPath = parsed.values
    .out;
  return {
    kind: 'ready-bugs',
    repository: readRepository({
      ...(parsed.values
        .repo
        === undefined ? {} : { value: parsed.values
          .repo, }),
      subcommand: 'ready-bugs',
    },),
    ...((outPath === undefined) || (outPath === '') ? {} : { outPath, }),
  };
}

/**
 Parses the command line.
 
 @param tokens - Tokens after the executable and script paths.
 
 @returns The parsed command.
 
 @throws {@link CliInvocationError} when the tokens do not form one command.
 
 @example
 ```ts
 const command = parseArguments({ tokens: ['settle', '--plan', 'plan.json'] });
 ```
 */
export function parseArguments({ tokens, }: { readonly tokens: readonly string[]; },): Command {
  /**
   Parsed tokens, before they are required to form one command.
   */
  const parsed = parseRaw(tokens,);
  /**
   Flags the parser saw set.
   */
  const flags = FLAG_NAMES.filter(function isSet(name,): boolean {
    return parsed.values[name] !== undefined;
  },);
  if ((parsed.values
    .help
    === true) || (parsed.positionals[0] === 'help')) {
    return { kind: 'help', };
  }
  /**
   First positional, which names the subcommand.
   */
  const [subcommand,] = parsed.positionals;
  if (subcommand === undefined) {
    throw new CliInvocationError('a subcommand is required.',);
  }
  /**
   Tokens after the subcommand, which each reader requires to be empty or a list of numbers.
   */
  const rest: ParsedArguments = {
    values: parsed.values,
    positionals: parsed.positionals
      .slice(1,),
  };
  if (subcommand === 'settle') {
    return readSettle({
      parsed: rest,
      flags,
    },);
  }
  if (subcommand === 'show') {
    return readShow({
      parsed: rest,
      flags,
    },);
  }
  if (subcommand === 'ready-bugs') {
    return readReadyBugs({
      parsed: rest,
      flags,
    },);
  }
  throw new CliInvocationError(`${JSON.stringify(subcommand,)} is not a subcommand.`,);
}
