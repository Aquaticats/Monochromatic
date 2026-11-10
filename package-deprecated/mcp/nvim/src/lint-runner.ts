/**
 * Oxlint lint runner.
 *
 * Orchestrates oxlint invocations across files grouped by their
 * nearest `tsconfig.json` ancestor for correct `--type-aware` resolution.
 *
 * @module
 */

import { dirname, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts/tagged.ts';

import type { Diagnostic, } from './nvim-client.ts';
import { rootLogger, } from './log.ts';
import { findAncestorWithFile, } from './oxlint-parse.ts';
import { spawnOxlint, } from './oxlint-spawn.ts';

//region Logging

/**
 Logger for this module, re-tagged with the enclosing function name at each
 site so a log line names the call that produced it.
 */
const lintRunnerLogger = tagged({
  l: rootLogger,
  tag: 'lint-runner',
},);

//endregion Logging

//region Types: lint result shape

/**
 * Result from a lint run, including diagnostics and any caveat notes.
 *
 * @example
 * ```ts
 * const result: LintResult = {
 *
 *   diagnostics: new Map(),
 *
 *   notes: ["oxlint ran without --type-aware; some rules may not report."],
 * };
 * ```
 */
export type LintResult = {
  readonly diagnostics: Map<string, Diagnostic[]>;
  readonly notes: readonly string[];
};

//endregion Types

//region Runner: orchestrate oxlint across file groups

/**
 * Runs oxlint on the specified files and returns parsed diagnostics.
 * Groups files by their nearest `tsconfig.json` ancestor for correct `--type-aware` resolution.
 * Falls back to non-type-aware mode with a caveat note when no `tsconfig.json` is found.
 * Returns empty results gracefully when oxlint is unavailable.
 *
 * @param files - Absolute paths to lint.
 *
 * @returns Diagnostics grouped by absolute file path, plus any caveat notes.
 *
 * @example
 * ```ts
 * const result = await runOxlint({ files: ["/home/user/project/src/index.ts"] });
 * ```
 */
export async function runOxlint(
  { files, }: { files: readonly string[]; },
): Promise<LintResult> {
  if (files.length
    === 0) {
    return {
      diagnostics: new Map(),
      notes: [],
    };
  }

  /**
   * First file's directory as starting point for config search.
   */
  const [firstFile,] = files;
  if (firstFile === undefined) {
    return {
      diagnostics: new Map(),
      notes: [],
    };
  }
  /**
   * Directory containing the nearest `oxlint.config.ts`; used as cwd for orphan-file fallback so oxlint loads its config.
   */
  const configDir = findAncestorWithFile({
    startDir: dirname(firstFile,),
    filename: 'oxlint.config.ts',
  },);
  if (configDir === null) {
    tagged({
      tag: runOxlint.name,
      l: lintRunnerLogger,
    },)
      .error('Could not find oxlint.config.ts in any ancestor directory',);
    return {
      diagnostics: new Map(),
      notes: [],
    };
  }

  //region Group files by tsconfig ancestor: each group runs in its own cwd
  /**
   * Files keyed by their nearest tsconfig directory; one oxlint invocation runs per key with `--type-aware` enabled.
   */
  const groupsByPackageRoot = new Map<string, string[]>();
  /**
   * Files with no tsconfig ancestor; linted in a single fallback invocation without `--type-aware`.
   */
  const filesWithoutTsconfig: string[] = [];

  for (const filePath of files) {
    /**
     * Nearest tsconfig directory for this file; `null` triggers the fallback bucket.
     */
    const packageRoot = findAncestorWithFile({
      startDir: dirname(filePath,),
      filename: 'tsconfig.json',
    },);
    if (packageRoot !== null) {
      /**
       * Files already grouped under this package root; extended in place when present.
       */
      const existing = groupsByPackageRoot.get(packageRoot,);
      if (existing !== undefined)
        existing.push(filePath,);
      else {
        groupsByPackageRoot.set(
          packageRoot,
          [filePath,],
        );
      }
    }
    else {
      filesWithoutTsconfig.push(filePath,);
    }
  }
  //endregion Group files by tsconfig ancestor

  /**
   * Path-keyed accumulator that gathers diagnostics from every oxlint invocation before the final return.
   */
  const merged = new Map<string, Diagnostic[]>();
  /**
   * Mutable caveat list; may gain a fallback-mode warning when some files lack a tsconfig ancestor.
   */
  const notes: string[] = [];

  //region Run per-package-root invocations with --type-aware
  /**
   * Concurrent oxlint runs, one per package root; each uses its own cwd so type-aware rules resolve correctly.
   */
  const packageRuns = [...groupsByPackageRoot.entries(),].map(
    function runPackageOxlint([packageRoot, packageFiles,],) {
      return spawnOxlint({
        cwd: packageRoot,
        files: packageFiles,
        typeAware: true,
      },);
    },
  );
  //endregion Run per-package-root invocations with --type-aware

  //region Run fallback invocation without --type-aware for orphaned files
  /**
   * Promise placeholder for the optional fallback oxlint run; resolves to `null` when no orphaned files exist.
   */
  const fallbackRun: Promise<Map<string, Diagnostic[]> | null> =
    (function getFallbackRun() {
      if (filesWithoutTsconfig.length
        === 0)
        return Promise.resolve(null,);
      notes.push(
        'Some files have no tsconfig.json in any ancestor directory; '
          + 'oxlint ran without --type-aware for those files and some type-aware rules may not report.',
      );
      return spawnOxlint({
        cwd: configDir,
        files: filesWithoutTsconfig,
        typeAware: false,
      },);
    })();
  //endregion Run fallback invocation without --type-aware for orphaned files

  /**
   * Tuple of (per-package-root results, optional fallback result); awaited together so both lint groups overlap.
   */
  const [packageResults, fallbackResult,] = await Promise.all([
    Promise.all(packageRuns,),
    fallbackRun,
  ],);

  for (const resultMap of packageResults) {
    mergeInto({
      target: merged,
      source: resultMap,
    },);
  }
  if (fallbackResult !== null) {
    mergeInto({
      target: merged,
      source: fallbackResult,
    },);
  }

  return {
    diagnostics: merged,
    notes,
  };
}

//endregion Runner

//region Utilities: map merging

/**
 * Merges diagnostics from a source map into a target map, mutating target in place.
 *
 * @param target - Map to merge into.
 *
 * @param source - Map to merge from.
 *
 * @example
 * ```ts
 * const target = new Map<string, Diagnostic[]>([['/a.ts', [diagA]]]);
 * const source = new Map<string, Diagnostic[]>([['/a.ts', [diagB]], ['/b.ts', [diagC]]]);
 * mergeInto({ target, source });
 * // target is now Map { '/a.ts' => [diagA, diagB], '/b.ts' => [diagC] }
 * ```
 */
function mergeInto(
  {
    target,
    source,
  }: {
    target: Map<string, Diagnostic[]>;
    source: Map<string, Diagnostic[]>;
  },
): void {
  for (const [filePath, diagnostics,] of source) {
    /**
     * Diagnostics already collected under this path in the target map; extended in place when present.
     */
    const existing = target.get(filePath,);
    if (existing !== undefined)
      existing.push(...diagnostics,);
    else {
      target.set(
        filePath,
        [...diagnostics,],
      );
    }
  }
}

//endregion Utilities
