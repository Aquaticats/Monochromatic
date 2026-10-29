import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type Plugin,
  rolldown,
  VERSION,
} from 'rolldown';

import {
  CHECKOUT_BUNDLER_PLUGIN_NAME,
  checkoutBundlerPlugin,
} from './checkout-bundler.ts';
import { familyLibraryNodeConfig, } from './index.node.ts';

/**
 Disposable entry-source fixture directory.
 */
type EntryFixture = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates a disposable directory holding one entry source and a manifest with no family members.

 @returns Disposable fixture outside repository state.

 @example
 ```ts
 using fixture = createEntryFixture();
 ```
 */
function createEntryFixture(): EntryFixture {
  /**
   Fresh fixture directory under the system temp root.
   */
  const path = mkdtempSync(join(scratchRoot(), 'checkout-bundler-',),);
  writeFileSync(join(path, 'entry.ts',), 'export const cat = 1;\n',);
  writeFileSync(join(path, 'package.json',), JSON.stringify({ name: '@x/family-markup', },),);
  return {
    path,
    [Symbol.dispose]: function removeFixture(): void {
      rmSync(path, {
        force: true,
        recursive: true,
      },);
    },
  };
}

/**
 Builds the fixture entry through the guard, told the given installed version.

 @param packageDir - fixture directory holding entry.ts

 @param installedVersion - version the guard is told this checkout installs, the running one or another

 @example
 ```ts
 await buildGuarded({ packageDir: fixture.path, installedVersion: VERSION, },);
 ```
 */
async function buildGuarded({
  packageDir,
  installedVersion,
}: {
  readonly packageDir: string;
  readonly installedVersion: string;
},): Promise<void> {
  /**
   Real rolldown build of the fixture entry; the bundler runs its hooks only once output is asked for.
   */
  await using build = await rolldown({
    input: [join(packageDir, 'entry.ts',),],
    plugins: [checkoutBundlerPlugin({ installedVersion, },),],
  },);
  await build.generate({ format: 'es', },);
}

await describe({
  name: checkoutBundlerPlugin.name,
  children: [
    it({
      name: 'lets a build through when the running bundler is the version this checkout installs',
      fn: async function admitsInstalledVersion(): Promise<void> {
        using fixture = createEntryFixture();
        await buildGuarded({
          packageDir: fixture.path,
          installedVersion: VERSION,
        },);
      },
    },),
    it({
      name: 'refuses a build run by another version, naming both versions, PATH and each way to fix it',
      fn: async function refusesOtherVersion(): Promise<void> {
        using fixture = createEntryFixture();
        /**
         Every part the refusal must say: both versions, where the bundler came from, and each way to fix it.
         */
        const parts = [
          `running rolldown ${VERSION}`,
          'installs rolldown 0.0.0-installed',
          'first on PATH',
          'node_modules/.bin',
          '--setenv=PATH',
          'reinstall',
        ] as const;
        await Promise.all(parts.map(async function refusalSays(part: string,): Promise<void> {
          await expect(buildGuarded({
            packageDir: fixture.path,
            installedVersion: '0.0.0-installed',
          },),)
            .rejects
            .toThrow(part,);
        },),);
      },
    },),
  ],
},);

await describe({
  name: `${familyLibraryNodeConfig.name} bundler guard`,
  children: [
    it({
      // NOT "FIRST": `buildStart` is a parallel hook, so a plugin's place in the
      // list does not order it; the guard holds because an error there ends the
      // build before any output hook runs.
      name: 'carries the checkout bundler guard',
      fn: async function carriesGuard(): Promise<void> {
        using fixture = createEntryFixture();
        /**
         Library config for a member with no family siblings.
         */
        const config = await familyLibraryNodeConfig({
          familyPrefix: '@x/family-',
          packageDir: fixture.path,
        },);
        /**
         Plugins the config carries, in order.
         */
        const plugins = config.plugins as readonly Plugin[];
        expect(plugins.map(function nameOf(plugin: Plugin,): string {
          return plugin.name;
        },),).toContain(CHECKOUT_BUNDLER_PLUGIN_NAME,);
      },
    },),
  ],
},);
