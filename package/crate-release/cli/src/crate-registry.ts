import { join } from 'node:path';

import { caughtValueText } from '@monochromatic-dev/caught-value-core/ts';
import { isRecord, } from '@monochromatic-dev/type-guard-core/ts';

import { registryFileName } from './constants.ts';
import { CrateReleaseError } from './error.ts';
import {
  parseCommitScope,
  parseCrateName,
  parseManifestPath,
  parseRepositoryPath,
} from './validate.ts';

/**
 The arguments of one releasable crate: everything the release command needs besides the versions and the note.
 */
export type CrateEntry = {
  /**
   Repository-relative directory of the crate, holding its `Cargo.toml` and `Cargo.lock`.
   */
  readonly crateDirectory: string;
  /**
   Package scope of the release commit subject.
   */
  readonly scope: string;
  /**
   Repository-relative manifests that pin the crate exactly.
   */
  readonly pinFiles: readonly string[];
  /**
   Binaries the check builds; empty to check the whole crate.
   */
  readonly checkBins: readonly string[];
  /**
   Repository-relative directories whose lockfile records the changed version of a workspace member.
   */
  readonly lockDirectories: readonly string[];
};

/**
 Every releasable crate by its name on crates.io, which is also the `crate` choice of the publish workflow.
 */
export type CrateRegistry = ReadonlyMap<string, CrateEntry>;

/**
 Keys an entry of the registry file may hold.
 */
const entryKeys: readonly string[] = [
  'crateDir',
  'scope',
  'pinFiles',
  'checkBins',
  'lockDirs',
];

/**
 Parse the registry file's text as JSON.

 @param text - contents of the registry file

 @returns the parsed value

 @throws CrateReleaseError carrying the parser's complaint when the text is not JSON

 @example
 ```ts
 readJson('{}');
 ```
 */
function readJson(text: string): unknown {
  try {
    return JSON.parse(text) as unknown;
  }
  catch (error) {
    throw new CrateReleaseError(
      `${registryFileName} is not JSON: ${caughtValueText(error)}`,
      { cause: error },
    );
  }
}

/**
 Read a required text field of an entry.

 @param entry - the entry's fields

 @param label - where the entry sits, for the message

 @param key - field name

 @returns the text

 @throws CrateReleaseError when the field is missing or not text

 @example
 ```ts
 requiredText({ entry, label: 'crate "a"', key: 'scope' });
 ```
 */
function requiredText({
  entry,
  label,
  key,
}: {
  readonly entry: Readonly<Record<string, unknown>>;
  readonly label: string;
  readonly key: string;
}): string {
  /**
   The field as written.
   */
  const value = entry[key];
  if ((typeof value) !== 'string')
    throw new CrateReleaseError(`${registryFileName}: ${label} needs "${key}" as text.`);
  return value as string;
}

/**
 Read an optional list-of-text field of an entry.

 @param entry - the entry's fields

 @param label - where the entry sits, for the message

 @param key - field name

 @returns the texts, empty when the field is absent

 @throws CrateReleaseError when the field is present and is not a list of text

 @example
 ```ts
 textList({ entry, label: 'crate "a"', key: 'pinFiles' });
 ```
 */
function textList({
  entry,
  label,
  key,
}: {
  readonly entry: Readonly<Record<string, unknown>>;
  readonly label: string;
  readonly key: string;
}): readonly string[] {
  /**
   The field as written.
   */
  const value = entry[key];
  if (value === undefined)
    return [];
  if ((!Array.isArray(value))
    || (!(value as readonly unknown[]).every(function isText(item) {
      return (typeof item) === 'string';
    })))
    throw new CrateReleaseError(`${registryFileName}: ${label} needs "${key}" as a list of text.`);
  return value as readonly string[];
}

/**
 Validate one entry of the registry file.

 @param name - the crate's name, already checked

 @param entry - the entry as parsed from JSON

 @returns the validated entry

 @throws CrateReleaseError naming the crate and the field when anything is missing, unknown or unsafe

 @example
 ```ts
 parseCrateEntry({ name: 'a', entry: { crateDir: 'package/a', scope: 'a' } });
 ```
 */
function parseCrateEntry({
  name,
  entry,
}: {
  readonly name: string;
  readonly entry: unknown;
}): CrateEntry {
  /**
   Where the entry sits, for messages.
   */
  const label = `crate "${name}"`;
  if (!isRecord(entry))
    throw new CrateReleaseError(`${registryFileName}: ${label} must be an object.`);
  /**
   Keys the entry holds that no field uses.
   */
  const unknownKeys = Object.keys(entry)
    .filter(function isUnknown(key) {
      return !entryKeys.includes(key);
    });
  if (unknownKeys.length > 0)
    throw new CrateReleaseError(
      `${registryFileName}: ${label} has unknown fields ${JSON.stringify(unknownKeys)}; the fields are ${
        JSON.stringify(entryKeys)
      }.`,
    );
  return {
    crateDirectory: parseRepositoryPath({
      option: `${registryFileName} ${label} "crateDir"`,
      value: requiredText({
        entry,
        label,
        key: 'crateDir'
      })
    }),
    scope: parseCommitScope({
      option: `${registryFileName} ${label} "scope"`,
      value: requiredText({
        entry,
        label,
        key: 'scope'
      })
    }),
    pinFiles: textList({
      entry,
      label,
      key: 'pinFiles'
    })
      .map(function parsePinFile(value) {
        return parseManifestPath({
          option: `${registryFileName} ${label} "pinFiles"`,
          value
        });
      }),
    checkBins: textList({
      entry,
      label,
      key: 'checkBins'
    })
      .map(function parseCheckBin(value) {
        return parseCrateName({
          option: `${registryFileName} ${label} "checkBins"`,
          value
        });
      }),
    lockDirectories: textList({
      entry,
      label,
      key: 'lockDirs'
    })
      .map(function parseLockDirectory(value) {
        return parseRepositoryPath({
          option: `${registryFileName} ${label} "lockDirs"`,
          value
        });
      }),
  };
}

/**
 Read and validate the registry file.

 @param text - contents of the registry file

 @returns every releasable crate by name, in file order

 @throws CrateReleaseError when the text is not a JSON object of valid entries, or holds no crate

 @example
 ```ts
 parseCrateRegistry('{"a":{"crateDir":"package/a","scope":"a"}}');
 ```
 */
export function parseCrateRegistry(text: string): CrateRegistry {
  /**
   The text as JSON.
   */
  const parsed = readJson(text);
  if (!isRecord(parsed))
    throw new CrateReleaseError(`${registryFileName} must be an object of crates by name.`);
  if (Object.keys(parsed)
    .length
    === 0)
    throw new CrateReleaseError(`${registryFileName} lists no crate.`);
  return new Map(Object.entries(parsed)
    .map(function parseNamedEntry([name, entry]): readonly [
      string,
      CrateEntry
    ] {
      return [
        parseCrateName({
          option: `${registryFileName} crate name`,
          value: name
        }),
        parseCrateEntry({
          name,
          entry
        }),
      ];
    }));
}

/**
 Read the registry file that sits in the package directory.

 @param packageDirectory - absolute directory of this package

 @param readText - reads a file by absolute path, injected so a test needs no real file

 @returns every releasable crate by name

 @throws CrateReleaseError when the file cannot be read or does not hold a valid registry

 @example
 ```ts
 await loadCrateRegistry({ packageDirectory: import.meta.dirname, readText: (path) => readFile(path, 'utf8') });
 ```
 */
export async function loadCrateRegistry({
  packageDirectory,
  readText,
}: {
  readonly packageDirectory: string;
  readonly readText: (path: string) => Promise<string>;
}): Promise<CrateRegistry> {
  /**
   Absolute path of the registry file.
   */
  const path = join(
    packageDirectory,
    registryFileName
  );
  try {
    return parseCrateRegistry(await readText(path));
  }
  catch (error) {
    if (error instanceof CrateReleaseError)
      throw error;
    throw new CrateReleaseError(
      `${path} could not be read: ${caughtValueText(error)}`,
      { cause: error },
    );
  }
}

/**
 List the crates a release can be started for.

 @param registry - validated registry

 @returns the crate names in file order

 @example
 ```ts
 registryCrateNames(registry);
 ```
 */
export function registryCrateNames(registry: CrateRegistry): readonly string[] {
  return [...registry.keys()];
}

/**
 Find the entry of a crate, or say which crates exist.

 @param registry - validated registry

 @param name - crate name as given by the person releasing

 @returns the entry

 @throws CrateReleaseError listing the known crates and the file to add a crate to when the name is not there

 @example
 ```ts
 findCrate({ registry, name: 'forbidden-strings' });
 ```
 */
export function findCrate({
  registry,
  name,
}: {
  readonly registry: CrateRegistry;
  readonly name: string;
}): CrateEntry {
  /**
   The entry, when the crate is listed.
   */
  const entry = registry.get(name);
  if (entry === undefined)
    throw new CrateReleaseError(
      `No releasable crate is named ${JSON.stringify(name)}; ${registryFileName} lists ${
        registryCrateNames(registry)
          .join(', ')
      }. Add the crate there to release it.`,
    );
  return entry;
}
