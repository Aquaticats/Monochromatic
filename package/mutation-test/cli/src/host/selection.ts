/**
 Source and test selection for one target package.
 
 @example
 ```ts
 const sources = await selectSources({ packageRoot });
 ```
 */

import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Module logger for host-side selection.
 */
const l = tagged({ tag: 'mutation-test', },);

/**
 Unit test file suffix.
 */
const UNIT_TEST_SUFFIX = '.unit.test.ts';

/**
 File name suffixes excluded from mutation with their reasons.
 */
const EXCLUDED_SUFFIXES: Readonly<Record<string, string>> = {
  '.test.ts': 'test file',
  '.bench.ts': 'benchmark file',
  '.d.ts': 'declaration file',
};

/**
 File selected or skipped during source scanning.
 */
export type SourceExclusion = {
  readonly file: string;
  readonly reason: string;
};

/**
 Source scan output: mutable targets plus documented exclusions.
 */
export type SourceSelection = {
  readonly files: readonly string[];
  readonly excluded: readonly SourceExclusion[];
};

/**
 Recursively lists package-relative .ts files under one directory.
 
 @param options - Package root and relative directory to scan.
 
 @returns Package-relative file paths.
 
 @example
 ```ts
 await listTsFiles({ packageRoot, dir: 'src' });
 ```
 */
async function listTsFiles(options: {
  readonly packageRoot: string;
  readonly dir: string;
},): Promise<readonly string[]> {
  /**
   Directory entries under the scanned directory.
   */
  const entries = await readdir(
    join(
      options.packageRoot,
      options.dir,
    ),
    { withFileTypes: true, },
  );
  /**
   Files collected from this level and subdirectories.
   */
  const nested = await Promise.all(entries.map(
    async function collect(entry: ForeignBorrowed<Dirent>,): Promise<readonly string[]> {
      /**
       Package-relative path of this entry.
       */
      const relative = `${options.dir}/${entry.name}`;

      if (entry.isDirectory())
        return await listTsFiles({
          packageRoot: options.packageRoot,
          dir: relative,
        },);

      return entry.name
        .endsWith('.ts',)
        ? [relative,]
        : [];
    },
  ),);
  return nested.flat();
}

/**
 Selects production source files for mutation.
 
 @param options - Package root directory.
 
 @returns Selected files and documented exclusions.
 
 @example
 ```ts
 const { files, excluded } = await selectSources({ packageRoot });
 ```
 */
export async function selectSources(options: {
  readonly packageRoot: string;
},): Promise<SourceSelection> {
  /**
   Every TypeScript file under src.
   */
  const all = await listTsFiles({
    packageRoot: options.packageRoot,
    dir: 'src',
  },);
  /**
   Files kept for mutation.
   */
  const files: string[] = [];
  /**
   Files skipped with reasons.
   */
  const excluded: SourceExclusion[] = [];

  for (const file of all) {
    /**
     Exclusion reason for this file, when any suffix matches.
     */
    const reason = Object.entries(EXCLUDED_SUFFIXES,)
      .find(function matches(entry,): boolean {
        return file.endsWith(entry[0],);
      },)
      ?.[1];

    if (reason === undefined)
      files.push(file,);
    else
      excluded.push({
        file,
        reason,
      },);
  }

  tagged({
    tag: selectSources.name,
    l,
  },)
    .info(`selected ${String(files.length,)} sources, excluded ${String(excluded.length,)}`,);
  return {
    files: files.toSorted(),
    excluded,
  };
}

/**
 Returns whether a test stem relates to a source stem.
 
 `foo` matches `foo.unit.test.ts` and dot-sidecars like
 `foo.regression.unit.test.ts`, never hyphen siblings like
 `foo-bar.unit.test.ts`, whose stem is a different word.
 
 @param options - Source stem and candidate test file name.
 
 @returns Whether the test targets the source.
 
 @example
 ```ts
 stemsRelated({ sourceStem: 'foo', testName: 'foo.regression.unit.test.ts' });
 // true
 ```
 */
export function stemsRelated(options: {
  readonly sourceStem: string;
  readonly testName: string;
},): boolean {
  if (!options.testName
    .endsWith(UNIT_TEST_SUFFIX,))
    return false;

  /**
   Test name with the unit suffix removed.
   */
  const bareStem = options.testName
    .slice(
    0,
    -UNIT_TEST_SUFFIX.length,
    );
  return (bareStem === options.sourceStem)
    || bareStem.startsWith(`${options.sourceStem}.`,);
}

/**
 Selects unit tests for one source file.
 
 @param options - Package root, source file, and full-suite toggle.
 
 @returns Package-relative test files, sorted.
 
 @example
 ```ts
 await selectTests({ packageRoot, sourceFile: 'src/trim.ts', fullSuite: false });
 ```
 */
export async function selectTests(options: {
  readonly packageRoot: string;
  readonly sourceFile: string;
  readonly fullSuite: boolean;
},): Promise<readonly string[]> {
  /**
   Every TypeScript file under src.
   */
  const all = await listTsFiles({
    packageRoot: options.packageRoot,
    dir: 'src',
  },);
  /**
   All unit tests in the package.
   */
  const unitTests = all.filter(function isUnitTest(file,): boolean {
    return file.endsWith(UNIT_TEST_SUFFIX,);
  },);

  if (options.fullSuite)
    return unitTests.toSorted();

  /**
   Source file name without directory or extension.
   */
  const sourceStem = (options.sourceFile
    .split('/',)
    .at(-1,)
    ?? options.sourceFile)
    .replace(
    '.ts',
    '',
    );
  /**
   Package-level integration test included for every source file when
   present, mirroring the old tool's default selection.
   */
  const integrationTests = unitTests.filter(function isIntegration(file,): boolean {
    return file === 'src/integration.unit.test.ts';
  },);
  return [
    ...new Set([
      ...unitTests
        .filter(function related(file,): boolean {
          /**
           Bare test file name without directories.
           */
          const testName = file
            .split('/',)
            .at(-1,)
            ?? file;
          return stemsRelated({
            sourceStem,
            testName,
          },);
        },),
      ...integrationTests,
    ],),
  ]
    .toSorted();
}
