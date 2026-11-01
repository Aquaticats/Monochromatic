/**
 Guards the one way the package lists a directory (ledger B65): through
 `corpus-run/directory-listing.ts`, which hands each reader only the kind of
 entry its writer makes. Readers that called `readdir` for themselves took a
 directory named like a record as one, read a record twice through a symlink,
 or read a write still in flight under its temporary name, in the artifact
 directory (ledger B64) and then in the ledger, the published tree, the probe
 runs and the slice cache.

 WHAT THE SCAN READS. Every import of a directory-listing function
 (`readdir`, `opendir`, `glob` and their synchronous forms) from `node:fs` or
 `node:fs/promises` in the package's source, by the name imported rather than
 the local alias, and any namespace, default or dynamic import of those
 modules, which would reach the same functions out of the scan's sight.
 Tests and test fixtures build their own fixtures and are not read. The
 walkers named here must see every kind of entry, each for the reason given.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

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
  isTreeNode,
  literalText,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectFindingsAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Modules whose listing functions the scan reads.
 */
const FS_MODULES: ReadonlySet<string> = new Set(['node:fs', 'node:fs/promises',],);

/**
 Functions that list a directory.
 */
const LISTING_NAMES: ReadonlySet<string> = new Set([
  'glob',
  'globSync',
  'opendir',
  'opendirSync',
  'readdir',
  'readdirSync',
],);

/**
 Imports allowed to list a directory for themselves, each as `path: name`,
 with why.
 */
const WALKERS: Readonly<Record<string, string>> = {
  'run/corpus-run/cap-census-walk.ts: readdir':
    'walks the log directories named on its command line by stat, following links into other '
      + 'agents\' directories on purpose, and takes every kind of entry as a path to visit',
  'glossary/corpus-run/directory-listing.ts: readdir': 'the one lister, handing each reader the kind of entry it takes',
  'glossary/corpus-run/pipeline-digest.ts: readdir':
    'digests the whole build recursively and must see every entry to refuse a link or special file '
      + 'it cannot identify',
};

/**
 What one import declaration brings in from a listing module, as the names
 the scan reports.

 @param declaration - import declaration node

 @returns Listing functions imported by name, `*` for a namespace import and
 `default` for a default import; empty for any other module

 @example
 ```ts
 const listed = listingImportsOf({ declaration, },);
 ```
 */
function listingImportsOf({ declaration, }: { readonly declaration: TreeNode; },): readonly string[] {
  if (!FS_MODULES.has(literalText({ node: declaration.source, },),))
    return [];
  return (declaration.specifiers as readonly TreeNode[])
    .flatMap(function reported(specifier,): readonly string[] {
      if (specifier.type === 'ImportNamespaceSpecifier')
        return ['*',];
      if (specifier.type === 'ImportDefaultSpecifier')
        return ['default',];
      /**
       Name imported, whatever the local alias.
       */
      const imported = isTreeNode(specifier.imported,) ? (specifier.imported.name ?? specifier.imported.value) : '';
      return LISTING_NAMES.has(String(imported,),) ? [String(imported,),] : [];
    },);
}

/**
 Directory listings the package's source imports outside the listing module,
 as `path: name`.

 @param files - files read; tests and fixtures are skipped

 @returns Findings, sorted and without repeats

 @example
 ```ts
 const listings = directoryListings({ files, },);
 ```
 */
function directoryListings({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found = new Set<string>();
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Nodes still to visit.
     */
    const pending: TreeNode[] = [parseSource({ file, },).program,];
    while (pending.length > 0) {
      /**
       Node visited now.
       */
      const node = pending.pop() as TreeNode;
      pending.push(...childNodes({ node, },),);
      if (node.type === 'ImportDeclaration')
        for (const name of listingImportsOf({ declaration: node, },))
          found.add(`${file.path}: ${name}`,);
      if ((node.type === 'ImportExpression') && FS_MODULES.has(literalText({ node: node.source, },),))
        found.add(`${file.path}: import()`,);
    }
  }
  return [...found,].toSorted();
}

await describe({
  name: 'directory listings (ledger B65)',
  children: [
    it({
      name: 'FINDS a listing function imported by name or under an alias, a namespace, default or dynamic '
        + 'import of the fs modules, and leaves other fs functions, other modules and tests',
      fn: async () => {
        expect(directoryListings({
          files: [
            {
              path: 'cat.ts',
              text: [
                'import { readdir, readFile, } from \'node:fs/promises\';',
                'import { readdirSync as napList, } from \'node:fs\';',
                'import { opendir, } from \'./not-fs.ts\';',
                'export const nap = [readdir, readFile, napList, opendir,];',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'kitten.ts',
              text: [
                'import * as whiskers from \'node:fs/promises\';',
                'import paws from \'node:fs\';',
                'export const purr = async () => [whiskers, paws, await import(\'node:fs/promises\'),];',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'import { readdir, } from \'node:fs/promises\'; export const nap = readdir;',
              isTest: true,
            },
          ],
        },),).toEqual([
          'cat.ts: readdir',
          'cat.ts: readdirSync',
          'kitten.ts: *',
          'kitten.ts: default',
          'kitten.ts: import()',
        ],);
      },
    },),
    it({
      name: 'FINDS NO DIRECTORY LISTING across the package but the listing module and the walkers named '
        + 'as needing every kind of entry',
      fn: async () => {
        expectFindingsAsListed({
          findings: directoryListings({ files: await readPackageSource(), },),
          listed: Object.keys(WALKERS,).toSorted(),
        },);
      },
    },),
  ],
},);
