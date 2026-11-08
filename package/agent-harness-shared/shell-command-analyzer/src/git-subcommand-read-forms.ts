/**
 Read-only argument shapes for Git subcommands that also write.
 
 `git branch --list` only prints,
 `git branch topic` creates,
 and the guard has to tell those apart to keep the read direction allowed.
 
 @module
 */

import {
  hasAnyOption,
  scanGitArguments,
} from './git-subcommand-arguments.ts';
import {
  BRANCH_OPTIONS_WITH_VALUE,
  BRANCH_WRITE_OPTIONS,
  CONFIG_OPTIONS_WITH_VALUE,
  CONFIG_READ_OPTIONS,
  CONFIG_WRITE_OPTIONS,
  SYMBOLIC_REF_WRITE_OPTIONS,
  TAG_OPTIONS_WITH_VALUE,
  TAG_READ_OPTIONS,
  TAG_WRITE_OPTIONS,
} from './git-subcommand-option-sets.ts';

//region Argument shapes

/**
 Reports whether `git branch` only lists branches.
 
 @param args - subcommand arguments in source order
 
 @returns whether the invocation lists instead of writing
 
 @example
 ```ts
 branchReadsOnly(['--list']); // true
 branchReadsOnly(['new-branch']); // false
 ```
 */
function branchReadsOnly(args: readonly string[],): boolean {
  /**
   Positional and option words of the invocation.
   */
  const scan = scanGitArguments({
    args,
    optionsWithValue: BRANCH_OPTIONS_WITH_VALUE,
  },);
  /**
   Whether the invocation names a branch to write.
   */
  const writesNamedBranch = hasAnyOption({
    words: scan.options,
    names: BRANCH_WRITE_OPTIONS,
  },);
  return (scan
    .positionals
    .length
    === 0) && (!writesNamedBranch);
}

/**
 Reports whether `git tag` only lists tags.
 
 @param args - subcommand arguments in source order
 
 @returns whether the invocation lists instead of writing
 
 @example
 ```ts
 tagReadsOnly(['--list', 'v*']); // true
 tagReadsOnly(['v1.0.0']); // false
 ```
 */
function tagReadsOnly(args: readonly string[],): boolean {
  /**
   Positional and option words of the invocation.
   */
  const scan = scanGitArguments({
    args,
    optionsWithValue: TAG_OPTIONS_WITH_VALUE,
  },);
  if (hasAnyOption({
    words: scan.options,
    names: TAG_WRITE_OPTIONS,
  },))
    return false;
  if (hasAnyOption({
    words: scan.options,
    names: TAG_READ_OPTIONS,
  },))
    return true;
  return scan
    .positionals
    .length
    === 0;
}

/**
 Reports whether `git config` only prints configuration values.
 
 @param args - subcommand arguments in source order
 
 @returns whether the invocation reads instead of writing
 
 @example
 ```ts
 configReadsOnly(['--get', 'user.name']); // true
 configReadsOnly(['user.name', 'Someone']); // false
 ```
 */
function configReadsOnly(args: readonly string[],): boolean {
  /**
   Positional and option words of the invocation.
   */
  const scan = scanGitArguments({
    args,
    optionsWithValue: CONFIG_OPTIONS_WITH_VALUE,
  },);
  if (hasAnyOption({
    words: scan.options,
    names: CONFIG_WRITE_OPTIONS,
  },))
    return false;
  if (hasAnyOption({
    words: scan.options,
    names: CONFIG_READ_OPTIONS,
  },))
    return true;
  if (scan.positionals[0] === 'get')
    return true;
  return scan
    .positionals
    .length
    === 1;
}

/**
 Reports whether `git symbolic-ref` only prints a reference.
 
 @param args - subcommand arguments in source order
 
 @returns whether the invocation reads instead of writing
 
 @example
 ```ts
 symbolicRefReadsOnly(['HEAD']); // true
 symbolicRefReadsOnly(['HEAD', 'refs/heads/topic']); // false
 ```
 */
function symbolicRefReadsOnly(args: readonly string[],): boolean {
  /**
   Positional and option words of the invocation.
   */
  const scan = scanGitArguments({
    args,
    optionsWithValue: new Set(),
  },);
  if (hasAnyOption({
    words: scan.options,
    names: SYMBOLIC_REF_WRITE_OPTIONS,
  },))
    return false;
  return scan
    .positionals
    .length
    <= 1;
}

//endregion Argument shapes

//region Verb-first subcommands

/**
 Subcommand shape whose first positional word selects a read or write operation.
 */
type GitVerbFirstForm = {
  /**
   Verbs that only read.
   */
  readonly readVerbs: ReadonlySet<string>;
  /**
   Whether the bare subcommand without a verb only reads.
   */
  readonly readsWithoutVerb: boolean;
};

/**
 Reports whether a verb-first subcommand only lists with the given verb.
 
 @param args - subcommand arguments in source order
 
 @param form - read verbs and the bare-subcommand behavior
 
 @returns whether the invocation reads instead of writing
 
 @example
 ```ts
 verbFirstReadsOnly({ args: ['list'], form: { readVerbs, readsWithoutVerb: true } });
 ```
 */
function verbFirstReadsOnly({
  args,
  form,
}: {
  readonly args: readonly string[];
  readonly form: GitVerbFirstForm;
},): boolean {
  /**
   Positional and option words of the invocation.
   */
  const scan = scanGitArguments({
    args,
    optionsWithValue: new Set(),
  },);
  /**
   Verb naming the operation, when one is given.
   */
  const [verb,] = scan.positionals;
  if (verb === undefined)
    return form.readsWithoutVerb;
  return form
    .readVerbs
    .has(verb,);
}

/**
 Subcommands whose first positional word decides between reading and writing.
 */
const GIT_VERB_FIRST_FORMS: ReadonlyMap<string, GitVerbFirstForm> = new Map([
  [
    'bisect',
    {
      readVerbs: new Set([
        'log',
        'view',
      ],),
      readsWithoutVerb: false,
    },
  ],
  [
    'bundle',
    {
      readVerbs: new Set([
        'list-heads',
        'verify',
      ],),
      readsWithoutVerb: false,
    },
  ],
  [
    'commit-graph',
    {
      readVerbs: new Set(['verify',],),
      readsWithoutVerb: false,
    },
  ],
  [
    'notes',
    {
      readVerbs: new Set([
        'get-ref',
        'list',
        'show',
      ],),
      readsWithoutVerb: true,
    },
  ],
  [
    'reflog',
    {
      readVerbs: new Set([
        'exists',
        'show',
      ],),
      readsWithoutVerb: true,
    },
  ],
  [
    'remote',
    {
      readVerbs: new Set([
        'get-url',
        'show',
      ],),
      readsWithoutVerb: true,
    },
  ],
  [
    'sparse-checkout',
    {
      readVerbs: new Set(['list',],),
      readsWithoutVerb: false,
    },
  ],
  [
    'stash',
    {
      readVerbs: new Set([
        'list',
        'show',
      ],),
      readsWithoutVerb: false,
    },
  ],
  [
    'submodule',
    {
      readVerbs: new Set([
        'status',
        'summary',
      ],),
      readsWithoutVerb: true,
    },
  ],
  [
    'worktree',
    {
      readVerbs: new Set(['list',],),
      readsWithoutVerb: true,
    },
  ],
]);

//endregion Verb-first subcommands

//region Read-only shapes

/**
 Subcommands that only read in some argument shapes.
 */
const READ_ONLY_GIT_SUBCOMMAND_FORMS: ReadonlyMap<
  string,
  (args: readonly string[]) => boolean
  > = new Map([
    [
      'branch',
      branchReadsOnly,
    ],
    [
      'config',
      configReadsOnly,
    ],
    [
      'symbolic-ref',
      symbolicRefReadsOnly,
    ],
    [
      'tag',
      tagReadsOnly,
    ],
  ]);

//endregion Read-only shapes

export type { GitVerbFirstForm, };

export {
  READ_ONLY_GIT_SUBCOMMAND_FORMS,
  GIT_VERB_FIRST_FORMS,
  verbFirstReadsOnly,
};
