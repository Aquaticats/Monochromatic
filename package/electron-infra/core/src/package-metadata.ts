/**
 Package metadata readers shared by Electron staging and distribution.
 
 @example
 ```ts
 const metadata = await readRootPackageMetadata({ packageRoot: process.cwd() });
 ```
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 Minimal package metadata used by staged Electron apps.
 
 @example
 ```ts
 const metadata: RootPackageMetadata = { name: 'x', productName: 'X', version: '0.0.1' };
 ```
 */
export type RootPackageMetadata = {
  readonly description?: string;
  readonly license?: string;
  readonly name: string;
  readonly productName: string;
  readonly version: string;
};

/**
 Runtime manifest fields Electron reads from the staged app directory.
 
 @example
 ```ts
 const metadata: StagedPackageMetadata = { name: 'x', productName: 'X', version: '0.0.1', main: 'main.mjs', type: 'module' };
 ```
 */
export type StagedPackageMetadata = RootPackageMetadata & {
  readonly main: string;
  readonly type: 'module';
};

/**
 Asserts that parsed JSON contains package metadata required by Electron infra.
 
 @param value - Parsed JSON value from package manifest.
 
 @returns Narrowed package metadata.
 
 @throws Error when required package metadata is missing.
 
 @example
 ```ts
 parseRootPackageMetadata({ value: { name: 'x', productName: 'X', version: '0.0.1' } });
 ```
 */
export function parseRootPackageMetadata(
  { value, }: { readonly value: unknown; },
): RootPackageMetadata {
  if (
    ((typeof value) !== 'object')
      || (value === null)
      || (!('name' in value))
      || (!('productName' in value))
      || (!('version' in value))
  )
    throw new Error('Package manifest is missing name, productName, or version.',);

  /**
   Manifest after structural checks have proven required keys exist.
   */
  const manifest = value as {
    readonly description?: unknown;
    readonly license?: unknown;
    readonly name: unknown;
    readonly productName: unknown;
    readonly version: unknown;
  };

  if (
    ((typeof manifest.name) !== 'string')
      || ((typeof manifest.productName) !== 'string')
      || ((typeof manifest.version) !== 'string')
  )
    throw new Error('Package manifest name, productName, and version must be strings.',);

  /**
   Required package metadata shared by root and staged manifests.
   */
  const metadata: RootPackageMetadata = {
    name: manifest.name,
    productName: manifest.productName,
    version: manifest.version,
  };

  return {
    ...metadata,
    ...((typeof manifest.description) === 'string'
      ? { description: manifest.description, }
      : {}),
    ...((typeof manifest.license) === 'string'
      ? { license: manifest.license, }
      : {}),
  };
}

/**
 Reads root package metadata from `package.json`.
 
 @param packageRoot - Directory containing package manifest.
 
 @returns Package metadata required for staged app and distribution manifests.
 
 @example
 ```ts
 await readRootPackageMetadata({ packageRoot: process.cwd() });
 ```
 */
export async function readRootPackageMetadata(
  { packageRoot, }: { readonly packageRoot: string; },
): Promise<RootPackageMetadata> {
  /**
   Raw package manifest text.
   */
  const manifestText = await readFile(
    join(
      packageRoot,
      'package.json',
    ),
    'utf8',
  );
  return parseRootPackageMetadata({ value: JSON.parse(manifestText,), },);
}
