/**
 Main entry point for the test matrix runner.
 
 Generates the cartesian product of `files x os x user x runtime`,
 filters out excluded combinations, and executes each combination
 via the appropriate backend (container, host, or vm).
 Results are reported through `describe`/`it` from `\@monochromatic-dev/test-harness-core`.
 */

import {
  logger as defaultLogger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { resolve, } from 'node:path';

import {
  findRootCached,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';

import {
  parseOs,
  runContainer,
} from './container.ts';
import { resolveBaseImage, } from './cachyos-image.ts';
import { discoverTestFiles, } from './discover.ts';
import { runHost, } from './host.ts';
import type {
  Combination,
  ExcludeEntry,
  ExcludeMatcher,
  MatrixOptions,
  Runtime,
  UserContext,
} from './types.ts';

//region Cartesian product

/**
 Generates the cartesian product of all axes, producing one {@link Combination}
 per unique `file x os x user x runtime` tuple.
 
 @param files - Absolute paths to test files
 
 @param os - OS specifications with protocol prefixes
 
 @param user - User contexts
 
 @param runtime - JS runtimes
 
 @returns all combinations before exclusion
 */
function generateCombinations({
  files,
  os,
  user,
  runtime,
}: {
  readonly files: readonly string[];
  readonly os: readonly string[];
  readonly user: readonly UserContext[];
  readonly runtime: readonly Runtime[];
},): readonly Combination[] {
  /**
   Accumulator; four nested loops would be unwieldy as a single `.flatMap` chain.
   */
  const combinations: Combination[] = [];

  for (const file of files) {
    for (const osEntry of os) {
      for (const userEntry of user) {
        for (const runtimeEntry of runtime) {
          combinations.push({
            file,
            os: osEntry,
            user: userEntry,
            runtime: runtimeEntry,
          },);
        }
      }
    }
  }

  return combinations;
}

/**
 Tests whether a single exclude matcher matches a value.
 String matchers use exact equality; function matchers are called as predicates.
 
 @param matcher - String or predicate function
 
 @param value - Combination field value to test
 
 @returns whether the matcher matches the value
 */
function testMatcher<T,>({
  matcher,
  value,
}: {
  readonly matcher: ExcludeMatcher<T>;
  readonly value: T;
},): boolean {
  if ((typeof matcher) === 'function') {
    // oxlint-disable-next-line no-unsafe-type-assertion -- typeof guard guarantees matcher is a function; T is always a string literal union, never a function type
    return (matcher as (v: T,) => boolean)(value,);
  }
  return matcher === value;
}

/**
 Checks whether a combination matches an exclude entry.
 All specified fields in the exclude entry must match.
 Each field can be an exact value or a predicate function.
 
 @param combination - Combination to check
 
 @param exclude - Exclude entry with partial match fields or predicates
 
 @returns whether the combination should be excluded
 */
function matchesExclude({
  combination,
  exclude,
}: {
  readonly combination: Combination;
  readonly exclude: ExcludeEntry;
},): boolean {
  if ((exclude.os
    !== undefined) && (!testMatcher({
      matcher: exclude.os,
      value: combination.os,
    },))) {
    return false;
  }
  if ((exclude.user
    !== undefined) && (!testMatcher({
      matcher: exclude.user,
      value: combination.user,
    },))) {
    return false;
  }
  if ((exclude.runtime
    !== undefined) && (!testMatcher({
      matcher: exclude.runtime,
      value: combination.runtime,
    },))) {
    return false;
  }
  if ((exclude.file
    !== undefined) && (!testMatcher({
      matcher: exclude.file,
      value: combination.file,
    },))) {
    return false;
  }
  return true;
}

/**
 Filters out combinations that match any exclude entry.
 
 @param combinations - All combinations from the cartesian product
 
 @param excludes - Exclude entries to match against
 
 @returns combinations that do not match any exclude entry
 */
function applyExcludes({
  combinations,
  excludes,
}: {
  readonly combinations: readonly Combination[];
  readonly excludes: readonly ExcludeEntry[];
},): readonly Combination[] {
  if (excludes.length
    === 0)
    return combinations;

  return combinations.filter(function isNotExcluded(combination,) {
    return !excludes.some(function isExcluded(exclude,) {
      return matchesExclude({
        combination,
        exclude,
      },);
    },);
  },);
}

//endregion Cartesian product

//region Label formatting

/**
 Formats a human-readable label for a combination.
 Used in test names and log output.
 
 @param combination - Combination to format
 
 @returns label like `'container:ubuntu / root / bun'`
 */
function formatLabel(combination: Combination,): string {
  return `${combination.os} / ${combination.user} / ${combination.runtime}`;
}

/**
 Extracts a short filename from an absolute path for display.
 
 @param filePath - Absolute path
 
 @returns filename without directory
 */
function shortFileName(filePath: string,): string {
  /**
   Captured to reuse in both the absent-separator guard and the slice offset.
   */
  const lastSlash = filePath.lastIndexOf('/',);
  if (lastSlash === (-1))
    return filePath;
  return filePath.slice(lastSlash + 1,);
}

//endregion Label formatting

//region Execution dispatch

/**
 Executes a single combination using the appropriate backend.
 Routes to {@link runContainer} or {@link runHost} based on the OS protocol.
 
 @param combination - Fully resolved combination to execute
 
 @param monorepoRoot - Absolute path to the monorepo root on the host
 
 @param cachyosImage - CachyOS image resolved once for the matrix; absent when no entry uses `cachyos`
 
 @returns stdout from the execution
 
 @throws Error when the combination fails or uses an unimplemented protocol
 */
function executeCombination({
  combination,
  monorepoRoot,
  cachyosImage,
}: {
  readonly combination: Combination;
  readonly monorepoRoot: string;
  readonly cachyosImage?: string;
},): Promise<string> {
  /**
   Protocol drives the backend choice; parsed once and inspected by each branch.
   */
  const parsed = parseOs(combination.os,);

  if (parsed.protocol
    === 'host')
    return runHost({ combination, },);

  if (parsed.protocol
    === 'container') {
    return runContainer({
      combination,
      monorepoRoot,
      ...(cachyosImage === undefined ? {} : { cachyosImage, }),
    },);
  }

  throw new Error(
    `vm: protocol not yet implemented (in "${combination.os}"). Use container: or host: protocol instead.`,
  );
}

//endregion Execution dispatch

/**
 Default maximum number of concurrent combination executions.
 */
const DEFAULT_CONCURRENCY = 4;

/**
 Runs test files across a cartesian product of environments.
 
 Generates all combinations of `files x os x user x runtime`,
 filters out excluded entries, and executes each combination via
 {@link executeCombination}. Results are reported through
 {@link describe}/{@link it} from `\@monochromatic-dev/test-harness-core`.
 
 @param files - Files to execute inside each environment (defaults to auto-discovery)
 
 @param os - OS specifications with protocol prefix
 
 @param user - User contexts (defaults to `['root']`)
 
 @param runtime - JS runtimes (defaults to `['bun']`)
 
 @param exclude - Combinations to exclude from the cartesian product
 
 @param concurrency - Maximum concurrent combination executions (defaults to 4)
 
 @throws Error (via describe/it) when any combination fails
 
 @example
 ```ts
 await matrix({
   os: ['container:cachyos', 'container:ubuntu'],
   user: ['root', 'user'],
 });
 ```
 */
export async function matrix({
  files: filesOption,
  os,
  user = ['root',],
  runtime = ['bun',],
  exclude = [],
  concurrency = DEFAULT_CONCURRENCY,
}: MatrixOptions,): Promise<void> {
  /**
   Tagged logger so each line in this function carries the `matrix` scope.
   */
  const l: Logger = tagged({
    tag: matrix.name,
    l: defaultLogger,
  },);

  //region Validate OS specifications
  for (const osSpec of os) {
    /**
     Pre-validates the protocol up-front; failing here surfaces config errors before any work.
     */
    const parsed = parseOs(osSpec,);
    if (parsed.protocol
      === 'vm') {
      throw new Error(
        `vm: protocol not yet implemented (in "${osSpec}"). Use container: or host: protocol instead.`,
      );
    }
  }
  //endregion Validate OS specifications

  //region Resolve the CachyOS image
  /**
   Whether any cell runs the `cachyos` distro.
   */
  const usesCachyos = os.includes('container:cachyos',);
  /**
   Probed once so every `container:cachyos` cell runs the same image; empty when no cell needs it.
   A host that is not x86-64 throws here, before any work, because a `cachyos` row is a pacman test.
   */
  const cachyosOption: { readonly cachyosImage?: string; } = usesCachyos
    ? {
      cachyosImage: (await resolveBaseImage({
        imageName: 'container:cachyos',
        alpineVariant: false,
        l,
      },)).image,
    }
    : {};
  //endregion Resolve the CachyOS image

  //region Discover monorepo root
  /**
   Resolved once and threaded into every container invocation as the bind-mount source.
   */
  const monorepoRoot = await findRootCached({ marker: MISE_MONOREPO, },);
  l.debug(`monorepo root: ${monorepoRoot}`,);
  //endregion Discover monorepo root

  //region Resolve files
  /**
   Either the consumer's explicit list (resolved against cwd) or auto-discovered tests.
   */
  const files = filesOption !== undefined
    ? filesOption.map(function resolveFile(filePath,) {
      return resolve(
        process.cwd(),
        filePath,
      );
    },)
    : await discoverTestFiles(process.cwd(),);

  l.info(`${String(files.length,)} test file(s) discovered`,);
  for (const file of files)
    l.debug(`  ${file}`,);
  //endregion Resolve files

  //region Generate and filter combinations
  /**
   Raw cartesian product before exclusion; retained so the log line below can report the delta.
   */
  const allCombinations = generateCombinations({
    files,
    os,
    user,
    runtime,
  },);
  /**
   Survivors after applying the user-supplied exclude entries; what actually executes.
   */
  const combinations = applyExcludes({
    combinations: allCombinations,
    excludes: exclude,
  },);

  l.info(
    `${String(combinations.length,)} combination(s) `
      + `(${String(allCombinations.length,)} total, ${
        String(allCombinations.length
          - combinations
          .length,)
      } excluded)`,
  );
  //endregion Generate and filter combinations

  //region Execute via describe/it

  l.info(`concurrency limit: ${String(concurrency,)}`,);

  /**
   Group combinations by file so each file gets its own describe block.
   */
  const fileGroups = new Map<string, Combination[]>();
  for (const combination of combinations) {
    /**
     Existing per-file bucket, if any; absent first iteration per file.
     */
    const existing = fileGroups.get(combination.file,);
    if (existing !== undefined)
      existing.push(combination,);
    else {
      fileGroups.set(
        combination.file,
        [combination,],
      );
    }
  }

  /**
   One describe-tree per file, each containing one `it` per combination for that file.
   */
  const children = [...fileGroups.entries(),].map(
    function createFileDescribe([filePath, fileCombinations,],) {
      /**
       Trimmed-path display label; full path is too long for nested describe output.
       */
      const fileName = shortFileName(filePath,);

      return describe({
        name: fileName,
        l,
        concurrency,
        children: fileCombinations.map(
          function createCombinationIt(combination,) {
            /**
             Human-readable axis tuple used as the `it` name.
             */
            const label = formatLabel(combination,);

            return it({
              name: label,
              l,
              fn: async function runCombination() {
                /**
                 Captured so the empty-output check below does not log a blank line.
                 */
                const output = await executeCombination({
                  combination,
                  monorepoRoot,
                  ...cachyosOption,
                },);
                if (output !== '')
                  l.info(output,);
              },
            },);
          },
        ),
      },);
    },
  );

  await describe({
    name: '',
    l,
    children,
  },);

  //endregion Execute via describe/it
}
