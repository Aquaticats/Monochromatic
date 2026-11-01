/**
 Package-level enumeration: mutants and tests per source file.
 
 @example
 ```ts
 const { groups, ignored } = await enumeratePackage(options);
 ```
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  enumerateMutants,
  type IgnoredMutant,
} from '../engine/enumerate.ts';
import {
  selectSources,
  selectTests,
} from './selection.ts';
import type { MutantGroup, } from './shards.ts';

/**
 Inputs shared with the orchestrator for one run.
 */
export type EnumerationInputs = {
  repoRoot: string;
  packagePath: string;
  sourceFiles: readonly string[];
  fullSuite: boolean;
};

/**
 Enumerates mutants for every selected source file.
 
 @param options - Repository root, package path, and selection toggles.
 
 @returns Per-file groups plus suppression-ignored mutants.
 
 @example
 ```ts
 const { groups, ignored } = await enumeratePackage(options);
 ```
 */
export async function enumeratePackage(options: Readonly<EnumerationInputs>,): Promise<{
  readonly groups: readonly MutantGroup[];
  readonly ignored: readonly IgnoredMutant[];
}> {
  /**
   Absolute package root on the host.
   */
  const packageRoot = join(
    options.repoRoot,
    options.packagePath,
  );
  /**
   Source files under mutation.
   */
  const sources = options.sourceFiles
    .length
    > 0
    ? options.sourceFiles
    : (await selectSources({ packageRoot, },)).files;
  /**
   Per-file enumeration joined with test selection.
   */
  const groups = await Promise.all(sources.map(
    async function toGroup(file,): Promise<{
      readonly group: MutantGroup;
      readonly ignored: readonly IgnoredMutant[];
    }> {
      /**
       Enumeration for this source file.
       */
      const enumeration = enumerateMutants({
        file,
        source: await readFile(
          join(
            packageRoot,
            file,
          ),
          'utf8',
        ),
      },);
      return {
        group: {
          file,
          mutants: enumeration.mutants,
          tests: await selectTests({
            packageRoot,
            sourceFile: file,
            fullSuite: options.fullSuite,
          },),
        },
        ignored: enumeration.ignored,
      };
    },
  ),);

  return {
    groups: groups
      .map(function toGroupOnly(entry,): MutantGroup {
        return entry.group;
      },)
      .filter(function hasWork(group,): boolean {
        return group.mutants
          .length
          > 0;
      },),
    ignored: groups.flatMap(function toIgnored(entry,): readonly IgnoredMutant[] {
      return entry.ignored;
    },),
  };
}
