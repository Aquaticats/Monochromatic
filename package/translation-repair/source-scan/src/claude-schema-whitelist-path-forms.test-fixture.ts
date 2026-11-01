import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A path literal that names a module carrying a builder, outside an import and a load
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) for a path
// literal that names a module which carries a builder and sits outside an import, an export from and a load: an
// argument of a call or of `new`, a constant, a template, or the module of an import-equals declaration. The case is
// one imaginary file that holds one place on each line, with the files the search lists and the places the check
// reports, by file, line, shape and position. A loader built by `createRequire` and `require.resolve` are also code
// the check reports, on the lines that hold them.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The file of the case.
 */
const FILE = 'imaginary/src/path-literal-stage.ts';

/**
 The place the check reports in the file of the case.

 @param line - line of the place

 @param shape - shape of the place

 @param position - where the place sits, or what it is

 @returns Report

 @example
 ```ts
 const spot = spotAt({ line: 3, shape: 'module-path-outside-a-load', position: 'CallExpression.arguments', },);
 ```
 */
function spotAt(
  {
    line,
    shape,
    position,
  }: {
    readonly line: number;
    readonly shape: BlindSpot['shape'];
    readonly position: string;
  },
): BlindSpot {
  return {
    file: FILE,
    line,
    shape,
    position,
  };
}

/**
 Cases of a path literal that names a module carrying a builder, outside an import and a load.
 */
export const WHITELIST_PATH_CASES: readonly ShapeCase[] = [
  {
    name: 'LEAVES UNREAD A PATH LITERAL THAT NAMES A MODULE CARRYING A BUILDER OUTSIDE AN IMPORT, AN EXPORT FROM AND A '
      + 'LOAD, as an argument of a call or of new, a constant, or the module of an import-equals declaration, and '
      + 'the check reports each',
    files: [
      {
        file: FILE,
        text: textOf([
          'import { createRequire, } from \'node:module\';',
          '',
          `export const viaCreate = createRequire(import.meta.url,)('${CONTEST_SPECIFIER}');`,
          `export const resolved = require.resolve('${CONTEST_SPECIFIER}');`,
          `export const meta = import.meta.resolve('${CONTEST_SPECIFIER}');`,
          `export const url = new URL('${CONTEST_SPECIFIER}', import.meta.url,);`,
          `export const kept = '${CONTEST_SPECIFIER}';`,
          `import equalled = require('${CONTEST_SPECIFIER}');`,
          `export type Typed = typeof import('${CONTEST_SPECIFIER}');`,
          `export const templated = \`${CONTEST_SPECIFIER}\`;`,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      spotAt({
        line: 1,
        shape: 'code-the-search-cannot-read',
        position: 'createRequire builds a loader the search cannot follow',
      },),
      spotAt({
        line: 3,
        shape: 'module-path-outside-a-load',
        position: 'CallExpression.arguments',
      },),
      spotAt({
        line: 3,
        shape: 'code-the-search-cannot-read',
        position: 'createRequire builds a loader the search cannot follow',
      },),
      spotAt({
        line: 4,
        shape: 'module-path-outside-a-load',
        position: 'CallExpression.arguments',
      },),
      spotAt({
        line: 4,
        shape: 'code-the-search-cannot-read',
        position: 'require used beyond a direct call',
      },),
      spotAt({
        line: 5,
        shape: 'module-path-outside-a-load',
        position: 'CallExpression.arguments',
      },),
      spotAt({
        line: 6,
        shape: 'module-path-outside-a-load',
        position: 'NewExpression.arguments',
      },),
      spotAt({
        line: 7,
        shape: 'module-path-outside-a-load',
        position: 'VariableDeclarator.init',
      },),
      spotAt({
        line: 8,
        shape: 'module-path-outside-a-load',
        position: 'TSExternalModuleReference.expression',
      },),
      spotAt({
        line: 10,
        shape: 'module-path-outside-a-load',
        position: 'VariableDeclarator.init',
      },),
    ],
  },
];

//endregion A path literal that names a module carrying a builder, outside an import and a load
