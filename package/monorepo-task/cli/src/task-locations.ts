import { existsSync, readdirSync, } from 'node:fs';
import { dirname, join, } from 'node:path';

/** Workspace root marker. */
const ROOT_MARKER = 'pnpm-workspace.yaml';
/** Category and package segments beneath package/. */
const PACKAGE_SEGMENT_COUNT = 2;

//region Repo layout: locating the root and packages

/**
 Finds the monorepo root by walking up for the workspace marker.

 @returns Absolute path of the monorepo root.
 @throws When no root marker is found from the current directory.
 */
export function findRepoRoot(): string {
  let current = process.cwd();
  while (true) {
    if (existsSync(join(current, ROOT_MARKER))) { return current; }
    const parent = dirname(current);
    if (parent === current)
      throw new Error(`task-cli: no ${ROOT_MARKER} above ${process.cwd()}`);
    current = parent;
  }
}

/**
 Manifest files that mark a package directory: the npm, Cargo and Gradle homes
 this monorepo's packages use.
 */
const PACKAGE_MARKERS = ['package.json', 'Cargo.toml', 'build.gradle.kts', 'build.gradle',] as const;

/**
 Resolves the package directory for a `//package/<category>/<name>[:task]` name.

 @param root - Monorepo root.
 @param packagePath - Path like `package/logger/core`.
 @returns Absolute package directory.
 @throws When the path is not a `package/` path with two segments below it, or has no manifest.
 */
export function packageDirectory({ root, packagePath, }: { readonly root: string; readonly packagePath: string; },): string {
  const segments = packagePath.split('/');
  if (segments[0] !== 'package' || segments.length !== 1 + PACKAGE_SEGMENT_COUNT)
    throw new Error(`task-cli: not a package path: ${packagePath}`);
  const directory = join(root, packagePath);
  if (!PACKAGE_MARKERS.some((marker) => existsSync(join(directory, marker))))
    throw new Error(`task-cli: no package manifest (${PACKAGE_MARKERS.join(', ')}) at ${packagePath}`);
  return directory;
}

/**
 Walks `package/` for task files without reading ignored dependency trees.

 @param root - Monorepo root.
 @returns Absolute paths of every package `task.ts`, sorted.
 */
export function findPackageTaskFiles(root: string): readonly string[] {
  const packageRoot = join(root, 'package');
  const files: string[] = [];
  for (const first of readdirSync(packageRoot, { withFileTypes: true, })) {
    if (!first.isDirectory()) { continue; }
    const categoryDirectory = join(packageRoot, first.name);
    for (const second of readdirSync(categoryDirectory, { withFileTypes: true, })) {
      if (!second.isDirectory()) { continue; }
      const taskPath = join(categoryDirectory, second.name, 'task.ts');
      if (existsSync(taskPath)) { files.push(taskPath); }
    }
  }
  return files.toSorted();
}

//endregion
