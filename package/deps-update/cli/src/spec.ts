/**
 Parsing for pnpm `name@version` package specs, as pnpm writes them into
 `minimumReleaseAgeExclude` when it records an immature pick.

 @module
 */

//region Types

/**
 One exact package version, split out of a `name@version` spec.
 */
export type PackageSpec = {
  /**
   npm package name, scoped or unscoped.
   */
  readonly name: string;
  /**
   Exact version string.
   */
  readonly version: string;
};

//endregion Types

//region Errors

/**
 Thrown when a string is not a `name@version` spec with both parts present.
 */
export class PackageSpecError extends Error {
  /**
   Builds the error for one unparseable spec.

   @param spec - rejected input, echoed so the caller sees what pnpm wrote

   @example
   ```ts
   throw new PackageSpecError('chord');
   ```
   */
  constructor(spec: string,) {
    super(`Expected a name@version spec, got ${JSON.stringify(spec,)}`,);
    this.name = 'PackageSpecError';
  }
}

//endregion Errors

//region Parse

/**
 Splits a `name@version` spec at its last `@`.

 The last `@` is the version separator because a scope's leading `@`
 sits at index 0 and npm names contain no other `@`.

 @param spec - `name@version` string, e.g. `@earendil-works/chord@0.87.1`

 @returns name and version parts

 @throws PackageSpecError when either part is empty

 @example
 ```ts
 parsePackageSpec('@earendil-works/chord@0.87.1');
 // { name: '@earendil-works/chord', version: '0.87.1' }
 ```
 */
export function parsePackageSpec(spec: string,): PackageSpec {
  /**
   Index of version separator; 0 or -1 means no separator after a name.
   */
  const separator = spec.lastIndexOf('@',);
  if ((separator <= 0) || (separator === (spec.length
    - 1)))
    throw new PackageSpecError(spec,);
  return {
    name: spec.slice(
      0,
      separator,
    ),
    version: spec.slice(separator + 1,),
  };
}

/**
 Separator pnpm writes between versions when it merges exact versions of
 one package into a single exclude entry (`name@1.0.0 || 1.0.1`).
 */
const VERSION_UNION_SEPARATOR = ' || ';

/**
 Expands one `minimumReleaseAgeExclude` entry into exact versions.

 Bare names and globs (`left-pad`, `@scope/*`) exempt every version, so they
 name no specific pick and expand to nothing.

 @param entry - exclude list entry as written in `pnpm-workspace.yaml`

 @returns exact versions the entry names, possibly none

 @throws PackageSpecError when a versioned entry has an empty part

 @example
 ```ts
 expandExcludeEntry('a@1.0.0 || 1.0.1');
 // [{ name: 'a', version: '1.0.0' }, { name: 'a', version: '1.0.1' }]
 expandExcludeEntry('@scope/*'); // []
 ```
 */
export function expandExcludeEntry(entry: string,): readonly PackageSpec[] {
  if (entry.lastIndexOf('@',) <= 0)
    return [];
  /**
   Name and possibly merged version list.
   */
  const {
    name,
    version,
  } = parsePackageSpec(entry,);
  return version.split(VERSION_UNION_SEPARATOR,)
    .map(function toSpec(part,): PackageSpec {
    /**
     One version with surrounding whitespace removed.
     */
      const trimmed = part.trim();
      if (trimmed === '')
        throw new PackageSpecError(entry,);
      return {
        name,
        version: trimmed,
      };
    },);
}

//endregion Parse
