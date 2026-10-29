import type { Dirent, } from 'node:fs';
import { readdir, } from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  basename,
  join,
} from 'node:path';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { caughtErrorHasCode, } from '../io/error.ts';
import { trackGlob, } from '../tracker.ts';

//region Shapes and sentinels: product directories and absence markers

/**
 Metadata for a JetBrains product config directory discovered under the config root.
 */
export type JetbrainsOptionsDirectory = {
  readonly optionsDirectory: string;
  readonly productDirectory: string;
  readonly versionParts: readonly number[];
};

/**
 Sentinel for a product directory that is not a requested, version-named product.
 */
export const NOT_A_MATCHING_PRODUCT: unique symbol = Symbol('file-enforcer/jetbrains: directory is not a requested versioned product',);

/**
 Sentinel returned when no matching JetBrains product options directory exists.
 */
export const NO_JETBRAINS_OPTIONS_DIRECTORY: unique symbol = Symbol('file-enforcer/jetbrains: no matching product options directory found',);

//endregion Shapes and sentinels

//region Version parsing: compare JetBrains product config directories numerically

/**
 Checks whether text consists only of decimal digits.
 
 @param value - Text to inspect.
 
 @returns Whether every character is a decimal digit.
 
 @example
 ```ts
 isDecimalDigits({ value: '2026' });
 ```
 */
function isDecimalDigits({ value, }: { readonly value: string; },): boolean {
  if (value.length === 0) return false;
  for (const char of value) {
    if ((char < '0') || (char > '9')) return false;
  }
  return true;
}

/**
 Parses a numeric version tuple from a product directory name, checking each
 dot-separated segment with {@link isDecimalDigits}.
 
 @param productName - Directory name under the JetBrains config root.
 
 @param prefixes - Product name prefixes whose numeric suffix is a version.
 
 @returns Parsed version parts, or {@link NOT_A_MATCHING_PRODUCT} when none match.
 
 @example
 ```ts
 parseVersionParts({ productName: 'IntelliJIdea2026.2', prefixes: ['IntelliJIdea'] });
 ```
 */
export function parseVersionParts(
  {
    productName,
    prefixes,
  }: {
    readonly prefixes: readonly string[];
    readonly productName: string
  },
): readonly number[] | typeof NOT_A_MATCHING_PRODUCT {
  /**
   Matching product prefix, if any.
   */
  const prefix = prefixes.find(function prefixMatches(candidate,): boolean {
    return productName.startsWith(candidate,);
  },);
  if (prefix === undefined) return NOT_A_MATCHING_PRODUCT;
  /**
   Dotted version suffix after the prefix.
   */
  const suffix = productName.slice(prefix.length,);
  /**
   Dot-separated version segments.
   */
  const parts = suffix.split('.',);
  if (parts.length === 0) return NOT_A_MATCHING_PRODUCT;
  /**
   Numeric value of each segment, NaN for non-numeric segments.
   */
  const parsedParts = parts.map(function parseVersionPart(part,): number {
    return isDecimalDigits({ value: part, }) ? Number(part,) : Number.NaN;
  },);
  if (parsedParts.some(function isNotANumber(part,): boolean {
    return Number.isNaN(part,);
  },)) {
    return NOT_A_MATCHING_PRODUCT;
  }
  return parsedParts;
}

/**
 Compares two version tuples element by element.
 
 @param left - Left version tuple.
 
 @param right - Right version tuple.
 
 @returns Positive when left is newer, negative when right is newer, zero when equal.
 
 @example
 ```ts
 compareVersionParts({ left: [2026, 2], right: [2026, 1] });
 ```
 */
export function compareVersionParts(
  {
    left,
    right,
  }: {
    readonly left: readonly number[];
    readonly right: readonly number[]
  },
): number {
  /**
   Length of the longer tuple, padding the shorter with zeros.
   */
  const maxLength = Math.max(
    left.length,
    right.length,
  );
  for (let cursorIndex = 0; cursorIndex < maxLength; cursorIndex += 1) {
    /**
     Left segment at this index, or zero when absent.
     */
    const leftPart = left[cursorIndex] ?? 0;
    /**
     Right segment at this index, or zero when absent.
     */
    const rightPart = right[cursorIndex] ?? 0;
    if (leftPart !== rightPart) return leftPart - rightPart;
  }
  return 0;
}

/**
 Chooses the newer of an accumulator and a candidate options directory,
 ranking via {@link compareVersionParts}.
 
 @param current - Current accumulator, or the no-directory sentinel.
 
 @param candidate - Candidate being considered.
 
 @returns Newer options directory.
 
 @example
 ```ts
 keepLatestOptions({ current: NO_JETBRAINS_OPTIONS_DIRECTORY, candidate });
 ```
 */
function keepLatestOptions(
  {
    current,
    candidate,
  }: {
    readonly candidate: JetbrainsOptionsDirectory;
    readonly current: JetbrainsOptionsDirectory | typeof NO_JETBRAINS_OPTIONS_DIRECTORY;
  },
): JetbrainsOptionsDirectory {
  if (current === NO_JETBRAINS_OPTIONS_DIRECTORY) return candidate;
  /**
   Version ordering between candidate and current.
   */
  const comparison = compareVersionParts({
    left: candidate.versionParts,
    right: current.versionParts,
  },);
  if (comparison > 0) return candidate;
  if ((comparison === 0) && (candidate.productDirectory > current.productDirectory)) return candidate;
  return current;
}

//endregion Version parsing

//region Discovery: track product directories and pick the latest version

/**
 Converts one product directory path into an options candidate, parsing its
 version via {@link parseVersionParts}.
 
 @param productDirectory - Candidate JetBrains product directory.
 
 @param prefixes - Product name prefixes recognized as the requested product.
 
 @returns Options candidate, or {@link NOT_A_MATCHING_PRODUCT} for non-matching paths.
 
 @example
 ```ts
 optionsCandidate({ productDirectory, prefixes: ['IntelliJIdea'] });
 ```
 */
function optionsCandidate(
  {
    productDirectory,
    prefixes,
  }: {
    readonly prefixes: readonly string[];
    readonly productDirectory: string
  },
): JetbrainsOptionsDirectory | typeof NOT_A_MATCHING_PRODUCT {
  /**
   Parsed version parts, or the non-matching sentinel.
   */
  const versionParts = parseVersionParts({
    productName: basename(productDirectory,),
    prefixes,
  },);
  if (versionParts === NOT_A_MATCHING_PRODUCT) return NOT_A_MATCHING_PRODUCT;
  return {
    optionsDirectory: join(
      productDirectory,
      'options',
    ),
    productDirectory,
    versionParts,
  };
}

/**
 Lists product directories under the JetBrains config root and records them
 via {@link trackGlob} as a glob dependency so file-enforcer staleness
 tracking reruns when installs change.
 
 @param jetBrainsConfigDirectory - JetBrains config root directory.
 
 @returns Product directory paths found directly under the config root.
 
 @throws When the JetBrains config root exists but cannot be listed; absence
 itself is detected via {@link caughtErrorHasCode} and treated as no products.
 
 @example
 ```ts
 await trackedProductDirectories({ jetBrainsConfigDirectory });
 ```
 */
async function trackedProductDirectories(
  { jetBrainsConfigDirectory, }: { readonly jetBrainsConfigDirectory: string; },
): Promise<readonly string[]> {
  /**
   Glob pattern recorded for staleness tracking of the config root.
   */
  const pattern = join(
    jetBrainsConfigDirectory,
    '*',
  );
  try {
    /**
     Directory entries directly under the config root.
     */
    const entries = await readdir(
      jetBrainsConfigDirectory,
      { withFileTypes: true, },
    );
    /**
     Absolute paths of the product subdirectories.
     */
    const paths = entries
      .filter(function keepDirectory(entry: ForeignBorrowed<Dirent>,): boolean {
        return entry.isDirectory();
      },)
      .map(function toProductDirectory(entry: ForeignBorrowed<Dirent>,): string {
        return join(
          jetBrainsConfigDirectory,
          entry.name,
        );
      },);
    trackGlob({
      pattern,
      paths,
    },);
    return paths;
  }
  catch (directoryReadError: unknown) {
    if (!caughtErrorHasCode({
      error: directoryReadError,
      code: 'ENOENT',
    },))
      throw directoryReadError;

    trackGlob({
      pattern,
      paths: [],
    },);
    return [];
  }
}

/**
 Finds the latest JetBrains product options directory matching the given prefixes,
 by listing candidates via {@link trackedProductDirectories}, filtering them to
 matching products via {@link optionsCandidate}, and reducing with
 {@link keepLatestOptions}. Whether the chosen options directory actually contains
 the desired files is left to the caller, so a newer install without those files
 yields a clear miss rather than mutating an older install.
 
 @param productPrefixes - Product name prefixes (for example `IntelliJIdea`, `IdeaIC`).
 
 @returns Latest matching options directory, or {@link NO_JETBRAINS_OPTIONS_DIRECTORY}.
 
 @example
 ```ts
 await latestJetbrainsOptionsDirectory({ productPrefixes: ['IntelliJIdea', 'IdeaIC'] });
 ```
 */
export async function latestJetbrainsOptionsDirectory(
  { productPrefixes, }: { readonly productPrefixes: readonly string[]; },
): Promise<JetbrainsOptionsDirectory | typeof NO_JETBRAINS_OPTIONS_DIRECTORY> {
  /**
   JetBrains config root derived from XDG or the home directory.
   */
  const jetBrainsConfigDirectory = join(
    process.env
      .XDG_CONFIG_HOME
      ?? join(
        homedir(),
        '.config',
      ),
    'JetBrains',
  );
  /**
   Product directories discovered under the config root.
   */
  const productDirectories = await trackedProductDirectories({ jetBrainsConfigDirectory, },);
  /**
   Matching product options candidates.
   */
  const candidates = productDirectories
    .map(function toCandidate(productDirectory,): JetbrainsOptionsDirectory | typeof NOT_A_MATCHING_PRODUCT {
      return optionsCandidate({
        productDirectory,
        prefixes: productPrefixes,
      },);
    },)
    .filter(function keepCandidate(candidate,): candidate is JetbrainsOptionsDirectory {
      return candidate !== NOT_A_MATCHING_PRODUCT;
    },);
  return candidates.reduce<JetbrainsOptionsDirectory | typeof NO_JETBRAINS_OPTIONS_DIRECTORY>(
    function pickLatest(
      current,
      candidate,
    ): JetbrainsOptionsDirectory {
      return keepLatestOptions({
        current,
        candidate,
      },);
    },
    NO_JETBRAINS_OPTIONS_DIRECTORY,
  );
}

//endregion Discovery
