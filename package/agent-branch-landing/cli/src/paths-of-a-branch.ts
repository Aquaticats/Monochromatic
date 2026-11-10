/**
 What a branch's changed paths say about it: which packages it touches, which paths a release gate depends on, and whether it
 changes the lockfile.

 @module
 */

import {
  LOCKFILE_PATH,
  PACKAGES_PREFIX,
  PACKAGE_PATH_SEGMENTS,
  PATHS_A_RELEASE_DEPENDS_ON,
} from './constants.ts';

/**
 Lists the packages a branch changed, each once, in the order the paths named them.

 @param changedPaths - the repository-relative paths a branch changed

 @returns each changed package's path, as `package/<category>/<name>`

 @example
 ```ts
 const packages = packagesAmong(['package/a/b/src/x.ts',],);
 ```
 */
export function packagesAmong(changedPaths: readonly string[],): readonly string[] {
  return [...new Set(changedPaths
    .filter(function isInAPackage(path: string,) {
      return path.startsWith(PACKAGES_PREFIX,);
    },)
    .map(function packageOf(path: string,) {
      return path.split('/',)
        .slice(
        0,
        PACKAGE_PATH_SEGMENTS,
        )
        .join('/',);
    },),),];
}

/**
 Tells whether one path is one a release gate depends on: anything below a watched directory, or a watched file name at any
 depth, so every workspace root's manifest and lockfile counts.

 @param path - a repository-relative path

 @returns true when a release's evidence stops covering the repository once this path changes

 @example
 ```ts
 const guarded = isReleasePath('package/git-policy/cli/src/lib.rs',);
 ```
 */
export function isReleasePath(path: string,): boolean {
  /**
   The path's own name, which is all a watched file name matches.
   */
  const name = path.split('/',)
    .at(-1,)
    ?? path;
  return PATHS_A_RELEASE_DEPENDS_ON.fileNames
    .includes(name,)
    || PATHS_A_RELEASE_DEPENDS_ON.directories
      .some(function contains(path_: string,) {
        return (path === path_) || path.startsWith(`${path_}/`,);
      },);
}

/**
 Picks the paths among a branch's changes that a release gate depends on.

 @param changedPaths - the repository-relative paths a branch changed

 @returns those a release depends on, each once, in the order given

 @example
 ```ts
 const guarded = releasePathsAmong(['Cargo.toml', 'doc/a.md',],);
 ```
 */
export function releasePathsAmong(changedPaths: readonly string[],): readonly string[] {
  return [...new Set(changedPaths.filter(isReleasePath,),),];
}

/**
 Tells whether a branch changed the lockfile, which leaves the main checkout needing an install once the branch lands.

 @param changedPaths - the repository-relative paths a branch changed

 @returns true when the lockfile changed

 @example
 ```ts
 const needsInstall = changesTheLockfile(['pnpm-lock.yaml',],);
 ```
 */
export function changesTheLockfile(changedPaths: readonly string[],): boolean {
  return changedPaths.includes(LOCKFILE_PATH,);
}
