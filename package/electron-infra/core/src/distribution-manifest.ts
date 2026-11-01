/**
 Distribution manifest writer for Electron Packager runs.
 
 @example
 ```ts
 await writeDistributionManifest({ appBundleId: 'dev.example.app', electronVersion: '42.0.0', manifestPath: '/tmp/manifest.json', targets: [] });
 ```
 */

import { writeFile, } from 'node:fs/promises';

import {
  targetKey,
  type DistributionTarget,
} from './distribution-targets.js';

/**
 Writes a machine-readable manifest for dry-run and completed distribution runs.
 
 @param appBundleId - Stable Electron application bundle identifier.
 
 @param electronVersion - Electron version used for target bundles.
 
 @param manifestPath - JSON path receiving the manifest.
 
 @param targets - Distribution targets represented in the manifest.
 
 @example
 ```ts
 await writeDistributionManifest({ appBundleId: 'dev.example.app', electronVersion: '42.0.0', manifestPath: '/tmp/manifest.json', targets: [] });
 ```
 */
export async function writeDistributionManifest(
  {
    appBundleId,
    electronVersion,
    manifestPath,
    targets,
  }: {
    readonly appBundleId: string;
    readonly electronVersion: string;
    readonly manifestPath: string;
    readonly targets: readonly DistributionTarget[];
  },
): Promise<void> {
  await writeFile(
    manifestPath,
    `${JSON.stringify(
      {
        appBundleId,
        electronVersion,
        targets: targets.map(function toManifestTarget(target,) {
          return {
            ...target,
            key: targetKey({ target, },),
          };
        },),
      },
      null,
      2,
    )}\n`,
    'utf8',
  );
}
