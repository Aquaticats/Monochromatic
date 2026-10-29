import { readFile, } from 'node:fs/promises';

import { isStalenessManifest, } from './staleness-guards.ts';
import { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';

import {
  caughtErrorHasCode,
  StalenessManifestPersistenceError,
} from './staleness-manifest-error.ts';
import {
  MANIFEST_VERSION,
  type StalenessEntry,
  type StalenessManifest,
} from './staleness-types.ts';

/**
 Manifest shape accepted by persistence helpers that may serialize caller hooks.
 */
export type PersistableStalenessManifest = {
  /**
   Manifest schema version.
   */
  version: number;

  /**
   Entry map for serialization and merge.
   */
  entries: Readonly<Record<string, StalenessEntry>>;
};

//region Manifest parsing and serialization

/**
 Returns default empty manifest.
 
 @returns Empty manifest for absent cache file.
 
 @example
 ```ts
 const manifest = emptyManifest();
 ```
 */
export function emptyManifest(): StalenessManifest {
  return {
    version: MANIFEST_VERSION,
    entries: {},
  };
}

/**
 Serializes manifest as pretty JSON with trailing newline.
 
 @param manifest - Manifest object to serialize.
 
 @returns Manifest JSON text.
 
 @mutates manifest - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 const text = serializeManifest(manifest);
 ```
 */
export function serializeManifest(manifest: PersistableStalenessManifest,): string {
  return `${JSON.stringify(
    manifest,
    null,
    2,
  )}\n`;
}

/**
 Reads manifest from disk via {@link readManifestContent}, parses it with
 {@link parseManifestJson}, and validates the shape with {@link isStalenessManifest}.
 Returns empty only when file is absent.
 
 @param manifestPath - Absolute manifest path.
 
 @returns Parsed manifest.
 
 @throws {@link StalenessManifestPersistenceError} When manifest exists but is unreadable or invalid.
 
 @example
 ```ts
 const manifest = await readManifestFromDisk('/tmp/manifest.json');
 ```
 */
export async function readManifestFromDisk(manifestPath: string,): Promise<StalenessManifest> {
  /**
   Raw manifest JSON content read from disk.
   */
  const rawManifest = await readManifestContent(manifestPath,);
  /**
   Parsed manifest JSON value.
   */
  const parsedManifest = parseManifestJson({
    manifestPath,
    rawManifest,
  },);

  if (isStalenessManifest(parsedManifest,))
    return parsedManifest;

  throw new StalenessManifestPersistenceError(
    `Invalid staleness manifest schema at ${manifestPath}`,
  );
}

/**
 Reads raw manifest JSON text from disk.
 
 @param manifestPath - Absolute manifest path.
 
 @returns Raw manifest text, or empty manifest JSON when absent.
 
 @throws When manifest exists but cannot be read.
 
 @example
 ```ts
 const raw = await readManifestContent('/tmp/manifest.json');
 ```
 */
async function readManifestContent(manifestPath: string,): Promise<string> {
  try {
    return await readFile(
      manifestPath,
      'utf8',
    );
  }
  catch (readError: unknown) {
    if (caughtErrorHasCode({
      error: readError,
      code: 'ENOENT',
    },))
      return serializeManifest(emptyManifest(),);

    throw new StalenessManifestPersistenceError(
      `Could not read staleness manifest ${manifestPath}: ${caughtErrorMessage(readError,)}`,
      { cause: readError, },
    );
  }
}

/**
 Parses raw manifest JSON text.
 
 @param manifestPath - Absolute manifest path for diagnostics.
 
 @param rawManifest - Raw manifest JSON text.
 
 @returns Parsed JSON value.
 
 @throws When JSON parsing fails.
 
 @example
 ```ts
 const parsed = parseManifestJson({ manifestPath, rawManifest });
 ```
 */
function parseManifestJson(
  {
    manifestPath,
    rawManifest,
  }: {
    readonly manifestPath: string;
    readonly rawManifest: string;
  },
): unknown {
  try {
    return JSON.parse(rawManifest,);
  }
  catch (parseError: unknown) {
    throw new StalenessManifestPersistenceError(
      `Invalid staleness manifest ${manifestPath}: ${caughtErrorMessage(parseError,)}`,
      { cause: parseError, },
    );
  }
}

//endregion Manifest parsing and serialization
