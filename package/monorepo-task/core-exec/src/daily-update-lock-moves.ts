/**
 Finds which locked versions moved between two texts of a lock, so an issue can say what the update changed.

 Each reader makes one pass over its text, with string operations.

 @module
 */

import type { LockMove, } from './daily-update-package-check-types.ts';

/**
 A `Cargo.lock` name line starts with this and ends with a closing quote.
 */
const CARGO_NAME_START = 'name = "';

/**
 A `Cargo.lock` version line starts with this and ends with a closing quote.
 */
const CARGO_VERSION_START = 'version = "';

/**
 The closing quote of a `Cargo.lock` value.
 */
const CARGO_QUOTE_END = '"';

/**
 The line that opens the importers section of a `pnpm-lock.yaml`.
 */
const PNPM_IMPORTERS_HEADING = 'importers:';

/**
 Spaces before an importer key in a `pnpm-lock.yaml`.
 */
const PNPM_IMPORTER_INDENT = 2;

/**
 Spaces before a dependency name inside an importer.
 */
const PNPM_DEPENDENCY_INDENT = 6;

/**
 Spaces before a field of a dependency, such as its `version`.
 */
const PNPM_FIELD_INDENT = 8;

/**
 A dependency field line that carries the locked version starts with this.
 */
const PNPM_VERSION_START = 'version: ';

/**
 A YAML key line ends with this.
 */
const YAML_KEY_END = ':';

/**
 The quote YAML puts around a key with special characters.
 */
const YAML_QUOTE = "'";

/**
 Removes the single quotes YAML puts around a key with special characters.

 @param key - key text

 @returns the key without surrounding single quotes

 @example
 ```ts
 unquote("'@types/node'",); // '@types/node'
 ```
 */
export function unquote(key: string,): string {
  return key.startsWith(YAML_QUOTE,) && key.endsWith(YAML_QUOTE,) ? key.slice(
    1,
    -1,
  ) : key;
}

/**
 How many spaces a line starts with.

 @param line - one line

 @returns the count of leading spaces

 @example
 ```ts
 leadingSpaces('    a',); // 4
 ```
 */
export function leadingSpaces(line: string,): number {
  return line.length
    - line.trimStart()
      .length;
}

/**
 A name with one version, as a lock lists them.
 */
export type NamedVersion = {
  /**
   Crate, package or tool name.
   */
  readonly name: string;
  /**
   Locked version.
   */
  readonly version: string;
};

/**
 Collects the versions of every named entry of a text.

 @param versions - pairs of name and version found in the text

 @returns name to the set of versions

 @example
 ```ts
 groupVersions([{ name: 'a', version: '1', },],); // Map { 'a' => Set { '1' } }
 ```
 */
export function groupVersions(versions: readonly NamedVersion[],): ReadonlyMap<string, ReadonlySet<string>> {
  /**
   Versions seen per name.
   */
  const grouped = new Map<string, Set<string>>();
  for (const {
    name,
    version,
  } of versions) {
    /**
     Versions already seen for the name.
     */
    const seen = grouped.get(name,) ?? new Set<string>();
    seen.add(version,);
    grouped.set(
      name,
      seen,
    );
  }
  return grouped;
}

/**
 Turns two groupings into the moves between them: for each name, the versions that left and the versions that arrived.

 @param before - versions per name before

 @param after - versions per name after

 @returns the moves ordered by name; a name whose versions did not change is left out

 @example
 ```ts
 movesBetween({ before: new Map([['a', new Set(['1',]),],],), after: new Map([['a', new Set(['2',]),],],), },); // a: 1 to 2
 ```
 */
export function movesBetween({
  before,
  after,
}: {
  readonly before: ReadonlyMap<string, ReadonlySet<string>>;
  readonly after: ReadonlyMap<string, ReadonlySet<string>>;
},): readonly LockMove[] {
  /**
   Every name on either side, ordered.
   */
  const names = [
    ...new Set([
      ...before.keys(),
      ...after.keys(),
    ],),
  ].toSorted();
  /**
   Moves found so far.
   */
  const moves: LockMove[] = [];
  for (const name of names) {
    /**
     Versions on each side.
     */
    const was = before.get(name,) ?? new Set<string>();
    /**
     Versions after.
     */
    const is = after.get(name,) ?? new Set<string>();
    /**
     Versions that left.
     */
    const left = [...was,].filter(function gone(version,) {
      return !is.has(version,);
    },)
      .toSorted();
    /**
     Versions that arrived.
     */
    const arrived = [...is,].filter(function fresh(version,) {
      return !was.has(version,);
    },)
      .toSorted();
    if ((left.length > 0) || (arrived.length > 0)) {
      moves.push({
        name,
        from: left.join(', ',),
        to: arrived.join(', ',),
      },);
    }
  }
  return moves;
}

/**
 Reads the name and version of every `[[package]]` of a `Cargo.lock`.

 @param text - lock text

 @returns pairs of name and version

 @example
 ```ts
 cargoVersionsOf('[[package]]\nname = "a"\nversion = "1"\n',); // [{ name: 'a', version: '1', }]
 ```
 */
export function cargoVersionsOf(text: string,): readonly NamedVersion[] {
  /**
   Pairs found so far.
   */
  const pairs: NamedVersion[] = [];
  /**
   Name of the package block being read.
   */
  let name = '';
  for (const line of text.split('\n',)) {
    if (line.startsWith(CARGO_NAME_START,) && line.endsWith(CARGO_QUOTE_END,)) {
      name = line.slice(
        CARGO_NAME_START.length,
        -CARGO_QUOTE_END.length,
      );
    }
    else if (line.startsWith(CARGO_VERSION_START,) && line.endsWith(CARGO_QUOTE_END,)
      && (name !== '')) {
      pairs.push({
        name,
        version: line.slice(
          CARGO_VERSION_START.length,
          -CARGO_QUOTE_END.length,
        ),
      },);
      name = '';
    }
  }
  return pairs;
}

/**
 Lists the crates whose locked versions differ between two texts of a `Cargo.lock`.

 @param before - lock text before

 @param after - lock text after

 @returns moves ordered by crate name

 @example
 ```ts
 cargoLockMoves({ before, after, },); // [{ name: 'zbus', from: '5.19.0', to: '6.2.1', }]
 ```
 */
export function cargoLockMoves({
  before,
  after,
}: {
  readonly before: string;
  readonly after: string;
},): readonly LockMove[] {
  return movesBetween({
    before: groupVersions(cargoVersionsOf(before,),),
    after: groupVersions(cargoVersionsOf(after,),),
  },);
}

/**
 Where the reader of a `pnpm-lock.yaml` is.
 */
type PnpmPlace = {
  /**
   Whether the reader is inside the importers section.
   */
  inImporters: boolean;
  /**
   Whether the reader is inside the importer asked for.
   */
  inWanted: boolean;
  /**
   Name of the dependency being read.
   */
  dependency: string;
};

/**
 The key a YAML key line names, without its colon.

 @param line - one line

 @returns the key, unquoted; the empty string when the line is not a key line

 @example
 ```ts
 yamlKeyOf("      '@types/node':",); // '@types/node'
 ```
 */
export function yamlKeyOf(line: string,): string {
  /**
   The line without surrounding spaces.
   */
  const trimmed = line.trim();
  return trimmed.endsWith(YAML_KEY_END,) ? unquote(trimmed.slice(
    0,
    -YAML_KEY_END.length,
  ),) : '';
}

/**
 Reads the locked version of every dependency of one importer in a `pnpm-lock.yaml`.

 @param text - lock text

 @param importer - importer key, a repository-relative package directory

 @returns pairs of dependency name and version

 @example
 ```ts
 pnpmVersionsOf({ text, importer: 'package/a/b', },); // [{ name: 'typescript', version: '6.0.1', }]
 ```
 */
export function pnpmVersionsOf({
  text,
  importer,
}: {
  readonly text: string;
  readonly importer: string;
},): readonly NamedVersion[] {
  /**
   Pairs found so far.
   */
  const pairs: NamedVersion[] = [];
  /**
   Where the reader is.
   */
  const place: PnpmPlace = {
    inImporters: false,
    inWanted: false,
    dependency: '',
  };
  for (const line of text.split('\n',)) {
    if (line === PNPM_IMPORTERS_HEADING) {
      place.inImporters = true;
      continue;
    }
    /**
     How deep the line is indented.
     */
    const indent = leadingSpaces(line,);
    if (place.inImporters && (indent === 0)
      && (line !== ''))
      place.inImporters = false;
    if (!place.inImporters)
      continue;
    if ((indent === PNPM_IMPORTER_INDENT) && (yamlKeyOf(line,) !== '')) {
      place.inWanted = yamlKeyOf(line,) === importer;
    }
    else if (place.inWanted && (indent === PNPM_DEPENDENCY_INDENT)
      && (yamlKeyOf(line,) !== '')) {
      place.dependency = yamlKeyOf(line,);
    }
    else if (
      place.inWanted
        && (indent === PNPM_FIELD_INDENT)
        && (place.dependency !== '')
        && line.trim()
          .startsWith(PNPM_VERSION_START,)
    ) {
      pairs.push({
        name: place.dependency,
        version: line.trim()
          .slice(PNPM_VERSION_START.length,),
      },);
    }
  }
  return pairs;
}

/**
 Lists the dependencies of one importer whose locked versions differ between two texts of a `pnpm-lock.yaml`.

 @param before - lock text before

 @param after - lock text after

 @param importer - importer key, a repository-relative package directory

 @returns moves ordered by dependency name

 @example
 ```ts
 pnpmImporterMoves({ before, after, importer: 'package/a/b', },); // [{ name: 'typescript', from: '5.9.0', to: '6.0.1', }]
 ```
 */
export function pnpmImporterMoves({
  before,
  after,
  importer,
}: {
  readonly before: string;
  readonly after: string;
  readonly importer: string;
},): readonly LockMove[] {
  return movesBetween({
    before: groupVersions(pnpmVersionsOf({
      text: before,
      importer,
    },),),
    after: groupVersions(pnpmVersionsOf({
      text: after,
      importer,
    },),),
  },);
}

/**
 Reads the locked version of every tool in a `toolchain.lock.json`.

 @param text - lock text

 @returns pairs of tool name and version, none when the text is empty or not a lock

 @example
 ```ts
 toolchainVersionsOf('{"tools":{"zig":{"version":"0.17.1"}}}',); // [{ name: 'zig', version: '0.17.1', }]
 ```
 */
export function toolchainVersionsOf(text: string,): readonly NamedVersion[] {
  if (text === '')
    return [];
  try {
    /**
     The parsed lock.
     */
    const lock: unknown = JSON.parse(text,);
    if (((typeof lock) !== 'object') || (lock === null)
      || (!('tools' in lock))
      || ((typeof lock.tools) !== 'object')
      || (lock.tools === null))
      return [];
    /**
     The table of tools.
     */
    const tools: object = lock.tools;
    return Object.keys(tools,)
      .flatMap(function versionOf(name,) {
        /**
         The tool's row.
         */
        const row: unknown = Reflect.get(
          tools,
          name,
        );
        return ((typeof row) === 'object') && (row !== null)
          && ('version' in row)
          && ((typeof row.version) === 'string')
          ? [{
            name,
            version: row.version,
          },]
          : [];
      },);
  }
  catch (error) {
    process.stderr
      .write(`daily-update: a toolchain lock text could not be read, so its moves are not listed: ${String(error,)}\n`,);
    return [];
  }
}

/**
 Lists the tools whose locked versions differ between two texts of a `toolchain.lock.json`.
 A side that is empty or unreadable lists nothing, because a missing file says nothing about what moved.

 @param before - lock text before

 @param after - lock text after

 @returns moves ordered by tool name

 @example
 ```ts
 toolchainLockMoves({ before, after, },); // [{ name: 'rust', from: 'nightly-2026-11-01', to: 'nightly-2026-11-07', }]
 ```
 */
export function toolchainLockMoves({
  before,
  after,
}: {
  readonly before: string;
  readonly after: string;
},): readonly LockMove[] {
  /**
   Versions on each side.
   */
  const was = toolchainVersionsOf(before,);
  /**
   Versions after.
   */
  const is = toolchainVersionsOf(after,);
  if ((was.length === 0) || (is.length === 0))
    return [];
  return movesBetween({
    before: groupVersions(was,),
    after: groupVersions(is,),
  },);
}
