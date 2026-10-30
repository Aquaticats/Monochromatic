import {
  parseArgs,
  type ParseArgsOptionsConfig,
} from 'node:util';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import { CrateReleaseError } from './error.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';

/**
 Options and positional arguments of one command line, before any value is validated.
 */
export type RawArguments = {
  /**
   Values of the options that may appear once, by long name.
   */
  readonly singles: ReadonlyMap<string, string>;
  /**
   Values of the options that may repeat, by long name, in command-line order.
   */
  readonly repeated: ReadonlyMap<string, readonly string[]>;
  /**
   Arguments that are not options, in order.
   */
  readonly positionals: readonly string[];
};

/**
 Check that an option value is text; the option definitions only declare string options, so a boolean never occurs.

 @param value - value as `parseArgs` delivers it

 @returns true for text

 @example
 ```ts
 isText('a');
 ```
 */
function isText(value: unknown): value is string {
  return (typeof value) === 'string';
}

/**
 Split a command line into options and positional arguments, refusing unknown options and options without a value.

 A positional argument that starts with a hyphen goes after a lone `--`.

 @param argv - arguments after the script path

 @param options - option definitions in the `node:util` `parseArgs` format

 @returns the raw values and positionals

 @throws CrateReleaseError naming the problem option when the command line is malformed

 @example
 ```ts
 splitArguments({ argv: ['--scope', 'x', 'a'], options: { scope: { type: 'string' } } });
 ```
 */
export function splitArguments({
  argv,
  options,
}: {
  readonly argv: readonly string[];
  readonly options: ParseArgsOptionsConfig;
}): RawArguments {
  try {
    /**
     Values and positionals as `parseArgs` delivers them.
     */
    const parsed = parseArgs({
      args: [...argv],
      options,
      allowPositionals: true,
      strict: true,
    });
    /**
     Every given option with its value or values.
     */
    const entries: readonly (readonly [
      string,
      unknown
    ])[] = Object.entries(parsed.values);
    return {
      singles: new Map(entries.flatMap(function singleEntry([name, value]): [
        string,
        string
      ][] {
        return isText(value) ? [[
          name,
          value
        ]] : [];
      })),
      repeated: new Map(entries.flatMap(function repeatedEntry([name, value]): [
        string,
        readonly string[]
      ][] {
        return Array.isArray(value) ? [[
          name,
          value.filter(isText)
        ]] : [];
      })),
      positionals: parsed.positionals,
    };
  }
  catch (error) {
    throw new CrateReleaseError(
      `The command line is malformed: ${caughtValueText(error)}`,
      { cause: error },
    );
  }
}

/**
 Look for an option that may appear once.

 @param raw - split command line

 @param name - long option name without the hyphens

 @returns the text value, or a miss when the option was not given

 @example
 ```ts
 optionValue({ raw, name: 'scope' });
 ```
 */
export function optionValue({
  raw,
  name,
}: {
  readonly raw: RawArguments;
  readonly name: string;
}): Lookup<string> {
  /**
   Value as given.
   */
  const value = raw.singles
    .get(name);
  return value === undefined ? missingValue(`--${name} was not given.`) : foundValue(value);
}

/**
 Read an option that may appear once, or its default when absent.

 @param raw - split command line

 @param name - long option name without the hyphens

 @param fallback - value used when the option was not given

 @returns the given value or the fallback

 @example
 ```ts
 valueOr({ raw, name: 'memory-max', fallback: '12G' });
 ```
 */
export function valueOr({
  raw,
  name,
  fallback,
}: {
  readonly raw: RawArguments;
  readonly name: string;
  readonly fallback: string;
}): string {
  return raw.singles
    .get(name)
    ?? fallback;
}

/**
 Read an option that may repeat.

 @param raw - split command line

 @param name - long option name without the hyphens

 @returns every given value in command-line order, empty when the option was not given

 @example
 ```ts
 repeatedValues({ raw, name: 'pin-file' });
 ```
 */
export function repeatedValues({
  raw,
  name,
}: {
  readonly raw: RawArguments;
  readonly name: string;
}): readonly string[] {
  return raw.repeated
    .get(name)
    ?? [];
}

/**
 Read an option that must be present.

 @param raw - split command line

 @param name - long option name without the hyphens

 @returns the text value

 @throws CrateReleaseError naming the option when it is absent

 @example
 ```ts
 requiredValue({ raw, name: 'crate-name' });
 ```
 */
export function requiredValue({
  raw,
  name,
}: {
  readonly raw: RawArguments;
  readonly name: string;
}): string {
  /**
   The value, when given.
   */
  const lookup = optionValue({
    raw,
    name
  });
  if (!lookup.found)
    throw new CrateReleaseError(`--${name} is required.`);
  return lookup.value;
}

/**
 Check whether the command line asks for help, looking only at options (before a lone `--`), so a note that happens to
 read `--help` is still a note.

 @param argv - arguments after the script path

 @returns true when `--help` appears before the end of the options

 @example
 ```ts
 wantsHelp(['--help']);
 ```
 */
export function wantsHelp(argv: readonly string[]): boolean {
  /**
   Position of the lone `--` that ends the options.
   */
  const end = argv.indexOf('--');
  return (end === (-1) ? argv : argv.slice(
    0,
    end
  )).includes('--help');
}

/**
 Resolve the repository root: the `--root` option when given, otherwise the default, which is looked up only then.

 @param raw - split command line

 @param root - the default root, or a function that finds it and may throw

 @returns the root as given or found

 @example
 ```ts
 rootOf({ raw, root: '/repo' });
 ```
 */
export function rootOf({
  raw,
  root,
}: {
  readonly raw: RawArguments;
  readonly root: string | (() => string);
}): string {
  /**
   Root named on the command line.
   */
  const named = optionValue({
    raw,
    name: 'root'
  });
  if (named.found)
    return named.value;
  return (typeof root) === 'string' ? root : (root as () => string)();
}
