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
  anchoredExternal,
  matchesBundlePattern,
  packageExternals,
} from './package-externals.ts';

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
  const path = mkdtempSync(join(scratchRoot(), 'package-externals-',),);
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

await describe({
  name: matchesBundlePattern.name,
  children: [
    it({
      name: 'matches scope prefixes, name prefixes, and exact names',
      fn: async function patternForms(): Promise<void> {
        expect(matchesBundlePattern({
          name: '@monochromatic-dev/logger-core',
          pattern: '@monochromatic-dev/**',
        },),).toBe(true,);
        expect(matchesBundlePattern({
          name: 'lezer-german',
          pattern: 'lezer-**',
        },),).toBe(true,);
        expect(matchesBundlePattern({
          name: 'nano-spawn',
          pattern: 'nano-spawn',
        },),).toBe(true,);
      },
    },),
    it({
      name: 'rejects names outside the pattern',
      fn: async function patternRejections(): Promise<void> {
        expect(matchesBundlePattern({
          name: '@other-scope/module-logger',
          pattern: '@monochromatic-dev/**',
        },),).toBe(false,);
        expect(matchesBundlePattern({
          name: 'nano-spawn-extra',
          pattern: 'nano-spawn',
        },),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: anchoredExternal.name,
  children: [
    it({
      name: 'matches the bare name and subpaths only',
      fn: async function anchoring(): Promise<void> {
        /**
         Matcher for one scoped package name.
         */
        const matcher = anchoredExternal('@earendil-works/pi-ai',);
        expect(matcher.test('@earendil-works/pi-ai',),).toBe(true,);
        expect(matcher.test('@earendil-works/pi-ai/tools',),).toBe(true,);
        expect(matcher.test('@earendil-works/pi-ai-extras',),).toBe(false,);
      },
    },),
    it({
      name: 'escapes regex metacharacters in names',
      fn: async function metacharacterEscape(): Promise<void> {
        expect(anchoredExternal('weird+name',).test('weird+name',),).toBe(true,);
        expect(anchoredExternal('weird+name',).test('weirddname',),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: packageExternals.name,
  children: [
    it({
      name: 'externalizes declared deps minus bundle patterns and keeps node builtins external',
      fn: async function externalsComputation(): Promise<void> {
        using fixture = createManifestFixture(JSON.stringify({
          dependencies: {
            '@monochromatic-dev/logger-core': 'workspace:*',
            'toml-eslint-parser': '>=0.10.0',
          },
          peerDependencies: { oxlint: '>=1.0.0', },
        },),);
        /**
         Matchers produced for the fixture manifest.
         */
        const external = await packageExternals({
          packageDir: fixture.path,
          alwaysBundle: ['@monochromatic-dev/**',],
        },);
        /**
         Convenience probe deciding whether any matcher hits a specifier.
         
         @param specifier - Import specifier to test against every matcher.
         
         @returns Whether any matcher treats specifier as external.
         
         @example
         ```ts
         hits('node:path');
         ```
         */
        function hits(specifier: string,): boolean {
          return external.some(function matches(matcher: RegExp,): boolean {
            return matcher.test(specifier,);
          },);
        }
        expect(hits('node:path',),).toBe(true,);
        expect(hits('toml-eslint-parser',),).toBe(true,);
        expect(hits('oxlint/plugins',),).toBe(true,);
        expect(hits('@monochromatic-dev/logger-core',),).toBe(false,);
        expect(hits('undeclared-transitive',),).toBe(false,);
      },
    },),
    it({
      name: 'rejects manifests that do not parse to objects',
      fn: async function invalidManifest(): Promise<void> {
        using fixture = createManifestFixture('"just a string"',);
        await expect(packageExternals({
          packageDir: fixture.path,
          alwaysBundle: [],
        },),).rejects.toThrow('package.json must parse to an object',);
      },
    },),
    it({
      name: 'rejects non-object dependency maps',
      fn: async function invalidDependencyMaps(): Promise<void> {
        using fixture = createManifestFixture('{"dependencies": 5}',);
        await expect(packageExternals({
          packageDir: fixture.path,
          alwaysBundle: [],
        },),).rejects.toThrow('dependency maps must be objects',);
      },
    },),
  ],
},);
