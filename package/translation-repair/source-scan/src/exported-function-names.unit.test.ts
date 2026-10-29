/**
 Guards against two exported functions sharing a name (ledger B29): the
 package exported `settleTranslateSlice` from two files that settle a slice
 at different depths, `withoutComments` from two files that strip different
 comment grammars under one signature, and `wordsOf` from two files that
 split words differently. An import of the wrong one compiles and runs, a
 logger tagged with `fn.name` names two functions alike, and a reader or a
 scan keyed by name conflates them (the ledger B29 census did).

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find a shared name (ledger M21). Fixtures are
 cat-themed; the package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Names of the functions one file exports where it declares them: function
 declarations, and constants initialised with a function.

 @param program - parsed program

 @returns Exported function names, in source order

 @example
 ```ts
 const names = exportedFunctionNames({ program, },);
 ```
 */
function exportedFunctionNames({ program, }: { readonly program: TreeNode; },): readonly string[] {
  return (program.body as readonly TreeNode[]).flatMap(function exported(statement,): readonly string[] {
    /**
     What the statement exports, when it declares what it exports.
     */
    const { declaration, } = statement;
    if ((statement.type !== 'ExportNamedDeclaration') || (!isTreeNode(declaration,)))
      return [];
    if ((declaration.type === 'FunctionDeclaration') && isTreeNode(declaration.id,))
      return [declaration.id.name as string,];
    if (declaration.type !== 'VariableDeclaration')
      return [];
    return (declaration.declarations as readonly TreeNode[])
      .filter(function initialisedWithFunction(item,): boolean {
        return isTreeNode(item.id,) && (item.id.type === 'Identifier') && isTreeNode(item.init,)
          && ((item.init.type === 'ArrowFunctionExpression') || (item.init.type === 'FunctionExpression'));
      },)
      .map(function named(item,): string {
        return (item.id as TreeNode).name as string;
      },);
  },);
}

/**
 Exported function names that more than one source file exports, each with
 the files that do, as `name: path | path`.

 @param files - files read; tests and fixtures are skipped

 @returns Shared names, sorted

 @example
 ```ts
 const shared = sharedExportedNames({ files, },);
 ```
 */
function sharedExportedNames({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Files exporting each name.
   */
  const exporters = new Map<string, string[]>();
  for (const file of files) {
    if (file.isTest)
      continue;
    for (const name of exportedFunctionNames({ program: parseSource({ file, },).program, },))
      exporters.set(name, [...(exporters.get(name,) ?? []), file.path,],);
  }
  return [...exporters,]
    .filter(function shared([, paths,],): boolean {
      return paths.length > 1;
    },)
    .map(function described([name, paths,],): string {
      return `${name}: ${paths.toSorted()
        .join(' | ',)}`;
    },)
    .toSorted();
}

await describe({
  name: 'exported function names',
  children: [
    it({
      name: 'FINDS a name two source files export, whether declared or as a function constant, and leaves a name '
        + 'one file keeps private, a test re-declaring a name, and non-function exports',
      fn: async () => {
        expect(sharedExportedNames({
          files: [
            {
              path: 'cat.ts',
              text: [
                'export function nap(): number { return 1; }',
                'export const groom = (): number => 2;',
                'function purr(): number { return 3; }',
                'export const WHISKERS = 4;',
                'export function knead(): number { return purr(); }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'kitten.ts',
              text: [
                'export function nap(): number { return 5; }',
                'export function groom(): number { return 6; }',
                'function purr(): number { return 7; }',
                'export const WHISKERS = 8;',
                'export function pounce(): number { return purr(); }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'export function knead(): number { return 9; }',
              isTest: true,
            },
          ],
        },),).toEqual([
          'groom: cat.ts | kitten.ts',
          'nap: cat.ts | kitten.ts',
        ],);
      },
    },),
    it({
      name: 'FINDS NO NAME TWO SOURCE FILES EXPORT across the package',
      fn: async () => {
        /**
         Every package file, tests among them to be skipped.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: sharedExportedNames({ files, },), },);
      },
    },),
  ],
},);
