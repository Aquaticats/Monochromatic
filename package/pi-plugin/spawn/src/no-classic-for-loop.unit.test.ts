/**
 Guards the review standard that replaces classic `for` loops with `for...of`
 or a functional pipeline, so the shipped modules of the `pi-plugin` packages
 keep the shape the standard requires. No linter rule enforces that standard, so
 this scan is the only mechanical check.

 The scan covers every package under `package/pi-plugin`, not this package
 alone: the standard governs the unit the repository review read, and a copy of
 this scan per package would be the same shape kept in many places and left to
 drift. The check therefore lives in the package that wrote it first, which also
 means deleting that package would delete the check.

 One package the review named has since left the category: Advisor moved to
 `package/agent-harness-shared/advisor`, so the scan holds that package too and
 the fix its reverse message scan now carries cannot regress.
 */

import type { Dirent, } from 'node:fs';

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Scan inputs

/**
 Directory holding the packages this guard scans, resolved from this test file so
 the scan follows the sources wherever the category is checked out.
 */
const packageDirectory = join(
  import.meta.dirname,
  '..',
  '..',
  '..',
);

/**
 Category whose packages the scan covers.
 */
const CATEGORY_NAME = 'pi-plugin';

/**
 Packages outside {@link CATEGORY_NAME} this scan covers, named relative to
 {@link packageDirectory}.

 `agent-harness-shared/advisor` was `pi-plugin/advisor` when the review read its
 reverse message scan, and a package that leaves the category would otherwise
 leave the scan with it.
 */
const MOVED_PACKAGE_PATHS: readonly string[] = [
  join(
    'agent-harness-shared',
    'advisor',
  ),
];

/**
 Name of the folder holding one package's shipped modules.
 */
const MODULE_DIRECTORY_NAME = 'src';

/**
 Suffix marking a unit test file, which the shipped-module scan skips because
 the standard governs the modules the package ships for consumers.
 */
const UNIT_TEST_SUFFIX = '.unit.test.ts';

/**
 Source prefix opening every `for` head this scan inspects.
 */
const FOR_HEAD_PREFIX = 'for (';

/**
 Source suffix marking a module file this scan reads.
 */
const MODULE_SUFFIX = '.ts';

/**
 Statement separator appearing only in a classic `for` head, never in a
 `for...of` or `for...in` head.
 */
const HEAD_CLAUSE_SEPARATOR = ';';

/**
 Classic `for` sample the detector must report before its verdicts count.
 */
const CLASSIC_FOR_SAMPLE = 'for (let index = 0; index < 1; index += 1) { }';

/**
 `for...of` sample the detector must leave alone, so a detector matching every
 loop head cannot pass itself off as a classic-`for` detector.
 */
const FOR_OF_SAMPLE = 'for (const item of items) { }';

//endregion Scan inputs

//region Types

/**
 One package's shipped modules, named relative to {@link packageDirectory}.
 */
type PackageModules = {
  /**
   Package path under the repository's `package` directory.
   */
  readonly packagePath: string;
  /**
   Shipped module paths this guard reads, empty when the package ships none.
   */
  readonly modulePaths: readonly string[];
};

//endregion Types

//region Module scan

/**
 Reads one module's authored source.

 @param modulePath - path under {@link packageDirectory}

 @returns the file's source text

 @example
 ```typescript
 const source = await moduleSource('pi-plugin/spawn/src/message-extract.ts');
 ```
 */
async function moduleSource(modulePath: string,): Promise<string> {
  return await readFile(
    join(
      packageDirectory,
      modulePath,
    ),
    'utf8',
  );
}

/**
 Lists the category's package paths.

 @returns package paths relative to {@link packageDirectory}, in directory order

 @example
 ```typescript
 const paths = await categoryPackagePaths();
 ```
 */
async function categoryPackagePaths(): Promise<readonly string[]> {
  /**
   Category entries, of which only directories name a package.
   */
  const entries = await readdir(
    join(
      packageDirectory,
      CATEGORY_NAME,
    ),
    { withFileTypes: true, },
  );
  return entries
    .filter(function isPackageDirectory(entry: Dirent,): boolean {
      return entry.isDirectory();
    },)
    .map(function packagePath(entry: Dirent,): string {
      return join(
        CATEGORY_NAME,
        entry.name,
      );
    },);
}

/**
 Lists one package's shipped module paths.

 A package with no {@link MODULE_DIRECTORY_NAME} folder reports an empty module
 list rather than a failed read, so the coverage test names it instead of a
 thrown filesystem error naming a path.

 @param packagePath - package path relative to {@link packageDirectory}

 @returns that package's shipped module paths, in directory order

 @example
 ```typescript
 const row = await packageModules('pi-plugin/spawn');
 ```
 */
async function packageModules(packagePath: string,): Promise<PackageModules> {
  /**
   Entries of this package, of which one folder holds its modules.
   */
  const entries = await readdir(
    join(
      packageDirectory,
      packagePath,
    ),
    { withFileTypes: true, },
  );
  /**
   Whether this package keeps a shipped-module folder at all.
   */
  const hasModuleDirectory = entries.some(function isModuleDirectory(entry: Dirent,): boolean {
    return entry.isDirectory() && (entry.name === MODULE_DIRECTORY_NAME);
  },);
  if (!hasModuleDirectory)
    return {
      packagePath,
      modulePaths: [],
    };

  /**
   File names of this package's shipped-module folder.
   */
  const moduleNames = await readdir(
    join(
      packageDirectory,
      packagePath,
      MODULE_DIRECTORY_NAME,
    ),
  );
  return {
    packagePath,
    modulePaths: moduleNames
      .filter(isShippedModuleName,)
      .map(function modulePath(moduleName: string,): string {
        return join(
          packagePath,
          MODULE_DIRECTORY_NAME,
          moduleName,
        );
      },),
  };
}

/**
 Lists every scanned package's shipped module paths.

 @returns one row per scanned package, in {@link categoryPackagePaths} order
 followed by {@link MOVED_PACKAGE_PATHS} order

 @example
 ```typescript
 const packages = await scannedPackages();
 ```
 */
async function scannedPackages(): Promise<readonly PackageModules[]> {
  /**
   Category packages the scan covers.
   */
  const categoryPaths = await categoryPackagePaths();
  /**
   Every package path the scan covers.
   */
  const packagePaths = [
    ...categoryPaths,
    ...MOVED_PACKAGE_PATHS,
  ];
  return await Promise.all(packagePaths.map(async function packageRows(
    packagePath: string,
  ): Promise<PackageModules> {
    return await packageModules(packagePath,);
  },),);
}

/**
 Selects the file names of one shipped-module folder that the scan reads.

 @param name - file name inside a shipped-module folder

 @returns whether the file is a shipped module

 @example
 ```typescript
 isShippedModuleName('message-extract.ts'); // true
 isShippedModuleName('message-extract.unit.test.ts'); // false
 ```
 */
function isShippedModuleName(name: string,): boolean {
  if (name.endsWith(UNIT_TEST_SUFFIX,))
    return false;
  return name.endsWith(MODULE_SUFFIX,);
}

/**
 Detects a classic `for` head on one source line.

 A classic head separates its clauses with semicolons, which no `for...of` or
 `for...in` head carries between its own parentheses. The head is read from its
 `for (` prefix to the last `)` on the line, so a head split across lines reads
 as compliant; no loop head in these packages spans a line break.

 @param line - one line of module source

 @returns whether the line opens a classic `for` loop

 @example
 ```typescript
 isClassicForHead('for (let index = 0; index < 1; index += 1) { }'); // true
 isClassicForHead('for (const item of items) { }'); // false
 ```
 */
function isClassicForHead(line: string,): boolean {
  if (!line.includes(FOR_HEAD_PREFIX,))
    return false;

  /**
   Head text from the `for` keyword to the line's last closing parenthesis, so
   a body holding calls still ends the slice past the head.
   */
  const headText = line.slice(
    line.lastIndexOf(FOR_HEAD_PREFIX,),
    line.lastIndexOf(')',),
  );

  return headText.includes(HEAD_CLAUSE_SEPARATOR,);
}

/**
 Reports classic `for` heads in one module text.

 @param source - module text to scan

 @returns one report per offending line, naming its line number and trimmed text

 @example
 ```typescript
 classicForHeads('for (let index = 0; index < 1; index += 1) { }');
 ```
 */
function classicForHeads(source: string,): readonly string[] {
  return source
    .split('\n',)
    .flatMap(function reportLine(line: string, lineIndex: number,): readonly string[] {
      if (!isClassicForHead(line,))
        return [];
      return [`line ${String(lineIndex + 1,)}: ${line.trim()}`];
    },);
}

/**
 Reports every classic `for` head across the scanned packages' shipped modules,
 grouped by file.

 @returns `package/src/file name` prefixed reports, empty when every module
 complies

 @example
 ```typescript
 const reports = await shippedClassicForHeads();
 ```
 */
async function shippedClassicForHeads(): Promise<readonly string[]> {
  /**
   Shipped modules the reports cover.
   */
  const packages = await scannedPackages();
  /**
   Per-module report lists, flattened after every module is read.
   */
  const perModule = await Promise.all(packages.flatMap(function packageRows(
    row: PackageModules,
  ): readonly Promise<readonly string[]>[] {
    return row.modulePaths.map(async function moduleReports(modulePath: string,) {
      /**
       That module's authored source.
       */
      const source = await moduleSource(modulePath,);
      return classicForHeads(source,).map(function qualify(report: string,): string {
        return `${modulePath} ${report}`;
      },);
    },);
  },),);

  return perModule.flat();
}

//endregion Module scan

await describe({
  name: 'classic for loop scan',
  children: [
    it({
      name: 'positive control: the detector reports a classic for head and leaves a for...of head alone',
      fn: async function testDetectorSeesBothShapes() {
        expect(isClassicForHead(CLASSIC_FOR_SAMPLE,),).toBe(true,);
        expect(isClassicForHead(FOR_OF_SAMPLE,),).toBe(false,);
      },
    },),
    it({
      name: 'positive control: the scan reads the shipped modules of every package it covers',
      fn: async function testScanReachesSources() {
        /**
         Shipped modules the scan covers.
         */
        const packages = await scannedPackages();
        expect(packages.length,).toBeGreaterThan(0,);
        /**
         Packages that shipped no module the scan could read, which would let
         the negative assertion below pass without seeing their source.
         */
        const withoutModules = packages
          .filter(function shipsNoModule(row: PackageModules,): boolean {
            return row.modulePaths.length === 0;
          },)
          .map(function nameOf(row: PackageModules,): string {
            return row.packagePath;
          },);
        expect(withoutModules.join('\n',),).toBe('',);
        /**
         Module texts the scan covers.
         */
        const sources = await Promise.all(packages.flatMap(function moduleReads(
          row: PackageModules,
        ): readonly Promise<string>[] {
          return row.modulePaths.map(async function readModule(modulePath: string,) {
            return await moduleSource(modulePath,);
          },);
        },),);
        /**
         Modules whose read came back empty, which no file in these packages is.
         */
        const emptySources = sources.filter(function isEmptyText(source: string,): boolean {
          return source.length === 0;
        },);
        expect(emptySources.length,).toBe(0,);
      },
    },),
    it({
      name: 'shipped modules iterate with for...of or a pipeline instead of a classic for loop',
      fn: async function testNoClassicForHeads() {
        /**
         Offending heads across every shipped module, one per line, so the
         failure names each file and line instead of an elided array.
         */
        const reports = await shippedClassicForHeads();
        expect(reports.join('\n',),).toBe('',);
      },
    },),
  ],
},);
