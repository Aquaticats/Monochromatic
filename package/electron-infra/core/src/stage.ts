/**
 Stages no-Vite Electron app assets and runtime package metadata.
 
 @example
 ```ts
 await stageElectronApp({ packageRoot: process.cwd(), staticAssets: ['index.html'] });
 ```
 */

import {
  access,
  cp,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  readRootPackageMetadata,
  type StagedPackageMetadata,
} from './package-metadata.js';

/**
 Static file copy mapping from source tree into staged app.
 
 @example
 ```ts
 const asset: StaticAssetMapping = { source: 'index.html', destination: 'index.html' };
 ```
 */
export type StaticAssetMapping = {
  readonly destination: string;
  readonly source: string;
};

/**
 Options for staging a no-Vite Electron app.
 
 @example
 ```ts
 const options: ElectronAppStageOptions = { packageRoot: process.cwd(), staticAssets: ['index.html'] };
 ```
 */
export type ElectronAppStageOptions = {
  readonly appOutDir?: string;
  readonly licenseDir?: string;
  readonly main?: string;
  readonly packageRoot: string;
  readonly sourceRoot?: string;
  readonly staticAssets: readonly (string | StaticAssetMapping)[];
};

/**
 Returns copy mapping for a static asset shorthand or explicit mapping.
 
 @param asset - Asset shorthand or mapping.
 
 @returns Source and destination relative paths.
 
 @example
 ```ts
 normalizeStaticAsset({ asset: 'index.html' });
 ```
 */
function normalizeStaticAsset(
  { asset, }: { readonly asset: string | StaticAssetMapping; },
): StaticAssetMapping {
  if ((typeof asset) === 'string')
    return {
      destination: asset,
      source: asset,
    };

  return asset;
}

/**
 Checks whether an unknown error is an `ENOENT` filesystem error.
 
 @param error - Caught value from a filesystem operation.
 
 @returns Whether the error reports an absent path.
 
 @example
 ```ts
 isMissingPathError({ error: new Error('x') });
 ```
 */
function isMissingPathError({ error, }: { readonly error: unknown; },): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && (error.code === 'ENOENT');
}

/**
 Copies optional package license texts into the staged app.
 
 @param appOutDir - Staged app output directory.
 
 @param licenseDir - Source LICENSES directory.
 
 @example
 ```ts
 await copyLicenseTexts({ appOutDir: '/tmp/app', licenseDir: '/tmp/LICENSES' });
 ```
 */
async function copyLicenseTexts(
  {
    appOutDir,
    licenseDir,
  }: {
    readonly appOutDir: string;
    readonly licenseDir: string;
  },
): Promise<void> {
  try {
    await access(licenseDir,);
  }
  catch (error: unknown) {
    if (isMissingPathError({ error, },))
      return;

    throw error;
  }

  await cp(
    licenseDir,
    join(
      appOutDir,
      'LICENSES',
    ),
    {
      force: true,
      recursive: true,
    },
  );
}

/**
 Writes static assets and runtime manifest into a staged Electron app directory.
 
 @param appOutDir - Optional staged app output directory, defaulting to `dist/app`.
 
 @param licenseDir - Optional source license directory, defaulting to `LICENSES`.
 
 @param main - Staged Electron entry file, defaulting to `main.mjs`.
 
 @param packageRoot - Directory containing package manifest.
 
 @param sourceRoot - Optional source asset directory, defaulting to `src`.
 
 @param staticAssets - Static files copied from source root into staged app.
 
 @example
 ```ts
 await stageElectronApp({ packageRoot: process.cwd(), staticAssets: ['index.html', 'styles.css'] });
 ```
 */
export async function stageElectronApp(
  {
    appOutDir,
    licenseDir,
    main = 'main.mjs',
    packageRoot,
    sourceRoot,
    staticAssets,
  }: ElectronAppStageOptions,
): Promise<void> {
  /**
   Staged Electron app directory consumed by Electron and Packager.
   */
  const resolvedAppOutDir = appOutDir ?? join(
    packageRoot,
    'dist',
    'app',
  );

  /**
   Source directory containing renderer assets.
   */
  const resolvedSourceRoot = sourceRoot ?? join(
    packageRoot,
    'src',
  );

  await mkdir(
    resolvedAppOutDir,
    { recursive: true, },
  );
  await Promise.all(staticAssets.map(async function copyStaticAsset(asset,): Promise<void> {
    /**
     Normalized source and destination relative paths.
     */
    const mapping = normalizeStaticAsset({ asset, },);

    await cp(
      join(
        resolvedSourceRoot,
        mapping.source,
      ),
      join(
        resolvedAppOutDir,
        mapping.destination,
      ),
    );
  },),);
  await copyLicenseTexts({
    appOutDir: resolvedAppOutDir,
    licenseDir: licenseDir ?? join(
      packageRoot,
      'LICENSES',
    ),
  },);

  /**
   Runtime manifest consumed by Electron inside the staged app directory.
   */
  const stagedMetadata: StagedPackageMetadata = {
    ...await readRootPackageMetadata({ packageRoot, },),
    main,
    type: 'module',
  };

  await writeFile(
    join(
      resolvedAppOutDir,
      'package.json',
    ),
    `${JSON.stringify(
      stagedMetadata,
      null,
      2,
    )}\n`,
    'utf8',
  );
}
