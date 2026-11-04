import {
  decimalDigits,
  isPositiveWholeNumber,
  usesOnly,
} from '@monochromatic-dev/shell-argument-core/ts/character-set.ts';
import {
  buildTimeoutFlag,
  examineFlag,
  passesFlag,
  timeoutFlag,
} from './constants.ts';
import {
  defaultBuildTimeoutSeconds,
  maximumBuildTimeoutSeconds,
} from './campaign-files.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Characters a mutation scope's file-name glob may contain between the product's prefix and suffix.
 */
const scopeNameCharacters = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_*?.-';

/**
 Characters of a `--examine` pattern: names, alternation, grouping and the wildcards of the mutant names.
 */
const examineCharacters = String.raw`ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_|:.*?()\ -`;

/**
 Lowercase letters and the underscore: the characters a Rust test-path segment may start with.
 */
const segmentStartCharacters = 'abcdefghijklmnopqrstuvwxyz_';

/**
 Segments a Rust test path has at least: a module and a test name.
 */
const minimumTestPathSegments = 2;

/**
 Rules a product gives the campaign option parser.
 */
export type TimeoutRules = {
  /**
   Test timeout, in seconds, that applies to each mutant when no `--timeout` option is given.
   */
  readonly defaultTimeoutSeconds: number;
  /**
   Largest test timeout, in seconds, a `--timeout` option may request.
   */
  readonly maximumTimeoutSeconds: number;
  /**
   Build timeout, in seconds, that applies to each mutant when no `--build-timeout` option is given; the shared
   default when absent.
   */
  readonly defaultBuildTimeoutSeconds?: number;
  /**
   Largest build timeout, in seconds, a `--build-timeout` option may request; the shared maximum when absent.
   */
  readonly maximumBuildTimeoutSeconds?: number;
  /**
   Whether the product runs a two-pass campaign for `--passes 2`; without it `--passes` is left for the scope parser,
   which rejects it.
   */
  readonly acceptPasses?: boolean;
};

/**
 Parsed `--timeout`, `--build-timeout` and `--examine` options and the arguments left over.
 */
export type TimeoutAndExamine = {
  readonly timeoutSeconds: number;
  readonly buildTimeoutSeconds: number;
  readonly examine: readonly string[];
  readonly scopeOptions: readonly string[];
  /**
   Passes the campaign runs, 1 unless `--passes 2` is given; absent when the product does not accept the option.
   */
  readonly passes?: 1 | 2;
};

/**
 Read one timeout value, whichever flag carried it.

 @param flag - the flag that carried the value, named in the message
 
 @param value - text after the flag
 
 @param maximumSeconds - largest accepted timeout
 
 @returns the whole number of seconds it names
 
 @throws CargoMutantsToolingError when the value is absent, not a positive whole number, or over the maximum
 
 @example
 ```ts
 secondsOf({ flag: '--timeout', value: '60', maximumSeconds: 3600 });
 ```
 */
function secondsOf({
  flag,
  value,
  maximumSeconds,
}: {
  readonly flag: string;
  readonly value?: string;
  readonly maximumSeconds: number;
}): number {
  if ((value === undefined) || (!isPositiveWholeNumber({
    text: value,
    maximumDigits: Number.POSITIVE_INFINITY,
  }))
    || (Number(value) > maximumSeconds))
    throw new CargoMutantsToolingError(`${flag} must be a whole number of seconds from 1 to ${maximumSeconds}, got ${value}.`);
  return Number(value);
}

/**
 Check one `--examine` pattern, for the campaign parser and the launcher alike.

 @param value - text after the flag
 
 @returns the pattern, unchanged
 
 @throws CargoMutantsToolingError when the value is absent or holds a character outside the plain set
 
 @example
 ```ts
 examinePatternOf('replace foo|bar');
 ```
 */
export function examinePatternOf(value?: string): string {
  if ((value === undefined) || (!usesOnly({
    text: value,
    allowed: examineCharacters,
  })))
    throw new CargoMutantsToolingError(`${examineFlag} must be a regular expression of plain characters, got ${value}.`);
  return value;
}

/**
 Check a `--passes` value, for the campaign parser and the launcher alike.

 @param value - text after the flag

 @returns 1 or 2

 @throws CargoMutantsToolingError when the value is absent or is not exactly `1` or `2`

 @example
 ```ts
 passesOf('2');
 ```
 */
export function passesOf(value?: string): 1 | 2 {
  if (value === '1')
    return 1;
  if (value === '2')
    return 2;
  throw new CargoMutantsToolingError(`${passesFlag} must be 1 or 2, got ${value}.`);
}

/**
 Split `--timeout <seconds>`, `--build-timeout <seconds>` and the repeated `--examine <regex>` from the other options;
 the last `--timeout` and the last `--build-timeout` win.

 @param options - command-line arguments after the script path
 
 @param rules - the product's default and maximum timeouts
 
 @returns the test timeout, the build timeout, the mutant-name patterns of the repeated `--examine <regex>` options (only matching mutants are
 tried), the passes when the product accepts `--passes <1|2>`, and the arguments left for the scope parser
 
 @throws CargoMutantsToolingError when a timeout or pattern is malformed
 
 @example
 ```ts
 parseTimeoutAndExamine({ options: ['--timeout', '60'], rules: { defaultTimeoutSeconds: 300, maximumTimeoutSeconds: 3600 } });
 ```
 */
export function parseTimeoutAndExamine({
  options,
  rules,
}: {
  readonly options: readonly string[];
  readonly rules: TimeoutRules;
}): TimeoutAndExamine {
  /**
   Every `--timeout` value in order, of which the last wins.
   */
  const timeouts: number[] = [];
  /**
   Every `--build-timeout` value in order, of which the last wins.
   */
  const buildTimeouts: number[] = [];
  /**
   Arguments that are not `--timeout` or `--examine`, left for the scope parser to accept or reject.
   */
  const scopeOptions: string[] = [];
  /**
   Mutant-name patterns of the `--examine` options.
   */
  const examine: string[] = [];
  /**
   Every `--passes` value in order, of which the last wins.
   */
  const passes: (1 | 2)[] = [];
  for (let index = 0; index < options.length; index += 2) {
    /**
     The flag at this position.
     */
    const flag = options[index];
    /**
     The text after the flag, absent when the flag ends the arguments.
     */
    const value = options[index + 1];
    if (flag === examineFlag)
      examine.push(examinePatternOf(value));
    else if (flag === timeoutFlag)
      timeouts.push(secondsOf({
        flag: timeoutFlag,
        ...(value === undefined ? {} : { value }),
        maximumSeconds: rules.maximumTimeoutSeconds,
      }));
    else if (flag === buildTimeoutFlag)
      buildTimeouts.push(secondsOf({
        flag: buildTimeoutFlag,
        ...(value === undefined ? {} : { value }),
        maximumSeconds: rules.maximumBuildTimeoutSeconds ?? maximumBuildTimeoutSeconds,
      }));
    else if ((flag === passesFlag) && (rules.acceptPasses === true))
      passes.push(passesOf(value));
    else
      scopeOptions.push(
        flag ?? '',
        value ?? '',
      );
  }
  /**
   Build timeout of the product when no `--build-timeout` option is given.
   */
  const fallbackBuildTimeout = rules.defaultBuildTimeoutSeconds ?? defaultBuildTimeoutSeconds;
  return {
    timeoutSeconds: timeouts.at(-1) ?? rules.defaultTimeoutSeconds,
    buildTimeoutSeconds: buildTimeouts.at(-1) ?? fallbackBuildTimeout,
    examine,
    scopeOptions,
    ...(rules.acceptPasses === true ? { passes: passes.at(-1) ?? 1 } : {}),
  };
}

/**
 Where a product's mutation scopes may point.
 */
export type ScopeRules = {
  /**
   Directory every scope must name, such as `src/native/`.
   */
  readonly prefix: string;
  /**
   Extension every scope must name, such as `.rs`.
   */
  readonly suffix: string;
};

/**
 Check that a mutation scope is a file glob between the product's prefix and suffix, with one linear scan.

 @param scope - `--file` value from the command line
 
 @param rules - the product's prefix and suffix
 
 @returns true when the name between prefix and suffix is non-empty and every `/`-separated segment of it is non-empty,
   is not `.` or `..`, and uses only glob-safe characters, so a subdirectory of the product directory is a valid scope
   and a path out of it is not
 
 @example
 ```ts
 isSourceGlob({ scope: 'src/native/lock.rs', rules: { prefix: 'src/native/', suffix: '.rs' } });
 ```
 */
function isSourceGlob({
  scope,
  rules,
}: {
  readonly scope: string;
  readonly rules: ScopeRules;
}): boolean {
  if ((!scope.startsWith(rules.prefix)) || (!scope.endsWith(rules.suffix)))
    return false;
  /**
   The part between prefix and suffix, which may name subdirectories of the product directory.
   */
  const name = scope.slice(
    rules.prefix
      .length,
    scope.length
      - rules.suffix
        .length,
  );
  return name.split('/')
    .every(function isSegment(segment) {
      return (segment !== '') && (segment !== '.') && (segment !== '..') && usesOnly({
        text: segment,
        allowed: scopeNameCharacters,
      });
    });
}

/**
 Accept only repeated `--file <glob>` scopes; anything else is a usage error.

 @param options - command-line arguments left after the timeout and examine options
 
 @param rules - the product's prefix and suffix
 
 @returns scopes in command-line order, forwarded to cargo-mutants unchanged
 
 @throws CargoMutantsToolingError when an option is not `--file <glob>` or a glob is outside the product's directory
 
 @example
 ```ts
 parseFileScopes({ options: ['--file', 'src/native/lock.rs'], rules: { prefix: 'src/native/', suffix: '.rs' } });
 ```
 */
export function parseFileScopes({
  options,
  rules,
}: {
  readonly options: readonly string[];
  readonly rules: ScopeRules;
}): string[] {
  /**
   Accepted `--file` globs in command-line order.
   */
  const scopes: string[] = [];
  for (let index = 0; index < options.length; index += 2) {
    /**
     The flag at this position, which must be `--file`.
     */
    const flag = options[index];
    /**
     The glob after the flag, absent when the flag ends the arguments.
     */
    const scope = options[index + 1];
    if ((flag !== '--file') || (scope === undefined))
      throw new CargoMutantsToolingError('Only repeated --file <glob> options are accepted.');
    if (!isSourceGlob({
      scope,
      rules,
    }))
      throw new CargoMutantsToolingError(`--file must be a ${rules.prefix}*${rules.suffix} glob, got ${scope}.`);
    scopes.push(scope);
  }
  return scopes;
}

/**
 Check for a Rust test path: two or more `::`-separated segments, each starting with a lowercase letter or underscore.

 @param text - value of the baseline-skip environment variable
 
 @returns true for `module::tests::name`
 
 @example
 ```ts
 isTestPath('module::tests::name');
 ```
 */
function isTestPath(text: string): boolean {
  /**
   The `::`-separated segments of the path.
   */
  const segments = text.split('::');
  return (segments.length >= minimumTestPathSegments) && segments.every(function isSegment(segment) {
    return usesOnly({
      text: segment,
      allowed: segmentStartCharacters + decimalDigits,
    }) && segmentStartCharacters.includes(segment[0] ?? '');
  });
}

/**
 Test-path arguments that skip one named test in the baseline and every mutant run.

 Opt-in through an environment variable that names one test path such as `module::tests::name`.
 Unset or empty, the campaign runs every test.

 @param variable - name of the environment variable, for the message
 
 @param value - its value, or absent when it is unset
 
 @returns `['--', '--', '--skip', name]` or no arguments
 
 @throws CargoMutantsToolingError when the value is not a test path
 
 @example
 ```ts
 baselineSkipArguments({ variable: 'NATIVE_MUTATION_BASELINE_SKIP' });
 ```
 */
export function baselineSkipArguments({
  variable,
  value,
}: {
  readonly variable: string;
  readonly value?: string;
}): string[] {
  if ((value === undefined) || (value === ''))
    return [];
  if (!isTestPath(value))
    throw new CargoMutantsToolingError(`${variable} must be a test path, got ${value}.`);
  return [
    '--',
    '--',
    '--skip',
    value,
  ];
}
