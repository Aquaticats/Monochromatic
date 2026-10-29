/**
 Single-target Electron Packager wrapper.
 
 @example
 ```ts
 await packageTarget({ appBundleId: 'dev.example.app', appCategoryType: 'public.app-category.developer-tools', appCopyright: 'Copyright Example', appDir: '/tmp/app', appVersion: '0.0.1', distributionDir: '/tmp/dist', electronVersion: '42.0.0', executableName: 'example-app', productName: 'Example', target: { platform: 'linux', arch: 'x64' } });
 ```
 */

import { packager, } from '@electron/packager';

import type { DistributionTarget, } from './distribution-targets.js';

/**
 Inputs required to package one Electron distribution target.
 
 @example
 ```ts
 const options: PackageTargetOptions = { appBundleId: 'dev.example.app', appCategoryType: 'public.app-category.developer-tools', appCopyright: 'Copyright Example', appDir: '/tmp/app', appVersion: '0.0.1', distributionDir: '/tmp/dist', electronVersion: '42.0.0', executableName: 'example-app', productName: 'Example', target: { platform: 'linux', arch: 'x64' } };
 ```
 */
export type PackageTargetOptions = {
  readonly appBundleId: string;
  readonly appCategoryType: string;
  readonly appCopyright: string;
  readonly appDir: string;
  readonly appVersion: string;
  readonly distributionDir: string;
  readonly electronVersion: string;
  readonly executableName: string;
  readonly productName: string;
  readonly target: DistributionTarget;
};

/**
 Runs Electron Packager for a single distribution target.
 
 @param appBundleId - Stable Electron application bundle identifier.
 
 @param appCategoryType - macOS Finder category.
 
 @param appCopyright - Copyright string embedded into bundles.
 
 @param appDir - Staged app directory.
 
 @param appVersion - Version embedded into native package metadata.
 
 @param distributionDir - Output directory for native bundles.
 
 @param electronVersion - Electron version to download and embed.
 
 @param executableName - Executable basename across target platforms.
 
 @param productName - Human-facing application name.
 
 @param target - Platform and architecture target.
 
 @example
 ```ts
 await packageTarget({ appBundleId: 'dev.example.app', appCategoryType: 'public.app-category.developer-tools', appCopyright: 'Copyright Example', appDir: '/tmp/app', appVersion: '0.0.1', distributionDir: '/tmp/dist', electronVersion: '42.0.0', executableName: 'example-app', productName: 'Example', target: { platform: 'linux', arch: 'x64' } });
 ```
 */
export async function packageTarget(
  {
    appBundleId,
    appCategoryType,
    appCopyright,
    appDir,
    appVersion,
    distributionDir,
    electronVersion,
    executableName,
    productName,
    target,
  }: PackageTargetOptions,
): Promise<void> {
  await packager({
    appBundleId,
    appCategoryType,
    appCopyright,
    appVersion,
    arch: target.arch,
    asar: true,
    dir: appDir,
    electronVersion,
    executableName,
    name: productName,
    out: distributionDir,
    overwrite: true,
    platform: target.platform,
    prune: true,
    quiet: false,
    win32metadata: {
      CompanyName: 'Monochromatic',
      FileDescription: productName,
      InternalName: executableName,
      OriginalFilename: `${executableName}.exe`,
      ProductName: productName,
    },
  },);
}
