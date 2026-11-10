/**
 Cross-package imports reach a sibling's TypeScript source, never its build.
 
 Rule ST3 in `AGENTS.md` and `doc/decision/workspace-ts-source-imports.md` make
 the `/ts` subpath the one cross-package channel, so type-check, lint, test, and
 bundle read sibling source with no build step. A bare workspace specifier
 resolves through the sibling's exports map to its `dist/` output, which this
 package would then need built first. This suite reads the workspace specifiers
 out of every source module here and fails on one that skips `/ts`.
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
 Workspace scope every package name shares, without its trailing separator so
 this file's own text never reads as an import site.
 */
const workspaceScope = '@monochromatic-dev';

/**
 Workspace scope plus the separator that follows it.
 */
const workspaceSpecifierPrefix = `${workspaceScope}/`;

/**
 Subpath segment serving a package's TypeScript source.
 */
const sourceSubpath = 'ts';

/**
 Quote characters a module specifier may be written in.
 */
const specifierQuotes = [
  "'",
  '"',
] as const;

/**
 Suffix of a source module; test modules count because they import siblings
 too.
 */
const moduleSuffix = '.ts';

/**
 Keywords and call forms whose last token before a quote is a module
 specifier, so a quoted fixture string elsewhere in a module is not one.
 */
const specifierIntroducers = [
  'from',
  'import',
  'import(',
  'require(',
] as const;

/**
 Whether the quote at one offset opens a module specifier.
 
 @param sourceText - Text of the module being scanned
 
 @param quoteIndex - Offset of the opening quote
 
 @returns whether an import keyword directly precedes the quote
 
 @example
 ```ts
 opensSpecifier({ sourceText: "import x from 'y';", quoteIndex: 14, }); // true
 ```
 */
function opensSpecifier({
  sourceText,
  quoteIndex,
}: {
  readonly sourceText: string;
  readonly quoteIndex: number;
},): boolean {
  /**
   Code before the quote, without the whitespace between token and quote.
   */
  const beforeQuote = sourceText.slice(0, quoteIndex,)
    .trimEnd();
  return specifierIntroducers
    .some(function namesIntroducer(introducer,) {
      return beforeQuote.endsWith(introducer,);
    },);
}

/**
 Source modules of this package.
 
 @returns Absolute paths of every module in this package's source directory.
 */
function sourceModulePaths(): string[] {
  return readdirSync(sourceDirectory,)
    .filter(function isModule(entryName,) {
      return entryName.endsWith(moduleSuffix,);
    },)
    .map(function toAbsolutePath(entryName,) {
      return join(
        sourceDirectory,
        entryName,
      );
    },);
}

/**
 Workspace specifiers one module imports or re-exports.
 
 A specifier counts only when a quote holding the workspace scope follows an
 import keyword, so the scan needs no parser and cannot mistake prose or a
 fixture string for an import.
 
 @param sourceText - Text of one source module.
 
 @returns Specifier text of every workspace import site in that module.
 */
function workspaceSpecifiers(sourceText: string,): string[] {
  return specifierQuotes.flatMap(function specifiersInQuote(quote,) {
    /**
     Opening text of a workspace specifier written in this quote character.
     */
    const marker = `${quote}${workspaceSpecifierPrefix}`;
    /**
     Specifiers found in this quote character.
     */
    const specifiers: string[] = [];
    /**
     Offset the next search starts at, past the specifier just read.
     */
    let searchFrom = 0;

    for (;;) {
      /**
       Offset of the next workspace specifier opening, absent past the end.
       */
      const markerIndex = sourceText.indexOf(
        marker,
        searchFrom,
      );

      if (markerIndex === (-1))
        break;

      /**
       Offset of the specifier text, one character past the opening quote.
       */
      const specifierStart = markerIndex + 1;
      /**
       Offset of the quote closing this specifier, absent when unclosed.
       */
      const specifierEnd = sourceText.indexOf(
        quote,
        specifierStart,
      );

      if (specifierEnd === (-1))
        break;

      if (opensSpecifier({
        sourceText,
        quoteIndex: markerIndex,
      },))
        specifiers.push(sourceText.slice(
          specifierStart,
          specifierEnd,
        ),);
      searchFrom = specifierEnd + 1;
    }

    return specifiers;
  },);
}

/**
 Whether a workspace specifier names the sibling's TypeScript source.
 
 @param specifier - Workspace specifier text, scope included.
 
 @returns True when the segment after the package name is the `/ts` subpath.
 */
function namesSourceSubpath(specifier: string,): boolean {
  return specifier
    .slice(workspaceSpecifierPrefix.length,)
    .split('/',)[1] === sourceSubpath;
}

/**
 Workspace import sites that skip the `/ts` source subpath.
 
 @returns One `fileName: specifier` entry per site reaching a sibling build.
 */
function builtOutputImportSites(): string[] {
  return sourceModulePaths()
    .flatMap(function sitesInModule(filePath,) {
      /**
       Source text of the module being scanned.
       */
      const sourceText = readFileSync(
        filePath,
        'utf8',
      );
      return workspaceSpecifiers(sourceText,)
        .filter(function skipsSourceSubpath(specifier,) {
          return !namesSourceSubpath(specifier,);
        },)
        .map(function describeSite(specifier,) {
          return `${basename(filePath,)}: ${specifier}`;
        },);
    },);
}

//endregion Source scanning

await describe({
  name: 'workspace import sites',
  children: [
    it({
      name: 'every cross-package import names the /ts source subpath',
      fn: async () => {
        expect(builtOutputImportSites(),)
          .toEqual([],);
      },
    },),
  ],
},);
