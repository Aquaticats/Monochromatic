import { expect, } from '@monochromatic-dev/test-harness-core/ts';

import { reportingRefusals, } from '../../dist/final/node/index.mjs';

//region CLI refusal cases
// WHAT THE CLI REFUSAL SUITES SHARE: the two disposable swaps they make, the
// process arguments and environment they hand in, and the name and the body
// of the case an operator's own mistake is reported by.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. `cli-refusal.unit.test.ts`,
// `bench-report-refusal.unit.test.ts` and `coverage-census-refusals.unit.test.ts`
// each need these, and a copy in each would drift.

/**
 Exit code a CLI leaves behind when it declined in its own words.
 */
export const REFUSED_AS_STATED = 6;

/**
 Process arguments of a run that typed nothing after the script, which every
 command reads (ledger B75).
 */
export const BARE_ARGV: readonly string[] = [
  '/usr/bin/node',
  'dist/final/node/runner.mjs',
];

/**
 Environment of a process that holds no provider key and was started by no
 task, which no case here reads for anything but the guard.
 */
export const NO_KEYS: Readonly<NodeJS.ProcessEnv> = {};

/**
 Collects what would have gone to stderr, restoring the real one on disposal.

 @param lines - collector the caller reads afterwards

 @returns Collected lines, and the restore that disposal runs

 @example
 ```ts
 using printed = collectingErrors({ lines: [], },);
 ```
 */
export function collectingErrors(
  { lines, }: { readonly lines: string[]; },
): { readonly lines: readonly string[]; } & Disposable {
  /**
   Real reporter, put back on disposal.
   */
  const reported = console.error;

  console.error = function collect(text: unknown,): void {
    lines.push(String(text,),);
  };
  return {
    lines,
    [Symbol.dispose]: function putBack(): void {
      console.error = reported;
    },
  };
}

/**
 Puts the process exit code back to whatever it was, however a case ends.

 @returns Restore that disposal runs

 @example
 ```ts
 using held = holdingExitCode();
 ```
 */
export function holdingExitCode(): Disposable {
  /**
   Exit code standing before this case ran.
   */
  const before = process.exitCode;

  return {
    [Symbol.dispose]: function putBack(): void {
      process.exitCode = before;
    },
  };
}

/**
 Holds that an operator's own mistake is reported in its own words at the
 stated-refusal code, with no fault line or frames; the case body, which each
 suite hands to `it` itself.

 @param refusal - refusal the command body throws

 @example
 ```ts
 await reportsMistake({ refusal: new HardCapOverrideError({ value: 'soon', },), },);
 ```
 */
export async function reportsMistake({ refusal, }: { readonly refusal: Error; },): Promise<void> {
  /**
   Puts the exit code back when this scope ends.
   */
  using held = holdingExitCode();
  /**
   Lines the reporter printed, with the real reporter put back when this scope ends.
   */
  using printed = collectingErrors({ lines: [], },);

  await reportingRefusals({
    what: 'draw-sample',
    argv: BARE_ARGV,
    env: NO_KEYS,
    run: function refuse(): Promise<never> {
      return Promise.reject(refusal,);
    },
  },);

  expect(process.exitCode,)
    .toBe(REFUSED_AS_STATED,);
  expect(printed.lines,)
    .toEqual([`draw-sample: ${refusal.message}`,],);
}

/**
 Names the case holding that an operator's own mistake is reported in its own
 words, so the suites that split the list of classes keep one name.

 @param refusal - refusal the case reports

 @returns Case name, starting with the class it is about

 @example
 ```ts
 it({ name: caseName({ refusal, },), fn: () => reportsMistake({ refusal, },), },);
 ```
 */
export function caseName(
  { refusal, }: { readonly refusal: Readonly<Pick<Error, 'name'>>; },
): string {
  return `REPEATS ${refusal.name}, the operator's own mistake, in its own words at the stated-refusal code `
    + 'with no fault line or frames';
}

//endregion CLI refusal cases
