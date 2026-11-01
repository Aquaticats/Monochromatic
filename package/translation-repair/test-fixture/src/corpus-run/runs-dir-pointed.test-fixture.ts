//region Runs directory pointed
// POINTS THE RUNS DIRECTORY VARIABLE AT A DIRECTORY THE CODE UNDER TEST MUST
// NOT USE, for the cases showing that a reader or writer handed its runs
// directory uses that one rather than `TRANSLATION_REPAIR_RUNS_DIR`.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Four test files kept a copy each, since
// the global-writes scan once followed a writer only within its own file
// (ledger B111); it follows a writer into every file that imports it by name
// since ledger B120, so the copies are this one. `process.env` is
// process-wide, so every suite around a case calling this runs one case at a
// time (`global-writes-sequenced.unit.test.ts`).

/**
 Variable naming the runs directory.
 */
const RUNS_DIR_VARIABLE = 'TRANSLATION_REPAIR_RUNS_DIR';

/**
 Points the runs directory variable at a path until the handle's scope ends.

 The directory itself is the caller's own `scratchDir`, bound first so it is
 removed after the variable is restored.

 @param path - directory the variable names meanwhile

 @returns Disposable handle restoring the variable as it stood

 @example
 ```ts
 await using runs = await scratchDir({ prefix: 'whiskers-relabel-artifact-', },);
 using pointed = runsDirPointedAt({ path: runs.path, },);
 ```
 */
export function runsDirPointedAt({ path, }: { readonly path: string; },): Disposable {
  /**
   Runs directory standing before this case ran.
   */
  const before = process.env[RUNS_DIR_VARIABLE];
  process.env[RUNS_DIR_VARIABLE] = path;
  return {
    [Symbol.dispose]: function restore(): void {
      if (before === undefined) {
        Reflect.deleteProperty(
          process.env,
          RUNS_DIR_VARIABLE,
        );
      } else
        process.env[RUNS_DIR_VARIABLE] = before;
    },
  };
}

//endregion Runs directory pointed
