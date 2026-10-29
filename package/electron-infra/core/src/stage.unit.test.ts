/**
 Unit tests for reusable Electron app staging helpers.
 
 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stageElectronApp, } from '../dist/final/node/index.mjs';

/**
 Disposable staging fixture.
 */
type StageFixture = {
  /** Package root path. */
  readonly packageRoot: string;
  /** Removes package root recursively. */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a minimal Electron package source tree.
 
 @returns Disposable staging fixture.
 
 @example
 ```ts
 await using fixture = await makeStageFixture();
 ```
 */
async function makeStageFixture(): Promise<StageFixture> {
  /**
   Temporary package root.
   */
  const packageRoot = await mkdtemp(join(
    scratchRoot(),
    'electron-infra-stage-',
  ),);

  await mkdir(
    join(
      packageRoot,
      'src',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      packageRoot,
      'package.json',
    ),
    `${JSON.stringify({
      name: 'fixture-electron-app',
      productName: 'Fixture Electron App',
      version: '1.2.3',
      description: 'Fixture description',
      license: 'LGPL-3.0-or-later',
    },)}\n`,
    'utf8',
  );
  await writeFile(
    join(
      packageRoot,
      'src',
      'index.html',
    ),
    '<!doctype html>\n',
    'utf8',
  );

  return {
    packageRoot,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        packageRoot,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

await describe({
  name: stageElectronApp.name,
  children: [
    it({
      name: 'copies static assets and writes runtime package manifest',
      fn: async () => {
        await using fixture = await makeStageFixture();

        await stageElectronApp({
          packageRoot: fixture.packageRoot,
          staticAssets: ['index.html',],
        },);

        expect(await readFile(
          join(
            fixture.packageRoot,
            'dist',
            'app',
            'index.html',
          ),
          'utf8',
        ),).toBe('<!doctype html>\n',);

        expect(
          JSON.parse(await readFile(
            join(
              fixture.packageRoot,
              'dist',
              'app',
              'package.json',
            ),
            'utf8',
          ),),
        ).toEqual({
          name: 'fixture-electron-app',
          productName: 'Fixture Electron App',
          version: '1.2.3',
          description: 'Fixture description',
          license: 'LGPL-3.0-or-later',
          main: 'main.mjs',
          type: 'module',
        },);
      },
    },),
  ],
},);
