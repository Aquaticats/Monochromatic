import type { UsageOption, } from './usage-declarations.ts';

//region Preserve option values and literal passthrough words

/** A task declaration cannot interpret the supplied argument words. */
class TaskUsageError extends Error {
  /** @param message - Invalid option or missing value with its task-facing spelling. */
  constructor(message: string,) {
    super(message,);
    this.name = 'TaskUsageError';
  }
}

/**
 Separates declared options from literal passthrough after the second separator.

 @param argv - Arguments following the task name.
 @returns Option words and raw passthrough words.
 @example
 ```ts
 const { options, rest } = splitPassthrough(['--', '--mode', 'debug', '--', 'a b']);
 ```
 */
export function splitPassthrough(argv: readonly string[],): {
  readonly options: readonly string[];
  readonly rest: readonly string[];
} {
  /** The leading separator belongs to the task command itself. */
  const afterSeparator = argv[0] === '--' ? argv.slice(1,) : argv;
  /** A later separator begins words the task must receive literally. */
  const index = afterSeparator.indexOf('--',);
  if (index < 0)
    return { options: afterSeparator, rest: [], };
  return { options: afterSeparator.slice(0, index,), rest: afterSeparator.slice(index + 1,), };
}

/**
 Quotes words for the restricted shell-word grammar consumed by parseUsageArgs.

 @param word - Literal argument word.
 @returns Reversible encoding for the usage_args environment variable.
 */
function shellQuote(word: string,): string {
  if (word.length === 0)
    return "''";
  for (const character of word) {
    if (character.trim() === '' || "'\"\\".includes(character,)) {
      return `'${word.replaceAll("'", "'\\''",)}'`;
    }
  }
  return word;
}

/**
 Joins encoded words without losing empty values, whitespace, quotes or backslashes.

 @param words - Literal argument vector.
 @returns Value recoverable by parseUsageArgs.
 */
function shellJoin(words: readonly string[],): string {
  return words.map(shellQuote,).join(' ',);
}

/**
 Interprets declared options while preserving positional and passthrough arguments.

 @param options - Parsed usage declarations.
 @param argv - Task option arguments.
 @param rest - Passthrough words after the separator.
 @returns Environment entries consumed by the task body.
 @throws TaskUsageError when arguments do not match the declarations.
 @example
 ```ts
 const env = usageEnvironment({ options: [], argv: ['a b'], rest: [] });
 ```
 */
export function usageEnvironment({ options, argv, rest, }: {
  readonly options: readonly UsageOption[];
  readonly argv: readonly string[];
  readonly rest: readonly string[];
},): Record<string, string> {
  /** Only declaration-derived environment entries are returned to the caller. */
  const env: Record<string, string> = {};
  if (options.length === 0) {
    /** Undeclared tasks receive all words both as argv and through usage_args. */
    const raw = [...argv, ...rest,];
    if (raw.length > 0)
      env.usage_args = shellJoin(raw,);
    return env;
  }
  /** Positional words retain order even when flags appear between them. */
  const positional: string[] = [];
  for (let index = 0; index < argv.length;) {
    /** Index advances once for a flag and twice for a flag with its value. */
    const token = argv[index];
    if (token === undefined)
      throw new TaskUsageError('task-cli: argument list changed while parsing',);
    if (!token.startsWith('--')) {
      positional.push(token,);
      index += 1;
      continue;
    }
    /** Command-line spelling drops only the leading dashes. */
    const name = token.slice(2,);
    /** Each option is resolved against the task's declarations. */
    const option = options.find(function sameFlag(candidate) {
      return candidate.kind === 'flag' && candidate.name === name;
    },);
    if (option === undefined)
      throw new TaskUsageError(`task-cli: unknown option --${name}`,);
    /** Environment variable names use underscores for declared hyphens. */
    const variable = `usage_${name.replaceAll('-', '_',)}`;
    if (option.valueName === undefined) {
      env[variable] = 'true';
      index += 1;
      continue;
    }
    /** A missing value is distinct from a present empty string. */
    const value = argv[index + 1];
    if (value === undefined)
      throw new TaskUsageError(`task-cli: option --${name} needs <${option.valueName}>`,);
    env[variable] = value;
    index += 2;
  }
  /** Positional declaration order determines named environment variables. */
  const args = options.filter(function positionalOption(option) { return option.kind === 'arg'; },);
  if (positional.length > 0 && args.length === 0)
    throw new TaskUsageError(`task-cli: unexpected positional arguments: ${positional.join(' ',)}`,);
  if (args.length > 0 || rest.length > 0)
    env.usage_args = shellJoin([...positional, ...rest,],);
  /** The variadic args name denotes all words and must not overwrite usage_args. */
  const named = args.filter(function namedPosition(option) { return option.name !== 'args'; },);
  for (const [position, option,] of named.entries()) {
    /** Optional positional values are omitted rather than converted to an empty string. */
    const word = positional[position];
    if (word !== undefined)
      env[`usage_${option.name.replaceAll('-', '_',)}`] = word;
  }
  return env;
}

//endregion Preserve option values and literal passthrough words
