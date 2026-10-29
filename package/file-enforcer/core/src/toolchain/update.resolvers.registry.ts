/**
 Version-only package registry resolvers: crates.io, the npm registry, and PyPI.

 @module
 */

import type { ToolPlanEntry, } from './types.ts';
import {
  type Fetcher,
  parseJson,
  stringField,
  stripPrefix,
  type ToolResolution,
} from './update.resolvers.shared.ts';

/**
 Resolves a `cargo:` tool from crates.io's max stable version.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version with the recorded (empty) rows

 @throws Error when the crate is unknown

 @example
 ```ts
 await resolveCargo({ entry, fetcher, },); // { version: '9.9.9', artifacts: {} }
 ```
 */
export async function resolveCargo(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Crate name behind the entry.
   */
  const crate = entry.packageId
    ?? stripPrefix({
      text: entry.id,
      prefix: 'cargo:',
    },);
  /**
   Crate document from the registry.
   */
  const doc = parseJson({
    response: await fetcher(`https://crates.io/api/v1/crates/${crate}`,),
    label: `crates.io/${crate}`,
  },);
  /**
   Max stable version the registry reports.
   */
  const version = stringField({
    doc,
    path: [
      'crate',
      'max_stable_version',
    ],
  },);
  if (version === '')
    throw new Error(`crates.io reports no version for ${crate}`,);
  return {
    version,
    artifacts: {},
  };
}

/**
 Resolves an `mpm-pnpm` tool from the npm registry's dist-tags: the `latest`
 tag for the floating track (and for pins, whose newer-availability note
 reads it), the named channel's tag for `{channel}` requests (`next`, ...).

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns bare registry version with the recorded (empty) rows

 @throws Error when the registry reports no matching dist-tag

 @example
 ```ts
 await resolveNpm({ entry, fetcher, },); // { version: '3.2.1', artifacts: {} }
 ```
 */
export async function resolveNpm(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Package name behind the entry (the scoped names keep their slash).
   */
  const name = entry.packageId
    ?? stripPrefix({
      text: entry.id,
      prefix: 'npm:',
    },);
  /**
   Dist-tag the request tracks (`latest`, `next`, ...).
   */
  const tag = 'channel' in entry.request
    ? entry.request
      .channel
    : 'latest';
  /**
   Packument from the registry.
   */
  const doc = parseJson({
    response: await fetcher(`https://registry.npmjs.org/${name}`,),
    label: `registry.npmjs.org/${name}`,
  },);
  /**
   Version the dist-tag points at.
   */
  const version = stringField({
    doc,
    path: [
      'dist-tags',
      tag,
    ],
  },);
  if (version === '') {
    throw new Error(`registry.npmjs.org reports no ${
      tag
    } dist-tag for ${
      name
    }`,);
  }
  return {
    version,
    artifacts: {},
  };
}

/**
 Resolves an `mpm-uvx` tool from PyPI's project metadata.

 @param entry - plan entry to resolve

 @param fetcher - injected GET

 @returns new version with the recorded rows (empty for tools)

 @throws Error when the project is unknown

 @example
 ```ts
 await resolvePypi({ entry, fetcher, },); // { version: '0.5.0', artifacts: {} }
 ```
 */
export async function resolvePypi(
  {
    entry,
    fetcher,
  }: {
    readonly entry: ToolPlanEntry;
    readonly fetcher: Fetcher;
  },
): Promise<ToolResolution> {
  /**
   Project name behind the entry.
   */
  const name = entry.packageId
    ?? stripPrefix({
      text: entry.id,
      prefix: 'pipx:',
    },);
  /**
   Project metadata from PyPI.
   */
  const doc = parseJson({
    response: await fetcher(`https://pypi.org/pypi/${name}/json`,),
    label: `pypi.org/${name}`,
  },);
  /**
   Latest version PyPI reports.
   */
  const version = stringField({
    doc,
    path: [
      'info',
      'version',
    ],
  },);
  if (version === '')
    throw new Error(`pypi.org reports no version for ${name}`,);
  return {
    version,
    artifacts: entry.id === 'mpm'
      ? entry.artifacts
      : {},
  };
}
