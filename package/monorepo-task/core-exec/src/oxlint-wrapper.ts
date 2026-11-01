#!/usr/bin/env node

/**
 CLI wrapper for `oxlint` that augments diagnostic output.
 
 Runs `oxlint` with all provided arguments, captures the output, augments
 diagnostics with enhanced guidance via {@link augmentOxlintOutput}, and
 preserves the original exit code.
 
 When the caller passes a fix-applying flag (`--fix`, `--fix-suggestions`, or
 `--fix-dangerously`), the wrapper loops oxlint until the codebase reaches a
 fixpoint (see {@link fixUntilStable}) instead of running a single pass, so
 overlapping autofixes that oxlint defers to a later pass fully converge in
 one `task-oxlint` invocation.
 
 @example
 ```bash
 task-oxlint
 task-oxlint --fix
 ```
 */

import spawn from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import { augmentOxlintOutput, } from './oxlint-augment.ts';
import {
  fixUntilStable,
  MAX_AUTOFIX_PASSES,
  type OxlintRunResult,
} from './oxlint-fix-loop.ts';

//region Argument construction

/**
 Thread count override from environment.
 
 When set, injects `--threads <value>` into the oxlint arguments.
 oxlint ignores `RAYON_NUM_THREADS` because it always passes an explicit
 count to rayon's `ThreadPoolBuilder`, so this env var is the only way
 to control threads without modifying every call site.
 
 @example Set in a mise task env block:
 ```toml
 [tasks.lint.env]
 OXLINT_THREADS = "1"
 ```
 */
const threadOverride = process.env
  .OXLINT_THREADS;

/**
 Whether the caller already passed an explicit output format flag.
 
 The wrapper pins `--format=default` so diagnostic augmentation always receives
 Oxlint's graphical block reporter. Oxlint's piped default reporter varies by
 version and can emit a compact one-line format that has no source-context
 boundary for guidance injection. An explicit caller `--format`/`-f` wins.
 */
const hasExplicitFormat = process.argv
  .slice(2,)
  .some(function isFormatFlag(arg,) {
    return (arg === '--format')
      || (arg === '-f')
      || arg.startsWith('--format=',)
      || arg.startsWith('-f=',);
  },);

/**
 Arguments forwarded to oxlint, identical on every pass of the fix loop.
 */
const oxlintArgs = [
  ...(((threadOverride !== undefined) && (threadOverride !== ''))
    ? [
      '--threads',
      threadOverride,
    ]
    : []),
  ...(hasExplicitFormat ? [] : ['--format=default',]),
  ...process.argv
    .slice(2,),
];

/**
 oxlint's fix-applying flags, stripped to build the oracle lint arguments.
 */
const FIX_FLAGS = new Set([
  '--fix',
  '--fix-suggestions',
  '--fix-dangerously',
],);

/**
 Whether the caller passed `--fix`, the documented multi-pass convergence case.
 
 `--fix` alone is the loop trigger. `--fix-suggestions` or `--fix-dangerously`
 passed without `--fix` stay single-pass: the oracle lint is not verified to
 track suggestion-applied changes, and "may change program behavior" fixes
 should not be iterated on their own. A plain lint run produces identical
 output every pass, so it never loops.
 */
const hasFixFlag = process.argv
  .slice(2,)
  .includes('--fix',);

/**
 Plain lint arguments: the forwarded arguments with every fix flag removed.
 
 Used as the convergence oracle, so it reports the post-fix file's remaining
 violations (fixable and unfixable) rather than applying anything.
 */
const oracleArgs = oxlintArgs
  .filter(function isNotFixFlag(arg,) {
    return !FIX_FLAGS.has(arg,);
  },);

//endregion Argument construction

//region Execution

/**
 Runs `oxlint` once and normalizes nano-spawn's success and failure shapes.
 
 nano-spawn resolves on a zero exit and throws a `SubprocessError` (carrying
 captured stdout/stderr plus `exitCode`/`signalName`) on a non-zero exit or
 signal; a missing binary throws a plain error. This collapses all three into
 one {@link OxlintRunResult} so the fix loop and {@link finalizeResult} read
 them uniformly.
 
 @param args - fully constructed oxlint arguments
 
 @returns normalized run result
 */
async function runOxlint(args: readonly string[],): Promise<OxlintRunResult> {
  try {
    /**
     Successful (exit 0) oxlint result; stdout still carries the `Found 0 ...` summary.
     */
    const result = await spawn(
      'oxlint',
      [...args,],
    );
    return {
      stdout: result.stdout,
      stderr: result.stderr,
      exitCode: 0,
    };
  }
  catch (error) {
    if ((error !== null) && ((typeof error) === 'object')
      && ('exitCode' in error)) {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- 'exitCode' in check above narrows error to the captured-subprocess shape */
      /**
       Re-typed thrown error so its captured stdout, stderr, and exit fields can be normalized.
       */
      const subprocessError = error as {
        stdout?: string;
        stderr?: string;
        exitCode?: number;
        signalName?: string;
      };
      /* oxlint-enable typescript/no-unsafe-type-assertion */
      return {
        stdout: subprocessError.stdout
          ?? '',
        stderr: subprocessError.stderr
          ?? '',
        ...((subprocessError.exitCode === undefined)
          ? {}
          : { exitCode: subprocessError.exitCode, }),
        ...((subprocessError.signalName === undefined)
          ? {}
          : { signalName: subprocessError.signalName, }),
      };
    }
    return {
      stdout: '',
      stderr: '',
      executionError: caughtValueText(error,),
    };
  }
}

/**
 Writes a normalized oxlint result to the parent streams and sets the exit code.
 
 Mirrors Oxlint's result exactly: standard output receives diagnostic guidance,
 standard error passes through, and every nonzero exit remains nonzero.
 An execution failure reports and exits with status one.
 
 @param result - final run result from the loop or a single pass
 */
function finalizeResult(result: OxlintRunResult,): void {
  if (result.executionError !== undefined) {
    console.error(`[task-oxlint] failed to execute oxlint: ${result.executionError}`,);
    process.exitCode = 1;
    return;
  }

  if (result.exitCode === 0) {
    if (result.stdout
      .length
      > 0)
      process.stdout
        .write(augmentOxlintOutput(result.stdout,),);
    if (result.stderr
      .length
      > 0)
      process.stderr
        .write(result.stderr,);
    return;
  }

  /**
   Oxlint diagnostics with wrapper guidance appended.
   */
  const augmentedStdout = augmentOxlintOutput(result.stdout,);
  if (augmentedStdout.length > 0) {
    process.stdout
      .write(augmentedStdout,);
    if (!augmentedStdout.endsWith('\n',))
      process.stdout
        .write('\n',);
  }

  if (result.stderr
    .length
    > 0) {
    process.stderr
      .write(result.stderr,);
    if (!result.stderr
      .endsWith('\n',))
      process.stderr
        .write('\n',);
  }

  process.exitCode = result.exitCode
    ?? 1;

  if ((result.signalName
    !== undefined)
    && (result.signalName
      !== ''))
    {
    console.error(`[task-oxlint] oxlint terminated by signal: ${result.signalName}`,);
    process.exitCode = 1;
  }
}

//endregion Execution

//region Main

if (hasFixFlag) {
  /**
   Fix-loop outcome; the `--fix` passes only apply fixes, the final oracle lint is forwarded.
   */
  const outcome = await fixUntilStable({
    runFix: function runFixPass() {
      return runOxlint(oxlintArgs,);
    },
    runLint: function runOraclePass() {
      return runOxlint(oracleArgs,);
    },
    maxPasses: MAX_AUTOFIX_PASSES,
  },);
  if (outcome.stopReason === 'cycle')
    console.error(
      `[task-oxlint] autofix oscillates between two states after ${outcome.passes} passes; two rules' fixes conflict, so this cannot converge. Fix the rule conflict or disable one rule's autofix.`,
    );
  if (outcome.stopReason === 'cap')
    console.error(
      `[task-oxlint] reached ${MAX_AUTOFIX_PASSES} autofix passes without converging; remaining diagnostics may be incomplete`,
    );
  finalizeResult(outcome.result,);
}
else {
  finalizeResult(await runOxlint(oxlintArgs,),);
}

//endregion Main
