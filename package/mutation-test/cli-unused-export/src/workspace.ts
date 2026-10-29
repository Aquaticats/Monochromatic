/**
 Workspace package discovery for unused-export analysis.
 
 @example
 ```ts
 const packages = await discoverWorkspacePackages({ workspaceRoot: '/repo' });
 ```
 */

import {
  glob,
  readFile,
} from 'node:fs/promises';
import { posix, } from 'node:path';
import { parse as parseYaml, } from 'yaml';
import { tagged, } from '@monochromatic-dev/module-logger/ts';

/**
 Module logger for workspace discovery.
 */
const l = tagged({ tag: 'unused-export', },);

/**
 One discovered workspace package with its analyzable sources.
 */
export type WorkspacePackage = {
  /**
   Workspace-relative package directory, forward slashes.
   */
  readonly dir: string;
  /**
   Published package name from the manifest.
   */
  readonly name: string;
  /**
   Workspace-relative TypeScript source paths under `src/`.
   */
  readonly sourceFiles: readonly string[];
};

/**
 Reads the pnpm workspace package globs.
 
 @param workspaceRoot - Absolute workspace root holding `pnpm-workspace.yaml`.
 
 @returns Workspace package glob patterns.
 
 @throws Error when the workspace manifest carries no package globs.
 
 @example
 ```ts
 await workspaceGlobs({ workspaceRoot: '/repo' });
 // ['package/*\/*']
 ```
 */
async function workspaceGlobs({
  workspaceRoot,
}: Readonly<{
  workspaceRoot: string;
}>,): Promise<readonly string[]> {
  /**
   Parsed workspace manifest before shape validation.
   */
  const manifest: unknown = parseYaml(await readFile(
    posix.join(
      workspaceRoot,
      'pnpm-workspace.yaml',
    ),
    'utf8',
  ),);

  if (((typeof manifest) !== 'object') || (manifest === null)
    || (!('packages' in manifest)))
    throw new Error(`pnpm-workspace.yaml under ${workspaceRoot} declares no packages list`,);

  /**
   Raw package globs before element validation.
   */
  const { packages, } = manifest;

  if ((!Array.isArray(packages,)) || (!packages.every(function isString(entry,): entry is string {
    return (typeof entry) === 'string';
  },)))
    throw new Error(`pnpm-workspace.yaml under ${workspaceRoot} packages list is not a string array`,);

  return packages;
}

/**
 Discovers workspace packages and their TypeScript sources.
 
 Declaration files are excluded; generated `.d.ts` carries no runtime
 exports worth reporting.
 
 @param workspaceRoot - Absolute workspace root holding `pnpm-workspace.yaml`.
 
 @returns Packages sorted by directory.
 
 @throws Error when the workspace manifest is missing or malformed, or
 when a discovered package manifest declares no name.
 
 @example
 ```ts
 const packages = await discoverWorkspacePackages({ workspaceRoot: '/repo' });
 ```
 */
export async function discoverWorkspacePackages({
  workspaceRoot,
}: Readonly<{
  workspaceRoot: string;
}>,): Promise<readonly WorkspacePackage[]> {
  /**
   Discovery logger tagged with the calling function.
   */
  const dl = tagged({
    tag: discoverWorkspacePackages.name,
    l,
  },);
  /**
   Workspace package glob patterns.
   */
  const globs = await workspaceGlobs({ workspaceRoot, },);
  /**
   Manifest paths matched by every package glob.
   */
  const manifestPaths: string[] = [];

  for await (const entry of glob(
    globs.map(function toManifestGlob(pattern,): string {
      return posix.join(
        pattern,
        'package.json',
      );
    },),
    { cwd: workspaceRoot, },
  ))
    manifestPaths.push(entry,);

  /**
   Discovered packages, unsorted.
   */
  const discovered = await Promise.all(manifestPaths
    .filter(function outsideDependencies(manifestPath,): boolean {
      return !manifestPath.includes('node_modules',);
    },)
    .map(async function toPackage(manifestPath,): Promise<WorkspacePackage> {
      /**
       Workspace-relative package directory.
       */
      const dir = posix.dirname(manifestPath,);
      /**
       Parsed package manifest before name validation.
       */
      const manifest: unknown = JSON.parse(
        await readFile(
          posix.join(
            workspaceRoot,
            manifestPath,
          ),
          'utf8',
        ),
      );

      if (((typeof manifest) !== 'object') || (manifest === null)
        || (!('name' in manifest))
        || ((typeof manifest.name) !== 'string'))
        throw new Error(`workspace package manifest under ${dir} declares no package name`,);

      dl.debug(`discovered ${dir}`,);

      /**
       Source paths matched under the package's src tree.
       */
      const sourceFiles: string[] = [];

      for await (const sourcePath of glob(
        [
          'src/**/*.ts',
          'src/**/*.tsx',
        ],
        {
          cwd: posix.join(
            workspaceRoot,
            dir,
          ),
        },
      )) {
        if (!sourcePath.endsWith('.d.ts',))
          sourceFiles.push(posix.join(
            dir,
            sourcePath,
          ),);
      }

      return {
        dir,
        name: manifest.name,
        sourceFiles: sourceFiles.toSorted(),
      };
    },),);

  return discovered.toSorted(function byDir(
    a,
    b,
  ): number {
    return a.dir < b.dir ? -1 : 1;
  },);
}
