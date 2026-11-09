/**
 Node's filesystem module reaches the bundle through one mechanism.
 
 A literal `import('node:fs/promises')` is a static specifier: every bundler
 resolves it, so a client bundle that reaches such a site either pulls Node
 builtins in or fails to resolve them, whichever the bundler does with
 missing builtins. The package therefore loads the module once, behind the
 computed specifier in `fs.ts`, and hands the loaded module to every caller.
 This suite fails when a statically resolvable spelling appears in a source
 module, or when the computed specifier the readers rely on disappears.
 */
import { readdirSync, readFileSync, } from 'node:fs';
import { basename, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Source scanning

/**
 Directory holding the modules under test; this suite sits beside them.
 */
const sourceDirectory = import.meta.dirname;

/**
 Suffix marking a test file, which no bundle includes and which may reach
 `node:fs` directly to build fixtures.
 */
const testFileSuffix = '.test.ts';

/**
 Dynamic-import spelling of Node's filesystem module that a bundler resolves
 statically: `import('node:fs')` or `import('node:fs/promises')` in any quote
 style. A computed specifier (`import(nodeFsSpecifier)`) never matches,
 because its argument is not a string literal.
 */
const literalNodeFsImport = /import\(\s*['"`]node:fs(?:\/promises)?['"`]/u;

/**
 Source modules of this package, test files excluded.
 
 @returns Absolute paths of the modules a bundle can include.
 */
function bundledSourcePaths(): string[] {
  return readdirSync(sourceDirectory,)
    .filter(function isBundledSource(entryName,) {
      return entryName.endsWith('.ts',) && (!entryName.endsWith(testFileSuffix,));
    },)
    .map(function toAbsolutePath(entryName,) {
      return join(
        sourceDirectory,
        entryName,
      );
    },);
}

/**
 Line numbers of every statically resolvable `node:fs` import in one module.
 
 @param filePath - Absolute path of the module to inspect.
 
 @returns One `relativePath:lineNumber` entry per literal import site.
 */
function literalNodeFsImportSites(filePath: string,): string[] {
  return readFileSync(
    filePath,
    'utf8',
  )
    .split('\n',)
    .flatMap(function sitesOnLine(line, index,) {
      return literalNodeFsImport.test(line,)
        ? [`${basename(filePath,)}:${index + 1}`,]
        : [];
    },);
}

//endregion Source scanning

await describe({
  name: 'node:fs import sites',
  children: [
    it({
      name: 'no source module spells node:fs as a static import specifier',
      fn: () => {
        const sites = bundledSourcePaths()
          .flatMap(literalNodeFsImportSites,);
        expect(sites,).toEqual([],);
      },
    },),
  ],
},);
