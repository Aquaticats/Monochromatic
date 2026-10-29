/**
 Stages static files and package metadata for Electron Packager.
 
 @example
 ```ts
 await stageElectronCounterApp();
 ```
 */

import { stageElectronApp, } from '@monochromatic-dev/electron-infra-core/ts/stage';

/**
 Stages this package's no-Vite Electron app output.
 
 @example
 ```ts
 await stageElectronCounterApp();
 ```
 */
async function stageElectronCounterApp(): Promise<void> {
  await stageElectronApp({
    packageRoot: process.cwd(),
    staticAssets: [
      'index.html',
      'styles.css',
    ],
  },);
}

await stageElectronCounterApp();
