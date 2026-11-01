import {
  foundValue,
  type Lookup,
  missingValue,
} from './lookup.ts';
import {
  endsWithComment,
  locateValue,
  skipBlanks,
  splitManifest,
  type ValueLocation,
} from './toml-line.ts';

/**
 Key that holds the version in both a package table and an inline dependency table.
 */
export const versionKey = 'version';

/**
 Header of the table that holds the crate's own version.
 */
const packageHeader = '[package]';

/**
 Check whether a trimmed line is the header of the package table, with at most a comment after it.

 @param trimmed - line without surrounding blanks

 @returns true for `[package]` and `[package] # note`

 @example
 ```ts
 isPackageHeader('[package]');
 ```
 */
function isPackageHeader(trimmed: string): boolean {
  return trimmed.startsWith(packageHeader) && endsWithComment(trimmed.slice(packageHeader.length));
}

/**
 Mark each line that belongs to the package table: the lines after its header up to the next table header.

 @param bodies - line bodies

 @returns one flag per line

 @example
 ```ts
 packageLineFlags(['[package]', 'version = "1"', '[dependencies]']);
 ```
 */
function packageLineFlags(bodies: readonly string[]): readonly boolean[] {
  return bodies.reduce<boolean[]>(
    function mark(
      flags,
      body
    ) {
      /**
       Whether the line before this one belonged to the package table.
       */
      const previous = flags.at(-1) ?? false;
      flags.push(body.charAt(skipBlanks({
        line: body,
        from: 0
      })) === '[' ? isPackageHeader(body.trim()) : previous);
      return flags;
    },
    [],
  );
}

/**
 A manifest line that holds the crate's own version, with where its value sits.
 */
export type PackageVersionLine = {
  readonly index: number;
  readonly body: string;
  readonly location: ValueLocation;
};

/**
 Find the lines of the package table that carry a quoted `version` key.

 @param bodies - line bodies

 @returns the lines in file order, each with the place of its value

 @example
 ```ts
 packageVersionLines(['[package]', 'version = "1"']);
 ```
 */
export function packageVersionLines(bodies: readonly string[]): readonly PackageVersionLine[] {
  /**
   Flag per line for membership in the package table.
   */
  const inPackage = packageLineFlags(bodies);
  return bodies.flatMap(function packageVersion(
    body,
    index
  ) {
    if (inPackage[index] !== true)
      return [];
    /**
     The version value, when this line is the version key.
     */
    const location = locateValue({
      line: body,
      from: skipBlanks({
        line: body,
        from: 0
      }),
      key: versionKey
    });
    return location.found ? [
      {
        index,
        body,
        location: location.value
      }
    ] : [];
  });
}

/**
 Read the crate's own version out of the package table of a manifest, changing nothing.

 Only the `version` key of the `[package]` table counts, as in the release edit, so a `rust-version` key, the version
 of another table or a dependency's version is never read.

 @param text - manifest text

 @returns the version, or a miss that says the package table holds no quoted `version` (an inherited
 `version.workspace` key is such a miss)

 @example
 ```ts
 readPackageVersion('[package]\nversion = "0.4.2"\n');
 ```
 */
export function readPackageVersion(text: string): Lookup<string> {
  /**
   The first line of the package table that holds a quoted version.
   */
  const [first] = packageVersionLines(splitManifest(text)
    .bodies);
  return first === undefined
    ? missingValue('The manifest has no quoted version = "..." line in its [package] table.')
    : foundValue(first.location
      .value);
}
