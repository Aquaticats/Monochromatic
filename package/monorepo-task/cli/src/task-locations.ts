import {
  readdir,
  stat,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

//region Repository and package discovery without entering dependency trees

/**
 Workspace root marker.
 */
const ROOT_MARKER = 'pnpm-workspace.yaml';

/**
 Category and package segments beneath package/.
 */
const PACKAGE_SEGMENT_COUNT = 2;

/**
 Manifest files that mark the npm, Cargo and Gradle packages in this repository.
 */
const PACKAGE_MARKERS = [
  'package.json',
  'Cargo.toml',
  'build.gradle.kts',
  'build.gradle',
] as const;

/**
 Finds the repository root by walking parent directories for the workspace marker.

 @returns Absolute repository root.
 *
 @throws When no parent contains the workspace marker.
 *
 @example
 ```ts
 const root = await findRepoRoot();
 ```
 */
export async function findRepoRoot(): Promise<string> {
  for (let current = process.cwd(); ;) {
    if (await stat(
      join(current, ROOT_MARKER,),
      { throwIfNoEntry: false, },
    ) !== undefined)
      return current;
    /**
     Reaching a parent equal to itself means the filesystem root was examined.
     */
    const parent = dirname(current,);
    if (parent === current)
      throw new Error(`task-cli: no ${ROOT_MARKER} above ${process.cwd()}`,);
    current = parent;
  }
}

/**
 Resolves a package path and checks that it contains a supported package manifest.

 @param root - Repository boundary for package paths.
 *
 @param packagePath - Category and package path from the requested task.
 *
 @returns Absolute package directory.
 *
 @throws When the path shape or package manifest is missing.
 *
 @example
 ```ts
 const cwd = await packageDirectory({ root, packagePath: 'package/logger/core' });
 ```
 */
export async function packageDirectory({
  root,
  packagePath,
}: {
  readonly root: string;
  readonly packagePath: string;
},): Promise<string> {
  /**
   Accepted paths have exactly the repository prefix, category and package segments.
   */
  const segments = packagePath.split('/',);
  if ((segments[0] !== 'package') || (segments.length !== 1 + PACKAGE_SEGMENT_COUNT))
    throw new Error(`task-cli: not a package path: ${packagePath}`,);
  /**
   All manifest checks address this one candidate package.
   */
  const directory = join(
    root,
    packagePath,
  );
  /**
   Independent filesystem checks can complete concurrently.
   */
  const markers = await Promise.all(PACKAGE_MARKERS.map(async function manifest(marker) {
    return await stat(
      join(directory, marker,),
      { throwIfNoEntry: false, },
    ) !== undefined;
  }),);
  if (!markers.includes(true,))
    throw new Error(`task-cli: no package manifest (${PACKAGE_MARKERS.join(', ',)}) at ${packagePath}`,);
  return directory;
}

/**
 Enumerates task files at the known category/package depth.

 @param root - Repository containing the package directory.
 *
 @returns Sorted task paths without traversing ignored dependency trees.
 *
 @example
 ```ts
 const taskFiles = await findPackageTaskFiles(root);
 ```
 */
export async function findPackageTaskFiles(root: string,): Promise<readonly string[]> {
  /**
   Fixed package depth keeps discovery independent of dependency tree size.
   */
  const packageRoot = join(
    root,
    'package',
  );
  /**
   Non-directory entries cannot contain package tasks.
   */
  const categories = await readdir(
    packageRoot,
    { withFileTypes: true, },
  );
  /**
   Each category contributes its existing package task files.
   */
  const groups = await Promise.all(categories.map(async function categoryTasks(category) {
    if (!category.isDirectory())
      return [];
    /**
     Only immediate children are candidate packages.
     */
    const directory = join(
      packageRoot,
      category.name,
    );
    /**
     Package entries are read once before checking their task files.
     */
    const packages = await readdir(
      directory,
      { withFileTypes: true, },
    );
    /**
     Missing task files contribute no task paths.
     */
    const paths = await Promise.all(packages.map(async function packageTasks(entry) {
      if (!entry.isDirectory())
        return [];
      /**
       Task definitions have a fixed name within each package.
       */
      const path = join(
        directory,
        entry.name,
        'task.ts',
      );
      return await stat(
        path,
        { throwIfNoEntry: false, },
      ) === undefined ? [] : [path,];
    }),);
    return paths.flat();
  }),);
  return groups.flat()
    .toSorted(function byPath(left, right) { return left.localeCompare(right,); });
}

//endregion Repository and package discovery without entering dependency trees
