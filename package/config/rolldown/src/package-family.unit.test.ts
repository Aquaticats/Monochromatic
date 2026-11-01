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
import type { Plugin, } from 'rolldown';

import { familyLibraryNodeConfig, } from './index.node.ts';
import {
  familyDependencies,
  familyEntrySpecifier,
} from './package-family.ts';

/**
 Disposable manifest fixture directory.
 */
type ManifestFixture = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Creates a disposable directory holding one package.json fixture.

 @param manifestText - Raw package.json content to write.

 @returns Disposable fixture outside repository state.

 @example
 ```ts
 using fixture = createManifestFixture('{}');
 ```
 */
function createManifestFixture(manifestText: string,): ManifestFixture {
  /**
   Fresh fixture directory under the system temp root.
   */
  const path = mkdtempSync(join(scratchRoot(), 'package-family-',),);
  writeFileSync(join(path, 'package.json',), manifestText,);
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
 Package-name prefix the fixtures' family shares.
 */
const PREFIX = '@x/family-';

await describe({
  name: familyDependencies.name,
  children: [
    it({
      name: 'keeps production dependencies that carry the prefix and leaves others and dev dependencies out',
      fn: async function membersOnly(): Promise<void> {
        using fixture = createManifestFixture(JSON.stringify({
          dependencies: {
            '@x/family-markup': 'workspace:*',
            '@x/family-stage': 'workspace:*',
            '@x/other': 'workspace:*',
            'family-markup': '1.0.0',
          },
          devDependencies: { '@x/family-test-fixture': 'workspace:*', },
        },),);
        expect(await familyDependencies({
          packageDir: fixture.path,
          familyPrefix: PREFIX,
        },),).toEqual([
          '@x/family-markup',
          '@x/family-stage',
        ],);
      },
    },),
    it({
      name: 'reads a manifest with no dependencies as a package with no family members',
      fn: async function noDependencies(): Promise<void> {
        using fixture = createManifestFixture(JSON.stringify({ name: '@x/family-markup', },),);
        expect(await familyDependencies({
          packageDir: fixture.path,
          familyPrefix: PREFIX,
        },),).toEqual([],);
      },
    },),
    it({
      name: 'refuses a manifest that is not a JSON object, naming the file',
      fn: async function manifestNotObject(): Promise<void> {
        using fixture = createManifestFixture('[]',);
        await expect(familyDependencies({
          packageDir: fixture.path,
          familyPrefix: PREFIX,
        },),).rejects.toThrow(`${join(fixture.path, 'package.json',)} must hold a JSON object`,);
      },
    },),
    it({
      name: 'refuses dependencies that are not a JSON object, an array among them',
      fn: async function dependenciesNotObject(): Promise<void> {
        using fixture = createManifestFixture(JSON.stringify({ dependencies: ['@x/family-markup',], },),);
        await expect(familyDependencies({
          packageDir: fixture.path,
          familyPrefix: PREFIX,
        },),).rejects.toThrow('"dependencies" must be a JSON object when present',);
      },
    },),
  ],
},);

await describe({
  name: familyEntrySpecifier.name,
  children: [
    it({
      name: 'trades a member\'s source subpath for its bare name and keeps every other id',
      fn: async function specifiers(): Promise<void> {
        /**
         Family the specifiers are read against.
         */
        const family: ReadonlySet<string> = new Set(['@x/family-markup',],);
        expect(familyEntrySpecifier({
          id: '@x/family-markup/ts',
          family,
        },),).toBe('@x/family-markup',);
        expect(familyEntrySpecifier({
          id: '@x/family-stage/ts',
          family,
        },),).toBe('@x/family-stage/ts',);
        expect(familyEntrySpecifier({
          id: '@x/family-markup',
          family,
        },),).toBe('@x/family-markup',);
        expect(familyEntrySpecifier({
          id: '@x/family-markup-extra/ts',
          family,
        },),).toBe('@x/family-markup-extra/ts',);
        expect(familyEntrySpecifier({
          id: 'node:fs',
          family,
        },),).toBe('node:fs',);
      },
    },),
    it({
      name: 'refuses one file under a member\'s source subpath, which the member\'s built entry does not expose',
      fn: async function deepSourcePath(): Promise<void> {
        expect(function emitDeep(): string {
          return familyEntrySpecifier({
            id: '@x/family-markup/ts/han-only-text.ts',
            family: new Set(['@x/family-markup',],),
          },);
        },).toThrow('@x/family-markup/ts/han-only-text.ts names one file of family member @x/family-markup',);
      },
    },),
  ],
},);

await describe({
  name: familyLibraryNodeConfig.name,
  children: [
    it({
      name: 'keeps family members external and emits their imports against each member\'s built entry',
      fn: async function libraryConfig(): Promise<void> {
        using fixture = createManifestFixture(JSON.stringify({
          dependencies: {
            '@x/family-markup': 'workspace:*',
            'nano-spawn': '^1.0.0',
          },
        },),);
        /**
         Config built for the fixture package.
         */
        const config = await familyLibraryNodeConfig({
          familyPrefix: PREFIX,
          packageDir: fixture.path,
        },);
        /**
         Matchers the config hands rolldown.
         */
        const external = config.external as readonly RegExp[];
        /**
         Whether any matcher keeps a specifier external.

         @param specifier - import the bundle would emit

         @returns Whether rolldown leaves it an import

         @example
         ```ts
         isExternal('\@x/family-markup/ts',); // true
         ```
         */
        function isExternal(specifier: string,): boolean {
          return external.some(function hits(matcher: RegExp,): boolean {
            return matcher.test(specifier,);
          },);
        }
        expect(isExternal('@x/family-markup/ts',),).toBe(true,);
        expect(isExternal('@x/family-stage/ts',),).toBe(false,);
        /**
         Output options of the config, holding the emitted-specifier mapping.
         */
        const output = config.output as { readonly paths: (id: string,) => string; };
        expect(output.paths('@x/family-markup/ts',),).toBe('@x/family-markup',);
        expect(output.paths('node:fs',),).toBe('node:fs',);
      },
    },),
    it({
      name: 'appends the plugins it is handed after the shared ones, so a member can carry a data file into its output',
      fn: async function extraPlugins(): Promise<void> {
        using fixture = createManifestFixture(JSON.stringify({ name: '@x/family-markup', },),);
        /**
         Plugin the member hands the build.
         */
        const copier: Plugin = { name: 'copy-config', };
        /**
         Config built with the extra plugin.
         */
        const config = await familyLibraryNodeConfig({
          familyPrefix: PREFIX,
          packageDir: fixture.path,
          extraPlugins: [copier,],
        },);
        /**
         Plugins the config hands rolldown, in order.
         */
        const plugins = config.plugins as readonly Plugin[];
        expect(plugins.at(-1,),).toBe(copier,);
        expect(plugins.length > 1,).toBe(true,);
      },
    },),
  ],
},);
