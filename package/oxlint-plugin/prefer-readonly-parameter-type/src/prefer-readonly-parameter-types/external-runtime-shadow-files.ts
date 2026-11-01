/**
 Runtime siblings selected for declaration-shadowed package imports.
 
 @module
 */

import { existsSync, } from 'node:fs';

import type { Project, } from 'typescript/sync';

/**
 Declaration suffix and possible shipped runtime siblings.
 */
type RuntimeShadowRule = {
  readonly declarationSuffix: string;
  readonly runtimeSuffixes: readonly string[];
};

/**
 Runtime sibling rules in module-resolution preference order.
 */
const RUNTIME_SHADOW_RULES: readonly RuntimeShadowRule[] = [
  {
    declarationSuffix: '.d.mts',
    runtimeSuffixes: [
      '.mjs',
      '.mts',
    ],
  },
  {
    declarationSuffix: '.d.cts',
    runtimeSuffixes: [
      '.cjs',
      '.cts',
    ],
  },
  {
    declarationSuffix: '.d.ts',
    runtimeSuffixes: [
      '.js',
      '.mjs',
      '.cjs',
      '.ts',
      '.mts',
      '.cts',
      '.jsx',
      '.tsx',
    ],
  },
];

/**
 Finds runtime siblings for declarations selected while resolving exact package entry imports.
 
 TypeScript may prefer an adjacent declaration over runtime JavaScript for an
 explicit relative runtime import. Rooting the existing runtime sibling makes
 its implementation available without enumerating unrelated package source.
 
 @param project - External project whose reached imports selected declarations.
 
 @param packageRoot - Exact package root allowed for implementation analysis.
 
 @returns sorted existing runtime siblings within package root.
 
 @example
 ```ts
 externalRuntimeShadowFiles({ project, packageRoot: '/package' });
 ```
 */
export function externalRuntimeShadowFiles({
  project,
  packageRoot,
}: {
  readonly project: Project;
  readonly packageRoot: string;
}): readonly string[] {
  return project.program
    .getSourceFileNames()
    .filter(function packageDeclaration(fileName,): boolean {
      return fileName.startsWith(packageRoot,);
    },)
    .flatMap(function runtimeSiblings(fileName,): readonly string[] {
      /**
       Declaration suffix rule matching reached file.
       */
      const rule = RUNTIME_SHADOW_RULES.find(function matchesSuffix(candidate,): boolean {
        return fileName.endsWith(candidate.declarationSuffix,);
      },);
      if (rule === undefined)
        return [];
      /**
       Path stem shared by declaration and possible runtime siblings.
       */
      const stem = fileName.slice(
        0,
        fileName.length
          - rule.declarationSuffix
            .length,
      );
      return rule.runtimeSuffixes
        .map(function runtimePath(suffix,): string {
          return `${stem}${suffix}`;
        },)
        .filter(function runtimeFileExists(runtimePath,): boolean {
          // oxlint-disable-next-line no-restricted-syntax/no-sync -- External semantic setup validates reached shipped runtime siblings synchronously.
          return existsSync(runtimePath,);
        },)
        .slice(
        0,
        1,
        );
    },)
    .toSorted();
}
