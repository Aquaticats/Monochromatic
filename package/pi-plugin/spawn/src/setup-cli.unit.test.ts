import {
  existsSync,
  mkdirSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  autoSetupCli,
  cliIsOnPath,
  cliPathFromExtensionPath,
  isBuiltExtensionPath,
  NO_CLI_SETUP_WARNING,
  packageRootFromExtensionPath,
} from './setup-cli.ts';
import {
  envVar,
  tempDir,
} from './test-support.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: isBuiltExtensionPath.name,
      children: [
        it({
          name: 'detects tsdown node output path shape',
          fn: async function testBuiltPathDetection() {
            expect(isBuiltExtensionPath('/pkg/dist/final/node/index.mjs',),).toBe(true,);
            expect(isBuiltExtensionPath('/pkg/src/index.ts',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: packageRootFromExtensionPath.name,
      children: [
        it({
          name: 'resolves source package root',
          fn: async function testSourceRoot() {
            expect(packageRootFromExtensionPath('/pkg/src/index.ts',),).toBe('/pkg',);
          },
        },),
        it({
          name: 'resolves built package root',
          fn: async function testBuiltRoot() {
            expect(packageRootFromExtensionPath('/pkg/dist/final/node/index.mjs',),).toBe('/pkg',);
          },
        },),
      ],
    },),
    describe({
      name: cliPathFromExtensionPath.name,
      children: [
        it({
          name: 'points at source cli beside extension source root',
          fn: async function testSourceCliPath() {
            expect(cliPathFromExtensionPath('/pkg/src/index.ts',),).toBe('/pkg/src/cli.ts',);
          },
        },),
        it({
          name: 'points at built cli beside built extension output',
          fn: async function testBuiltCliPath() {
            expect(cliPathFromExtensionPath('/pkg/dist/final/node/index.mjs',),)
              .toBe('/pkg/dist/final/node/cli.mjs',);
          },
        },),
      ],
    },),
    describe({
      name: cliIsOnPath.name,
      children: [
        it({
          name: 'returns a boolean for current PATH',
          fn: async function testCliIsOnPath() {
            expect(typeof await cliIsOnPath(),).toBe('boolean',);
          },
        },),
      ],
    },),
    describe({
      name: autoSetupCli.name,
      children: [
        it({
          name: 'symlinks source cli into user local bin when command is absent',
          fn: async function testAutoSetupCli() {
            await using packageDir = await tempDir({ prefix: 'spawn-pi-package-', },);
            await using homeDir = await tempDir({ prefix: 'spawn-pi-home-', },);
            using _path = envVar({
              name: 'PATH',
              value: '/usr/bin:/bin',
            },);

            mkdirSync(
              join(
                packageDir.path,
                'src',
              ),
              { recursive: true, },
            );
            writeFileSync(
              join(
                packageDir.path,
                'src',
                'cli.ts',
              ),
              '#!/usr/bin/env node\n',
            );

            /**
             User-local bin path expected by auto setup.
             */
            const localBin = join(
              homeDir.path,
              '.local',
              'bin',
            );
            /**
             CLI setup result.
             */
            const result = await autoSetupCli({
              extensionPath: join(
                packageDir.path,
                'src',
                'index.ts',
              ),
              env: {
                HOME: homeDir.path,
                PATH: localBin,
              },
            },);
            /**
             Symlink expected from automatic setup.
             */
            const symlinkPath = join(
              localBin,
              'spawn-pi',
            );

            expect(result,).toBe(NO_CLI_SETUP_WARNING,);
            expect(existsSync(symlinkPath,),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
