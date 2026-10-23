/**
 Guards the identity of the classes the family tests with `instanceof` (round 7 of the D4 and E2 work, the cross-package
 census behind it).

 A library entry of a package keeps a sibling external only when the sibling is listed under that package's
 `dependencies`; every other workspace import is bundled into its dist, and a class imported from it is a COPY. An
 `instanceof` against that copy is false for the real instances the sibling's own dist makes, and the failure is
 silent: a refusal that ends the run is taken for an ordinary failure, a bench error that the pass could park on is not
 parked, and a unit test written against the same copy passes while production differs. The slice-cache package met it
 when its test of `EveryProviderDryError` printed no return fields until the check went through stage's `isDryRefusal`.

 WHAT THE SCAN READS, in the family's source (tests and their fixtures are left out): every `instanceof` against a
 name imported from a workspace package, and every `.constructor` comparison against such a name, by `===`, `!==`,
 `==` or `!=`, with or without an optional chain on the read.
 A site passes only when the class's package is a family sibling (`@monochromatic-dev/translation-repair-*`) listed
 under the importing package's `dependencies`. A `module-*` package is bundled whatever the listing says, so it never
 passes. A structural guard function that the owning package exports (`isDryRefusal`) is a call and not a site.

 THE FIXTURE CASES COME FIRST, so the package-wide case is read against a scan shown able to find each form (ledger
 M21). The package-wide case must then reach a floor of sites and packages and a few named sites (QIV), so an empty
 result cannot come from a wrong glob or an unparsed file. Fixtures are cat-themed.

 @module
 */

import { readFile, } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  familyPackages,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  literalText,
  memberName,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Bundled class identity
// The scan itself: which names a file imports from a workspace package, which of them an `instanceof` or a
// `.constructor` comparison tests, and whether the importing package keeps that package external.

/**
 Prefix of the family's own packages, the only workspace packages a library build keeps external.
 */
const FAMILY_PREFIX = '@monochromatic-dev/translation-repair-';

/**
 Scope every workspace package is published under.
 */
const WORKSPACE_SCOPE = '@monochromatic-dev/';

/**
 Names the importing package lists under `dependencies`, per package directory.
 */
type Manifests = ReadonlyMap<string, readonly string[]>;

/**
 What the scan found in the files it was given.
 */
type ClassIdentityReading = {
  /**
   Every site that tests a class imported from a workspace package, as `path#Class`, repeated once per site.
   */
  readonly sites: readonly string[];

  /**
   The sites whose class is bundled into the importing package's dist, each saying which and why.
   */
  readonly findings: readonly string[];
};

/**
 The package a specifier names.

 @param specifier - import specifier, such as `@monochromatic-dev/translation-repair-run/ts`

 @returns Scope and the package name, without any subpath

 @example
 ```ts
 const owner = ownerOf({ specifier: '@monochromatic-dev/translation-repair-run/ts', },);
 ```
 */
function ownerOf({ specifier, }: { readonly specifier: string; },): string {
  return specifier.split('/',)
    .slice(
    0,
    2,
    )
    .join('/',);
}

/**
 The names a file imports as values from a workspace package, each with that package.

 @param program - parsed file

 @returns Local name to owning package, type-only imports left out

 @example
 ```ts
 const imported = workspaceValueImports({ program, },);
 ```
 */
function workspaceValueImports({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, string> {
  /**
   Names found so far.
   */
  const found = new Map<string, string>();
  for (const node of nodesUnder({ root: program, },)) {
    if ((node.type !== 'ImportDeclaration') || (node.importKind === 'type'))
      continue;
    /**
     The specifier the declaration imports from.
     */
    const specifier = literalText({ node: node.source, },);
    if (!specifier.startsWith(WORKSPACE_SCOPE,))
      continue;
    /**
     The specifiers it binds.
     */
    const bound = Array.isArray(node.specifiers,) ? node.specifiers as readonly unknown[] : [];
    for (const binding of bound) {
      if ((!isTreeNode(binding,)) || (binding.importKind === 'type'))
        continue;
      /**
       Name the file reads it by.
       */
      const local = identifierName({ node: binding.local, },);
      if (local !== '')
        found.set(
          local,
          ownerOf({ specifier, },),
        );
    }
  }
  return found;
}

/**
 The operators that compare a `.constructor` read with a class, strict and loose.
 */
const CONSTRUCTOR_COMPARISONS: ReadonlySet<unknown> = new Set(['===', '!==', '==', '!=',],);

/**
 A node without the optional chain that wraps it, so `caught?.constructor` reads as the member read it holds.

 @param value - node, or whatever a field held

 @returns Member read inside a chain, or the value unchanged

 @example
 ```ts
 const read = withoutChain({ value: node.left, },); // the MemberExpression of `caught?.constructor`
 ```
 */
function withoutChain({ value, }: { readonly value: unknown; },): unknown {
  return (isTreeNode(value,) && (value.type === 'ChainExpression')) ? value.expression : value;
}

/**
 The class a comparison tests, when it is one of the two forms the scan reads.

 The forms read: `x instanceof Class`, and a `.constructor` read (plain, through `?.`, or by a string key) compared
 with `Class` on either side by `===`, `!==`, `==` or `!=`.

 Outside the scan, each shown by a planted file to give no site, and the reason for each:
 a namespace import (`p.BError`), because the class is a member read and not a name the file imports; a re-export hop
 (the file imports the name from a local file that re-exports it from the workspace package), because the scan
 follows no import past the file; a class held in a variable or passed as a parameter (`const A = BError`,
 `instanceof cls`), because its origin is a value flow and the scan reads names; a prototype comparison
 (`Object.getPrototypeOf(e) === B.prototype`), because it compares a prototype and not a class; a dynamic
 `await import()`, because the binding is a destructuring of a call result and not an import declaration; and the
 thrower side (`extends B` or `new B` in a package that bundles `b`), because it derives from or builds the copy and
 compares nothing, so no test is made there.

 Also outside the scan, each shown by a planted file to give no site, one reason each:
 - a conditional class operand (`e instanceof (c ? B : Error)`): the operand is a conditional, not a name
 - an operand in parentheses (`e instanceof (B)`, `(e.constructor) === B`): only an optional chain is unwrapped
 - an operand under an `as` assertion (`(e.constructor as unknown) === B`): the assertion wraps the node it reads
 - an operand under a `!` assertion (`e.constructor! === B`, `e.constructor === B!`): wrapped as the `as` form is
 - `B.prototype.isPrototypeOf(e)`: a method call on a prototype, not a comparison
 - `B[Symbol.hasInstance](e)`: a call on a computed member, not a comparison
 - `switch (e.constructor) { case B: }`: the class is a `case` label, not a comparison
 - `[B].includes(e.constructor)`: the class is an array element, not a comparison

 @param node - node read

 @returns Name of the class tested against, empty where the node tests none

 @example
 ```ts
 const tested = testedClassOf({ node, },); // 'BError' for `caught instanceof BError`
 ```
 */
function testedClassOf({ node, }: { readonly node: TreeNode; },): string {
  if (node.type !== 'BinaryExpression')
    return '';
  if (node.operator === 'instanceof')
    return identifierName({ node: node.right, },);
  if (!CONSTRUCTOR_COMPARISONS.has(node.operator,))
    return '';
  /**
   The class named on either side of a `.constructor` read.
   */
  const sides = [
    [withoutChain({ value: node.left, },), node.right,],
    [withoutChain({ value: node.right, },), node.left,],
  ] as const;
  for (const [read, named,] of sides) {
    if (isTreeNode(read,) && (read.type === 'MemberExpression') && (memberName({ node: read, },) === 'constructor'))
      return identifierName({ node: named, },);
  }
  return '';
}

/**
 Reads the class tests of the files given against the packages their importers keep external.

 @param files - files read; tests and fixtures among them are passed over

 @param manifests - names each package directory lists under `dependencies`

 @returns Every site and the ones that test a bundled copy

 @example
 ```ts
 const { sites, findings, } = classIdentityOf({ files, manifests, },);
 ```
 */
function classIdentityOf(
  {
    files,
    manifests,
  }: {
    readonly files: readonly SourceText[];
    readonly manifests: Manifests;
  },
): ClassIdentityReading {
  /**
   Sites and findings so far.
   */
  const sites: string[] = [];
  const findings: string[] = [];
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     Parsed file.
     */
    const { program, } = parseSource({ file, },);
    /**
     Names it imports from workspace packages.
     */
    const imported = workspaceValueImports({ program, },);
    /**
     Package directory the file sits in, which names its manifest.
     */
    const [directory = '',] = file.path.split('/',);
    /**
     Packages that directory keeps external.
     */
    const external = manifests.get(directory,) ?? [];
    for (const node of nodesUnder({ root: program, },)) {
      /**
       Class this node tests.
       */
      const tested = testedClassOf({ node, },);
      /**
       Package the class is imported from, none where it is not an import from a workspace package.
       */
      const owner = imported.get(tested,);
      if (owner === undefined)
        continue;
      sites.push(`${file.path}#${tested}`,);
      if (owner.startsWith(FAMILY_PREFIX,) && external.includes(owner,))
        continue;
      findings.push(
        `${file.path}#${tested}: imported from ${owner}, which ${directory} ${
          owner.startsWith(FAMILY_PREFIX,)
            ? 'does not list under dependencies, so its dist bundles a copy'
            : 'always bundles, so its dist holds a copy'
        }; test the class through a guard its owner exports, or list the owner under dependencies`,
      );
    }
  }
  return {
    sites: sites.toSorted(),
    findings: findings.toSorted(),
  };
}

/**
 The `dependencies` of every family package.

 @returns Package directory to the names it lists under `dependencies`

 @example
 ```ts
 const manifests = await readManifests();
 ```
 */
async function readManifests(): Promise<Manifests> {
  /**
   Directory holding the family's packages.
   */
  const familyRoot = resolve(
    import.meta.dirname,
    '..',
    '..',
  );
  return new Map(
    await Promise.all((await familyPackages()).map(
      async function readManifest(directory,): Promise<readonly [string, readonly string[],]> {
      /**
       The manifest, parsed.
       */
        const manifest: unknown = JSON.parse(await readFile(
          join(
            familyRoot,
            directory,
            'package.json',
          ),
          'utf8',
        ),);
      /**
       Its dependencies field.
       */
        const dependencies = isJsonRecord(manifest,) ? manifest.dependencies : undefined;
        return [directory, isJsonRecord(dependencies,) ? Object.keys(dependencies,) : [],];
      },
    ),),
  );
}

//endregion Bundled class identity

/**
 The fewest sites the package-wide case accepts. The census found 48 at the head (round 7), and the floor keeps margin
 below it: the remedy the scan's own message gives, testing the class through a guard its owner exports, removes a site,
 so a floor at the count would fail the next fix for a reason that reads as a defect in that fix. A fix through an
 owner's guard may lower the floor in the same commit, with the reason in the message.
 */
const SITE_FLOOR = 40;

/**
 The fewest packages those sites sit in. The census found 11 at the head (round 7); the lowering rule of the site floor
 applies to this floor too.
 */
const PACKAGE_FLOOR = 8;

/**
 Sites the package-wide case must find by name, as `path#Class` with the path as `readPackageSource` gives it (the
 package directory, then the path under `src`): one in each of three packages, each a class from another family
 package. A floor on the count alone would pass with these gone, so a scan that stopped reading one whole package
 would go unseen. A site that a fix through an owner's guard removes leaves this list in the same commit, and the
 commit message gives the reason.
 */
const ANCHORED_SITES = [
  'slice-cache/corpus-run/entry-error-outcome.ts#WritingBenchUnreachableError',
  'provider/provider-budget-refusal.ts#ProviderHttpError',
  'stage/stage-run-end.ts#EveryProviderDryError',
] as const;

/**
 One file importing a class from the family package `b` and testing it.

 @param test - test expression, using `BError`

 @returns File, of package `a`

 @example
 ```ts
 const file = catFile({ test: 'caught instanceof BError', },);
 ```
 */
function catFile({ test, }: { readonly test: string; },): SourceText {
  return {
    path: 'a/cat.ts',
    text: [
      'import { BError, } from \'@monochromatic-dev/translation-repair-b/ts\';',
      `export const isCat = (caught: unknown,): boolean => ${test};`,
    ].join('\n',),
    isTest: false,
  };
}

await describe({
  name: 'class identity across bundled packages',
  children: [
    it({
      name: 'FINDS A CLASS TESTED THAT THE IMPORTER DOES NOT KEEP EXTERNAL: a sibling absent from dependencies (it '
        + 'is listed under devDependencies only, or not at all) is bundled, and the same file passes when the '
        + 'sibling is listed',
      fn: async () => {
        /**
         The package-a manifests without and with the sibling.
         */
        const without: Manifests = new Map([['a', ['@monochromatic-dev/translation-repair-z',],],],);
        const unlisted: Manifests = new Map();
        const listed: Manifests = new Map([['a', ['@monochromatic-dev/translation-repair-b',],],],);
        /**
         The file under test.
         */
        const files = [catFile({ test: 'caught instanceof BError', },),];
        expect(classIdentityOf({ files, manifests: without, },).findings,).toHaveLength(1,);
        expect(classIdentityOf({ files, manifests: without, },).findings[0],).toContain(
          'a/cat.ts#BError: imported from @monochromatic-dev/translation-repair-b, which a does not list',
        );
        expect(classIdentityOf({ files, manifests: unlisted, },).findings,).toHaveLength(1,);
        expect(classIdentityOf({ files, manifests: listed, },),).toEqual({
          sites: ['a/cat.ts#BError',],
          findings: [],
        },);
      },
    },),

    it({
      name: 'FINDS A .constructor COMPARISON AGAINST A BUNDLED CLASS on either side, and a module-* package whatever '
        + 'the listing says, since a module-* package is always bundled',
      fn: async () => {
        /**
         Package a lists the sibling under dependencies.
         */
        const listed: Manifests = new Map([['a', ['@monochromatic-dev/translation-repair-b',],],],);
        expect(classIdentityOf({
          files: [catFile({ test: 'caught.constructor === BError', },),],
          manifests: new Map(),
        },).findings,).toHaveLength(1,);
        expect(classIdentityOf({
          files: [catFile({ test: 'BError !== caught.constructor', },),],
          manifests: new Map(),
        },).findings,).toHaveLength(1,);
        expect(classIdentityOf({
          files: [catFile({ test: 'caught.constructor === BError', },),],
          manifests: listed,
        },).findings,).toEqual([],);
        /**
         A module-* package listed under dependencies is still bundled.
         */
        const moduleFile: SourceText = {
          path: 'a/cat.ts',
          text: [
            'import { ModuleError, } from \'@monochromatic-dev/module-x/ts\';',
            'export const isCat = (caught: unknown,): boolean => caught instanceof ModuleError;',
          ].join('\n',),
          isTest: false,
        };
        expect(classIdentityOf({
          files: [moduleFile,],
          manifests: new Map([['a', ['@monochromatic-dev/module-x',],],],),
        },).findings,).toHaveLength(1,);
      },
    },),

    it({
      name: 'FINDS A .constructor COMPARISON READ THROUGH AN OPTIONAL CHAIN, and one written with == or !=, since '
        + 'each is the same test of a bundled copy as the strict form',
      fn: async () => {
        /**
         Package a lists no sibling, so every site found is a finding.
         */
        const unlisted: Manifests = new Map();
        /**
         The forms planted, each of which must give exactly one finding.
         */
        const planted = [
          'caught?.constructor === BError',
          'BError === caught?.constructor',
          'caught?.constructor !== BError',
          'caught.constructor == BError',
          'BError != caught.constructor',
          'caught?.constructor != BError',
        ];
        expect(planted.filter(function missed(test,): boolean {
          return classIdentityOf({ files: [catFile({ test, },),], manifests: unlisted, },).findings
            .length
            !== 1;
        },),).toEqual([],);
      },
    },),

    it({
      name: 'LEAVES WHAT IS NOT A CLASS TEST OF A WORKSPACE IMPORT: a class defined in the file, a guard function '
        + 'the owner exports, a type-only import, a name that is not imported, and tests',
      fn: async () => {
        /**
         Files none of which tests a bundled workspace class.
         */
        const files: readonly SourceText[] = [
          catFile({ test: 'isBError(caught,)', },),
          {
            path: 'a/local.ts',
            text: 'class LocalError extends Error {}\nexport const isLocal = (c: unknown) => c instanceof LocalError;',
            isTest: false,
          },
          {
            path: 'a/type.ts',
            text: [
              'import type { BError, } from \'@monochromatic-dev/translation-repair-b/ts\';',
              'export const isCat = (c: unknown) => c instanceof Error;',
              'export type Cat = BError;',
            ].join('\n',),
            isTest: false,
          },
          {
            path: 'a/unimported.ts',
            text: 'export const isCat = (c: unknown) => c instanceof BError;',
            isTest: false,
          },
          {
            path: 'a/cat.unit.test.ts',
            text: [
              'import { BError, } from \'@monochromatic-dev/translation-repair-b\';',
              'export const mock = (c: unknown) => c instanceof BError;',
            ].join('\n',),
            isTest: true,
          },
        ];
        expect(classIdentityOf({ files, manifests: new Map(), },),).toEqual({ sites: [], findings: [], },);
      },
    },),

    it({
      name: 'TESTS A CLASS IMPORTED FROM A WORKSPACE PACKAGE ONLY WHERE THE IMPORTER KEEPS THAT PACKAGE EXTERNAL, '
        + 'across the family, and reaches the floor of sites and packages and the named sites',
      fn: async () => {
        /**
         Every package file, tests among them.
         */
        const files = await readPackageSource();
        /**
         What the family's sources test, against what each package lists.
         */
        const reading = classIdentityOf({
          files,
          manifests: await readManifests(),
        },);
        expectNoFindings({ findings: reading.findings, },);
        // REACH: a wrong glob or an unparsed file would leave this short, and an empty finding list would pass.
        expect(reading.sites.length,).toBeGreaterThanOrEqual(SITE_FLOOR,);
        expect(new Set(reading.sites.map(function packageOf(site,): string {
          return site.split('/',)[0] ?? '';
        },),).size,).toBeGreaterThanOrEqual(PACKAGE_FLOOR,);
        for (const anchored of ANCHORED_SITES)
          expect(reading.sites,).toContain(anchored,);
      },
    },),
  ],
},);
