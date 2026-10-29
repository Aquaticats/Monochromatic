/**
 Shared guardrail helpers for misleading `bun test` invocations.
 
 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';

//region Constants

/**
 Refusal reason used when a shell command invokes `bun test`.
 
 @example
 ```ts
 return BUN_TEST_BAN_REASON;
 ```
 */
const BUN_TEST_BAN_REASON: string = [
  'Blocked: `bun test` invocations are banned in this repo.',
  'The custom `@monochromatic-dev/test-harness-core` harness runs tests as a side effect of module import,',
  "so `bun test <file>` reports `0 pass / 0 fail` even when every test passed (the harness's `PASS`",
  "log lines are not measured by bun's test runner).",
  'Use `mise run //package/<path>:test:unit` instead. When no such task exists, add one to the',
  "target package's `mise.toml` first. For ad-hoc single-file runs use `node <file>` directly",
  '(no `test` subcommand).',
]
  .join(' ',);

//endregion Constants

//region Predicates

/**
 Checks whether shell command contains a `bun test` command anywhere in parsed shell syntax.
 
 Uses the shared `unbash` analyzer so quoted prose, escaped characters,
 nested command substitutions, and function definitions are classified by
 shell grammar instead of text boundaries. Function bodies stay visible so
 `f(){ bun test; }; f` cannot hide the banned invocation behind a shell name.
 
 @param command - shell command from agent harness Bash tool input
 
 @returns whether command contains `bun test`
 
 @example
 ```ts
 invokesBunTest('cd x && bun test'); // true
 invokesBunTest('echo "bun test"'); // false
 ```
 */
function invokesBunTest(command: string,): boolean {
  /**
   Parsed shell command analysis.
   */
  const analysis = analyzeShellCommand(command,);
  if (!analysis.parsed)
    return false;

  return analysis.commands
    .some(function commandIsBunTest(info,): boolean {
      return (info.name === 'bun')
        && (info.args[0] === 'test');
    },);
}

//endregion Predicates

export {
  BUN_TEST_BAN_REASON,
  invokesBunTest,
};
