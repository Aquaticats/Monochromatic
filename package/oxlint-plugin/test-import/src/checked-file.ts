/**
 Which files the rule inspects, and which import targets it forgives.
 
 Test and benchmark files are the subject of the convention. Test-only helper
 modules are inspected too, and for the same reason: a test may import a
 permitted helper, and if that helper were unchecked it could re-export
 straight from source, bypassing the rule without any change to the test's own
 import.
 
 @module
 */

import { matchesAnyGlob, } from './path-glob.ts';

/**
 Filename suffix marking a unit, integration, or end-to-end test.
 */
const TEST_SUFFIX = '.test.ts';

/**
 Filename suffix marking a benchmark.
 */
const BENCH_SUFFIX = '.bench.ts';

/**
 Globs naming test-only modules that are not package behavior.
 
 Every entry must identify test-only code by name alone, because a match has
 two consequences: the module is exempt as an import target, and the module is
 itself inspected. A glob that also catches package behavior therefore fails
 twice over, exempting real behavior from tests while reporting ordinary
 source for importing its own siblings.
 
 The three literal `test-` names are listed individually rather than as a
 `test-*` prefix glob, which would also match real package behavior such as
 `package/mutation-test/cli/src/container/test-run.ts` and silently exempt it.
 
 `*-helpers.ts` and `*-harness.ts` are deliberately absent. Measured across
 this repository, all 22 files carrying those suffixes are imported by
 package behavior (`cli-helpers.ts` by `cli.ts`, `render-helpers.ts` by four
 i18n modules, `tasks-helpers.ts` by three database modules) and none are
 test-only. The suffix describes what a module does, not who may load it.
 
 @internal
 */
export const DEFAULT_FIXTURE_PATTERNS: readonly string[] = [
  '**/fixture.*',
  '**/*-fixture*.ts',
  '**/test-support.ts',
  '**/test-setup.ts',
  '**/test-fixtures.ts',
];

/**
 Tests whether a path names a test or benchmark file.
 
 @param path - normalized absolute path
 
 @returns true for `.test.ts` and `.bench.ts` files
 
 @example
 ```ts
 isTestFile({ path: '/repo/package/module/x/src/x.unit.test.ts' });
 ```
 
 @internal
 */
export function isTestFile({ path, }: {
  /**
   Normalized absolute path to classify.
   */
  readonly path: string;
},): boolean {
  return path.endsWith(TEST_SUFFIX,) || path.endsWith(BENCH_SUFFIX,);
}

/**
 Tests whether a path names a test-only fixture or helper module.
 
 Matching runs against a resolved path, never against raw specifier text, so
 two spellings of the same target cannot classify differently.
 
 @param patterns - configured fixture globs
 
 @param path - normalized absolute path
 
 @returns true when any configured glob covers the path
 
 @example
 ```ts
 isFixtureModule({ patterns: DEFAULT_FIXTURE_PATTERNS, path: '/repo/src/tree-helpers.ts' });
 ```
 
 @internal
 */
export function isFixtureModule({
  patterns,
  path,
}: {
  /**
   Configured fixture globs.
   */
  readonly patterns: readonly string[];
  /**
   Normalized absolute path to classify.
   */
  readonly path: string;
},): boolean {
  return matchesAnyGlob({
    patterns,
    path,
  },);
}

/**
 Tests whether the rule inspects a file at all.
 
 @param patterns - configured fixture globs
 
 @param path - normalized absolute path of the file under lint
 
 @returns true for test and benchmark files and for fixture or helper modules
 
 @example
 ```ts
 isCheckedFile({ patterns: DEFAULT_FIXTURE_PATTERNS, path: '/repo/src/x.unit.test.ts' });
 ```
 
 @internal
 */
export function isCheckedFile({
  patterns,
  path,
}: {
  /**
   Configured fixture globs.
   */
  readonly patterns: readonly string[];
  /**
   Normalized absolute path of the file under lint.
   */
  readonly path: string;
},): boolean {
  if (isTestFile({ path, },))
    return true;
  return isFixtureModule({
    patterns,
    path,
  },);
}
