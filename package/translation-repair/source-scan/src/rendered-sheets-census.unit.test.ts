/**
 Guards ledger X17: every sheet builder the package defines is rendered by the
 rendered-sheets fixtures, so a guard over "every model-facing sheet" reads
 every one. The fixture once claimed every sheet and rendered fifteen, and a
 British spelling in the picture readers' instruction went unread.

 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  pathUnderSrc,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  nodesUnder,
  parseSource,
  type SourceText,
} from './source-scan.test-fixture.ts';
import { expectRecordAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Declaration that opens a sheet builder.
 */
const BUILDER_OPENING = 'export function build';

/**
 Suffix every sheet builder's name carries.
 */
const BUILDER_SUFFIX = 'Messages';

/**
 Names of the sheet builders one source text declares, read by an index scan.

 @param text - source file

 @returns Each `build…Messages` function it exports

 @example
 ```ts
 const names = buildersIn({ text: 'export function buildCatMessages(', },); // ['buildCatMessages']
 ```
 */
function buildersIn({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Names found.
   */
  const names: string[] = [];
  for (
    let at = text.indexOf(BUILDER_OPENING,);
    at !== (-1);
    at = text.indexOf(
      BUILDER_OPENING,
      at + BUILDER_OPENING.length,
    )
  ) {
    /**
     Where the name ends: the parameter list or a type parameter.
     */
    const nameStart = at + 'export function '.length;
    /**
     End of the identifier.
     */
    const open = [
      text.indexOf('(', nameStart,),
      text.indexOf('<', nameStart,),
    ].filter(function found(index,): boolean {
      return index !== (-1);
    },)
      .reduce(function first(least, index,): number {
        return Math.min(least, index,);
      }, text.length,);
    /**
     The function's name.
     */
    const name = text.slice(nameStart, open,)
      .trim();
    if (name.endsWith(BUILDER_SUFFIX,))
      names.push(name,);
  }
  return names;
}

/**
 Names every fixture whose file name opens with a prefix calls, read from the syntax tree so that a name in a comment,
 in a string or inside a longer name is not a call.

 @param files - every file of the family

 @param prefix - how the fixtures' file names open

 @returns Each name a call expression of a fixture calls plainly

 @example
 ```ts
 const called = namesCalledBy({ files, prefix: 'rendered-sheets', },); // Set { 'buildCatMessages', }
 ```
 */
function namesCalledBy(
  {
    files,
    prefix,
  }: {
    readonly files: readonly SourceText[];
    readonly prefix: string;
  },
): ReadonlySet<string> {
  return new Set(
    files
      .filter(function isSheetFixture({ path, },): boolean {
        /**
         Path under the package's source directory.
         */
        const name = pathUnderSrc({ path, },);
        return name.startsWith(prefix,) && name.endsWith('.test-fixture.ts',);
      },)
      .flatMap(function callsOf(file,): readonly string[] {
        return nodesUnder({ root: parseSource({ file, },).program, },)
          .filter(function isCall({ type, },): boolean {
            return type === 'CallExpression';
          },)
          .map(function calleeName({ callee, },): string {
            return identifierName({ node: callee, },);
          },);
      },),
  );
}

await describe({
  name: 'rendered-sheets census (ledger X17)',
  children: [
    it({
      name: 'RENDERS EVERY SHEET BUILDER THE PACKAGE DEFINES, so "every sheet" means every sheet',
      fn: async () => {
        /**
         Every file of every family package.
         */
        const files = await readPackageSource();
        /**
         Every builder, by name.
         */
        const builders = files
          .filter(function isSource({ isTest, },): boolean {
            return !isTest;
          },)
          .flatMap(function buildersOf({ text, },): readonly string[] {
            return buildersIn({ text, },);
          },);
        /**
         Names the sheet fixtures call.
         */
        const called = namesCalledBy({ files, prefix: 'rendered-sheets', },);
        expectRecordAsListed({
          found: {
            // The census reads the builders at all: two it must find.
            readsBuilders: builders.includes('buildCriticMessages',)
              && builders.includes('buildArchiveBlockReviewMessages',),
            unrendered: builders.filter(function unrendered(name,): boolean {
              return !called.has(name,);
            },),
          },
          listed: {
            readsBuilders: true,
            unrendered: [],
          },
        },);
      },
    },),
    it({
      name: 'RENDERS EVERY SHEET BUILDER FOR TWO SUBJECTS in the system prompt guard (R4), so a stage whose system text '
        + 'embeds the words of one call cannot be added without the guard reading it',
      fn: async () => {
        /**
         Every file of every family package.
         */
        const files = await readPackageSource();
        /**
         Every builder, by name.
         */
        const builders = files
          .filter(function isSource({ isTest, },): boolean {
            return !isTest;
          },)
          .flatMap(function buildersOf({ text, },): readonly string[] {
            return buildersIn({ text, },);
          },);
        /**
         Names the system prompt guard's stage fixtures call.
         */
        const called = namesCalledBy({ files, prefix: 'system-prompt-stages', },);
        expectRecordAsListed({
          found: {
            unguarded: builders.filter(function unguarded(name,): boolean {
              return !called.has(name,);
            },),
          },
          listed: { unguarded: [], },
        },);
      },
    },),
    it({
      name: 'COUNTS A BUILDER ONLY WHERE A FIXTURE CALLS IT, not where a comment, a string or a longer name holds its '
        + 'name (F11 of the re-check of the second fix round)',
      fn: async () => {
        /**
         Fixture that calls one builder and names three others without calling them.
         */
        const fixture: SourceText = {
          path: 'cat-package/rendered-sheets-cats.test-fixture.ts',
          text: [
            '// buildCommentedMessages(): not called',
            'const note = \'buildQuotedMessages(\';',
            'rebuildPrefixedMessages({});',
            'buildCalledMessages({});',
          ].join('\n',),
          isTest: true,
        };
        /**
         Names the fixture calls.
         */
        const called = namesCalledBy({ files: [fixture,], prefix: 'rendered-sheets', },);
        /**
         Rendered or not, for each builder the fixture holds.
         */
        const rendered = Object.fromEntries(
          [
            'buildCalledMessages',
            'buildCommentedMessages',
            'buildQuotedMessages',
            'buildPrefixedMessages',
          ].map(function readOne(name,): readonly [string, boolean] {
            return [name, called.has(name,),];
          },),
        );
        expectRecordAsListed({
          found: rendered,
          listed: {
            buildCalledMessages: true,
            buildCommentedMessages: false,
            buildQuotedMessages: false,
            buildPrefixedMessages: false,
          },
        },);
      },
    },),
  ],
},);
