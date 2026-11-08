/**
 Shared guardrail helpers for Git mutations that run the real Git binary by path.
 
 The `git` this repository exposes on PATH is a wrapper around the real Git binary:
 it applies commit-time policies and pushes after a commit.
 A command line that spells the binary by path skips the wrapper and every policy with it,
 while a read-only probe through the same path only reads.
 
 @module
 */

import { resolveRealGit, } from '@monochromatic-dev/git-executable-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { analyzeShellCommand, } from './analyzer.ts';
import {
  unwrapWrappers,
} from './command-wrapper-words.ts';
import {
  lastPathSegment,
  looksLikeExecutablePath,
  normalizeExecutableName,
} from './executable-path-words.ts';
import { mutatesRepository, } from './git-subcommand-mutation.ts';
import type { ShellCommandInfo, } from './types.ts';

//region Logging

/**
 Logger root for the real-Git mutation guard.
 */
const parentLogger = tagged({ tag: 'real-git-mutation', },);

//endregion Logging

//region Constants

/**
 Refusal reason used when a mutating Git command runs the real Git binary by path.

 @example
 ```ts
 return REAL_GIT_MUTATION_BAN_REASON;
 ```
 */
export const REAL_GIT_MUTATION_BAN_REASON: string = [
  'Blocked: this command calls Git through an absolute path to the real Git binary,',
  'which skips the Git wrapper this repository installs on PATH.',
  'That wrapper enforces the repository policies, commit-time checks and the automatic push after each commit;',
  'calling the binary directly (as in `/usr/bin/git commit`) bypasses every one of them.',
  'Run the same subcommand through the wrapper instead:',
  '`git -C <repository-root> <subcommand> ...` works from any directory,',
  'and `git <subcommand> ...` works from the repository root.',
  'Read-only probes through an absolute path (`rev-parse`, `for-each-ref`, `log`, `show`, `status`, `ls-remote`, `diff`) stay allowed.',
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
 shellScriptPayloads(['bash', '-lc', 'git commit']); // ['git commit']
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
   Whether the record runs a mutating real-Git path invocation.
   */
  readonly mutates: boolean;
  /**
   Nested shell source strings the record launches, still to analyze.
   */
  readonly nestedSources: readonly string[];
};

/**
 Reports whether one parsed command record runs a mutating real-Git path invocation.
 
 @param info - parsed command record
 
 @param realGitBasenames - normalized executable names that identify the real Git
 
 @param platform - runtime platform controlling name case sensitivity
 
 @returns verdict for the record and the shell sources it launches
 
 @example
 ```ts
 commandVerdict({ info, realGitBasenames, platform: 'linux' });
 ```
 */
function commandVerdict({
  info,
  realGitBasenames,
  platform,
}: {
  readonly info: ShellCommandInfo;
  readonly realGitBasenames: ReadonlySet<string>;
  readonly platform: NodeJS.Platform;
},): CommandVerdict {
  /**
   Command lines carried by wrapper options such as `env -S`.
   */
  const wrapperPayloads: string[] = [];
  /**
   Words of the innermost program the shell executes.
   */
  const words = unwrapWrappers({
    words: [
      info.name,
      ...info.args,
    ],
    payloads: wrapperPayloads,
  },);
  /**
   Every shell source string this record launches.
   */
  const nestedSources = [
    ...wrapperPayloads,
    ...shellScriptPayloads({ words, },),
  ];

  /**
   Command word that may name the real Git.
   */
  const [name,] = words;
  if ((name === undefined) || (!looksLikeExecutablePath(name,))) {
    return {
      mutates: false,
      nestedSources,
    };
  }
  if (!realGitBasenames.has(normalizeExecutableName({
    name: lastPathSegment(name,),
    platform,
  },),)) {
    return {
      mutates: false,
      nestedSources,
    };
  }

  return {
    mutates: mutatesRepository(words.slice(1,),),
    nestedSources,
  };
}

/**
 Reports whether shell source runs a mutating Git command through a real-Git path.
 
 Nested shell source strings are analyzed from a work list rather than by recursion,
 so stack depth stays constant as nesting grows.
 
 @param command - raw Bash source from a shell command string
 
 @param realGitBasenames - normalized executable names that identify the real Git
 
 @param platform - runtime platform controlling name case sensitivity
 
 @returns whether the source runs a mutating real-Git path invocation
 
 @example
 ```ts
 commandHasMutatingRealGitInvocation({ command: '/usr/bin/git commit', realGitBasenames, platform: 'linux' });
 ```
 */
function commandHasMutatingRealGitInvocation({
  command,
  realGitBasenames,
  platform,
}: {
  readonly command: string;
  readonly realGitBasenames: ReadonlySet<string>;
  readonly platform: NodeJS.Platform;
},): boolean {
  /**
   Shell source strings still to analyze, deepest last.
   */
  const pendingSources: string[] = [command,];

  while (pendingSources.length > 0) {
    /**
     Shell source string analyzed next.
     */
    const source = pendingSources.pop();
    if (source === undefined)
      break;

    /**
     Parsed shell command analysis.
     */
    const analysis = analyzeShellCommand(source,);
    if (!analysis.parsed)
      continue;

    for (const info of analysis.commands) {
      /**
       Verdict and nested shell sources for the current record.
       */
      const verdict = commandVerdict({
        info,
        realGitBasenames,
        platform,
      },);
      if (verdict.mutates)
        return true;
      pendingSources.push(...verdict.nestedSources,);
    }
  }

  return false;
}

//endregion Command verdicts

//region Predicates

/**
 Reads the executable names of the real Git this host exposes.
 
 Resolution failure leaves the guard inactive and logged rather than throwing,
 because a throwing hook would fail every Bash tool call.
 
 @returns normalized executable names, or an empty list when resolution fails
 
 @example
 ```ts
 await realGitBasenamesOrNone(); // ['git']
 ```
 */
async function realGitBasenamesOrNone(): Promise<readonly string[]> {
  try {
    /**
     Absolute path of the real Git outside the workspace wrapper.
     */
    const realGit = await resolveRealGit();
    return [lastPathSegment(realGit,),];
  }
  catch (error: unknown) {
    /**
     Function-tagged logger reporting the guard going inactive.
     */
    const innerLogger = tagged({
      tag: realGitBasenamesOrNone.name,
      l: parentLogger,
    },);
    innerLogger.debug(
      `real Git resolution failed, absolute-path guard inactive: ${String(error,)}`,
    );
    return [];
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
};

/**
 Checks whether shell source runs a mutating Git command through a real-Git path.
 
 Uses the shared `unbash` analyzer so quoting, chaining, and nested shells are classified by
 shell grammar instead of text boundaries.
 Wrappers that launch another program (`env`, `sudo`, `timeout`, a shell `-c`) are unwrapped first.
 Subcommands with unknown effects count as mutating; only a proven read stays allowed.
 
 @param command - shell command from agent harness Bash tool input
 
 @param realGitBasenames - executable names that identify the real Git, defaulting to the resolved install
 
 @returns whether command mutates the repository through a real-Git path
 
 @example
 ```ts
 await invokesMutatingGitThroughRealGitPath({ command: 'cd x && /usr/bin/git commit' }); // true
 await invokesMutatingGitThroughRealGitPath({ command: '/usr/bin/git rev-parse HEAD' }); // false
 ```
 */
export async function invokesMutatingGitThroughRealGitPath({
  command,
  realGitBasenames,
}: RealGitMutationCheck,): Promise<boolean> {
  /**
   Platform controlling executable-name case sensitivity.
   */
  const { platform, } = process;
  /**
   Executable names that mark a path word as the real Git.
   */
  const resolvedBasenames = realGitBasenames
    ?? await realGitBasenamesOrNone();

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

  return commandHasMutatingRealGitInvocation({
    command,
    realGitBasenames: normalizedBasenames,
    platform,
  },);
}

//endregion Predicates
