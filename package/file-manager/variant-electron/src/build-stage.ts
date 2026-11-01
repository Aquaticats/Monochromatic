/**
 Stages static files and package metadata for the Electron app directory.
 
 @example
 ```ts
 await stageFileManagerApp();
 ```
 */

import { cp, } from 'node:fs/promises';
import { join, } from 'node:path';

import { stageElectronApp, } from '@monochromatic-dev/electron-infra-core/ts/stage';

/**
 Stages this package's no-Vite Electron app output.
 
 @example
 ```ts
 await stageFileManagerApp();
 ```
 */
async function stageFileManagerApp(): Promise<void> {
  await stageElectronApp({
    packageRoot: process.cwd(),
    staticAssets: [
      'index.html',
      'styles.css',
    ],
  },);

  // The preload bundle builds into its own dist/preload subdir so tsdown's
  // default clean never deletes the main bundle; staging assembles dist/app.
  await cp(
    join(
      process.cwd(),
      'dist',
      'preload',
      'preload.cjs',
    ),
    join(
      process.cwd(),
      'dist',
      'app',
      'preload.cjs',
    ),
  );
}

await stageFileManagerApp();
