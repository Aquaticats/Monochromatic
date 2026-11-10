/**
 Shared guardrail helpers for misleading `bun test` invocations.
 
 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';
import { isBunTestCommandLine, } from './bun-test-command-line.ts';
import { unwrapWrappers, } from './command-wrapper-words.ts';
import { rawCommandWordRuns, } from './raw-command-word-runs.ts';
import { shellScriptPayloads, } from './shell-script-payloads.ts';

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
 
 Program wrappers are stripped before the subcommand is read, so `env bun test`,
 `timeout 10 bun test`, and `command bun test` reach the same answer as `bun test`,
 and `bun` global options are read so `bun --cwd /tmp test` does not hide `test` behind
 an option value. A shell `-c` script string and the source a wrapper such as `eval` joins
 are read as command lines of their own, so `bash -c 'bun test'` is refused too.
 
 A command the parser rejects is read from its raw words instead of answering "no":
 the visible text is the only evidence there is, and a guard that reads nothing from an
 unparsable command lets `bun test "unterminated` through.
 
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
    return rawTextInvokesBunTest({ source: command, },);

  return analysis.commands
    .some(function commandIsBunTest(info,): boolean {
      /**
       Innermost program words, with every wrapper layer the command line spelled stripped.
       */
      const wrapped = unwrapWrappers({
        words: [
          info.name,
          ...info.args,
        ],
      },);
      if (isBunTestCommandLine({ words: wrapped.words, },))
        return true;

      /**
       Command lines this record launches from among its own words.
       */
      const payloads = [
        ...wrapped.payloads,
        ...shellScriptPayloads({ words: wrapped.words, },),
      ];

      return payloads
        .some(function payloadIsBunTest(payload,): boolean {
          return rawTextInvokesBunTest({ source: payload, },);
        },);
    },);
}

/**
 Checks whether shell source text holds a command line that runs `bun test`.
 
 Reads raw words only, so it serves both an unparsable command and the command lines
 wrappers such as `eval` join into shell source. A work list rather than recursion keeps
 the walk flat, because each payload is itself a command line that may carry wrappers again.
 
 @param source - shell source text to read
 
 @returns whether any command position in the text spells `bun test`
 
 @example
 ```ts
 rawTextInvokesBunTest({ source: 'env bun test "' }); // true
 ```
 */
function rawTextInvokesBunTest({ source, }: { readonly source: string; },): boolean {
  /**
   Command lines still to read, the outer source text first.
   */
  const pending: string[] = [source,];
  /**
   Command lines already read, so a wrapper repeating its own text ends the walk.
   */
  const read = new Set<string>();

  while (pending.length > 0) {
    /**
     Command line popped from the work list.
     */
    const current = pending.pop();
    if (current === undefined)
      break;
    if (read.has(current,))
      continue;
    read.add(current,);

    for (const words of rawCommandWordRuns({ source: current, },)) {
      /**
       Innermost program words of this run, with its wrapper layers stripped.
       */
      const wrapped = unwrapWrappers({ words, },);
      if (isBunTestCommandLine({ words: wrapped.words, },))
        return true;
      pending.push(
        ...wrapped.payloads,
        ...shellScriptPayloads({ words: wrapped.words, },),
      );
    }
  }

  return false;
}

//endregion Predicates

export {
  BUN_TEST_BAN_REASON,
  invokesBunTest,
};
