/**
 The options the three gate commands share: how a command line is split into named values and positionals, and the caps
 one gate is started with.

 Options come before the positionals and are long form (`AGENTS.md`, rule `LFF`), each takes exactly one value, and an
 option a command does not know is refused instead of being read as a positional: a mistyped `--worktre` would otherwise
 become the worktree a gate runs in.
 */
import {
  parseArgs,
  type ParseArgsOptionDescriptor,
  type ParseArgsOptionsConfig,
} from 'node:util';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';

import {
  type GateCaps,
  checkedGateCaps,
  defaultGateCaps,
} from './native-gate-caps.ts';
import { NativeGateError } from './native-gate-error.ts';

/**
 Option that names the unit's CPU quota.
 */
export const cpuQuotaOption = '--unit-cpu-quota';

/**
 Option that names the unit's memory cap.
 */
export const memoryMaxOption = '--unit-memory-max';

/**
 Option that names the unit's task limit.
 */
export const tasksMaxOption = '--unit-tasks-max';

/**
 Option that names the unit's disk weight.
 */
export const ioWeightOption = '--unit-io-weight';

/**
 Option that names the container's process limit.
 */
export const containerPidsLimitOption = '--container-pids-limit';

/**
 Every option that overrides a gate's caps, in the order a refusal lists them.
 */
export const gateCapsOptionNames: readonly string[] = [
  cpuQuotaOption,
  memoryMaxOption,
  tasksMaxOption,
  ioWeightOption,
  containerPidsLimitOption,
];

/**
 One command line, split into the values it names and what is left.

 `positionals` holds the arguments after the options, and after a `--` if one was given.
 */
export type GateOptions = {
  /**
   Option values by their flag, in the order they were given.
   */
  readonly values: ReadonlyMap<string, string>;
  /**
   Arguments that are not option values.
   */
  readonly positionals: readonly string[];
};

/**
 Every flag the command line gives more than once.

 `parseArgs` keeps only the last value of a repeated option, and which of the two values names the gate would then be
 nobody's decision, so the repetition is refused before the parse.

 @param argv - the arguments after the script path

 @param knownOptions - every option this command takes

 @returns the repeated flags, in the order they were first given

 @example
 ```ts
 repeatedFlags({ argv: ['--repository', '/a', '--repository', '/b'], knownOptions: ['--repository'] });
 ```
 */
export function repeatedFlags({
  argv,
  knownOptions,
}: {
  readonly argv: readonly string[];
  readonly knownOptions: readonly string[];
}): readonly string[] {
  /**
   Where a `--` ends the options, past which a flag-shaped argument is a positional.
   */
  const separatorIndex = argv.indexOf('--');
  /**
   The flag-shaped arguments, which are the only ones a repeat can come from.
   */
  const flags = (separatorIndex === (-1) ? argv : argv.slice(
    0,
    separatorIndex
  ))
    .map(function flagOf(argument) {
      /**
       Where the argument's flag ends and an attached value begins.
       */
      const separator = argument.indexOf('=');
      return separator === (-1) ? argument : argument.slice(
        0,
        separator,
      );
    })
    .filter(function isKnown(flag) {
      return knownOptions.includes(flag);
    });
  /**
   How often each flag was given.
   */
  const counts = flags.reduce(
    function count(
      counted,
      flag
    ) {
      counted.set(
        flag,
        (counted.get(flag) ?? 0) + 1,
      );
      return counted;
    },
    new Map<string, number>()
  );
  /**
   The repeated flags, in the order the command lists its options, so a refusal reads the same twice.
   */
  const repeated: string[] = [];
  for (const flag of knownOptions)
    if ((counts.get(flag) ?? 0) > 1)
      repeated.push(flag);
  return repeated;
}

/**
 Tells whether a caught value is `parseArgs` refusing the command line.

 @param error - the caught value

 @returns true for the errors `parseArgs` throws in strict mode

 @example
 ```ts
 isParseArgsError({ error: new Error('nope') });
 ```
 */
function isParseArgsError({ error }: { readonly error: unknown; }): boolean {
  /**
   The code the error carries, when the value carries one.
   */
  const code: unknown = ((typeof error) === 'object') && (error !== null) ? Reflect.get(
    error,
    'code',
  ) : undefined;
  return (code === 'ERR_PARSE_ARGS_UNKNOWN_OPTION') || (code === 'ERR_PARSE_ARGS_INVALID_OPTION_VALUE');
}

/**
 Splits a command line into named values and positionals.

 @param argv - the arguments after the script path

 @param knownOptions - every option this command takes that takes one value

 @param flagOptions - the options that take no value, whose presence alone is the answer

 @returns the values by flag, a flag's own value being `true`, and the positionals

 @throws NativeGateError when an option is unknown, has no value, or is given twice

 @example
 ```ts
 gateOptions({ argv: ['--worktree', 'gate-wrapper-next-alpha'], knownOptions: ['--worktree'] });
 ```
 */
export function gateOptions({
  argv,
  knownOptions,
  flagOptions = [],
}: {
  readonly argv: readonly string[];
  readonly knownOptions: readonly string[];
  readonly flagOptions?: readonly string[];
}): GateOptions {
  /**
   The flags given more than once, which a parse would silently resolve.
   */
  const repeated = repeatedFlags({
    argv,
    knownOptions,
  });
  if (repeated.length > 0)
    throw new NativeGateError(`${repeated.join(', ')} is given twice, so which of the two values names the gate is unknown.`);
  /**
   Every option this command takes, which a refusal lists.
   */
  const everyOption = [
    ...knownOptions,
    ...flagOptions,
  ];
  /**
   What `parseArgs` is told each option is: one value, or none at all for a flag, named without the dashes it is typed with.
   */
  const entries: readonly (readonly [
    string,
    ParseArgsOptionDescriptor
  ])[] = [
    ...knownOptions.map(function valueSpec(flag) {
      return [
        flag.slice('--'.length),
        { type: 'string', },
      ] as const;
    }),
    ...flagOptions.map(function flagSpec(flag) {
      return [
        flag.slice('--'.length),
        { type: 'boolean', },
      ] as const;
    }),
  ];
  /**
   The option table `parseArgs` takes.
   */
  const specs: ParseArgsOptionsConfig = Object.fromEntries(entries);
  try {
    /**
     The parsed command line, which the standard library splits at a `--` and refuses a flag it was not given.
     */
    const parsed = parseArgs({
      args: [...argv,],
      options: specs,
      allowPositionals: true,
      strict: true,
    });
    return {
      values: new Map(Object.entries(parsed.values)
        .map(function named([key, value,]) {
          return [
            `--${key}`,
            String(value),
          ] as const;
        })),
      positionals: parsed.positionals,
    };
  }
  catch (error) {
    if (!isParseArgsError({ error, }))
      throw error;
    throw new NativeGateError(
      `${caughtValueText(error)} This command takes ${everyOption.length === 0 ? 'no options' : everyOption.join(', ')}, and everything after -- is a positional argument.`,
      { cause: error, }
    );
  }
}

/**
 Tells whether text is a whole number from zero, scanned by index rather than matched, so no regular expression decides it.

 @param text - the text the command line gave

 @returns true when every character is a digit and there is at least one

 @example
 ```ts
 isWholeNumberText('3600');
 ```
 */
export function isWholeNumberText(text: string): boolean {
  if (text === '')
    return false;
  for (let index = 0; index < text.length; index += 1) {
    /**
     The character at this position.
     */
    const character = text.charAt(index);
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Reads one whole number from a positional argument.

 @param text - what the caller typed

 @param name - what the argument is called, for the refusal

 @returns the number

 @throws NativeGateError when the text is not a whole number from zero

 @example
 ```ts
 wholeNumber({ text: '3600', name: 'the deadline' });
 ```
 */
export function wholeNumber({
  text,
  name,
}: {
  readonly text: string;
  readonly name: string;
}): number {
  if (!isWholeNumberText(text))
    throw new NativeGateError(`${name} "${text}" is not a whole number from zero.`);
  return Number(text);
}

/**
 Reads one text option.

 @param options - the parsed command line

 @param name - the flag to read

 @param fallback - what the option is when the command line does not name it

 @returns the value given, or the fallback

 @example
 ```ts
 textOption({ options, name: '--repository', fallback: '/repo' });
 ```
 */
export function textOption({
  options,
  name,
  fallback,
}: {
  readonly options: GateOptions;
  readonly name: string;
  readonly fallback: string;
}): string {
  return options.values
    .get(name)
    ?? fallback;
}

/**
 Reads one whole-number option.

 @param options - the parsed command line

 @param name - the flag to read

 @param fallback - what the option is when the command line does not name it

 @returns the number given, or the fallback

 @throws NativeGateError when the value is not a whole number from zero

 @example
 ```ts
 numberOption({ options, name: '--poll-milliseconds', fallback: 20_000 });
 ```
 */
export function numberOption({
  options,
  name,
  fallback,
}: {
  readonly options: GateOptions;
  readonly name: string;
  readonly fallback: number;
}): number {
  /**
   The value the command line gave, when it gave one.
   */
  const given = options.values
    .get(name);
  if (given === undefined)
    return fallback;
  return wholeNumber({
    text: given,
    name,
  });
}

/**
 Reads the caps one gate is started with, every one of them overridable.

 @param options - the parsed command line

 @returns the caps, checked against what systemd can read

 @throws NativeGateError when a cap is not a value systemd reads

 @example
 ```ts
 gateCapsFromOptions({ options });
 ```
 */
export function gateCapsFromOptions({ options }: { readonly options: GateOptions; }): GateCaps {
  return checkedGateCaps({
    caps: {
      unitCpuQuota: textOption({
        options,
        name: cpuQuotaOption,
        fallback: defaultGateCaps.unitCpuQuota,
      }),
      unitMemoryMax: textOption({
        options,
        name: memoryMaxOption,
        fallback: defaultGateCaps.unitMemoryMax,
      }),
      unitTasksMax: numberOption({
        options,
        name: tasksMaxOption,
        fallback: defaultGateCaps.unitTasksMax,
      }),
      unitIoWeight: numberOption({
        options,
        name: ioWeightOption,
        fallback: defaultGateCaps.unitIoWeight,
      }),
      containerPidsLimit: numberOption({
        options,
        name: containerPidsLimitOption,
        fallback: defaultGateCaps.containerPidsLimit,
      }),
    },
  });
}
