/**
 Command validation predicates for Bash output filter hook.
 
 @module
 */

import {
  analyzeShellCommand,
  type ShellCommandAnalysis,
  type ShellCommandInfo,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';

//region Allowlist

/**
 Non-alphanumeric characters that are still safe as a command's leading char.
 */
const ALLOW_LEADING_PUNCT = '_/.~"\'-';

/**
 Whether character is ASCII alphanumeric.
 
 @param c - one-character string to inspect
 
 @returns whether character is alphanumeric
 
 @example
 ```ts
 isAlphaNum('a');
 ```
 */
function isAlphaNum(c: string,): boolean {
  return ((c >= 'A') && (c <= 'Z'))
    || ((c >= 'a') && (c <= 'z'))
    || ((c >= '0') && (c <= '9'));
}

/**
 Whether `command` starts with an allowlisted leading character.
 
 @param command - full Bash command string
 
 @returns whether leading char is allowlisted
 
 @example
 ```ts
 startsWithSafeChar('git status');
 ```
 */
function startsWithSafeChar(command: string,): boolean {
  if (command.length === 0)
    return false;
  /**
   Leading char to test against allow-list set.
   */
  const c = command.charAt(0,);
  return isAlphaNum(c,)
    || ALLOW_LEADING_PUNCT.includes(c,);
}

/**
 Whether command looks like normal text command that is safe to pipe.
 
 @param command - full Bash command string from tool input
 
 @returns `true` if command matches allowlist predicate
 
 @example
 ```ts
 isAllowed('git status');
 ```
 */
function isAllowed(command: string,): boolean {
  return startsWithSafeChar(command,);
}

//endregion Allowlist

//region Denylist helpers

/**
 Binary-handling tools whose output would be mangled by filter pipeline.
 */
const BINARY_TOOL_NAMES: ReadonlySet<string> = new Set([
  'xxd',
  'hexdump',
  'od',
  'base64',
  'tar',
  'gzip',
  'gunzip',
  'zip',
  'unzip',
  'bzip2',
  'xz',
  'zstd',
],);

/**
 Detachment wrapper utilities that take child off controlling terminal.
 */
const DETACH_WRAPPER_NAMES: ReadonlySet<string> = new Set([
  'nohup',
  'setsid',
],);

/**
 Container runtimes whose `exec` and `run` subcommands may attach TTY.
 */
const CONTAINER_RUNTIMES: ReadonlySet<string> = new Set([
  'docker',
  'podman',
],);

/**
 Container subcommands that accept TTY flags.
 */
const CONTAINER_TTY_SUBCOMMANDS: ReadonlySet<string> = new Set([
  'exec',
  'run',
],);

/**
 Shell builtins that change shell state in ways filter cannot follow.
 */
const STATE_BUILTIN_NAMES: ReadonlySet<string> = new Set([
  'cd',
  'pushd',
  'popd',
  'export',
  'unset',
  'source',
  '.',
],);

/**
 Marker emitted by filter to indicate end-of-filter execution.
 */
const BOF_MARKER = '___BOF_EC:';

/**
 Filter script basenames that must not be recursively piped.
 */
const FILTER_SCRIPT_NAMES: ReadonlySet<string> = new Set([
  'filter.mjs',
  'filter.ts',
],);

/**
 Whether token names one of the filter scripts.
 
 @param token - parsed command word
 
 @returns whether token is filter script path or basename
 
 @example
 ```ts
 isFilterScriptToken('./filter.mjs');
 ```
 */
function isFilterScriptToken(token: string,): boolean {
  for (const scriptName of FILTER_SCRIPT_NAMES) {
    if (token === scriptName)
      return true;
    if (token.endsWith(`/${scriptName}`,))
      return true;
  }
  return false;
}

/**
 Command name plus argument words.
 
 @param command - parsed command info
 
 @returns command words in source order
 
 @example
 ```ts
 commandWords(command);
 ```
 */
function commandWords(command: ShellCommandInfo,): readonly string[] {
  return [
    command.name,
    ...command.args,
  ];
}

/**
 Whether token is short CLI flag containing `i` or `t`.
 
 @param token - parsed command argument
 
 @returns whether token is TTY-style short flag
 
 @example
 ```ts
 isTtyFlag('-it');
 ```
 */
function isTtyFlag(token: string,): boolean {
  if ((!token.startsWith('-',)) || (token.startsWith('--',)))
    return false;
  /**
   Flag body after leading dash.
   */
  const body = token.slice(1,);
  if (body.length === 0)
    return false;
  for (const c of body) {
    if ((c < 'a') || (c > 'z'))
      return false;
  }
  return body.includes('i',)
    || body.includes('t',);
}

/**
 Whether parsed command invokes container runtime with TTY flag.
 
 @param command - parsed command info
 
 @returns whether command attaches container TTY
 
 @example
 ```ts
 commandHasTtyContainerInvoke(command);
 ```
 */
function commandHasTtyContainerInvoke(command: ShellCommandInfo,): boolean {
  if (!CONTAINER_RUNTIMES.has(command.name,))
    return false;
  /**
   Container subcommand after runtime name.
   */
  const subcommand = command.args[0]
    ?? '';
  if (!CONTAINER_TTY_SUBCOMMANDS.has(subcommand,))
    return false;
  return command.args
    .slice(1,)
    .some(function argIsTtyFlag(arg,): boolean {
      return isTtyFlag(arg,);
    },);
}

/**
 Whether parsed command has output file redirect.
 
 @param command - parsed command info
 
 @returns whether command writes to redirected file target
 
 @example
 ```ts
 commandHasOutputRedirect(command);
 ```
 */
function commandHasOutputRedirect(command: ShellCommandInfo,): boolean {
  return command.redirects
    .some(function redirectWritesFile(redirect,): boolean {
      return redirect.writesFile;
    },);
}

/**
 Whether parsed command invokes `bun build` directly.
 
 @param command - parsed command info
 
 @returns whether command is `bun build`
 
 @example
 ```ts
 commandIsBunBuild(command);
 ```
 */
function commandIsBunBuild(command: ShellCommandInfo,): boolean {
  return (command.name === 'bun')
    && (command.args[0] === 'build');
}

/**
 Whether parsed command invokes filter helper script.
 
 @param command - parsed command info
 
 @returns whether command words reference filter script
 
 @example
 ```ts
 commandInvokesFilterScript(command);
 ```
 */
function commandInvokesFilterScript(command: ShellCommandInfo,): boolean {
  return commandWords(command,)
    .some(function commandWordIsFilterScript(token,): boolean {
      return isFilterScriptToken(token,);
    },);
}

/**
 Whether parsed command invokes a state-changing builtin.
 
 @param command - parsed command info
 
 @returns whether command changes shell state
 
 @example
 ```ts
 commandIsStateBuiltin(command);
 ```
 */
function commandIsStateBuiltin(command: ShellCommandInfo,): boolean {
  return STATE_BUILTIN_NAMES.has(command.name,);
}

/**
 Whether parsed command invokes a binary-handling tool.
 
 @param command - parsed command info
 
 @returns whether command output should not be piped through text filter
 
 @example
 ```ts
 commandIsBinaryTool(command);
 ```
 */
function commandIsBinaryTool(command: ShellCommandInfo,): boolean {
  return BINARY_TOOL_NAMES.has(command.name,);
}

/**
 Whether parsed command invokes a detachment wrapper.
 
 @param command - parsed command info
 
 @returns whether command detaches from terminal
 
 @example
 ```ts
 commandIsDetachWrapper(command);
 ```
 */
function commandIsDetachWrapper(command: ShellCommandInfo,): boolean {
  return DETACH_WRAPPER_NAMES.has(command.name,);
}

//endregion Denylist helpers

//region Public denylist

/**
 Whether parsed analysis has any command-level skip reason.
 
 @param analysis - parsed command analysis
 
 @returns whether command should not be piped through filter
 
 @example
 ```ts
 analysisHasCommandSkip(analysis);
 ```
 */
function analysisHasCommandSkip(analysis: ShellCommandAnalysis,): boolean {
  return analysis.commands
    .some(function commandShouldSkip(command,): boolean {
      return commandIsBinaryTool(command,)
        || commandHasOutputRedirect(command,)
        || commandInvokesFilterScript(command,)
        || commandIsDetachWrapper(command,)
        || commandHasTtyContainerInvoke(command,)
        || commandIsBunBuild(command,)
        || commandIsStateBuiltin(command,)
        || (command.name === 'eval');
    },);
}

/**
 Whether command should be skipped instead of piped through filter.
 
 @param command - full Bash command string from tool input
 
 @returns `true` if command matches any denylist predicate
 
 @example
 ```ts
 shouldSkip('xxd file.bin');
 ```
 */
function shouldSkip(command: string,): boolean {
  if (command.includes(BOF_MARKER,))
    return true;

  /**
   Parsed shell command analysis from shared analyzer.
   */
  const analysis = analyzeShellCommand(command,);
  if (!analysis.parsed)
    return true;

  return analysis.hasBackground
    || analysis.hasCommandSubstitution
    || analysis.hasProcessSubstitution
    || analysis.hasHeredoc
    || analysisHasCommandSkip(analysis,);
}

//endregion Public denylist

export {
  isAllowed,
  shouldSkip,
};
