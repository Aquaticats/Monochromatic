/**
 Bash command guardrails adopted from Claude Code guardrail.
 
 Two guards run on every Bash tool call:
 `bun test` is refused because it reads no package task,
 and a mutating Git subcommand spelled as a path to the real Git binary is refused
 because that path skips the repository's Git wrapper.
 Read-only Git probes through the same path stay allowed.
 
 @module
 */

import {
  BUN_TEST_BAN_REASON,
  hasNotUnderstoodShellSyntax,
  invokesBunTest,
  invokesMutatingGitThroughRealGitPath,
  NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
  REAL_GIT_MUTATION_BAN_REASON,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer/ts';
import {
  GUARDRAIL_NOT_BLOCKED,
  type GuardrailDecision,
} from './types.ts';
import { isRecord, } from './value.ts';

//region Types

/**
 Inputs for evaluating the Bash tool guards.
 */
type BashGuardCheck = {
  /**
   Pi Bash tool input, read defensively because it comes from a model.
   */
  readonly input: unknown;
  /**
   Whether the configured `bun test` guard is active; the real-Git guard has no off switch.
   */
  readonly blockBunTest?: boolean;
  /**
   Executable names that identify the real Git binary; tests inject them so fixtures need no host installation.
   */
  readonly realGitBasenames?: readonly string[];
};

//endregion Types

//region Bash guard evaluation

/**
 Applies the Bash tool guards to a pi Bash tool input.
 
 The real-Git check resolves the host's real Git executable, so the answer is asynchronous.
 Ordering is the guards' precedence: `bun test` first, then the absolute real-Git path.
 
 @param input - pi Bash tool input
 
 @param blockBunTest - whether the configured `bun test` guard is active, defaulting to active
 
 @param realGitBasenames - executable names treated as the real Git binary, defaulting to the resolved install
 
 @returns block decision when a guard matches, otherwise {@link GUARDRAIL_NOT_BLOCKED}
 
 @example
 ```typescript
 await evaluateBashGuard({ input: { command: 'bun test' } });
 ```
 */
async function evaluateBashGuard({
  input,
  blockBunTest = true,
  realGitBasenames,
}: BashGuardCheck,): Promise<GuardrailDecision> {
  if (!isRecord(input,))
    return GUARDRAIL_NOT_BLOCKED;

  /**
   Command candidate read defensively from external tool input.
   */
  const { command, } = input;
  if ((typeof command) !== 'string')
    return GUARDRAIL_NOT_BLOCKED;

  if (hasNotUnderstoodShellSyntax(command,)) {
    return {
      block: true,
      reason: NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
    };
  }

  if (blockBunTest && invokesBunTest(command,)) {
    return {
      block: true,
      reason: BUN_TEST_BAN_REASON,
    };
  }

  /**
   Whether the command mutates the repository by running the real Git binary as a path,
   which no Git wrapper on PATH can see.
   */
  const mutatesThroughRealGitPath = await invokesMutatingGitThroughRealGitPath({
    command,
    ...(realGitBasenames === undefined ? {} : { realGitBasenames, }),
  },);
  if (!mutatesThroughRealGitPath)
    return GUARDRAIL_NOT_BLOCKED;

  return {
    block: true,
    reason: REAL_GIT_MUTATION_BAN_REASON,
  };
}

//endregion Bash guard evaluation

export { evaluateBashGuard, };
export type { BashGuardCheck, };
