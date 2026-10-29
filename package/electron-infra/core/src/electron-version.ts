/**
 Reads installed Electron package metadata for distribution and tests.
 
 @example
 ```ts
 const version = readElectronVersion();
 ```
 */

import { createRequire, } from 'node:module';

/**
 CommonJS require rooted at this helper for package metadata reads.
 
 @example
 ```ts
 console.log(typeof require);
 ```
 */
const require = createRequire(import.meta.url,);

/**
 Shape read from `electron/package.json`.
 
 @example
 ```ts
 const metadata: ElectronPackageMetadata = { version: '42.0.0' };
 ```
 */
type ElectronPackageMetadata = {
  readonly version: string;
};

/**
 Throws when `value` is not Electron's package metadata shape.
 
 @param value - Package metadata loaded through `createRequire`.
 
 @returns Electron package metadata with version.
 
 @example
 ```ts
 parseElectronPackageMetadata({ value: { version: '42.0.0' } });
 ```
 */
function parseElectronPackageMetadata(
  { value, }: { readonly value: unknown; },
): ElectronPackageMetadata {
  if (
    ((typeof value) !== 'object')
      || (value === null)
      || (!('version' in value))
  )
    throw new Error('electron/package.json did not expose a version.',);

  /**
   Metadata after structural narrowing.
   */
  const metadata = value as { readonly version: unknown; };

  if ((typeof metadata.version) !== 'string')
    throw new Error('electron/package.json version must be a string.',);

  return { version: metadata.version, };
}

/**
 Reads the installed Electron version used by Packager downloads.
 
 @returns Electron version string from installed package metadata.
 
 @example
 ```ts
 const version = readElectronVersion();
 ```
 */
export function readElectronVersion(): string {
  return parseElectronPackageMetadata({
    value: require('electron/package.json',) as unknown,
  },)
    .version;
}
