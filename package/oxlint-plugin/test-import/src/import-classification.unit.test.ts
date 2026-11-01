import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  classifyImport,
  DEFAULT_FIXTURE_PATTERNS,
  type ImportOutcome,
  type OwningPackage,
} from '../dist/final/node/index.mjs';

/** Package root the sample owner is rooted at. */
const ROOT = '/repo/package/module/x';

/** Directory holding the importing test file. */
const CONTAINING_DIRECTORY = `${ROOT}/src`;

/** Owner standing in for a package that builds to `dist/final` and `dist/app`. */
const OWNER: OwningPackage = {
  root: ROOT,
  name: '@scope/my-package',
  buildsArtifact: true,
  artifactDirectories: [
    `${ROOT}/dist/final`,
    `${ROOT}/dist/app`,
  ],
};

/**
 Classifies one specifier against the sample owner.
 
 @param specifier - literal specifier text to classify
 
 @returns verdict for that specifier
 
 @example
 ```ts
 verdictFor('./parse.ts');
 ```
 */
function verdictFor(specifier: string,): ImportOutcome {
  return classifyImport({
    specifier,
    containingDirectory: CONTAINING_DIRECTORY,
    owner: OWNER,
    fixturePatterns: DEFAULT_FIXTURE_PATTERNS,
  },);
}

await describe({
  name: classifyImport.name,
  children: [
    it({
      name: 'allows a relative import landing in the default artifact root',
      fn: async () => {
        expect(verdictFor('../dist/final/node/index.mjs',),).toBe('allowed',);
      },
    },),
    it({
      name: 'allows a relative import landing in a declared entry directory',
      fn: async () => {
        expect(verdictFor('../dist/app/strip.js',),).toBe('allowed',);
      },
    },),
    it({
      name: 'rejects a relative import of sibling source',
      fn: async () => {
        expect(verdictFor('./parse.ts',),).toBe('relative-source',);
      },
    },),
    it({
      name: 'rejects a relative import of intermediate build output',
      fn: async () => {
        expect(verdictFor('../dist/temp/parse.mjs',),).toBe('relative-source',);
      },
    },),
    it({
      name: 'rejects a path that only looks like an artifact before normalization',
      fn: async () => {
        expect(verdictFor('./dist/final/fake.ts',),).toBe('relative-source',);
      },
    },),
    it({
      name: 'allows a relative import of a test-only fixture',
      fn: async () => {
        expect(verdictFor('./fixture.json',),).toBe('allowed',);
      },
    },),
    it({
      name: 'allows a relative import of a test-only support module',
      fn: async () => {
        expect(verdictFor('./test-support.ts',),).toBe('allowed',);
      },
    },),
    it({
      name: 'rejects a helper-suffixed module, which the default list treats as package behavior',
      fn: async () => {
        expect(verdictFor('./tree-helpers.ts',),).toBe('relative-source',);
      },
    },),
    it({
      name: 'allows the package\'s own bare name, which resolves through the exports map',
      fn: async () => {
        expect(verdictFor('@scope/my-package',),).toBe('allowed',);
      },
    },),
    it({
      name: 'rejects the package\'s own source subpath',
      fn: async () => {
        expect(verdictFor('@scope/my-package/ts',),).toBe('own-source-subpath',);
      },
    },),
    it({
      name: 'rejects a deeper own source subpath',
      fn: async () => {
        expect(verdictFor('@scope/my-package/ts/backend/api.ts',),).toBe('own-source-subpath',);
      },
    },),
    it({
      name: 'leaves another package alone',
      fn: async () => {
        expect(verdictFor('@scope/other-package',),).toBe('unchecked',);
      },
    },),
    it({
      name: 'leaves another package\'s source subpath alone, the sanctioned cross-package channel',
      fn: async () => {
        expect(verdictFor('@monochromatic-dev/test-harness-core/ts',),).toBe('unchecked',);
      },
    },),
    it({
      name: 'leaves a package whose name merely extends the owner\'s alone',
      fn: async () => {
        expect(verdictFor('@scope/my-package-extra',),).toBe('unchecked',);
      },
    },),
    it({
      name: 'leaves a package whose name extends the owner\'s and carries a ts subpath alone',
      fn: async () => {
        expect(verdictFor('@scope/my-package-extra/ts',),).toBe('unchecked',);
      },
    },),
    it({
      name: 'leaves a bare runtime module alone',
      fn: async () => {
        expect(verdictFor('node:path',),).toBe('unchecked',);
      },
    },),
  ],
},);
