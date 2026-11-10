/**
 Node's filesystem module reaches the bundle through one mechanism.
 
 A literal `import('node:fs/promises')` is a static specifier: every bundler
 resolves it, so a client bundle that reaches such a site either pulls Node
 builtins in or fails to resolve them, whichever the bundler does with
 missing builtins. The package therefore loads the module once, behind the
 computed specifier in `fs.ts`, and hands the loaded module to every caller.
 This suite fails when a statically resolvable spelling appears in a source
 module. It reads the argument of each `import(...)` call out of the source
 text, so a computed argument never counts and a literal split across lines
 still does.
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
 Text that opens a dynamic import call.
 */
const dynamicImportPrefix = 'import(';

/**
 Characters a literal import argument may be wrapped in, split by, or
 followed by, so a wrapped or quoted spelling compares equal to the bare
 module name.
 */
const argumentNoiseCharacters = [
  '"',
  "'",
  '`',
  ' ',
  '\t',
  '\n',
  '\r',
  ',',
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
 Arguments of every dynamic import call in one module, with quotes and
 whitespace removed so a wrapped spelling reads as one token.
 
 @param filePath - Absolute path of the module to scan.
 
 @returns Stripped argument text per call, empty arguments excluded.
 */
function dynamicImportArguments(filePath: string,): string[] {
  /**
   Source text of the module.
   */
  const sourceText = readFileSync(
    filePath,
    'utf8',
  );
  return sourceText
    .split(dynamicImportPrefix,)
    .slice(1,)
    .flatMap(function argumentOf(callText,) {
      /**
       Index of the parenthesis closing this call, absent when the call runs
       to the end of the file.
       */
      const closingIndex = callText.indexOf(')',);
      /**
       Argument as written, before quotes and line breaks are stripped.
       */
      const rawArgument = closingIndex === (-1)
        ? callText
        : callText.slice(
          0,
          closingIndex,
        );
      /**
       Argument without the characters a literal may be wrapped in.
       */
      const argument = argumentNoiseCharacters
        .reduce(function withoutNoise(text, character,) {
          return text.replaceAll(
            character,
            '',
          );
        }, rawArgument,);

      return argument === ''
        ? []
        : [argument,];
    },);
}

/**
 Statically resolvable `node:fs` import sites in one module.
 
 @param filePath - Absolute path of the module to inspect.
 
 @returns One `fileName: specifier` entry per literal import site.
 */
function literalNodeFsImportSites(filePath: string,): string[] {
  return dynamicImportArguments(filePath,)
    .filter(function namesNodeFs(argument,) {
      return literalNodeFsSpecifiers.has(argument,);
    },)
    .map(function describeSite(argument,) {
      return `${basename(filePath,)}: ${argument}`;
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
