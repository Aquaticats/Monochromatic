import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Specifiers that name nothing the check can resolve
// A specifier that is no built-in module, no package the lockfile holds for the family and no file can name a module
// that the search does not read, as an alias in an import map does (`#shared`). The check reports each such specifier
// of an import, an export from, an `import()` and a `require`, and reports nothing for a built-in, a package of the
// lockfile with any subpath, a file the sources hold, a file that exists beside them, a type-only import and a type.

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
];

//endregion Specifiers that name nothing the check can resolve
