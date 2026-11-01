/**
 The texts the family's reference guards read, shared so each guard reads the
 same set: the TypeScript under every family package's `src`, the family's own docs and README,
 each family package's README, docs and task file, and
 the living repository-level docs.

 MOVED OUT OF `task-list-numbers.unit.test.ts` when a second guard needed the
 same set (ledger D33), since two copies of one reader drift apart.

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  join,
  relative,
} from 'node:path';

import {
  FAMILY_PACKAGES,
  readLivingRepositoryDocs,
  readPackageDocPaths,
  REPOSITORY_ROOT,
} from './living-docs.test-fixture.ts';
import {
  familyPackages,
  readPackageSource,
} from './family-source.test-fixture.ts';

/**
 One file a guard reads, named from the family directory (a source file by its package directory and its path under
 `src`, as `readPackageSource` names it), or from the repository
 root for a living repository-level doc.

 @example
 ```ts
 const file: PackageText = { path: 'doc/cat.md', text: 'Naps.', };
 ```
 */
export type PackageText = {
  /**
   Path from the family directory, or package directory and path under `src` for source, which names locations.
   */
  readonly path: string;

  /**
   File text.
   */
  readonly text: string;
};

/**
 Name of a package-level doc as a guard names it: from the family directory, so the family's own docs go by the
 bare `doc/<name>.md` or `README.md` and a family package's by its directory.

 @param path - path from the repository root

 @returns The path from the family directory, led by the package directory for a family package's file

 @example
 ```ts
 const named = packageRootedName({ path: 'package/translation-repair/archive/README.md', },); // 'archive/README.md'
 ```
 */
function packageRootedName({ path, }: { readonly path: string; },): string {
  return relative(
    FAMILY_PACKAGES,
    path,
  );
}

/**
 Every file the guard reads: the TypeScript under every family package's `src`, the family's own docs and README,
 each family package's README and docs, and each family package's
 task file.

 @returns Files with their text

 @example
 ```ts
 const files = await readPackageTexts();
 ```
 */
export async function readPackageTexts(): Promise<readonly PackageText[]> {
  /**
   Docs and READMEs, from the repository root.
   */
  const docs = await readPackageDocPaths();
  /**
   Task files, one per family package, from the repository root.
   */
  const tasks = (await familyPackages()).map(function taskFileOf(name,): string {
    return join(
      FAMILY_PACKAGES,
      name,
      'mise.toml',
    );
  },);
  /**
   Non-source files, read from the repository root and named from the family directory.
   */
  const others = await Promise.all([
    ...docs,
    ...tasks,
  ].map(async function read(
    path,
  ): Promise<PackageText> {
    return {
      path: packageRootedName({ path, },),
      text: await readFile(
        join(
          REPOSITORY_ROOT,
          path,
        ),
        'utf8',
      ),
    };
  },),);
  /**
   Source and test files, named by package directory and path under `src`.
   */
  const source = (await readPackageSource()).map(function fromRoot(file,): PackageText {
    return {
      path: file.path,
      text: file.text,
    };
  },);
  return [
    ...source,
    ...others,
  ];
}

/**
 Every living repository-level doc: the translation-repair decision records,
 the canonical handover and its snapshot, the planning docs it links as
 current, and the translation-repair runbooks and troubleshooting docs.

 @returns Files with their text, named from the repository root

 @example
 ```ts
 const files = await readRepositoryTexts();
 ```
 */
export async function readRepositoryTexts(): Promise<readonly PackageText[]> {
  /**
   Living repository-level docs.
   */
  const {
    decisionRecords,
    handover,
    currentPlanning,
    operations,
  } = await readLivingRepositoryDocs();
  return Promise.all([
    ...decisionRecords,
    ...handover,
    ...currentPlanning,
    ...operations,
  ].map(async function read(
    path,
  ): Promise<PackageText> {
    return {
      path,
      text: await readFile(
        join(
          REPOSITORY_ROOT,
          path,
        ),
        'utf8',
      ),
    };
  },),);
}
