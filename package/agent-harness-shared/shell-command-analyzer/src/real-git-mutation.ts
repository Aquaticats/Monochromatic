/**
 Shared guardrail helpers for Git mutations that run the real Git binary by path.
 
 The `git` this repository exposes on PATH is a wrapper around the real Git binary:
 it applies commit-time policies and pushes after a commit.
 A command line that spells the binary by path skips the wrapper and every policy with it,
 while a read-only probe through the same path only reads.
 
 The same skip happens without a path word: assigning `PATH` on the command line,
 exporting `PATH` in an earlier statement,
 or running Git on a search path the wrapper is absent from (`command -p`, `env -i`)
 also reaches the real binary with a bare `git` word.
 
 @module
 */

import { dirname, } from 'node:path';

import { resolveRealGit, } from '@monochromatic-dev/git-executable-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { analyzeShellCommand, } from './analyzer.ts';
import { unwrapWrappers, } from './command-wrapper-words.ts';
import {
  lastPathSegment,
  looksLikeExecutablePath,
  normalizeExecutableName,
} from './executable-path-words.ts';
import { mutatesRepository, } from './git-subcommand-mutation.ts';
import {
  comparableDirectories,
  commandExcludesWrapper,
  statementReassignsSearchPath,
} from './search-path-reaches-wrapper.ts';
import type { ShellCommandInfo, } from './types.ts';

//region Logging

/**
 Logger root for the real-Git mutation guard.
 */
const parentLogger = tagged({ tag: 'real-git-mutation', },);

//endregion Logging

//region Constants

/**
 Refusal reason used when a mutating Git command runs the real Git binary outside the wrapper.

 @example
 ```ts
 return REAL_GIT_MUTATION_BAN_REASON;
 ```
 */
export const REAL_GIT_MUTATION_BAN_REASON: string = [
  'Blocked: this command runs a mutating Git subcommand without the Git wrapper this repository installs on PATH.',
  'Three spellings reach the real Git binary and skip it:',
  'an absolute path (as in `/usr/bin/git commit`),',
  'an assignment of `PATH` on the same command line (as in `PATH=/usr/bin:$PATH git commit`),',
  'and a search path this repository cannot install into (`command -p git commit`, `env -i git commit`).',
  'The wrapper enforces the repository policies, commit-time checks and the automatic push after each commit;',
  'every one of those forms bypasses all of them.',
  'Run the same subcommand through the wrapper instead:',
  '`git -C <repository-root> <subcommand> ...` works from any directory,',
  'and `git <subcommand> ...` works from the repository root.',
  'Read-only probes (`rev-parse`, `for-each-ref`, `log`, `show`, `status`, `ls-remote`, `diff`) stay allowed in every one of those forms.',
]
  .join(' ',);

/**
 Shell names whose `-c` argument carries a script string.
 */
const SHELL_NAMES: ReadonlySet<string> = new Set([
  'bash',
  'dash',
  'ksh',
  'mksh',
  'sh',
  'zsh',
],);

/**
 Shell flags that consume the next word as their value.
 */
const SHELL_OPTIONS_WITH_VALUE: ReadonlySet<string> = new Set([
  '-O',
  '-o',
  '+O',
  '+o',
],);

/**
 Sentinel returned when this host exposes no real-Git executable.
 */
const REAL_GIT_UNRESOLVED: unique symbol = Symbol('real Git executable not resolved',);

//endregion Constants



//region Shell script payloads

/**
 Reports whether a shell flag word selects `-c`, as in `-c` or `-lc`.
 
 @param word - shell argument word
 
 @returns whether the word is a short flag cluster containing `c`
 
 @example
 ```ts
 isShellCommandFlag('-lc'); // true
 isShellCommandFlag('-e'); // false
 ```
 */
function isShellCommandFlag(word: string,): boolean {
  return word.startsWith('-',)
    && (!word.startsWith('--',))
    && word.includes('c',);
}

/**
 Collects script strings passed to a shell through its `-c` flag.
 
 @param words - command name followed by its arguments
 
 @returns script strings the launched shell executes
 
 @example
 ```ts
 shellScriptPayloads({ words: ['bash', '-lc', 'git commit'] }); // ['git commit']
 ```
 */
function shellScriptPayloads({ words, }: { readonly words: readonly string[]; },): readonly string[] {
  /**
   Command word that may name a shell.
   */
  const [name,] = words;
  if ((name === undefined) || (!SHELL_NAMES.has(lastPathSegment(name,),)))
    return [];

  /**
   Script strings following `-c` flags.
   */
  const payloads: string[] = [];
  for (let index = 1; index < words.length; index += 1) {
    /**
     Shell argument inspected as flag or script string.
     */
    const word = words[index];
    if (word === undefined)
      break;
    if (word === '--')
      break;
    if (!word.startsWith('-',))
      break;
    if (SHELL_OPTIONS_WITH_VALUE.has(word,)) {
      index += 1;
      continue;
    }
    if (!isShellCommandFlag(word,))
      continue;

    /**
     Script string following the `-c` flag.
     */
    const payload = words[index + 1];
    if (payload !== undefined)
      payloads.push(payload,);
    break;
  }
  return payloads;
}

//endregion Shell script payloads

//region Command verdicts

/**
 Decision for one parsed command record.
 */
type CommandVerdict = {
  /**
   Whether the record runs a mutating real-Git invocation.
   */
  readonly mutates: boolean;
  /**
   Nested shell source strings the record launches, still to analyze.
   */
  readonly nestedSources: readonly string[];
  /**
   Whether the search path the record leaves its nested source excludes the wrapper.
   */
  readonly searchPathExcludesWrapper: boolean;
};

/**
 Reports whether one parsed command record runs a mutating real-Git invocation.
 
 @param info - parsed command record
 
 @param realGitBasenames - normalized executable names that identify the real Git
 
 @param realGitDirectories - comparable folders that hold the real Git binary
 
 @param platform - runtime platform controlling name case sensitivity
 
 @param searchPathExcludesWrapper - whether an enclosing assignment already left the program on a search path without the wrapper
 
 @returns verdict for the record, the shell sources it launches, and the search path it leaves them
 
 @example
 ```ts
 commandVerdict({ info, realGitBasenames, realGitDirectories, platform: 'linux', searchPathExcludesWrapper: false });
 ```
 */
function commandVerdict({
  info,
  realGitBasenames,
  realGitDirectories,
  platform,
  searchPathExcludesWrapper,
}: {
  readonly info: ShellCommandInfo;
  readonly realGitBasenames: ReadonlySet<string>;
  readonly realGitDirectories: ReadonlySet<string>;
  readonly platform: NodeJS.Platform;
  readonly searchPathExcludesWrapper: boolean;
},): CommandVerdict {
  /**
   Innermost program words and everything the stripped wrappers revealed.
   */
  const wrapped = unwrapWrappers({
    words: [
      info.name,
      ...info.args,
    ],
  },);
  /**
   Every shell source string this record launches.
   */
  const nestedSources = [
    ...wrapped.payloads,
    ...shellScriptPayloads({ words: wrapped.words, },),
  ];
  /**
   Whether this record leaves its program on a search path that does not find the wrapper.
   */
  const excludesWrapper = commandExcludesWrapper({
    info,
    wrapped,
    realGitDirectories,
    searchPathExcludesWrapper,
  },);

  /**
   Command word of the innermost program.
   */
  const [name,] = wrapped.words;
  if ((name === undefined)
    || (!realGitBasenames.has(normalizeExecutableName({
      name: lastPathSegment(name,),
      platform,
    },),))) {
    return {
      mutates: false,
      nestedSources,
      searchPathExcludesWrapper: excludesWrapper,
    };
  }

  /**
   Whether the command word spells the executable as a path, which needs no search path.
   */
  const spellsPath = looksLikeExecutablePath(name,);
  return {
    mutates: (spellsPath || excludesWrapper)
      && mutatesRepository(wrapped.words
        .slice(1,),),
    nestedSources,
    searchPathExcludesWrapper: excludesWrapper,
  };
}

/**
 Reports whether shell source runs a mutating Git command outside the wrapper.
 
 Nested shell source strings are analyzed from a work list rather than by recursion,
 so stack depth stays constant as nesting grows.
 A search-path assignment reaches the commands that follow it in the same source,
 so records are folded in traversal order.
 
 @param command - raw Bash source from a shell command string
 
 @param realGitBasenames - normalized executable names that identify the real Git
 
 @param realGitDirectories - comparable folders that hold the real Git binary
 
 @param platform - runtime platform controlling name case sensitivity
 
 @returns whether the source runs a mutating real-Git invocation
 
 @example
 ```ts
 commandHasMutatingRealGitInvocation({ command: '/usr/bin/git commit', realGitBasenames, realGitDirectories, platform: 'linux' });
 ```
 */
function commandHasMutatingRealGitInvocation({
  command,
  realGitBasenames,
  realGitDirectories,
  platform,
}: {
  readonly command: string;
  readonly realGitBasenames: ReadonlySet<string>;
  readonly realGitDirectories: ReadonlySet<string>;
  readonly platform: NodeJS.Platform;
},): boolean {
  /**
   Shell source strings still to analyze, deepest last, each with the search path it inherits.
   */
  const pendingSources: {
    readonly source: string;
    readonly searchPathExcludesWrapper: boolean;
  }[] = [
    {
      source: command,
      searchPathExcludesWrapper: false,
    },
  ];

  while (pendingSources.length > 0) {
    /**
     Shell source string analyzed next.
     */
    const pending = pendingSources.pop();
    if (pending === undefined)
      break;

    /**
     Parsed shell command analysis.
     */
    const analysis = analyzeShellCommand(pending.source,);
    if (!analysis.parsed)
      continue;

    /**
     Search path state carried from record to record, as the shell runs them in order.
     */
    const searchPathState = {
      excludesWrapper: pending.searchPathExcludesWrapper,
    };

    for (const info of analysis.commands) {
      if (statementReassignsSearchPath({
        info,
        realGitDirectories,
      },))
        searchPathState.excludesWrapper = true;

      /**
       Verdict and nested shell sources for the current record.
       */
      const verdict = commandVerdict({
        info,
        realGitBasenames,
        realGitDirectories,
        platform,
        searchPathExcludesWrapper: searchPathState.excludesWrapper,
      },);
      if (verdict.mutates)
        return true;
      for (const source of verdict.nestedSources) {
        pendingSources.push({
          source,
          searchPathExcludesWrapper: verdict.searchPathExcludesWrapper,
        },);
      }
    }
  }

  return false;
}

//endregion Command verdicts

//region Predicates

/**
 Reads the real-Git executable this host exposes.
 
 Resolution failure leaves the guard inactive and logged rather than throwing,
 because a throwing hook would fail every Bash tool call.
 
 @returns absolute path of the real Git, or {@link REAL_GIT_UNRESOLVED} when resolution failed
 
 @example
 ```ts
 await realGitPathOrNone(); // '/usr/bin/git'
 ```
 */
async function realGitPathOrNone(): Promise<string | typeof REAL_GIT_UNRESOLVED> {
  try {
    return await resolveRealGit();
  }
  catch (error: unknown) {
    /**
     Function-tagged logger reporting the guard going inactive.
     */
    const innerLogger = tagged({
      tag: realGitPathOrNone.name,
      l: parentLogger,
    },);
    innerLogger.debug(
      `real Git resolution failed, real-Git guard inactive: ${String(error,)}`,
    );
    return REAL_GIT_UNRESOLVED;
  }
}

/**
 Inputs for {@link invokesMutatingGitThroughRealGitPath}.
 */
export type RealGitMutationCheck = {
  /**
   Shell command from agent harness Bash tool input.
   */
  readonly command: string;
  /**
   Executable names that identify the real Git, for fixtures and for hosts that resolve it elsewhere.
   */
  readonly realGitBasenames?: readonly string[];
  /**
   Folders that hold the real Git binary, for fixtures.
   
   An assignment of `PATH` naming one of them reaches the real binary,
   so a fixture must inject a folder that this host need not have.
   */
  readonly realGitDirectories?: readonly string[];
};

/**
 Checks whether shell source runs a mutating Git command outside the wrapper.
 
 Uses the shared `unbash` analyzer so quoting, chaining, and nested shells are classified by
 shell grammar instead of text boundaries.
 Wrappers that launch another program (`env`, `sudo`, `timeout`, a shell `-c`) are unwrapped first.
 Subcommands with unknown effects count as mutating; only a proven read stays allowed.
 
 A Git path known only at run time (`bash -c "$CMD commit"`, `G=/usr/bin/git; $G commit`)
 stays undetected: the guard reads command text and runs no shell evaluator.
 
 @param command - shell command from agent harness Bash tool input
 
 @param realGitBasenames - executable names that identify the real Git, defaulting to the resolved install
 
 @param realGitDirectories - folders that hold the real Git, defaulting to the resolved install's folder
 
 @returns whether command mutates the repository outside the wrapper
 
 @example
 ```ts
 await invokesMutatingGitThroughRealGitPath({ command: 'cd x && /usr/bin/git commit' }); // true
 await invokesMutatingGitThroughRealGitPath({ command: 'PATH=/usr/bin git commit' }); // true
 await invokesMutatingGitThroughRealGitPath({ command: '/usr/bin/git rev-parse HEAD' }); // false
 ```
 */
export async function invokesMutatingGitThroughRealGitPath({
  command,
  realGitBasenames,
  realGitDirectories,
}: RealGitMutationCheck,): Promise<boolean> {
  /**
   Platform controlling executable-name case sensitivity.
   */
  const { platform, } = process;
  /**
   Whether either list must come from the executable this host resolves.
   */
  const needsResolution = (realGitBasenames === undefined) || (realGitDirectories === undefined);
  /**
   Absolute path of the real Git this host exposes, when a default is needed.
   */
  const resolvedPath = needsResolution
    ? await realGitPathOrNone()
    : REAL_GIT_UNRESOLVED;

  /**
   Executable names that mark a word as the real Git.
   */
  const resolvedBasenames = realGitBasenames
    ?? ((typeof resolvedPath) === 'symbol' ? [] : [lastPathSegment(resolvedPath,)]);
  /**
   Comparable executable names after platform normalization.
   */
  const normalizedBasenames = new Set(resolvedBasenames
    .map(function normalizedBasename(name,): string {
      return normalizeExecutableName({
        name,
        platform,
      },);
    },),);
  if (normalizedBasenames.size === 0)
    return false;

  /**
   Folders that hold the real Git, read in the comparison form a search-path entry uses.
   */
  const normalizedDirectories = comparableDirectories(realGitDirectories
    ?? ((typeof resolvedPath) === 'symbol' ? [] : [dirname(resolvedPath,)]),);

  return commandHasMutatingRealGitInvocation({
    command,
    realGitBasenames: normalizedBasenames,
    realGitDirectories: normalizedDirectories,
    platform,
  },);
}

//endregion Predicates
