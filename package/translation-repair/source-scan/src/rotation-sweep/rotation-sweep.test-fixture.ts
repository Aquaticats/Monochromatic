import { join, } from 'node:path';

import { assertSaltMovesRotation, } from './rotation-sweep-canary.test-fixture.ts';
import {
  FAMILY_ROOT,
  packagesReachingStage,
  unitTestFilesOf,
} from './rotation-sweep-packages.test-fixture.ts';
import {
  refusalLines,
  SweepOptionRefusalError,
  SWEEP_REFUSED_EXIT,
} from './rotation-sweep-refusal.test-fixture.ts';
import {
  caseKey,
  type FailedCase,
  newFailures,
  reportLines,
  type SaltResult,
  type SweepOutcome,
  sweepPassed,
} from './rotation-sweep-report.test-fixture.ts';
import {
  failedCasesOf,
  filesAtOnce,
  mapBounded,
} from './rotation-sweep-run.test-fixture.ts';

//region Rotation sweep
// THE GUARD OF LEDGER B701, an explicit task and not a seam in the stage package.
//
// WHAT IT ASKS. Does any scripted case pass only for the sheet text its first-round
// window happens to rotate a named seat into? The answer is found by running every
// unit test file of every package that can reach the stage package under each salt
// from 0 to 9 (ten salts cover every start of the largest bench, nine seats), and
// listing, for each salt, the cases it fails that salt 0 passes. A case red under
// salt 0 itself is listed and fails the sweep too, since it cannot be read against
// the salts and would hide its own rotation dependence.
//
// A FAILURE IS CONFIRMED. A file holding a case that a salt failed runs again under
// that salt, and only a case that fails both times is a finding; one that fails once
// is listed as a flake and does not fail the sweep, since a clock-bound case on a
// busy machine is not a rotation finding.

/**
 How many salts a full sweep runs: ten cover every start of the largest bench in the family, which holds nine seats.
 */
const SALT_COUNT = 10;

/**
 Salts a full sweep runs, from 0.
 */
const ALL_SALTS: readonly number[] = Array.from(
  { length: SALT_COUNT, },
  function saltAt(
    _unused,
    at,
  ): number {
    return at;
  },
);

/**
 Milliseconds in a second, for the wall clock a salt is reported with.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 What the sweep was asked to run.
 */
type SweepRequest = {
  /**
   Package directories to sweep; all that reach the stage package when empty.
   */
  readonly packages: readonly string[];
  /**
   Salts to run; salt 0 runs first whatever is asked, since every salt is read against it.
   */
  readonly salts: readonly number[];
};

/**
 Reads the command-line arguments of the sweep.

 @param args - words after the task name, `--package <directory>` and `--salt <n>` repeatable

 @returns The request

 @throws {@link SweepOptionRefusalError} on a word that is no option, an option without a value, or a salt that is not a whole number from 0 to 9

 @example
 ```ts
 const request = sweepRequestOf({ args: ['--package', 'stage', '--salt', '2',], },);
 ```
 */
export function sweepRequestOf({ args, }: { readonly args: readonly string[]; },): SweepRequest {
  /**
   Packages as they are read.
   */
  const packages: string[] = [];
  /**
   Salts as they are read.
   */
  const salts: number[] = [];
  for (const [at, option,] of args.entries()) {
    // Options come in pairs: an even position names the option, the next one holds its value.
    if ((at % 2) !== 0)
      continue;
    /**
     The value that follows the option.
     */
    const value = args[at + 1];
    if (value === undefined)
      throw new SweepOptionRefusalError({ reason: `${option} needs a value`, },);
    if (option === '--package') {
      packages.push(value,);
    }
    else if (option === '--salt') {
      /**
       The salt as a number.
       */
      const salt = Number(value,);
      if ((!Number.isInteger(salt,)) || (salt < 0)
        || (salt >= SALT_COUNT))
        throw new SweepOptionRefusalError({ reason: `--salt takes a whole number from 0 to ${String(SALT_COUNT - 1,)}, got ${value}`, },);
      salts.push(salt,);
    }
    else {
      throw new SweepOptionRefusalError({ reason: `${option} is not an option`, },);
    }
  }
  return {
    packages,
    salts: salts.length === 0
      ? ALL_SALTS
      : [...new Set(salts,),].toSorted(function ascending(
        first,
        second,
      ): number {
        return first - second;
      },),
  };
}

/**
 Runs every file once under a salt.

 @param files - absolute paths of the test files

 @param salt - seats the rotation start is moved by

 @returns Every case any file failed

 @example
 ```ts
 const failed = await runAll({ files, salt: 3, },);
 ```
 */
async function runAll(
  {
    files,
    salt,
  }: {
    readonly files: readonly string[];
    readonly salt: number;
  },
): Promise<readonly FailedCase[]> {
  return (await mapBounded({
    items: files,
    limit: filesAtOnce(),
    work: async function run(file: string,): Promise<readonly FailedCase[]> {
      return await failedCasesOf({
        file,
        salt,
      },);
    },
  },)).flat();
}

/**
 Runs one shifted salt over every file and confirms what it failed.

 @param files - absolute paths of the test files

 @param salt - seats the rotation start is moved by, above zero

 @param unsalted - cases salt 0 failed

 @returns The salt's findings, its flakes and its wall clock

 @example
 ```ts
 const result = await runShifted({ files, salt: 3, unsalted: [], },);
 ```
 */
async function runShifted(
  {
    files,
    salt,
    unsalted,
  }: {
    readonly files: readonly string[];
    readonly salt: number;
    readonly unsalted: readonly FailedCase[];
  },
): Promise<SaltResult> {
  /**
   Clock at the start of this salt.
   */
  const started = performance.now();
  /**
   Every case this salt failed that salt 0 did not.
   */
  const first = newFailures({
    unsalted,
    salted: await runAll({
      files,
      salt,
    },),
  },);
  /**
   Files holding a first failure, which run again to tell a failure of the salt from a failure of the moment.
   */
  const filesToRerun = [...new Set(first.map(function fileOf(failed,): string {
    return join(
      FAMILY_ROOT,
      failed.file,
    );
  },),),];
  /**
   Cases those files fail again.
   */
  const again = newFailures({
    unsalted,
    salted: await runAll({
      files: filesToRerun,
      salt,
    },),
  },);
  /**
   Keys of the cases that failed twice.
   */
  const confirmed = new Set(again.map(function keyOf(failed,): string {
    return caseKey({ failed, },);
  },),);
  return {
    salt,
    failures: first.filter(function failedTwice(failed,): boolean {
      return confirmed.has(caseKey({ failed, },),);
    },),
    flakes: first.filter(function failedOnce(failed,): boolean {
      return !confirmed.has(caseKey({ failed, },),);
    },),
    milliseconds: performance.now() - started,
  };
}

/**
 What reading the command line gave: the request, or the sentence that refuses it.
 */
type AskedRequest =
  | {
    readonly kind: 'request';
    readonly request: SweepRequest;
  }
    | {
      readonly kind: 'refused';
      readonly says: string;
    };

/**
 Reads the command line, keeping a refusal of it apart from a fault.

 @param args - words after the task name

 @returns The request, or the sentence that refuses the command line

 @throws Error on anything but a refusal of the command line, which is a fault of the reader

 @example
 ```ts
 const asked = requestOrRefusal({ args: ['--nap',], },); // { kind: 'refused', says: 'rotation sweep: refused, --nap is not an option; ...' }
 ```
 */
function requestOrRefusal({ args, }: { readonly args: readonly string[]; },): AskedRequest {
  try {
    return {
      kind: 'request',
      request: sweepRequestOf({ args, },),
    };
  }
  catch (error) {
    if (error instanceof SweepOptionRefusalError)
      return {
        kind: 'refused',
        says: error.message,
      };
    throw error;
  }
}

/**
 Writes one line of the report to standard output.

 @param line - the line, without its line break

 @example
 ```ts
 writeLine({ line: 'salt 3: cases failing that salt 0 passes 0', },);
 ```
 */
function writeLine({ line, }: { readonly line: string; },): void {
  process.stdout
    .write(`${line}\n`,);
}

/**
 Runs every file under each salt and reads each salt against salt 0.

 @param request - packages and salts to run

 @param write - receives each line of progress as it is known

 @returns Results per salt after salt 0, and the cases red with no shift

 @throws Error when the canary fails or a package named has no test file

 @example
 ```ts
 const outcome = await runSweep({ request: sweepRequestOf({ args: [], },), write: writeLine, },);
 ```
 */
async function runSweep(
  {
    request,
    write,
  }: {
    readonly request: SweepRequest;
    readonly write: (arguments_: { readonly line: string; },) => void;
  },
): Promise<SweepOutcome> {
  /**
   Directories to sweep.
   */
  const packages = request.packages
    .length
    === 0 ? await packagesReachingStage() : request.packages;
  /**
   Every test file of those packages.
   */
  const files = (await Promise.all(packages.map(function filesOf(packageDirectory,) {
    return unitTestFilesOf({ packageDirectory, },);
  },),)).flat();
  if (files.length === 0)
    throw new Error(`rotation sweep: no unit test file under ${packages.join(', ',)}`,);
  /**
   Salts to run, 0 first.
   */
  const salts = [...new Set([
    0,
    ...request.salts,
  ],),];
  await assertSaltMovesRotation({ salts, },);
  write({
    line: `rotation sweep: test files ${String(files.length,)}, packages ${String(packages.length,)}, salts ${
      salts.join(' ',)
    }; the canary saw each salt move the stage package's rotation`,
  },);
  /**
   Salt 0's run, which no shift touched.
   */
  const unsalted = await runAll({
    files,
    salt: 0,
  },);
  /**
   Results per salt after the first.
   */
  const results: SaltResult[] = [];
  for (const salt of salts.filter(function isShift(candidate,): boolean {
    return candidate !== 0;
  },)) {
    /* oxlint-disable no-await-in-loop -- one salt runs at a time, since each fills the machine with test files and a clock-bound case reads the load */
    /**
     This salt's result.
     */
    const result = await runShifted({
      files,
      salt,
      unsalted,
    },);
    /* oxlint-enable no-await-in-loop */
    results.push(result,);
    write({
      line: `salt ${String(salt,)}: cases failing that salt 0 passes ${String(result.failures
        .length,)}, wall clock ${
        String(Math.round(result.milliseconds / MILLISECONDS_PER_SECOND,),)
      } s`,
    },);
  }
  return {
    results,
    redAtSaltZero: unsalted,
  };
}

/**
 Writes one line of a refusal to standard error, where a reader collecting the report from standard output does not meet it.

 @param line - the line, without its line break

 @example
 ```ts
 writeErrorLine({ line: 'rotation sweep: refused, because ...', },);
 ```
 */
function writeErrorLine({ line, }: { readonly line: string; },): void {
  process.stderr
    .write(`${line}\n`,);
}

/**
 Runs the sweep from the command-line words of the `test:rotation-sweep` task and reports.

 A sweep that found cases to fix writes the report, then a refusal naming each case, and returns the stated-refusal code
 without throwing. An error out of the sweep itself is a fault and propagates with its stack.

 @param args - words after the task name

 @param sweep - the machinery that runs the salts, injected so that a case need not run the family's tests

 @param write - receives each line of the report

 @param writeError - receives each line of a refusal

 @returns The exit code: 0 for a clean sweep, `SWEEP_REFUSED_EXIT` for one that found cases to fix

 @example
 ```ts
 process.exitCode = await rotationSweepMain({ args: [], },);
 ```
 */
export async function rotationSweepMain(
  {
    args,
    sweep = runSweep,
    write = writeLine,
    writeError = writeErrorLine,
  }: {
    readonly args: readonly string[];
    readonly sweep?: typeof runSweep;
    readonly write?: (arguments_: { readonly line: string; },) => void;
    readonly writeError?: (arguments_: { readonly line: string; },) => void;
  },
): Promise<number> {
  /**
   The request the command line makes, or the sentence that refuses it.
   */
  const asked = requestOrRefusal({ args, },);
  if (asked.kind === 'refused') {
    writeError({ line: asked.says, },);
    return SWEEP_REFUSED_EXIT;
  }

  /**
   What the sweep found.
   */
  const outcome = await sweep({
    request: asked.request,
    write,
  },);
  for (const line of reportLines({ outcome, },))
    write({ line, },);
  if (sweepPassed({ outcome, },))
    return 0;
  for (const line of refusalLines({ outcome, },))
    writeError({ line, },);
  return SWEEP_REFUSED_EXIT;
}

//endregion Rotation sweep
