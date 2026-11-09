/**
 Node's filesystem module reaches the bundle through one mechanism.
 
 A literal `import('node:fs/promises')` is a static specifier: every bundler
 resolves it, so a client bundle that reaches such a site either pulls Node
 builtins in or fails to resolve them, whichever the bundler does with
 missing builtins. The package therefore loads the module once, behind the
 computed specifier in `fs.ts`, and hands the loaded module to every caller.
 This suite fails when a statically resolvable spelling appears in a source
 module, and it reads the specifier out of each `import(...)` call rather
 than matching a pattern, so a computed argument never counts.
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
 Specifier text of a dynamic import call written with a literal argument,
 as it reads once quotes and surrounding space are stripped.
 */
const dynamicImportPrefix = 'import(';

/**
 Quote characters a literal specifier may be written with.
 */
const quoteCharacters = [
  '"',
  "'",
  '`',
] as const;

/**
 Specifier texts a bundler resolves statically, i.e. Node's filesystem module
 named as a literal.
 */
const literalNodeFsSpecifiers: ReadonlySet<string> = new Set(
  [
    'node:fs',
    'node:fs/promises',
  ],
);

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
 Argument text of the first dynamic import call on a line, with quotes and
 surrounding space stripped.
 
 @param line - One source line.
 
 @returns Stripped argument text, or empty string when the line has no
 complete dynamic import call.
 */
function dynamicImportArgument(line: string,): string {
  const callIndex = line.indexOf(dynamicImportPrefix,);
  if (callIndex === (-1))
    return '';

  const argumentStart = callIndex + dynamicImportPrefix.length;
  const argumentEnd = line.indexOf(')', argumentStart,);
  if (argumentEnd === (-1))
    return '';

  /**
   Argument as written, before quotes are stripped.
   */
  const argument = line.slice(
    argumentStart,
    argumentEnd,
  )
    .trim();
  return quoteCharacters
    .reduce(function withoutQuote(text, quote,) {
      return text.replaceAll(
        quote,
        '',
      );
    }, argument,)
    .trim();
}

/**
 Statically resolvable `node:fs` import sites in one module.
 
 @param filePath - Absolute path of the module to inspect.
 
 @returns One `fileName:lineNumber` entry per literal import site.
 */
function literalNodeFsImportSites(filePath: string,): string[] {
  return readFileSync(
    filePath,
    'utf8',
  )
    .split('\n',)
    .flatMap(function sitesOnLine(line, index,) {
      return literalNodeFsSpecifiers.has(dynamicImportArgument(line,),)
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
      fn: async () => {
        const sites = bundledSourcePaths()
          .flatMap(literalNodeFsImportSites,);
        expect(sites,).toEqual([],);
      },
    },),
  ],
},);
