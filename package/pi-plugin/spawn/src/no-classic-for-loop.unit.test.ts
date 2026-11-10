/**
 Guards the review standard that replaces classic `for` loops with `for...of`
 or a functional pipeline, so this package's shipped modules keep the shape the
 standard requires. No linter rule enforces that standard, so this scan is the
 only mechanical check.
 */

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
 Directory holding the modules this guard scans, resolved from this test file so
 the scan follows the sources wherever the package is checked out.
 */
const moduleDirectory = import.meta.dirname;

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

/**
 Module-doc marker every shipped module carries in its header block, which
 proves a read returned real source rather than an empty file.
 */
const MODULE_DOC_MARKER = '@module';

//endregion Scan inputs

//region Module scan

/**
 Reads one module's authored source.

 @param name - file name under {@link moduleDirectory}

 @returns the file's source text

 @example
 ```typescript
 const source = await moduleSource('message-extract.ts');
 ```
 */
async function moduleSource(name: string,): Promise<string> {
  return await readFile(
    join(
      moduleDirectory,
      name,
    ),
    'utf8',
  );
}

/**
 Lists the shipped module file names this guard scans.

 @returns TypeScript file names that are not unit tests, in directory order

 @example
 ```typescript
 const names = await shippedModuleNames();
 ```
 */
async function shippedModuleNames(): Promise<readonly string[]> {
  return (await readdir(moduleDirectory,)).filter(function isShippedModule(name: string,): boolean {
    if (name.endsWith(UNIT_TEST_SUFFIX,))
      return false;
    return name.endsWith(MODULE_SUFFIX,);
  },);
}

/**
 Detects a classic `for` head on one source line.

 A classic head separates its clauses with semicolons, which no `for...of` or
 `for...in` head carries between its own parentheses. The head is read from its
 `for (` prefix to the last `)` on the line, so a head split across lines reads
 as compliant; every loop in this codebase sits on one line.

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
 Reports every classic `for` head across the shipped modules, grouped by file.

 @returns `file name` prefixed reports, empty when every module complies

 @example
 ```typescript
 const reports = await shippedClassicForHeads();
 ```
 */
async function shippedClassicForHeads(): Promise<readonly string[]> {
  /**
   Shipped module file names the reports cover.
   */
  const names = await shippedModuleNames();
  /**
   Per-file report lists, flattened after every module is read.
   */
  const perModule = await Promise.all(names.map(async function moduleReports(name: string,) {
    /**
     That module's authored source.
     */
    const source = await moduleSource(name,);
    return classicForHeads(source,).map(function qualify(report: string,): string {
      return `${name} ${report}`;
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
      name: 'positive control: the scan reads every shipped module\'s own source',
      fn: async function testScanReachesSources() {
        /**
         Shipped module file names the scan covers.
         */
        const names = await shippedModuleNames();
        expect(names.length,).toBeGreaterThan(0,);
        /**
         Module texts the scan covers.
         */
        const sources = await Promise.all(names.map(moduleSource,),);
        /**
         Modules whose read reached their header block; every module carries
         one, so a short count means a read came back empty and the negative
         assertion below would pass without seeing source.
         */
        const modulesWithHeader = sources.filter(function hasHeader(source: string,): boolean {
          return source.includes(MODULE_DOC_MARKER,);
        },);
        expect(modulesWithHeader.length,).toBe(names.length,);
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
