//region Usage options: mapping `usage` specs to `usage_*` variables

/**
 One parsed `usage` flag or arg from a task definition.
 */
type UsageOption = {
  readonly kind: 'flag' | 'arg';
  readonly name: string;
  readonly valueName?: string;
};

/**
 Parses a mise-style `usage` spec into flag and arg descriptors. Flag names
 drop their leading dashes so they match `--name` tokens and the `usage_name`
 variable the old mise mapping produced.

 @param usage - Raw usage string from a task definition.
 
 @returns Parsed options in declaration order.
 */
export function parseUsageSpec(usage: string | undefined): readonly UsageOption[] {
  if (usage === undefined) { return []; }
  const options: UsageOption[] = [];
  const pattern = /(flag|arg)\s+"([^"]+)"(?:\s+var=#true)?/gu;
  for (const match of usage.matchAll(pattern)) {
    const kind = match[1] === 'flag' ? 'flag' : 'arg';
    const body = match[2];
    if (body === undefined) { continue; }
    // A flag's name stops before its `<value>`; an arg's name is its whole first word, `<cache>` included.
    const nameMatch = (kind === 'arg' ? /^(?<name>\S+)/u : /^(?<name>[^\s<]+)/u).exec(body);
    if (nameMatch === null) { continue; }
    // Flags drop their dashes; positional args drop the `[ ]` or `< >` that mark them optional or required.
    const name = nameMatch.groups
      ?.name
      ?.replace(
      /^-+/u,
      ''
      )
      .replaceAll(
      /^[[<]|[\]>]$/gu,
      ''
      );
    if ((name === undefined) || (name === '')) { continue; }
    const valueName = /<(?<value>[^>]+)>/u.exec(body)
      ?.groups
      ?.value;
    options.push(valueName === undefined ? {
      kind,
      name,
    } : {
      kind,
      name,
      valueName,
    });
  }
  return options;
}

/**
 Splits task arguments the way `task-run <task> [-- options] [-- passthrough]`
 documents: one leading `--` separates them from anything before it, and the
 next `--` starts raw passthrough.

 @param argv - Arguments after the task name.
 
 @returns Option arguments before the passthrough separator and raw arguments after it.
 */
export function splitPassthrough(argv: readonly string[]): {
  options: readonly string[];
  rest: readonly string[]
} {
  const afterSeparator = argv[0] === '--' ? argv.slice(1) : argv;
  const index = afterSeparator.indexOf('--');
  if (index === (-1)) { return {
    options: afterSeparator,
    rest: [],
  }; }
  return {
    options: afterSeparator.slice(
      0,
      index
    ),
    rest: afterSeparator.slice(index + 1),
  };
}

/**
 Quotes one `usage_args` word for the shell-word grammar `parseUsageArgs`
 reads back: bare when plain, single-quoted with an escaped quote otherwise.

 @param word - Raw argument.
 
 @returns Encoded word.
 */
function shellQuote(word: string): string {
  if ((word.length > 0) && (!/[\s'"\\]/u.test(word))) { return word; }
  return `'${word.replaceAll(
    "'",
    `'\\''`
  )}'`;
}

/**
 Joins arguments into one `usage_args` value that `parseUsageArgs` recovers
 word for word, so paths keep their spaces the way the old mise var did.

 @param words - Raw arguments.
 
 @returns Shell-word encoded string.
 */
function shellJoin(words: readonly string[]): string {
  return words.map(shellQuote)
    .join(' ');
}

/**
 Builds the `usage_*` environment for a task from its options.

 Tasks without a usage spec take every post-`--` argument as command argv, the
 way mise appended it, and still see the words in `usage_args`. Tasks with a
 spec map declared flags to `usage_*` variables and keep positional and
 passthrough words in `usage_args`.

 @param options - Parsed usage descriptors.
 
 @param argv - Task option arguments.
 
 @param rest - Passthrough arguments after `--`.
 
 @returns Environment entries to set for the task body.
 
 @throws On unknown options, missing flag values, or misplaced positional args.
 */
export function usageEnvironment({
  options,
  argv,
  rest,
}: {
  readonly options: readonly UsageOption[];
  readonly argv: readonly string[];
  readonly rest: readonly string[]
},): Record<string, string> {
  const env: Record<string, string> = {};
  if (options.length === 0) {
    const raw = [
      ...argv,
      ...rest
    ];
    if (raw.length > 0) { env.usage_args = shellJoin(raw); }
    return env;
  }
  const positional: string[] = [];
  let index = 0;
  while (index < argv.length) {
    const token = argv[index]!;
    if (token.startsWith('--')) {
      const name = token.slice(2);
      const option = options.find((candidate) => (candidate.kind === 'flag') && (candidate.name === name),);
      if (option === undefined) { throw new Error(`task-cli: unknown option --${name}`); }
      const variable = `usage_${name.replaceAll(
        '-',
        '_'
      )}`;
      if (option.valueName === undefined) {
        env[variable] = 'true';
        index += 1;
        continue;
      }
      const value = argv[index + 1];
      if (value === undefined) { throw new Error(`task-cli: option --${name} needs <${option.valueName}>`); }
      env[variable] = value;
      index += 2;
      continue;
    }
    positional.push(token);
    index += 1;
  }
  const args = options.filter((option) => option.kind === 'arg',);
  if ((positional.length > 0) && (args.length === 0))
    throw new Error(`task-cli: unexpected positional arguments: ${positional.join(' ')}`);
  if ((args.length > 0) || (rest.length > 0))
    env.usage_args = shellJoin([
      ...positional,
      ...rest
    ]);
  // mise also set one `usage_<name>` variable per declared positional arg, in declaration order; bodies such as
  // the IDE bundle inspection read `usage_file`, `usage_runtime` and `usage_only`. The variadic `args` name keeps
  // meaning the whole word list in `usage_args` above.
  const named = args.filter((option) => option.name !== 'args',);
  for (const [index, option,] of named.entries()) {
    const word = positional[index];
    if (word !== undefined) { env[`usage_${option.name
      .replaceAll(
        '-',
        '_'
      )}`] = word; }
  }
  return env;
}

//endregion
