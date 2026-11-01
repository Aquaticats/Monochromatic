import type {
  BashToolInput,
  PreToolUseInput,
  PreToolUseOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';

import { shouldForceBackground, } from './force-background.ts';
import {
  isAllowed,
  shouldSkip,
} from './validation.ts';

/**
 Type guard that narrows a generic tool input to {@link BashToolInput}.
 
 @param input - tool input to check
 
 @returns `true` when `input` has a string `command` property
 
 @example
 ```ts
 isBashToolInput({ command: 'ls' }); // true
 isBashToolInput({ file_path: 'x' }); // false
 ```
 */
function isBashToolInput(input: Readonly<Record<string, unknown>>,): input is BashToolInput {
  return (typeof input.command) === 'string';
}

/**
 Output returned by the bash-output-filter handler. Either a {@link PreToolUseOutput}
 carrying a rewritten `updatedInput.command` that pipes the original command
 through the filter script, or the pass-through `{}` (when the command is
 non-Bash, malformed, disallowed, or skipped). Every {@link PreToolUseOutput} field
 is optional, so `{}` is itself a valid {@link PreToolUseOutput}.
 */
type BashOutputFilterOutput = PreToolUseOutput;

/**
 Rewrites the Bash command to pipe its merged stdout/stderr through the
 filter script. The filter script lives next to the bundled hook entry:
 `import.meta.dirname` resolves at runtime to the bundle's directory,
 which is `bundle/node/`, where `filter.mjs` is a sibling output of
 the same multi-entry tsdown build.
 
 Decision tree:
 
 1. **Tool gate**: non-Bash tools or malformed Bash inputs return `{}`.
 2. **Allowlist**: commands not matching the safe-prefix allowlist return `{}`.
 3. **Denylist**: commands matching any skip pattern (binary tools, redirects,
    background processes, command substitutions, shell builtins) return `{}`.
 4. **Rewrite**: the surviving command becomes:
    `set -o pipefail && <cmd> 2>&1 | node <filterPath> && true`.
    The trailing `&& true` absorbs the sandbox's `< /dev/null` append (see
    `bash-output-filter/TROUBLESHOOTING.md`) and lets pipefail surface the
    original command's exit code while keeping the filter's exit code (always 0)
    out of the way.
 
 @param event - parsed {@link PreToolUseInput} event from Claude Code
 
 @returns rewritten output when the command should be filtered, otherwise `{}`
 
 @example
 ```ts
 bashOutputFilterHandler({ tool_name: 'Bash', tool_input: { command: 'ls' }, ... });
 ```
 */
function bashOutputFilterHandler(event: ReadonlyDeep<PreToolUseInput>,): BashOutputFilterOutput {
  if ((event.tool_name
    !== 'Bash') || (!isBashToolInput(event.tool_input,)))
    return {};

  /**
   Narrowed Bash tool input; refined from `event.tool_input` by {@link isBashToolInput}.
   */
  const bashInput = event.tool_input;

  /**
   Whether this main-agent call is rewritten to run as a background task.
   */
  const forceBackground = shouldForceBackground({
    ...(event.agent_id === undefined ? {} : { agentId: event.agent_id, }),
    command: bashInput.command,
    env: process.env,
  },);

  if ((!isAllowed(bashInput.command,)) || shouldSkip(bashInput.command,)) {
    if (!forceBackground)
      return {};

    return {
      hookSpecificOutput: {
        hookEventName: 'PreToolUse',
        updatedInput: {
          ...bashInput,
          run_in_background: true,
        },
      },
    };
  }

  /**
   True when running from the bundled hook entry (`.mjs`); false during source-driven dev runs.
   */
  const isBuilt = import.meta.url
    .endsWith('.mjs',);
  /**
   Sibling path to the filter script chosen by build mode (`.mjs` vs `.ts`).
   */
  const filterPath = isBuilt
    ? `${import.meta.dirname}/filter.mjs`
    : `${import.meta.dirname}/filter.ts`;

  /**
   Rewritten command pipeline that streams the original command's merged output through the filter.
   */
  const wrappedCommand =
    `set -o pipefail && ${bashInput.command} 2>&1 | node ${filterPath} && true`;

  return {
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      updatedInput: {
        ...bashInput,
        command: wrappedCommand,
        ...(forceBackground ? { run_in_background: true, } : {}),
      },
    },
  };
}

/**
 Parses raw stdin as a {@link PreToolUseInput}.
 
 Input is trusted; it comes from Claude Code's hook dispatch system.
 
 @param raw - JSON payload from Claude Code stdin
 
 @returns parsed PreToolUse event
 
 @example
 ```ts
 const event = bashOutputFilterParser(await text(process.stdin));
 ```
 */
function bashOutputFilterParser(raw: string,): PreToolUseInput {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
  return JSON.parse(raw,) as PreToolUseInput;
}

/**
 Serializes the bash-output-filter output for stdout.
 
 No trailing newline; matches Claude Code's wire convention.
 
 @param output - {@link BashOutputFilterOutput} handler result to serialize
 
 @returns JSON string for stdout
 
 @mutates output - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 process.stdout.write(bashOutputFilterWriter({}));
 ```
 */
function bashOutputFilterWriter(output: BashOutputFilterOutput,): string {
  return JSON.stringify(output,);
}

export type { BashOutputFilterOutput, };

export {
  bashOutputFilterHandler,
  bashOutputFilterParser,
  bashOutputFilterWriter,
};
