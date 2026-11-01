import {
  dirname,
  isAbsolute,
  posix,
  relative,
  sep,
} from 'node:path';

import {
  lockFileName,
  manifestFileName,
} from './constants.ts';
import {
  isRecord,
  parseJson,
} from './json-value.ts';
import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';

/**
 The repository paths a release depends on: files that must not change, and directories none of whose files may change.
 Every path is repository-relative, with `/` as separator and no trailing slash.
 */
export type WatchSet = {
  readonly files: readonly string[];
  readonly directories: readonly string[];
};

/**
 Tell whether a changed path lies inside a watched path.

 A file matches only itself. A directory matches itself and what lies below it, and a name that merely starts with the
 directory's name, such as `package/ab` for `package/a`, is another path.

 @param path - repository-relative path that changed

 @param watch - the paths the release depends on

 @returns true when the path is one of the watched files, or is or lies below one of the watched directories

 @example
 ```ts
 isWatchedPath({ path: 'package/a/src/lib.rs', watch: { files: [], directories: ['package/a'] } });
 ```
 */
export function isWatchedPath({
  path,
  watch,
}: {
  readonly path: string;
  readonly watch: WatchSet;
}): boolean {
  return watch.files
    .includes(path)
    || watch.directories
      .some(function containsPath(directory) {
        return (path === directory) || path.startsWith(`${directory}/`);
      });
}

/**
 Pick, from the paths a comparison lists as changed, the ones the release depends on.

 @param changed - repository-relative paths the comparison lists, a renamed file's old and new path both

 @param watch - the paths the release depends on

 @returns the watched paths among them, each once, in the order given

 @example
 ```ts
 changedWatchedPaths({ changed: ['a/x', 'b/y'], watch: { files: [], directories: ['a'] } });
 ```
 */
export function changedWatchedPaths({
  changed,
  watch,
}: {
  readonly changed: readonly string[];
  readonly watch: WatchSet;
}): readonly string[] {
  return [...new Set(changed.filter(function isWatched(path) {
    return isWatchedPath({
      path,
      watch
    });
  }))];
}

/**
 Join watch sets into one.

 @param sets - the sets to join

 @returns every file and directory of any set, each once, in order of first appearance

 @example
 ```ts
 mergeWatchSets([{ files: ['a'], directories: [] }, { files: [], directories: ['b'] }]);
 ```
 */
export function mergeWatchSets(sets: readonly WatchSet[]): WatchSet {
  return {
    files: [...new Set(sets.flatMap(function filesOf(set) {
      return set.files;
    }))],
    directories: [...new Set(sets.flatMap(function directoriesOf(set) {
      return set.directories;
    }))],
  };
}

/**
 Arguments of `cargo metadata` that list a manifest's packages and their dependency closure.

 `--locked` keeps the call from rewriting a lockfile the release has already settled and committed.

 @param manifestPath - repository-relative path of the `Cargo.toml` to read

 @returns the arguments after `cargo`; the output is JSON

 @example
 ```ts
 buildCargoMetadataArguments('package/a/Cargo.toml');
 ```
 */
export function buildCargoMetadataArguments(manifestPath: string): readonly string[] {
  return [
    'metadata',
    '--format-version=1',
    '--locked',
    `--manifest-path=${manifestPath}`,
  ];
}

/**
 Turn an absolute path into a repository-relative one with `/` separators.

 @param root - absolute path of the repository root

 @param path - absolute path under inspection

 @returns the relative path, `''` for the root itself, or a miss when the path is outside the repository

 @example
 ```ts
 insideRepository({ root: '/r', path: '/r/package/a' });
 ```
 */
function insideRepository({
  root,
  path,
}: {
  readonly root: string;
  readonly path: string;
}): Lookup<string> {
  /**
   The path relative to the root, with the platform's separator.
   */
  const relativePath = relative(
    root,
    path
  );
  if ((relativePath === '..') || relativePath.startsWith(`..${sep}`)
    || isAbsolute(relativePath))
    return missingValue(`${path} is outside ${root}.`);
  return foundValue(relativePath.split(sep)
    .join(posix.sep));
}

/**
 Read the paths a manifest's build depends on from the answer of `cargo metadata`.

 Those are the directory of every package that comes from a path, not from a registry, which is the crate itself and
 every path dependency in its closure, and the manifest and lockfile of the workspace root, which hold what the members
 inherit and lock. A package outside the repository is left out, because a comparison of the repository cannot see it.

 @param body - JSON text printed by `cargo metadata --format-version=1`

 @param root - absolute path of the repository root, which the paths of the answer are relative to

 @returns the watch set of the manifest, or a miss that says what the answer lacked or why a path cannot be watched

 @example
 ```ts
 parseCargoMetadata({ body: '{"packages":[],"workspace_root":"/r/a"}', root: '/r' });
 ```
 */
export function parseCargoMetadata({
  body,
  root,
}: {
  readonly body: string;
  readonly root: string;
}): Lookup<WatchSet> {
  /**
   Parsed answer.
   */
  const answer = parseJson(body);
  if (!answer.found)
    return answer;
  if ((!isRecord(answer.value)) || (!Array.isArray(answer.value
    .packages))
    || ((typeof answer.value
      .workspace_root) !== 'string'))
    return missingValue('The cargo metadata answer lacks its packages or its workspace root.');
  /**
   Directories of the packages that come from a path.
   */
  const directories: string[] = [];
  for (const entry of answer.value
    .packages as readonly unknown[]) {
    if ((!isRecord(entry)) || (entry.source !== null))
      continue;
    if ((typeof entry.manifest_path) !== 'string')
      return missingValue('A path package of the cargo metadata answer lacks its manifest path.');
    /**
     The package directory, relative to the repository.
     */
    const directory = insideRepository({
      root,
      path: dirname(entry.manifest_path as string)
    });
    if (!directory.found)
      continue;
    if (directory.value === '')
      return missingValue(
        `The package ${JSON.stringify(entry.name)} sits at the repository root, so its directory cannot be watched without watching every file.`,
      );
    directories.push(directory.value);
  }
  /**
   The workspace root, relative to the repository.
   */
  const workspace = insideRepository({
    root,
    path: answer.value
      .workspace_root as string
  });
  /**
   The manifest and lockfile of the workspace root, when it is inside the repository.
   */
  const files = workspace.found ? [
    manifestFileName,
    lockFileName
  ].map(function inWorkspace(name) {
    return workspace.value === '' ? name : posix.join(
      workspace.value,
      name
    );
  }) : [];
  return foundValue({
    files,
    directories: [...new Set(directories)]
  });
}
