/**
 Unit tests for reusable Electron distribution helpers.
 
 @module
 */

import {
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

import {
  DISTRIBUTION_TARGETS,
  distributeElectronApp,
  parseElectronDistributionArgs,
  selectDistributionTargets,
  targetKey,
} from '../dist/final/node/index.mjs';

/**
 Expected default target keys.
 */
const expectedTargetKeys = [
  'linux-x64',
  'linux-arm64',
  'win32-x64',
  'win32-arm64',
  'darwin-x64',
  'darwin-arm64',
];

/**
 Disposable temp package fixture.
 */
type TempPackage = {
  /** Package root path. */
  readonly packageRoot: string;
  /** Removes package root recursively. */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a minimal package fixture for dry-run distribution.
 
 @returns Disposable package fixture.
 
 @example
 ```ts
 await using fixture = await makeTempPackage();
 ```
 */
async function makeTempPackage(): Promise<TempPackage> {
  /**
   Temporary package root.
   */
  const packageRoot = await mkdtemp(join(
    scratchRoot(),
    'electron-infra-distribution-',
  ),);

  await writeFile(
    join(
      packageRoot,
      'package.json',
    ),
    `${JSON.stringify({
      name: 'fixture-electron-app',
      productName: 'Fixture Electron App',
      version: '1.2.3',
    },)}\n`,
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
  name: '',
  children: [
    describe({
      name: targetKey.name,
      children: [
        it({
          name: 'formats stable distribution target key',
          fn: async () => {
            expect(targetKey({ target: { platform: 'linux', arch: 'x64', }, },),)
              .toBe('linux-x64',);
          },
        },),
      ],
    },),
    describe({
      name: 'DISTRIBUTION_TARGETS',
      children: [
        it({
          name: 'contains Linux, Windows, and macOS x64 and arm64 targets',
          fn: async () => {
            expect(DISTRIBUTION_TARGETS.map(function toKey(target,) {
              return targetKey({ target, },);
            },),).toEqual(expectedTargetKeys,);
          },
        },),
      ],
    },),
    describe({
      name: parseElectronDistributionArgs.name,
      children: [
        it({
          name: 'parses dry-run and repeated target filters',
          fn: async () => {
            expect(parseElectronDistributionArgs({
              argv: [
                '--dry-run',
                '--target',
                'linux-x64',
                '--target',
                'darwin-arm64',
              ],
            },),).toEqual({
              dryRun: true,
              selectedTargetKeys: [
                'linux-x64',
                'darwin-arm64',
              ],
            },);
          },
        },),
        it({
          name: 'throws when target option has no value',
          fn: async () => {
            expect(function parseInvalidArgs(): void {
              parseElectronDistributionArgs({ argv: ['--target',], },);
            },).toThrow('--target requires a target key.',);
          },
        },),
      ],
    },),
    describe({
      name: selectDistributionTargets.name,
      children: [
        it({
          name: 'filters selected targets by key',
          fn: async () => {
            expect(selectDistributionTargets({ selectedTargetKeys: ['linux-arm64',], },),)
              .toEqual([{ platform: 'linux', arch: 'arm64', },],);
          },
        },),
      ],
    },),
    describe({
      name: distributeElectronApp.name,
      children: [
        it({
          name: 'writes dry-run manifest without invoking packager',
          fn: async () => {
            await using fixture = await makeTempPackage();
            await distributeElectronApp({
              appBundleId: 'dev.example.fixture',
              appCategoryType: 'public.app-category.developer-tools',
              appCopyright: 'Copyright Fixture',
              dryRun: true,
              electronVersion: '42.0.0',
              executableName: 'fixture-electron-app',
              packageRoot: fixture.packageRoot,
              selectedTargetKeys: ['linux-x64',],
            },);

            /**
             Written manifest text.
             */
            const manifestText = await readFile(
              join(
                fixture.packageRoot,
                'dist',
                'distribution',
                'manifest.json',
              ),
              'utf8',
            );

            expect(JSON.parse(manifestText,),).toEqual({
              appBundleId: 'dev.example.fixture',
              electronVersion: '42.0.0',
              targets: [
                {
                  platform: 'linux',
                  arch: 'x64',
                  key: 'linux-x64',
                },
              ],
            },);
          },
        },),
      ],
    },),
  ],
},);
