import {
  glob,
  readFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  fieldOf,
  isJsonRecord,
} from './json-shape.ts';

//region Workspace and registry reads

/**
 Repository root, four levels above this file.
 */
export const REPOSITORY_ROOT: string = resolve(
  import.meta.dirname,
  '../../../..',
);

/**
 HTTP status pnpr returns for a package that has never been published.
 */
const HTTP_NOT_FOUND = 404;

/**
 Workspace package facts the publish run reads.
 */
export type WorkspaceEntry = {
  /**
   Package name.
   */
  readonly name: string;
  /**
   Repository-relative directory.
   */
  readonly directory: string;
  /**
   Parsed manifest.
   */
  readonly manifest: Readonly<Record<string, unknown>>;
};

/**
 Versions already in the registry for one package.
 */
export type PublishedVersions = {
  /**
   Every published version string.
   */
  readonly versions: ReadonlySet<string>;
  /**
   Current `latest` dist-tag, absent when none is set.
   */
  readonly latest?: string;
};

/**
 Reads every workspace manifest under a repository root.

 @param rootDir - Repository root whose two-level package directories hold the manifests.

 @returns Workspace packages in directory-listing order, one per manifest.

 @throws Error when a manifest is not an object with a string name.

 @example
 ```ts
 (await readWorkspaceEntries({ rootDir: REPOSITORY_ROOT })).map((entry) => entry.name);
 ```
 */
export async function readWorkspaceEntries(
  { rootDir, }: { readonly rootDir: string; },
): Promise<readonly WorkspaceEntry[]> {
  /**
   Manifest paths relative to the repository root.
   */
  const manifestPaths = await Array.fromAsync(glob(
    'package/*/*/package.json',
    { cwd: rootDir, },
  ),);
  return Promise.all(manifestPaths.map(async function readEntry(manifestPath,): Promise<WorkspaceEntry> {
    /**
     Parsed manifest, untrusted until narrowed.
     */
    const manifest: unknown = JSON.parse(await readFile(
      join(
        rootDir,
        manifestPath,
      ),
      'utf8',
    ),);
    if ((!isJsonRecord(manifest,)) || ((typeof manifest.name) !== 'string'))
      throw new Error(`${manifestPath} is not a manifest with a string name`,);
    return {
      name: manifest.name,
      directory: manifestPath.slice(
        0,
        -'/package.json'.length,
      ),
      manifest,
    };
  },),);
}

/**
 Reads every workspace manifest of this repository keyed by package name.

 @returns Workspace packages by name.

 @throws Error when a manifest is not an object with a string name.

 @example
 ```ts
 (await readWorkspace()).get('@monochromatic-dev/or-throw-core');
 ```
 */
export async function readWorkspace(): Promise<ReadonlyMap<string, WorkspaceEntry>> {
  /**
   Parsed workspace entries.
   */
  const entries = await readWorkspaceEntries({ rootDir: REPOSITORY_ROOT, },);
  return new Map(entries.map(function toPair(entry,) {
    return [
      entry.name,
      entry,
    ];
  },),);
}

/**
 Fetches published versions anonymously, since workload credentials cannot read.

 @param origin - Registry origin.

 @param name - Scoped package name.

 @returns Published versions and `latest`; a package never published has no versions and no `latest`.

 @throws Error on any response other than success or 404.

 @example
 ```ts
 await fetchPublishedVersions({ origin: 'https://pnpr.c.aquati.cat', name: '@monochromatic-dev/or-throw-core' });
 ```
 */
export async function fetchPublishedVersions(
  {
    origin,
    name,
  }: {
    readonly origin: string;
    readonly name: string;
  },
): Promise<PublishedVersions> {
  /**
   Anonymous packument response.
   */
  const response = await fetch(
    `${origin}/npm/${name.replace(
      '/',
      '%2f',
    )}`,
    { headers: { accept: 'application/json', }, },
  );
  if (response.status === HTTP_NOT_FOUND)
    return { versions: new Set(), };
  if (!response.ok)
    throw new Error(`packument read for ${name} failed with HTTP ${response.status}`,);
  /**
   Packument body, untrusted until narrowed.
   */
  const packument: unknown = await response.json();
  /**
   Version map from the packument.
   */
  const versions = fieldOf({
    value: packument,
    key: 'versions',
  },);
  /**
   `latest` dist-tag value.
   */
  const latest = fieldOf({
    value: fieldOf({
      value: packument,
      key: 'dist-tags',
    },),
    key: 'latest',
  },);
  return {
    versions: new Set(isJsonRecord(versions,) ? Object.keys(versions,) : [],),
    ...((typeof latest) === 'string' ? { latest, } : {}),
  };
}

//endregion Workspace and registry reads
