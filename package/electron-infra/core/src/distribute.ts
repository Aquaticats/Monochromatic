/**
 Electron Packager distribution runner for no-Vite staged apps.
 
 @example
 ```ts
 await distributeElectronApp({
   appBundleId: 'dev.example.app',
   appCategoryType: 'public.app-category.developer-tools',
   appCopyright: 'Copyright Example',
   dryRun: true,
   executableName: 'example-app',
   packageRoot: process.cwd(),
   selectedTargetKeys: [],
 });
 ```
 */

import {
  mkdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  DISTRIBUTION_TARGETS,
  targetKey,
  type DistributionTarget,
} from './distribution-targets.js';
import { readElectronVersion, } from './electron-version.js';
import { writeDistributionManifest, } from './distribution-manifest.js';
import { readRootPackageMetadata, } from './package-metadata.js';
import { packageTarget, } from './packager-target.js';

/**
 Options for distributing a staged Electron app.
 
 @example
 ```ts
 const options: ElectronAppDistributionOptions = {
   appBundleId: 'dev.example.app',
   appCategoryType: 'public.app-category.developer-tools',
   appCopyright: 'Copyright Example',
   dryRun: true,
   executableName: 'example-app',
   packageRoot: process.cwd(),
   selectedTargetKeys: [],
 };
 ```
 */
export type ElectronAppDistributionOptions = {
  readonly appBundleId: string;
  readonly appCategoryType: string;
  readonly appCopyright: string;
  readonly appDir?: string;
  readonly distributionDir?: string;
  readonly dryRun: boolean;
  readonly electronVersion?: string;
  readonly executableName: string;
  readonly packageRoot: string;
  readonly selectedTargetKeys: readonly string[];
};

/**
 Applies optional target-key filters to the default distribution matrix.
 
 @param selectedTargetKeys - Requested target keys, or empty for every target.
 
 @returns Distribution targets selected for this run.
 
 @throws Error when a requested target key is unknown.
 
 @example
 ```ts
 selectDistributionTargets({ selectedTargetKeys: ['linux-x64'] });
 ```
 */
export function selectDistributionTargets(
  { selectedTargetKeys, }: { readonly selectedTargetKeys: readonly string[]; },
): readonly DistributionTarget[] {
  if (selectedTargetKeys.length === 0)
    return DISTRIBUTION_TARGETS;

  /**
   Known target keys available in this package.
   */
  const knownKeys = new Set(DISTRIBUTION_TARGETS.map(function toTargetKey(target,) {
    return targetKey({ target, },);
  },),);

  /**
   Unknown target keys requested by the caller.
   */
  const unknownKeys = selectedTargetKeys.filter(function isUnknownKey(key,) {
    return !knownKeys.has(key,);
  },);

  if (unknownKeys.length > 0)
    throw new Error(`Unknown distribution target(s): ${unknownKeys.join(', ')}`,
    );

  return DISTRIBUTION_TARGETS.filter(function isSelectedTarget(target,) {
    return selectedTargetKeys.includes(targetKey({ target, },),);
  },);
}

/**
 Creates all requested distribution bundles or records the dry-run manifest.
 
 @param appBundleId - Stable Electron application bundle identifier.
 
 @param appCategoryType - macOS Finder category.
 
 @param appCopyright - Copyright string embedded into bundles.
 
 @param appDir - Optional staged app directory, defaulting to `dist/app`.
 
 @param distributionDir - Optional output directory, defaulting to `dist/distribution`.
 
 @param dryRun - Whether to skip Packager and only write the target manifest.
 
 @param electronVersion - Optional Electron version override.
 
 @param executableName - Executable basename across target platforms.
 
 @param packageRoot - Directory containing package manifest.
 
 @param selectedTargetKeys - Optional target keys to package.
 
 @example
 ```ts
 await distributeElectronApp({ appBundleId: 'dev.example.app', appCategoryType: 'public.app-category.developer-tools', appCopyright: 'Copyright Example', dryRun: true, executableName: 'example-app', packageRoot: process.cwd(), selectedTargetKeys: [] });
 ```
 */
export async function distributeElectronApp(
  {
    appBundleId,
    appCategoryType,
    appCopyright,
    appDir,
    distributionDir,
    dryRun,
    electronVersion,
    executableName,
    packageRoot,
    selectedTargetKeys,
  }: ElectronAppDistributionOptions,
): Promise<void> {
  /**
   Staged app directory produced by a build task.
   */
  const resolvedAppDir = appDir ?? join(
    packageRoot,
    'dist',
    'app',
  );

  /**
   Directory receiving Packager output bundles.
   */
  const resolvedDistributionDir = distributionDir ?? join(
    packageRoot,
    'dist',
    'distribution',
  );

  /**
   Root package metadata for product name and version.
   */
  const packageMetadata = await readRootPackageMetadata({ packageRoot, },);

  /**
   Electron version installed for Packager downloads.
   */
  const resolvedElectronVersion = electronVersion ?? readElectronVersion();

  /**
   Distribution targets selected for this run.
   */
  const targets = selectDistributionTargets({ selectedTargetKeys, },);

  await rm(
    resolvedDistributionDir,
    {
      recursive: true,
      force: true,
    },
  );
  await mkdir(
    resolvedDistributionDir,
    { recursive: true, },
  );
  await writeDistributionManifest({
    appBundleId,
    electronVersion: resolvedElectronVersion,
    manifestPath: join(
      resolvedDistributionDir,
      'manifest.json',
    ),
    targets,
  },);

  if (dryRun)
    return;

  await Promise.all(targets.map(function packageSelectedTarget(target,): Promise<void> {
    return packageTarget({
      appBundleId,
      appCategoryType,
      appCopyright,
      appDir: resolvedAppDir,
      appVersion: packageMetadata.version,
      distributionDir: resolvedDistributionDir,
      electronVersion: resolvedElectronVersion,
      executableName,
      productName: packageMetadata.productName,
      target,
    },);
  },),);
}
