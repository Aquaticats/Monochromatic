/**
 Holds the census's list of build entry lists to the packages that have one (ledger B1265). Every package that owns
 runners lists them in its own `src/build-entries.ts`, and the coverage census names the union of those lists in
 `FAMILY_ENTRY_LISTS` (`coverage-census/src/family-build-entries.ts`), or it sees a subset of the runners and counts a
 runner's main as library source. The case once sat in the census's own tests, which listed every package's folder;
 a test of one package that lists the whole family is not run by a gate that reached only that package, so the
 check whose subject is the whole family lives here, where the input is the whole family and the gate runs this
 package's whole `test:unit` whatever changed.

 WHAT THE SCAN READS: the keys of the `new Map([...])` that `FAMILY_ENTRY_LISTS` is declared as, each the first
 element of a pair, and the packages whose `src` holds a `build-entries.ts`. A key a package has no list for, and a
 package with a list no key names, are findings. A source that declares no such map is an error, so the rename of the
 constant cannot turn the scan into a pass.

 OUT OF ITS REACH: a key that is not a string literal; what a list holds (the census's own cases read the merged
 lists through its built entry).

 THE FIXTURE CASES COME FIRST (ledger M21). Fixtures are cat-themed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  packageDirectoryOf,
  pathUnderSrc,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  literalText,
  nodesUnder,
  parseSource,
  type SourceText,
} from './source-scan.test-fixture.ts';
import { expectFindingsAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Family build entries
// The keys of the census's map, and the packages that have a list.

/**
 Location of the census file that declares the map.
 */
const CENSUS_FILE = 'coverage-census/family-build-entries.ts';

/**
 Name of the constant that holds the map.
 */
const LISTS_CONSTANT = 'FAMILY_ENTRY_LISTS';

/**
 The package directories a source names as the owners of a list: the first element of each pair of the map it
 declares.

 @param file - census file

 @returns Keys, in the order written

 @throws {@link Error} when the file declares no `new Map([...])` under the constant's name, since a scan that
 found no keys would pass on any family

 @example
 ```ts
 const owners = ownersNamedBy({ file, },); // ['bench', 'corpus-pass', ...]
 ```
 */
function ownersNamedBy({ file, }: { readonly file: SourceText; }): readonly string[] {
  /**
   The declaration of the constant.
   */
  const declaration = nodesUnder({ root: parseSource({ file, },).program, },)
    .find(function declaresMap(node,): boolean {
      return (node.type === 'VariableDeclarator') && (identifierName({ node: node.id, },) === LISTS_CONSTANT)
        && isTreeNode(node.init,) && (node.init.type === 'NewExpression');
    },);
  if (declaration === undefined)
    throw new Error(`${file.path} declares no new Map under the name ${LISTS_CONSTANT}`,);
  /**
   The array of pairs the map is made from.
   */
  const pairs = childNodes({ node: declaration, },)
    .flatMap(function argumentsOf(node,) {
      return childNodes({ node, },);
    },)
    .find(function isArray(node,): boolean {
      return node.type === 'ArrayExpression';
    },);
  if (pairs === undefined)
    throw new Error(`${file.path} makes ${LISTS_CONSTANT} from no array of pairs`,);
  return childNodes({ node: pairs, },)
    .filter(function isPair(node,): boolean {
      return node.type === 'ArrayExpression';
    },)
    .map(function keyOf(pair,): string {
      return literalText({ node: childNodes({ node: pair, },)[0], },);
    },);
}

/**
 The package directories whose `src` holds a build entry list.

 @param files - every file of the family

 @returns Directories, sorted

 @example
 ```ts
 const owners = ownersWithList({ files, },); // ['bench', 'corpus-pass', ...]
 ```
 */
function ownersWithList({ files, }: { readonly files: readonly SourceText[]; }): readonly string[] {
  return files
    .filter(function isList({ path, },): boolean {
      return pathUnderSrc({ path, },) === 'build-entries.ts';
    },)
    .map(function directoryOf({ path, },): string {
      return packageDirectoryOf({ path, },);
    },)
    .toSorted();
}

//endregion Family build entries

await describe({
  name: 'the census names the build entry list of every package that has one',
  children: [
    it({
      name: 'READS THE KEYS OF THE MAP FROM THE CENSUS SOURCE and the packages with a list from the files, so a key with no '
        + 'list and a list with no key each show',
      fn: async () => {
        /**
         A census source naming two packages.
         */
        const census: SourceText = {
          path: CENSUS_FILE,
          text: [
            'export const FAMILY_ENTRY_LISTS = new Map([',
            '  [\'tabby\', tabbyEntries,],',
            '  [\'calico\', calicoEntries,],',
            '],);',
          ].join('\n',),
          isTest: false,
        };

        expect(ownersNamedBy({ file: census, },),).toEqual(['tabby', 'calico',],);
        expect(ownersWithList({
          files: [
            { path: 'tabby/build-entries.ts', text: '', isTest: false, },
            { path: 'ginger/build-entries.ts', text: '', isTest: false, },
            { path: 'ginger/corpus-run/build-entries.ts', text: '', isTest: false, },
            { path: 'calico/index.ts', text: '', isTest: false, },
          ],
        },),).toEqual(['ginger', 'tabby',],);
        expectFindingsAsListed({
          findings: ownersNamedBy({ file: census, },).toSorted(),
          listed: ['calico', 'tabby',],
        },);
      },
    },),
    it({
      name: 'REFUSES A SOURCE THAT DECLARES NO MAP UNDER THE CONSTANT\'S NAME, so renaming it cannot turn the scan into a pass',
      fn: async () => {
        expect(function readRenamed(): readonly string[] {
          return ownersNamedBy({
            file: {
              path: CENSUS_FILE,
              text: 'export const OTHER_LISTS = new Map([[\'tabby\', tabbyEntries,],],);\n',
              isTest: false,
            },
          },);
        },).toThrow(`${CENSUS_FILE} declares no new Map under the name ${LISTS_CONSTANT}`,);
      },
    },),
    it({
      name: 'READS THE BUILD ENTRIES OF EVERY PACKAGE OF THE FAMILY THAT HAS A LIST, not only the census\'s own: the map '
        + 'names exactly the packages whose src holds a build-entries.ts',
      fn: async () => {
        /**
         Every package file.
         */
        const files = await readPackageSource();
        /**
         The census file among them.
         */
        const census = files.find(function isCensusFile({ path, },): boolean {
          return path === CENSUS_FILE;
        },);
        if (census === undefined)
          throw new Error(`the family holds no ${CENSUS_FILE}`,);
        /**
         The packages with a list.
         */
        const listing = ownersWithList({ files, },);

        expectFindingsAsListed({
          findings: ownersNamedBy({ file: census, },).toSorted(),
          listed: listing,
        },);
        expect(listing.length > 1,).toBe(true,);
      },
    },),
  ],
},);
