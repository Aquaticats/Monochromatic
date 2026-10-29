//region Rotation sweep report
// WHAT A SUITE'S OUTPUT SAYS ABOUT THE CASES THAT FAILED. The test harness
// (`package/test-harness/core`) logs one error record per failed case, shaped
// `[error] [<timestamp>] [<suite>] [<case>] [FAIL] (<ms>) <cause>`, and one more
// for each suite that holds a failed case, shaped `[error] [<timestamp>] [<suite>]
// [FAIL] (<ms>) Error: <case>`. The tags before ` [FAIL] (` name the case by its
// path through the suites, so that path is the identity a salt's failure is
// compared by.

/**
 Marker that ends the tags of a failure record.
 */
const FAIL_MARKER = ' [FAIL] (';

/**
 Start of an error record: the level, whose tags then begin after the timestamp's closing bracket.
 */
const ERROR_RECORD_START = '[error] [';

/**
 End of the timestamp tag of a record, with the space that follows it.
 */
const TIMESTAMP_END = '] ';

/**
 One case that failed under a salt.
 */
export type FailedCase = {
  /**
   Test file the case ran in, as a path under `package/translation-repair`.
   */
  readonly file: string;
  /**
   Tags of the case, suite first, as the harness logged them.
   */
  readonly name: string;
};

/**
 Name of the failure of a process that exited with an error and logged no failed
 case, so the exit is compared like any case.
 */
export const EXIT_WITHOUT_CASE = '(the test file exited with an error and logged no failed case)';

/**
 Tags of every failure record in a suite's output.

 @param output - everything the test file wrote, standard output and standard error together

 @returns Tags before each failure marker, in order of appearance, a repeated one once

 @example
 ```ts
 const names = failureTags({ output: '[error] [2026-10-20T00:00:00.000Z] [suite] [case] [FAIL] (3ms) Error', },);
 ```
 */
export function failureTags({ output, }: { readonly output: string; },): readonly string[] {
  /**
   Tags found, each once.
   */
  const found = new Set<string>();
  for (const line of output.split('\n',)) {
    if (!line.startsWith(ERROR_RECORD_START,))
      continue;
    /**
     Where the timestamp's closing bracket sits.
     */
    const timestampEnd = line.indexOf(
      TIMESTAMP_END,
      ERROR_RECORD_START.length,
    );
    if (timestampEnd === (-1))
      continue;
    /**
     Where the error record's own tags start: after the timestamp.
     */
    const tagsStart = timestampEnd + TIMESTAMP_END.length;
    /**
     Where the failure marker sits in the line.
     */
    const marker = line.indexOf(
      FAIL_MARKER,
      tagsStart,
    );
    if (marker === (-1))
      continue;
    found.add(line.slice(
      tagsStart,
      marker,
    ),);
  }
  return [...found,];
}

/**
 The cases a suite failed: the deepest tags only, since a suite that holds a
 failed case logs its own failure with the tags of the suite alone.

 @param output - everything the test file wrote

 @param exitCode - code the test file exited with

 @returns Tags of each failed case, a suite's rollup left out; one pseudo case when the exit failed with no record

 @example
 ```ts
 const names = failedCaseNames({ output: '', exitCode: 1, },);
 ```
 */
export function failedCaseNames(
  {
    output,
    exitCode,
  }: {
    readonly output: string;
    readonly exitCode: number;
  },
): readonly string[] {
  /**
   Every failure record's tags.
   */
  const tags = failureTags({ output, },);
  /**
   Tags that no other failure record extends.
   */
  const deepest = tags.filter(function isDeepest(candidate,): boolean {
    return !tags.some(function extendsCandidate(other,): boolean {
      return (other !== candidate) && other.startsWith(`${candidate} [`,);
    },);
  },);
  if ((deepest.length === 0) && (exitCode !== 0))
    return [EXIT_WITHOUT_CASE,];
  return deepest;
}

/**
 A case's identity across salts.

 @param failed - case and its file

 @returns One string naming both

 @example
 ```ts
 const key = caseKey({ failed: { file: 'stage/src/a.unit.test.ts', name: '[suite] [case]', }, },);
 ```
 */
export function caseKey({ failed, }: { readonly failed: FailedCase; },): string {
  return `${failed.file} ${failed.name}`;
}

/**
 Cases a salt failed that the unsalted run passed.

 @param unsalted - cases salt 0 failed

 @param salted - cases the salt failed

 @returns Salted failures salt 0 did not have, in the order given

 @example
 ```ts
 const fresh = newFailures({ unsalted: [], salted: [], },);
 ```
 */
export function newFailures(
  {
    unsalted,
    salted,
  }: {
    readonly unsalted: readonly FailedCase[];
    readonly salted: readonly FailedCase[];
  },
): readonly FailedCase[] {
  /**
   Keys of the cases already red with no shift.
   */
  const red = new Set(unsalted.map(function keyOf(failed,): string {
    return caseKey({ failed, },);
  },),);
  return salted.filter(function isNew(failed,): boolean {
    return !red.has(caseKey({ failed, },),);
  },);
}


/**
 What the sweep found for one salt.
 */
export type SaltResult = {
  /**
   The salt.
   */
  readonly salt: number;
  /**
   Cases this salt failed that salt 0 passed, a rerun of the file having failed them again.
   */
  readonly failures: readonly FailedCase[];
  /**
   Cases that failed once under this salt and passed on a rerun.
   */
  readonly flakes: readonly FailedCase[];
  /**
   Wall clock the salt took, in milliseconds.
   */
  readonly milliseconds: number;
};

/**
 What a whole sweep found.
 */
export type SweepOutcome = {
  /**
   Results per salt after salt 0.
   */
  readonly results: readonly SaltResult[];
  /**
   Cases red with no shift.
   */
  readonly redAtSaltZero: readonly FailedCase[];
};

/**
 A case that failed under at least one salt, with the salts that failed it.
 */
type CaseBySalts = {
  /**
   The case and its file.
   */
  readonly failed: FailedCase;
  /**
   The salt that failed it first, which reruns it alone.
   */
  readonly firstSalt: number;
  /**
   Every salt that failed it, in the order the salts ran.
   */
  readonly salts: readonly number[];
};

/**
 The cases a sweep's salts failed that salt 0 passed, each once with every salt that failed it.

 @param outcome - what the sweep found

 @returns Cases in the order their first salt ran

 @example
 ```ts
 const cases = casesBySalts({ outcome: { results: [], redAtSaltZero: [], }, },);
 ```
 */
export function casesBySalts({ outcome, }: { readonly outcome: SweepOutcome; },): readonly CaseBySalts[] {
  /**
   Salts that fail each case, by file and tags.
   */
  const saltsOf = new Map<string, CaseBySalts>();
  for (const result of outcome.results) {
    for (const failed of result.failures) {
      saltsOf.set(
        caseKey({ failed, },),
        {
          failed,
          firstSalt: saltsOf.get(caseKey({ failed, },),)
            ?.firstSalt
            ?? result.salt,
          salts: [
            ...saltsOf.get(caseKey({ failed, },),)
              ?.salts
              ?? [],
            result.salt,
          ],
        },
      );
    }
  }
  return [...saltsOf.values(),];
}

/**
 Lines that report a sweep's outcome: every failing case with the salts that fail it, the flakes, the cases red with no shift, and the verdict.

 @param outcome - what the sweep found

 @returns Lines of the report, ending in the verdict

 @example
 ```ts
 const lines = reportLines({ outcome: { results: [], redAtSaltZero: [], }, },);
 ```
 */
export function reportLines({ outcome, }: { readonly outcome: SweepOutcome; },): readonly string[] {
  /**
   Cases failed under a salt that salt 0 passes.
   */
  const cases = casesBySalts({ outcome, },);
  /**
   Lines of the report.
   */
  const lines: string[] = cases.map(function failLine({
    failed,
    salts,
  },): string {
    return `FAIL under salts ${salts.join(', ',)}: ${caseKey({ failed, },)}`;
  },);
  for (const result of outcome.results) {
    for (const flaked of result.flakes) {
      lines.push(`flake under salt ${String(result.salt,)} (failed once, passed on rerun): ${caseKey({ failed: flaked, },)}`,);
    }
  }
  for (const red of outcome.redAtSaltZero) {
    lines.push(`red with no shift (salt 0), which cannot be read against the salts: ${caseKey({ failed: red, },)}`,);
  }
  lines.push(
    `rotation sweep: cases failing under a salt that salt 0 passes ${String(cases.length,)}, cases red with no shift ${
      String(outcome.redAtSaltZero
        .length,)
    }`,
    sweepPassed({ outcome, },) ? 'rotation sweep: clean' : 'rotation sweep: NOT clean'
  );
  return lines;
}

/**
 Whether a sweep passed: no case failed under a salt that salt 0 passed, and none is red with no shift.

 @param outcome - what the sweep found

 @returns Whether it is clean

 @example
 ```ts
 const clean = sweepPassed({ outcome: { results: [], redAtSaltZero: [], }, },);
 ```
 */
export function sweepPassed({ outcome, }: { readonly outcome: SweepOutcome; },): boolean {
  return (outcome.redAtSaltZero
    .length
    === 0)
    && outcome.results
      .every(function clean(result,): boolean {
        return result.failures
          .length
          === 0;
      },);
}

//endregion Rotation sweep report
