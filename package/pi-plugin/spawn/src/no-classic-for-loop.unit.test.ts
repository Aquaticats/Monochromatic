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
 Matches a classic `for` head declaring its own cursor, the shape the standard
 replaces with `for...of` or a pipeline over the collection.
 */
const CLASSIC_FOR_HEAD = /\bfor\s*\(\s*(?:let|var)\s/;

/**
 Matches a `for...of` head, which shows the scanned text is real TypeScript
 source rather than an empty read every negative assertion would pass on.
 */
const FOR_OF_HEAD = /\bfor\s*\(\s*const\s+[\w$]+\s+of\s/;

/**
 Classic `for` sample the detector must match before its verdicts count.
 */
const CLASSIC_FOR_SAMPLE = 'for (let index = 0; index < 1; index += 1) { }';

/**
 `for...of` sample the detector must leave alone, so a pattern matching every
 loop head cannot pass itself off as a classic-`for` detector.
 */
const FOR_OF_SAMPLE = 'for (const item of items) { }';

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
    return name.endsWith('.ts',) && !name.endsWith(UNIT_TEST_SUFFIX,);
  },);
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
    .map(function reportLine(line: string, lineIndex: number,): string | undefined {
      if (!CLASSIC_FOR_HEAD.test(line,))
        return undefined;
      return `line ${String(lineIndex + 1,)}: ${line.trim()}`;
    },)
    .filter(function isReport(report: string | undefined,): report is string {
      return report !== undefined;
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
      name: 'positive control: the detector matches a classic for head and leaves a for...of head alone',
      fn: async function testDetectorSeesBothShapes() {
        expect(CLASSIC_FOR_HEAD.test(CLASSIC_FOR_SAMPLE,),).toBe(true,);
        expect(CLASSIC_FOR_HEAD.test(FOR_OF_SAMPLE,),).toBe(false,);
      },
    },),
    it({
      name: 'positive control: the scan reads shipped modules and finds a for...of head among them',
      fn: async function testScanReachesSources() {
        /**
         Shipped module file names the scan covers.
         */
        const names = await shippedModuleNames();
        expect(names.length,).toBeGreaterThan(0,);
        /**
         Module texts joined so one match proves the read reached real source.
         */
        const sources = (await Promise.all(names.map(moduleSource,),)).join('\n',);
        expect(sources,).toMatch(FOR_OF_HEAD,);
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
