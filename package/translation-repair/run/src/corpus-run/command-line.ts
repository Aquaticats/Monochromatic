import {
  parseArgs,
  type ParseArgsOptionDescriptor,
} from 'node:util';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { isAsciiDigits, } from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import type {
  CommandLineFor,
  CommandLineSpec,
  FlagValue,
} from './command-line-types.ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Command line
// THE ONE READER OF THE WHOLE COMMAND LINE every runner takes (ledger B75).
//
// Each runner used to find its own flags with `indexOf` or `includes` on the
// exact token, so a flag written any other way read as not written at all.
// `--cap=0` bought every subject the settled audit could buy, `--olny Toka_ls`
// or a mistyped `--plan` ran every pending corpus entry for real, a second
// `--only` was dropped, and a command that takes no flags ran as if nothing
// had been typed after it. B73 gave the flag VALUES one reader; the line itself
// still had none, so nothing said which tokens a command reads at all.
//
// EVERY RUNNER NOW DECLARES WHAT IT READS, in `command-lines.ts`, and
// `reportingRefusals` reads the whole line against that before the command
// starts. Node's `parseArgs` splits the line, so the equals form, the `--`
// terminator and grouped short flags read as they do for every Node command.
// It runs with `strict` off and hands back its tokens, and every refusal is
// written here: all of them at once, in the words the B73 readers already
// used, and a number written with a minus sign after a flag still reaches the
// reader that answers it as under zero, which strict `parseArgs` refuses as
// ambiguous before any reader sees it.

/**
 Position of the script path in a process's arguments, after the runtime.
 */
const SCRIPT_AT = 1;

/**
 Position of the first argument the person typed.
 */
const FIRST_TYPED_AT = 2;

/**
 Marker a long flag starts with.
 */
const LONG_PREFIX = '--';

/**
 Sign a number under zero is written with.
 */
const MINUS = '-';

/**
 What an option token carried after its flag.

 NAMED RATHER THAN LEFT NULLISH, as `FlagValue` is: `parseArgs` answers
 `undefined` for a flag written with nothing after it, and this is that answer
 turned into one a reader has to name.

 @example
 ```ts
 const carried: Carried = { kind: 'text', text: 'tabby', };
 ```
 */
type Carried = {
  readonly kind: 'nothing';
} | {
  readonly kind: 'text';

  /**
   What was written after the flag, or after its `=`.
   */
  readonly text: string;
};

/**
 What one option token reads as against the declaration.

 @example
 ```ts
 const reading: OptionReading = { kind: 'value', name: 'only', value: 'tabby', };
 ```
 */
type OptionReading =
  | {
    readonly kind: 'value' | 'list';
    readonly name: string;
    readonly value: string;
  }
    | {
      readonly kind: 'switch';
      readonly name: string;
    }
    | {
      readonly kind: 'refused';
      readonly says: string;
    };

/**
 Whether an argument is a minus sign before digits, which this package's
 runners read as a number rather than as a flag.

 A COUNT TYPED BELOW ZERO STAYS A COUNT. `parseArgs` reads `-3` as the short
 flag `-3`, and no runner declares one, so without this a bench asked for
 `-3` slices would be told it has no flag `-3` rather than that a count cannot
 be below one, which is what was typed (ledger B73). Python's argparse reads
 such an argument as a value for the same reason when no flag looks like a
 number, and no flag here does.

 @param written - argument as typed

 @returns True for a minus sign followed by ASCII digits only

 @example
 ```ts
 isMinusNumber({ written: '-3', },); // true
 ```
 */
function isMinusNumber({ written, }: { readonly written: string; },): boolean {
  return written.startsWith(MINUS,)
    && isAsciiDigits({ text: written.slice(MINUS.length,), },);
}

/**
 The declaration in the form `parseArgs` takes, so a valued flag consumes the
 argument after it and a switch does not.

 @param spec - what the command reads

 @returns Option descriptors keyed by flag name

 @example
 ```ts
 const options = optionsOf({ spec, },);
 ```
 */
function optionsOf({ spec, }: { readonly spec: CommandLineSpec; },): Record<string, ParseArgsOptionDescriptor> {
  /**
   Descriptor of a flag that carries a value.
   */
  const carriesValue: ParseArgsOptionDescriptor = { type: 'string', };

  /**
   Descriptor of a switch.
   */
  const carriesNothing: ParseArgsOptionDescriptor = { type: 'boolean', };

  /**
   Flags that carry a value, written once or any number of times.
   */
  const valuedNames = [
    ...Object.keys(spec.valued,),
    ...Object.keys(spec.repeatable,),
  ];

  /**
   Each declared flag beside its descriptor.
   */
  const options = new Map<string, ParseArgsOptionDescriptor>();
  for (const name of valuedNames) {
    options.set(
      name,
      carriesValue,
    );
  }
  for (const name of Object.keys(spec.switches,)) {
    options.set(
      name,
      carriesNothing,
    );
  }
  return Object.fromEntries(options,);
}

/**
 The usage line a refusal ends with, built from the declaration so it cannot
 drift from what the command reads.

 @param command - command name as typed

 @param spec - what it reads

 @returns Command, then its flags, then its positions; a bracketed part may be
 left off

 @example
 ```ts
 usageOf({ command: 'corpus-pass', spec, },); // 'corpus-pass [--only <entry ids>] [--plan]'
 ```
 */
function usageOf(
  {
    command,
    spec,
  }: {
    readonly command: string;
    readonly spec: CommandLineSpec;
  },
): string {
  /**
   Names of the positions, how many must be written, and whether the last
   repeats.
   */
  const {
    names,
    least,
    rest,
  } = spec.positionals;

  /**
   The command, then each part of the line after it, in the order shown.
   */
  const parts = [command,];
  for (
    const [
      name,
      value,
    ] of Object.entries(spec.valued,)
  )
    parts.push(`[--${name} <${value}>]`,);
  for (
    const [
      name,
      value,
    ] of Object.entries(spec.repeatable,)
  )
    parts.push(`[--${name} <${value}> ...]`,);
  for (const name of Object.keys(spec.switches,))
    parts.push(`[--${name}]`,);
  for (const [at, name,] of names.entries()) {
    /**
     Whether this position must be written.
     */
    const required = at < least;

    /**
     Whether this is the last position and may repeat.
     */
    const repeats = rest && (at === (names.length - 1));

    if (repeats)
      parts.push(required ? `<${name}> [<${name}> ...]` : `[<${name}> ...]`,);
    else
      parts.push(required ? `<${name}>` : `[<${name}>]`,);
  }
  return parts.join(' ',);
}

/**
 Reads one option token against the declaration.

 @param name - flag name as `parseArgs` read it, without dashes or value

 @param flag - flag as the person typed it, without any value written with `=`

 @param carried - what the token carried after the flag

 @param written - whole argument the token came from

 @param spec - what the command reads

 @returns Value or switch it carries, or why it is refused

 @example
 ```ts
 const reading = optionReading({ name: 'only', flag: '--only', carried, written: '--only=tabby', spec, },);
 ```
 */
function optionReading(
  {
    name,
    flag,
    carried,
    written,
    spec,
  }: {
    readonly name: string;
    readonly flag: string;
    readonly carried: Carried;
    readonly written: string;
    readonly spec: CommandLineSpec;
  },
): OptionReading {
  /**
   Whether the declaration names this flag as written once with a value.
   */
  const once = Object.hasOwn(
    spec.valued,
    name,
  );

  /**
   Whether it names this flag as written any number of times with a value.
   */
  const repeatable = Object.hasOwn(
    spec.repeatable,
    name,
  );

  /**
   Whether it names this flag as a switch.
   */
  const isSwitch = Object.hasOwn(
    spec.switches,
    name,
  );

  if (once || repeatable) {
    /**
     What was written after the flag, empty when nothing was, which is
     refused the same way.
     */
    const text = (carried.kind === 'text') ? carried.text : '';

    // NOTHING AFTER IT, AN EMPTY ARGUMENT, OR THE NEXT FLAG standing where its
    // value should be: each once read as unwritten and took the default nobody
    // asked for (ledger B73).
    if ((text === '') || text.startsWith(LONG_PREFIX,))
      return {
        kind: 'refused',
        says: `${flag} needs a value written after it`,
      };
    return {
      kind: once ? 'value' : 'list',
      name,
      value: text,
    };
  }

  if (isSwitch) {
    if (carried.kind === 'text')
      return {
        kind: 'refused',
        says: `${flag} takes no value, and ${written} gives it one`,
      };
    return {
      kind: 'switch',
      name,
    };
  }

  return {
    kind: 'refused',
    says: `${flag.startsWith(LONG_PREFIX,) ? flag : written} is not a flag this command reads`,
  };
}

/**
 Refusals for flags written more than once and for positions written past or
 short of what the command reads.

 @param timesWritten - how often each once-only flag was written

 @param positionals - what was written by position

 @param spec - what the command reads

 @returns One refusal per problem, empty when there is none

 @example
 ```ts
 const said = countRefusals({ timesWritten, positionals, spec, },);
 ```
 */
function countRefusals(
  {
    timesWritten,
    positionals,
    spec,
  }: {
    readonly timesWritten: ReadonlyMap<string, number>;
    readonly positionals: readonly string[];
    readonly spec: CommandLineSpec;
  },
): readonly string[] {
  /**
   Names of the positions, as declared.
   */
  const {
    names,
    least,
    rest,
  } = spec.positionals;

  /**
   Flags written twice or more, each a refusal: a second value once replaced
   or was dropped in favour of the first, without a word either way.
   */
  const repeated = [...timesWritten,]
    .filter(function isRepeated([, times,]: ReadonlyEntry<string, number>,): boolean {
      return times > 1;
    },)
    .map(function repeatRefusal([name, times,]: ReadonlyEntry<string, number>,): string {
      return `--${name} is written ${String(times,)} times, and is read once`;
    },);

  /**
   Arguments past the last position the command reads.
   */
  const extra = rest
    ? []
    : positionals.slice(names.length,);

  /**
   Positions that must be written and were not, as the usage line shows them.
   */
  const missing = names
    .slice(
    positionals.length,
    least,
    )
    .map(function shown(name,): string {
      return `<${name}>`;
    },);

  /**
   Last position read, after which nothing more is.
   */
  const last = names.at(-1,);

  /**
   The arguments past it, each quoted so a blank or spaced one shows.
   */
  const shownExtra = extra
    .map(function quoted(argument,): string {
      return JSON.stringify(argument,);
    },)
    .join(' ',);

  /**
   Refusal of arguments past the last position, empty when there are none.
   */
  const past = (extra.length === 0)
    ? []
    : [
      (last === undefined)
        ? `this command takes no argument but its flags, and was given ${shownExtra}`
        : `this command takes nothing after <${last}>, and was given ${shownExtra}`,
    ];

  return [
    ...repeated,
    ...past,
    ...((missing.length === 0) ? [] : [`this command needs ${missing.join(' ',)}`,]),
  ];
}

/**
 Reads a runner's whole command line against what it declares.

 @param command - command name as typed, which starts the usage line

 @param spec - what it reads

 @param argv - process arguments: the runtime, the script, then what was typed

 @returns Flags, switches and positions written, offering only those
 declared

 @throws StatedRefusalError naming every problem at once, then the usage line:
 a flag the command does not read, a valued flag written with nothing usable
 after it, a switch given a value, a once-only flag written twice, an argument
 past the last position read, or a required position left out

 @example
 ```ts
 const line = readCommandLine({ command: 'corpus-pass', spec: COMMAND_LINES['corpus-pass'], argv: process.argv, },);
 ```
 */
export function readCommandLine<const Spec extends CommandLineSpec>(
  {
    command,
    spec,
    argv,
  }: {
    readonly command: string;
    readonly spec: Spec;
    readonly argv: readonly string[];
  },
): CommandLineFor<Spec> {
  /**
   Script the runtime ran.
   */
  const script = nonNullishOrThrow(argv[SCRIPT_AT],);

  /**
   What the person typed after it.
   */
  const typed = argv.slice(FIRST_TYPED_AT,);

  /**
   The line split into flags, values and positions.
   */
  const { tokens, } = parseArgs({
    args: [...typed,],
    options: optionsOf({ spec, },),
    strict: false,
    allowPositionals: true,
    tokens: true,
  },);

  /**
   Why the line is refused, in the order the problems were written.
   */
  const refusals: string[] = [];

  /**
   Value of each once-only flag written.
   */
  const values = new Map<string, string>();

  /**
   Values of each repeatable flag, in the order written.
   */
  const lists = new Map<string, string[]>();

  /**
   Switches written.
   */
  const switches = new Set<string>();

  /**
   How often each once-only flag or switch was written.
   */
  const timesWritten = new Map<string, number>();

  /**
   What was written by position.
   */
  const positionals: string[] = [];

  /**
   Arguments already read whole, since one grouped argument splits into
   several tokens that share its position.
   */
  const readWhole = new Set<number>();

  for (const token of tokens) {
    if (token.kind === 'option-terminator')
      continue;
    if (token.kind === 'positional') {
      positionals.push(token.value,);
      continue;
    }

    /**
     The whole argument this token came from.
     */
    const written = nonNullishOrThrow(typed[token.index],);

    if (readWhole.has(token.index,))
      continue;

    if (isMinusNumber({ written, },)) {
      readWhole.add(token.index,);
      positionals.push(written,);
      continue;
    }

    /**
     What this token reads as.
     */
    const reading = optionReading({
      name: token.name,
      flag: token.rawName,
      carried: (token.value === undefined)
        ? { kind: 'nothing', }
        : {
          kind: 'text',
          text: token.value,
        },
      written,
      spec,
    },);

    if (reading.kind === 'refused') {
      // A grouped short argument is refused once, whole, as it was typed.
      readWhole.add(token.index,);
      refusals.push(reading.says,);
      continue;
    }

    if (reading.kind === 'list') {
      /**
       Values this flag carried before this one.
       */
      const earlier = lists.get(reading.name,);
      if (earlier === undefined)
        lists.set(
          reading.name,
          [reading.value,],
        );
      else
        earlier.push(reading.value,);
      continue;
    }

    timesWritten.set(
      reading.name,
      (timesWritten.get(reading.name,) ?? 0) + 1,
    );
    if (reading.kind === 'switch')
      switches.add(reading.name,);
    else
      values.set(
        reading.name,
        reading.value,
      );
  }

  refusals.push(...countRefusals({
    timesWritten,
    positionals,
    spec,
  },),);

  if (refusals.length > 0)
    throw new StatedRefusalError({
      says: `${refusals.join('; ',)}. Usage: ${usageOf({
        command,
        spec,
      },)}`,
    },);

  return {
    script,
    flag: function flag(name: string,): FlagValue {
      /**
       What the flag carried, absent when it was not written.
       */
      const value = values.get(name,);
      return (value === undefined)
        ? {
          kind: 'unwritten',
          flag: `--${name}`,
        }
        : {
          kind: 'written',
          flag: `--${name}`,
          value,
        };
    },
    list: function list(name: string,): readonly string[] {
      return lists.get(name,) ?? [];
    },
    switched: function switched(name: string,): boolean {
      return switches.has(name,);
    },
    positionals,
  };
}

//endregion Command line
