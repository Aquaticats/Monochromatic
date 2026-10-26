import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Specifiers that name nothing the check can resolve
// A specifier that is no built-in module, no package the lockfile holds for the family and no file can name a module
// that the search does not read, as an alias in an import map does (`#shared`). The check reports each such specifier
// of an import, an export from, an `import()` and a `require`, and reports nothing for a built-in, a package of the
// lockfile with any subpath, a file the sources hold, a file that exists beside them, a type-only import and a type.
// A package of the family is the exception: it resolves only to a file of the sources.

/**
 The place the check reports for a specifier it cannot resolve.

 @param file - path of the imaginary file

 @param line - line of the specifier

 @param specifier - specifier as written

 @returns Report

 @example
 ```ts
 const spot = unresolvedAt({ file: 'imaginary/src/a.ts', line: 1, specifier: '#a', },);
 ```
 */
function unresolvedAt(
  {
    file,
    line,
    specifier,
  }: {
    readonly file: string;
    readonly line: number;
    readonly specifier: string;
  },
): BlindSpot {
  return {
    file,
    line,
    shape: 'unresolved-module-specifier',
    position: `${specifier} is no built-in module, no package of the lockfile and no file`,
  };
}

/**
 Cases of specifiers that name nothing the check can resolve.
 */
export const SPECIFIER_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A SPECIFIER THAT IS NO BUILT-IN MODULE, NO PACKAGE OF THE LOCKFILE AND NO FILE, in an import, an '
      + 'export from, an import() and a require, and nothing for a built-in, a package of the lockfile, a file, a '
      + 'type-only import or a type',
    files: [
      {
        file: 'imaginary/src/alias-stage.ts',
        text: textOf([
          'import { shared, } from \'#shared\';',
          'export { more, } from \'#more\';',
          'export * from \'missing-package\';',
          'export const loaded = await import(\'#dynamic\');',
          'export const required = require(\'./absent.ts\',);',
        ],),
      },
      {
        file: 'imaginary/src/resolved-stage.ts',
        text: textOf([
          'import { join, } from \'node:path\';',
          'import { readFileSync, } from \'fs\';',
          'import { nonNullishOrThrow, } from \'@monochromatic-dev/module-or-throw/ts\';',
          'import { sibling, } from \'./resolved-sibling.ts\';',
          'import pack from \'../../corpus-pass/package.json\' with { type: \'json\', };',
          'import type { Shared, } from \'#types\';',
          'export type Loaded = typeof import(\'#typed\');',
        ],),
      },
      {
        file: 'imaginary/src/resolved-sibling.ts',
        text: textOf([
          'export const sibling = 1;',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      unresolvedAt({
        file: 'imaginary/src/alias-stage.ts',
        line: 1,
        specifier: '#shared',
      },),
      unresolvedAt({
        file: 'imaginary/src/alias-stage.ts',
        line: 2,
        specifier: '#more',
      },),
      unresolvedAt({
        file: 'imaginary/src/alias-stage.ts',
        line: 3,
        specifier: 'missing-package',
      },),
      unresolvedAt({
        file: 'imaginary/src/alias-stage.ts',
        line: 4,
        specifier: '#dynamic',
      },),
      unresolvedAt({
        file: 'imaginary/src/alias-stage.ts',
        line: 5,
        specifier: './absent.ts',
      },),
    ],
  },
  {
    name: 'REPORTS THE SPECIFIER OF AN IMPORT-EQUALS DECLARATION, OF A SCOPED PACKAGE THE LOCKFILE DOES NOT HOLD AND '
      + 'OF A PACKAGE ONLY A TOOL OF THE REPOSITORY HOLDS',
    files: [
      {
        file: 'imaginary/src/equals-stage.ts',
        text: textOf([
          'import equalled = require(\'#equalled\');',
          'import { more, } from \'@monochromatic-dev/does-not-exist/ts\';',
          'import { lint, } from \'oxlint\';',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      unresolvedAt({
        file: 'imaginary/src/equals-stage.ts',
        line: 1,
        specifier: '#equalled',
      },),
      unresolvedAt({
        file: 'imaginary/src/equals-stage.ts',
        line: 2,
        specifier: '@monochromatic-dev/does-not-exist/ts',
      },),
      unresolvedAt({
        file: 'imaginary/src/equals-stage.ts',
        line: 3,
        specifier: 'oxlint',
      },),
    ],
  },
  {
    name: 'REPORTS A SPECIFIER OF A PACKAGE OF THE FAMILY THAT NAMES NO FILE OF THE SOURCES, a subpath that does not '
      + 'exist, one outside the exports map and a package nothing holds, and nothing for the package, its /ts subpath '
      + 'and a /ts file the sources hold: the lockfile lists the family\'s packages for each other, which says '
      + 'nothing of the files they hold',
    files: [
      {
        file: 'imaginary/src/family-stage.ts',
        text: textOf([
          'import make from \'@monochromatic-dev/translation-repair-lane-contest/ts/nothing.ts\';',
          'import pack from \'@monochromatic-dev/translation-repair-lane-contest/package.json\';',
          'export { built, } from \'@monochromatic-dev/translation-repair-lane-contest/dist/index.mjs\';',
          'export const loaded = await import(\'@monochromatic-dev/translation-repair-lane-contest/ts/absent.ts\');',
          'import { absent, } from \'@monochromatic-dev/translation-repair-nonesuch/ts\';',
        ],),
      },
      {
        file: 'provider/src/index.ts',
        text: textOf([
          'export const buildClaudeBody = 1;',
        ],),
      },
      {
        file: 'imaginary/src/family-resolved-stage.ts',
        text: textOf([
          'import { buildClaudeBody, } from \'@monochromatic-dev/translation-repair-provider\';',
          'import { buildClaudeBody as again, } from \'@monochromatic-dev/translation-repair-provider/ts\';',
          'import { wire, } from \'@monochromatic-dev/translation-repair-lane-contest/ts/contest-ballot-wire.ts\';',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      unresolvedAt({
        file: 'imaginary/src/family-stage.ts',
        line: 1,
        specifier: '@monochromatic-dev/translation-repair-lane-contest/ts/nothing.ts',
      },),
      unresolvedAt({
        file: 'imaginary/src/family-stage.ts',
        line: 2,
        specifier: '@monochromatic-dev/translation-repair-lane-contest/package.json',
      },),
      unresolvedAt({
        file: 'imaginary/src/family-stage.ts',
        line: 3,
        specifier: '@monochromatic-dev/translation-repair-lane-contest/dist/index.mjs',
      },),
      unresolvedAt({
        file: 'imaginary/src/family-stage.ts',
        line: 4,
        specifier: '@monochromatic-dev/translation-repair-lane-contest/ts/absent.ts',
      },),
      unresolvedAt({
        file: 'imaginary/src/family-stage.ts',
        line: 5,
        specifier: '@monochromatic-dev/translation-repair-nonesuch/ts',
      },),
    ],
  },
];

//endregion Specifiers that name nothing the check can resolve
