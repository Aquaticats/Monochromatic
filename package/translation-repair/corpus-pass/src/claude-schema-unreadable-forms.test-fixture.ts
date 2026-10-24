import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Code whose loads the search cannot read
// The first total follows `import()` and a call of `require`. Code that builds a loader, reaches `require` by another
// name, or makes code from text can load a module that carries a builder with no path the search could read, so the
// check reports each of these where it is written, and reports nothing for a direct call of `require`, a string or a
// name that only contains one of the words, or a type.

/**
 The position a report gives for a place that builds a loader.
 */
const MAKES_LOADER = 'createRequire builds a loader the search cannot follow';

/**
 The position a report gives for `require` reached by another way than a direct call.
 */
const REACHES_REQUIRE = 'require used beyond a direct call';

/**
 The position a report gives for `eval`.
 */
const RUNS_TEXT = 'eval runs text the search cannot read';

/**
 The position a report gives for `Function`.
 */
const BUILDS_FUNCTION = 'Function builds code from text the search cannot read';

/**
 The place the check reports for code the search cannot read.

 @param file - path of the imaginary file

 @param line - line of the place

 @param position - what the place is

 @returns The report

 @example
 ```ts
 const spot = spotAt({ file: 'imaginary/src/a.ts', line: 1, position: 'eval runs text the search cannot read', },);
 ```
 */
function spotAt(
  {
    file,
    line,
    position,
  }: {
    readonly file: string;
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file,
    line,
    shape: 'code-the-search-cannot-read',
    position,
  };
}

/**
 Cases of code whose loads the search cannot read.
 */
export const UNREADABLE_CODE_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS createRequire IN ANY FORM, require REACHED BY ANOTHER WAY THAN A DIRECT CALL, eval, Function AND '
      + 'new Function, and nothing for a direct call of require, a string, a longer name or a type',
    files: [
      {
        file: 'imaginary/src/made-loader-stage.ts',
        text: textOf([
          'import { createRequire, } from \'node:module\';',
          'const load = createRequire(import.meta.url,);',
          'export const loaded = load(\'./x.ts\',);',
          'export const namespaced = nodeModule.createRequire(import.meta.url,);',
        ],),
      },
      {
        file: 'imaginary/src/member-require-stage.ts',
        text: textOf([
          'export const loaded = module.require(\'./x.ts\',);',
        ],),
      },
      {
        file: 'imaginary/src/alias-require-stage.ts',
        text: textOf([
          'const alias = require;',
          'export const loaded = alias(\'./x.ts\',);',
          'export const called = require.call(undefined, \'./x.ts\',);',
          'export const sequenced = (0, require)(\'./x.ts\',);',
        ],),
      },
      {
        file: 'imaginary/src/text-code-stage.ts',
        text: textOf([
          'export const evaluated = eval(\'1\',);',
          'export const made = Function(\'return 1\',)();',
          'export const constructed = new Function(\'a\', \'return a\',);',
          'export const viaGlobal = globalThis.eval(\'1\',);',
          'export const viaGlobalFunction = globalThis.Function(\'return 1\',);',
        ],),
      },
      {
        file: 'imaginary/src/plain-code-stage.ts',
        text: textOf([
          'export const direct = require(\'node:path\',);',
          'export const word = \'eval Function createRequire\';',
          'export const evaluateFunction = 1;',
          'export type Runner = typeof eval;',
          'export type Maker = Function;',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/made-loader-stage.ts',
        line: 1,
        position: MAKES_LOADER,
      },),
      spotAt({
        file: 'imaginary/src/made-loader-stage.ts',
        line: 2,
        position: MAKES_LOADER,
      },),
      spotAt({
        file: 'imaginary/src/made-loader-stage.ts',
        line: 4,
        position: MAKES_LOADER,
      },),
      spotAt({
        file: 'imaginary/src/member-require-stage.ts',
        line: 1,
        position: REACHES_REQUIRE,
      },),
      spotAt({
        file: 'imaginary/src/alias-require-stage.ts',
        line: 1,
        position: REACHES_REQUIRE,
      },),
      spotAt({
        file: 'imaginary/src/alias-require-stage.ts',
        line: 3,
        position: REACHES_REQUIRE,
      },),
      spotAt({
        file: 'imaginary/src/alias-require-stage.ts',
        line: 4,
        position: REACHES_REQUIRE,
      },),
      spotAt({
        file: 'imaginary/src/text-code-stage.ts',
        line: 1,
        position: RUNS_TEXT,
      },),
      spotAt({
        file: 'imaginary/src/text-code-stage.ts',
        line: 2,
        position: BUILDS_FUNCTION,
      },),
      spotAt({
        file: 'imaginary/src/text-code-stage.ts',
        line: 3,
        position: BUILDS_FUNCTION,
      },),
      spotAt({
        file: 'imaginary/src/text-code-stage.ts',
        line: 4,
        position: RUNS_TEXT,
      },),
      spotAt({
        file: 'imaginary/src/text-code-stage.ts',
        line: 5,
        position: BUILDS_FUNCTION,
      },),
    ],
  },
];

//endregion Code whose loads the search cannot read
