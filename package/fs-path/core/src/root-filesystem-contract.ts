/**
 Filesystem contract shared by the upward-walk root discovery and its
 adapters: `root-filesystem.node.ts` (`node:fs/promises`),
 `root-filesystem.neutral.ts` (origin private file system), and
 `memory-root-filesystem.ts` (in-memory, for tests and callers that hold a
 tree in memory). Lives apart from all of them so no adapter imports the
 walker and no cycle forms.

 Paths are absolute POSIX strings; path arithmetic is not part of the
 contract, the walker and the markers do it through `#posix-path`.

 @module
 */

/**
 Sentinel returned by {@link RootFilesystem.readTextFile} when a path is absent.
 A unique `Symbol` distinguishes "file missing" from empty content without a
 nullish union; matchers narrow with `content !== ABSENT` before inspecting it.
 */
export const ABSENT: unique symbol = Symbol('root discovery path content absent on filesystem',);

/**
 Filesystem operations needed by upward root discovery.
 */
export type RootFilesystem = {
  /**
   Reads UTF-8 text, returning {@link ABSENT} when path is absent.
   */
  readonly readTextFile: (path: string,) => Promise<string | typeof ABSENT>;

  /**
   Reads symbolic-link target text,
   returning {@link ABSENT} when path is absent or not a symbolic link.
   */
  readonly readSymbolicLink: (path: string,) => Promise<string | typeof ABSENT>;

  /**
   Checks whether path exists as any filesystem entry kind.
   */
  readonly exists: (path: string,) => Promise<boolean>;

  /**
   Checks whether path resolves to a directory.
   */
  readonly isDirectory: (path: string,) => Promise<boolean>;

  /**
   Checks whether path resolves to a regular file.
   */
  readonly isFile: (path: string,) => Promise<boolean>;
};

/**
 Returns absence for runtimes without symbolic links.

 @param _path - ignored symbolic-link path

 @returns promise resolving to {@link ABSENT}

 @example
 ```ts
 await unsupportedReadSymbolicLink('/repo/.git/HEAD');
 ```
 */
export function unsupportedReadSymbolicLink(_path: string,): Promise<typeof ABSENT> {
  return Promise.resolve(ABSENT,);
}
