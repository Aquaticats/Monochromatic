/**
 Guards against imports nothing reads (ledger B32). No configured check
 reports one: the repository's lint leaves `no-unused-vars` off and its
 TypeScript configuration leaves `noUnusedLocals` false (GitHub issue 578),
 so imports outlived the code that read them, four type imports in the
 consolidate wire among them (ledger B30).

 A binding is read when its local name stands anywhere in the file outside
 its import declarations, type positions and export lists included, or
 when a TSDoc link names it (`{@link Name}`), since a `@throws` line may be
 the only place an error class a callee throws is named.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each kind (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source, tests included.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { isIdentifierCharacter, } from './source-text-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 What opens a TSDoc link.
 */
const LINK_OPENER = '{@link ';

/**
 Names the file's TSDoc links point at: the identifier after each
 `{@link `, up to its first character that continues no name.

 @param text - file text

 @returns Linked names, each once

 @example
 ```ts
 const linked = linkedNames({ text: '@throws {@link CatError} when asleep', },); // CatError
 ```
 */
function linkedNames({ text, }: { readonly text: string; },): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const names = new Set<string>();
  /**
   Start of the link opener read now, -1 once none is left.
   */
  let at = text.indexOf(LINK_OPENER,);
  while (at !== (-1)) {
    /**
     First character of the linked name.
     */
    const start = at + LINK_OPENER.length;
    /**
     One past the linked name's last character.
     */
    let end = start;
    while ((end < text.length) && isIdentifierCharacter({ character: text.charAt(end,), },))
      end += 1;
    if (end > start)
      names.add(text.slice(start, end,),);
    at = text.indexOf(LINK_OPENER, end,);
  }
  return names;
}

/**
 How often each identifier stands in a file outside its import declarations.

 @param program - parsed program

 @returns Count per name

 @example
 ```ts
 const counts = namesOutsideImports({ program, },);
 ```
 */
function namesOutsideImports({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, number> {
  /**
   Counts so far.
   */
  const counts = new Map<string, number>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = (program.body as readonly TreeNode[]).filter(function notImport(statement,): boolean {
    return statement.type !== 'ImportDeclaration';
  },);
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const node = pending.pop() as TreeNode;
    if ((node.type === 'Identifier') && ((typeof node.name) === 'string'))
      counts.set(node.name as string, (counts.get(node.name as string,) ?? 0) + 1,);
    pending.push(...childNodes({ node, },),);
  }
  return counts;
}

/**
 Import bindings nothing in their file reads, across a set of files.

 @param files - package's files

 @returns Each as `path#name`, sorted

 @example
 ```ts
 const unused = unusedImports({ files, },);
 ```
 */
function unusedImports({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  return files
    .flatMap(function unusedHere(file,): readonly string[] {
      /**
       The file's syntax tree.
       */
      const { program, } = parseSource({ file, },);
      /**
       Identifier counts outside the imports.
       */
      const counts = namesOutsideImports({ program, },);
      /**
       Names the file's TSDoc links point at.
       */
      const linked = linkedNames({ text: file.text, },);
      return (program.body as readonly TreeNode[])
        .filter(function isImport(statement,): boolean {
          return statement.type === 'ImportDeclaration';
        },)
        .flatMap(function bindings(statement,): readonly string[] {
          return ((statement.specifiers ?? []) as readonly TreeNode[]).map(function localName(specifier,): string {
            return (specifier.local as TreeNode).name as string;
          },);
        },)
        .filter(function unread(name,): boolean {
          return ((counts.get(name,) ?? 0) === 0) && (!linked.has(name,));
        },)
        .map(function located(name,): string {
          return `${file.path}#${name}`;
        },);
    },)
    .toSorted();
}

/**
 A fixture file, as the scan reads one.

 @param path - file name

 @param lines - file lines

 @returns Source file

 @example
 ```ts
 const file = fixture({ path: 'cat.ts', lines: ['export const nap = 1;',], },);
 ```
 */
function fixture({ path, lines, }: { readonly path: string; readonly lines: readonly string[]; },): SourceText {
  return {
    path,
    text: lines.join('\n',),
    isTest: false,
  };
}

/**
 The cat fixtures: one file holding an import binding of every kind, read
 and unread.
 */
const CAT_FILES: readonly SourceText[] = [
  fixture({
    path: 'cat.ts',
    lines: [
      'import \'./side-effect.ts\';',
      'import { nap, purr, } from \'./sleep.ts\';',
      'import type { Whisker, Paw, } from \'./body.ts\';',
      'import litter from \'./litter.ts\';',
      'import * as bowl from \'./bowl.ts\';',
      'import { groom as tidy, stretch, } from \'./care.ts\';',
      'import { CatError, } from \'./errors.ts\';',
      'import { knead, } from \'./knead.ts\';',
      'export { knead, };',
      '/**',
      ' Wakes the cat.',
      '',
      ' @throws {@link CatError} when the cat is asleep',
      ' */',
      'export function wake(paw: Paw): number {',
      '  return nap() + paw.length;',
      '}',
    ],
  },),
];

await describe({
  name: 'imports nothing reads',
  children: [
    it({
      name: 'FINDS a value import, a type import, a default import, a namespace import and a renamed import '
        + 'nothing reads; KEEPS one read in code, one read only in a type, one named only in a TSDoc link, '
        + 'one re-exported by an export list, and a bare side-effect import',
      fn: async () => {
        expect(unusedImports({ files: CAT_FILES, },),).toEqual([
          'cat.ts#Whisker',
          'cat.ts#bowl',
          'cat.ts#litter',
          'cat.ts#purr',
          'cat.ts#stretch',
          'cat.ts#tidy',
        ],);
      },
    },),
    it({
      name: 'READS a TSDoc link\'s name up to the first character that continues no name',
      fn: async () => {
        expect([...linkedNames({ text: 'see {@link Cat.nap} and {@link Kitten_2$}; {@link } is empty', },),],)
          .toEqual(['Cat', 'Kitten_2$',],);
      },
    },),
    it({
      name: 'FINDS NO IMPORT NOTHING READS across the package\'s source and tests',
      fn: async () => {
        /**
         Every package file.
         */
        const files = await readPackageSource();
        expect(files.length,).toBeGreaterThan(0,);
        expectNoFindings({ findings: unusedImports({ files, },), },);
      },
    },),
  ],
},);
