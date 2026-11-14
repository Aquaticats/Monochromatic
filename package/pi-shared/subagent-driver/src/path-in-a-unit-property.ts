/**
 One path written the way systemd reads it inside a `ReadOnlyPaths=` or `ReadWritePaths=` property.

 Why: systemd reads each of those properties as a space-separated list of paths, so a checkout folder or a worktree that carries a
 space, a quote or a line break is read as several paths or as a broken one, and the manager refuses the whole unit
 (`Invalid ReadOnlyPaths`, or `Failed to parse ReadWritePaths= value '…': Invalid argument`). On this machine the double quotes
 around each path are what makes one path out of such a name, and a backslash or a double quote inside it is escaped so that the
 quoting itself reads them as text (`systemd.syntax` 7; a `systemd-run --property=` value is read as a unit-file assignment, per
 `systemctl` 1's `set-property`).

 What is written here is only the item: the encoding happens at the last interpolation, where the path becomes part of the
 `--property=` word, so no caller builds a property value out of already-encoded pieces. A path that is not absolute cannot be
 written at all, because systemd's path properties are paths under the host's root, so it is refused here with words that name it.

 @module
 */

import { isAbsolute, } from 'node:path';

/**
 The prefix that tells systemd to leave out a path that is not there.
 */
const IGNORE_WHEN_MISSING_PREFIX: string = '-';

/**
 The backslash, which the quoting itself reads and which a path has to spell out to be read as text.
 */
const BACKSLASH: string = '\\';

/**
 The double quote, which wraps the item and which a path has to spell out to be read as text.
 */
const DOUBLE_QUOTE: string = '"';

/**
 A path that systemd's `ReadOnlyPaths=` and `ReadWritePaths=` cannot take.

 @example
 ```ts
 throw new PathThatIsNotAbsoluteError("the path 'sub/repo' is not absolute",);
 ```
 */
export class PathThatIsNotAbsoluteError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PathThatIsNotAbsoluteError';
}

/**
 Writes one path as the item of a systemd path property that systemd reads as that one path.

 @param path - the path to write, which must be absolute

 @param ignoredWhenMissing - whether systemd leaves the path out when it is not there, which is its `-` prefix

 @returns the item, quoted, and prefixed by `-` when it may be absent

 @throws PathThatIsNotAbsoluteError when the path is not absolute, because systemd's path properties take absolute paths only

 @example
 ```ts
 const item = pathInAUnitProperty({ path: '/data/my repo', },);
 ```
 */
export function pathInAUnitProperty({
  path,
  ignoredWhenMissing = false,
}: {
  /**
   The path to write, which must be absolute.
   */
  readonly path: string;

  /**
   Whether systemd leaves the path out when it is not there.
   */
  readonly ignoredWhenMissing?: boolean;
},): string {
  if (!isAbsolute(path,)) {
    throw new PathThatIsNotAbsoluteError(
      `the path '${path}' is not absolute, so systemd cannot take it: its ReadOnlyPaths= and ReadWritePaths= are lists of paths under the host's root, and a relative one refuses the whole unit with "Invalid ReadOnlyPaths". Name the path as an absolute one, starting with /.`,
    );
  }
  /**
   The path with each backslash spelled out, so that it reaches systemd as one backslash.
   */
  const withoutBackslashes = path.replaceAll(
    BACKSLASH,
    `${BACKSLASH}${BACKSLASH}`,
  );
  /**
   The path with each double quote spelled out too, so that the quoting reads it as text rather than as the end of the item.
   */
  const escaped = withoutBackslashes.replaceAll(
    DOUBLE_QUOTE,
    `${BACKSLASH}${DOUBLE_QUOTE}`,
  );
  /**
   The path as one quoted item, in which every other character is text.
   */
  const item = `${DOUBLE_QUOTE}${escaped}${DOUBLE_QUOTE}`;
  return ignoredWhenMissing ? `${IGNORE_WHEN_MISSING_PREFIX}${item}` : item;
}
