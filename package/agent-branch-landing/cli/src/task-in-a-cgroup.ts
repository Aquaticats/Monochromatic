/**
 One `mise` task in a cgroup of its own, and the few numbers a report reads from what it printed.

 @module
 */

import {
  COMPILE_ONLY_ARGUMENT,
  SYSTEMD_RUN_PROGRAM,
  TASK_OUTPUT_BYTES,
  type CommandLimits,
} from './constants.ts';
import type { Repository, } from './git-commands.ts';
import type { ProgramOutput, } from './run-a-program.ts';

/**
 The word one of the repository's suites writes before its passing or failing case count.
 */
const PASS_WORD = 'pass';

/**
 What the same suite writes before its failing case count.
 */
const FAIL_WORD = 'fail';

/**
 What a report shows for a count the output did not carry.
 */
const NO_COUNT = '-';

/**
 Words that put one command in its own cgroup with the limits named for it, which is what `AGENTS.md` rules `RXI` and `RXS`
 ask of every command that is not trivial.

 @param limits - the limits this kind of work is given

 @param collect - whether its unit is unloaded once it ends, which a compile asks for and a task does not

 @returns the words `systemd-run` is called with, before the command itself

 @example
 ```ts
 const words = cgroupWordsOf({ limits: PACKAGE_TASK_LIMITS, collect: true, },);
 ```
 */
export function cgroupWordsOf({
  limits,
  collect,
}: {
  readonly limits: CommandLimits;
  readonly collect: boolean;
},): readonly string[] {
  return [
    '--user',
    '--scope',
    ...(collect ? ['--collect',] : []),
    '--quiet',
    `--property=CPUQuota=${limits.cpuQuota}`,
    `--property=MemoryMax=${limits.memoryMax}`,
    ...(limits.memorySwapMax === undefined ? [] : [`--property=MemorySwapMax=${limits.memorySwapMax}`,]),
    `--property=TasksMax=${limits.tasksMax}`,
    `--property=IOWeight=${limits.ioWeight}`,
    '--',
  ];
}

/**
 What the words after a task are when the task must compile and stop, which is the whole check a landing runs on the wrapper's
 two targets.

 @returns the arguments placed after the task name

 @example
 ```ts
 const argumentsAfterTask = compileArgumentsOf();
 ```
 */
export function compileArgumentsOf(): readonly string[] {
  return [
    '--',
    COMPILE_ONLY_ARGUMENT,
  ];
}

/**
 Runs one `mise` task of a checkout in its own cgroup.

 @param repository - the checkout whose task runs

 @param task - the task, repository-qualified

 @param argumentsAfterTask - its arguments, each kept whole

 @param limits - the limits this kind of work is given

 @param collect - whether its unit is unloaded once it ends

 @returns what the task printed and how it ended

 @example
 ```ts
 const ran = await runTaskInACgroup({ repository: worktree, task: '//package/a/b:lint:types', argumentsAfterTask: [], limits, collect: false, },);
 ```
 */
export function runTaskInACgroup({
  repository,
  task,
  argumentsAfterTask,
  limits,
  collect,
}: {
  readonly repository: Repository;
  readonly task: string;
  readonly argumentsAfterTask: readonly string[];
  readonly limits: CommandLimits;
  readonly collect: boolean;
},): Promise<ProgramOutput> {
  return repository.run({
    program: SYSTEMD_RUN_PROGRAM,
    words: [
      ...cgroupWordsOf({
        limits,
        collect,
      },),
      'mise',
      'run',
      task,
      ...argumentsAfterTask,
    ],
    folder: repository.folder,
    timeoutMilliseconds: repository.timeoutMilliseconds,
    outputBytes: TASK_OUTPUT_BYTES,
  },);
}

/**
 Tells whether a text is a whole number, so a count is read only where the output carried one.

 @param text - what a line held

 @returns true when every character is a digit and there is at least one

 @example
 ```ts
 const numeric = isWholeNumber('12',);
 ```
 */
function isWholeNumber(text: string,): boolean {
  return (text !== '')
    && text.split('',)
      .every(function isDigit(character: string,) {
        return (character >= '0') && (character <= '9');
      },);
}

/**
 Reads the count one line carries after the suite's word, when it carries one at the end.

 @param line - one line of the output

 @param word - the word the suite writes before its count

 @returns the count the line carried, empty when it carried none

 @example
 ```ts
 const counts = countsOn({ line: ' pass 12', word: 'pass', },);
 ```
 */
function countsOn({
  line,
  word,
}: {
  readonly line: string;
  readonly word: string;
},): readonly string[] {
  /**
   The line's own words.
   */
  const words = line.trim()
    .split(' ',);
  /**
   Where the suite's word sits, which is nowhere on most lines.
   */
  const at = words.lastIndexOf(word,);
  /**
   What follows it.
   */
  const value = words.at(at + 1,);
  /**
   Whether the count the suite wrote ends this line.
   */
  const endsTheLine = (at !== (-1)) && (at === (words.length - 2));
  if (!endsTheLine) return [];
  if (value === undefined) return [];
  return isWholeNumber(value,) ? [value,] : [];
}

/**
 Reads the count one of the repository's suites wrote at the end of a line.

 @param lines - the output's lines

 @param word - the word the suite writes before its count

 @returns the last count the output carried, or what a report shows for none

 @example
 ```ts
 const passed = countAfter({ lines: [' pass 12',], word: 'pass', },);
 ```
 */
function countAfter({
  lines,
  word,
}: {
  readonly lines: readonly string[];
  readonly word: string;
},): string {
  /**
   Every count the output carried, in the order it carried them.
   */
  const counts = lines.flatMap(function countOn(line: string,) {
    return countsOn({
      line,
      word,
    },);
  },);
  return counts.at(-1,) ?? NO_COUNT;
}

/**
 Counts what a test task reported, in both output shapes the repository's suites use.

 @param output - what the task printed

 @returns the harness's passing and failing cases, and node's counts as the text it wrote

 @example
 ```ts
 const tally = tallyOf('[PASS] a\npass 3\n',);
 ```
 */
export function tallyOf(output: string,): {
  readonly passed: number;
  readonly failed: number;
  readonly nodePassed: string;
  readonly nodeFailed: string;
} {
  /**
   The output's lines, each of which may carry one case or one count.
   */
  const lines: readonly string[] = output.split('\n',);
  return {
    passed: lines.filter(function isPassed(line: string,) {
      return line.includes('[PASS]',);
    },)
      .length,
    failed: lines.filter(function isFailed(line: string,) {
      return line.includes('[FAIL]',);
    },)
      .length,
    nodePassed: countAfter({
      lines,
      word: PASS_WORD,
    },),
    nodeFailed: countAfter({
      lines,
      word: FAIL_WORD,
    },),
  };
}

/**
 Says what one task did, in the line a report keeps for it.

 @param task - the task that ran

 @param status - the status it exited with

 @param output - what it printed

 @returns the one line naming the task and its counts

 @example
 ```ts
 const line = tallyLineOf({ task: '//package/a/b:test:unit', status: 0, output: '', },);
 ```
 */
export function tallyLineOf({
  task,
  status,
  output,
}: {
  readonly task: string;
  readonly status: number;
  readonly output: string;
},): string {
  /**
   What the task's suites reported.
   */
  const tally = tallyOf(output,);
  return `${task}: status ${String(status,)}, PASS lines ${String(tally.passed,)}, FAIL lines ${String(tally.failed,)}, node pass ${tally.nodePassed}, node fail ${tally.nodeFailed}`;
}

/**
 Picks the lines of a compilation failure that name what broke, which is a Rust error's own line and the line naming its
 location.

 @param output - what the task printed

 @param limit - how many lines to keep

 @returns the lines, without repeats, at most the limit

 @example
 ```ts
 const errors = errorLinesOf({ output: 'error[E0308]: mismatched types\n', limit: 30, },);
 ```
 */
export function errorLinesOf({
  output,
  limit,
}: {
  readonly output: string;
  readonly limit: number;
},): readonly string[] {
  return [...new Set(output.split('\n',)
    .filter(function namesTheFailure(line: string,) {
      return line.startsWith('error',) || line.includes(' --> ',);
    },),),].slice(
    0,
    limit,
    );
}

/**
 Keeps the end of what a task printed, which is where a failing suite explains itself.

 @param output - what the task printed

 @param characters - how much of the end to keep

 @returns that many characters at most

 @example
 ```ts
 const tail = tailOf({ output: 'a long failure', characters: 8, },);
 ```
 */
export function tailOf({
  output,
  characters,
}: {
  readonly output: string;
  readonly characters: number;
},): string {
  return output.slice(-characters,);
}
