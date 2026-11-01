/**
 Package version reported by `search-fetch --version`.

 The version is read from the package's own `package.json` at run time, so the
 printed value cannot drift from the manifest.

 @module
 */

import { readFile, } from 'node:fs/promises';

import { SearchFetchCliConfigurationError, } from './errors.ts';

//region Constants

/**
 Manifest location relative to the built entry under `dist/final/node`.
 */
const packageJsonUrl = new URL(
  '../../../package.json',
  import.meta.url,
);

//endregion Constants

//region Public API

/**
 Read the `@monochromatic-dev/search-fetch-cli` package version.

 @param manifestUrl - manifest location to read, defaulting to this package's own manifest

 @returns version string from the package manifest

 @throws SearchFetchCliConfigurationError when the manifest cannot be read or carries no string version

 @example
 ```ts
 await packageVersion();
 ```
 */
export async function packageVersion(
  {
    manifestUrl = packageJsonUrl,
  }: {
    readonly manifestUrl?: URL;
  } = {},
): Promise<string> {
  /**
   Manifest text read from the manifest location.
   */
  const manifestText = await readManifestText({ manifestUrl, },);
  /**
   Parsed manifest value.
   */
  const manifestValue = parseManifestText({
    manifestText,
    manifestUrl,
  },);
  if (!hasStringVersion(manifestValue,))
    throw new SearchFetchCliConfigurationError({
      message: `Package manifest at ${manifestUrl.pathname} has no string version field`,
    },);
  return manifestValue.version;
}

//endregion Public API

//region Helpers

/**
 Read the manifest text, reporting a read failure as a configuration error.

 @param manifestUrl - manifest location to read

 @returns manifest text

 @throws SearchFetchCliConfigurationError when the manifest cannot be read

 @example
 ```ts
 await readManifestText({ manifestUrl: new URL('../package.json', import.meta.url) });
 ```
 */
async function readManifestText(
  { manifestUrl, }: {
    readonly manifestUrl: URL;
  },
): Promise<string> {
  try {
    return await readFile(
      manifestUrl,
      'utf8',
    );
  }
  catch (error: unknown) {
    throw new SearchFetchCliConfigurationError({
      message: `Could not read the package manifest at ${manifestUrl.pathname} for --version`,
      cause: error,
    },);
  }
}

/**
 Parse the manifest text, reporting malformed JSON as a configuration error.

 @param manifestText - manifest text to parse

 @param manifestUrl - manifest location named in the failure message

 @returns parsed manifest value

 @throws SearchFetchCliConfigurationError when the manifest is not JSON

 @example
 ```ts
 parseManifestText({ manifestText: '{"version":"0.0.1"}', manifestUrl });
 ```
 */
function parseManifestText(
  {
    manifestText,
    manifestUrl,
  }: {
    readonly manifestText: string;
    readonly manifestUrl: URL;
  },
): unknown {
  try {
    return JSON.parse(manifestText,);
  }
  catch (error: unknown) {
    throw new SearchFetchCliConfigurationError({
      message: `Package manifest at ${manifestUrl.pathname} is not JSON`,
      cause: error,
    },);
  }
}

/**
 Report whether a parsed manifest carries a string version field.

 @param value - parsed manifest value

 @returns whether value is a manifest record with a string version

 @example
 ```ts
 hasStringVersion({ version: '0.0.1' });
 ```
 */
function hasStringVersion(value: unknown,): value is { readonly version: string; } {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('version' in value)
    && ((typeof value.version) === 'string');
}

//endregion Helpers
