/**
 Distribution wrapper for the Electron counter package.
 
 @example
 ```ts
 await distributeElectronCounter();
 ```
 */

import { parseElectronDistributionArgs, } from '@monochromatic-dev/electron-infra-core/ts/distribution-args';
import { distributeElectronApp, } from '@monochromatic-dev/electron-infra-core/ts/distribute';

/**
 Stable Electron application bundle identifier.
 
 @example
 ```ts
 console.log(appBundleId);
 ```
 */
const appBundleId = 'dev.monochromatic.electron-counter';

/**
 macOS Finder category for this demonstrator.
 
 @example
 ```ts
 console.log(appCategoryType);
 ```
 */
const appCategoryType = 'public.app-category.developer-tools';

/**
 Executable basename used across target platforms.
 
 @example
 ```ts
 console.log(executableName);
 ```
 */
const executableName = 'monochromatic-electron-counter';

/**
 Runs shared Electron distribution machinery for this app.
 
 @example
 ```ts
 await distributeElectronCounter();
 ```
 */
async function distributeElectronCounter(): Promise<void> {
  await distributeElectronApp({
    ...parseElectronDistributionArgs({ argv: process.argv
      .slice(2,), },),
    appBundleId,
    appCategoryType,
    appCopyright: 'Copyright Monochromatic contributors',
    executableName,
    packageRoot: process.cwd(),
  },);
}

await distributeElectronCounter();
