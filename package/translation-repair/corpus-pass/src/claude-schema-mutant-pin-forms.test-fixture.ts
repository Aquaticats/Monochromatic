import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Inputs that tell the code from a change to it
// Each case here gives an input on which the check answers one way, and a change to one line of it answers the other
// way: a forwarder whose constants are declared against the order of their dependence, a builder call with a second
// argument that reads no parameter, an export read under two names, a name imported under another and bound again by a
// constant, a sender called through a namespace member and `.call`, a name read before the function that binds it, and
// an import-equals declaration of `vm`.

/**
 The file that calls the forwarder a case declares in `imaginary/src/pin-stage.ts`.
 */
const PIN_CALLER: SourceFile = {
  file: 'imaginary/src/pin-caller.ts',
  text: textOf([
    'import { mk, } from \'./pin-stage.ts\';',
    `export const SENT = mk(${ARGUMENTS},);`,
  ],),
};

/**
 A case whose stage file declares a forwarder and whose caller file calls it.

 @param name - name of the case

 @param lines - lines of the stage file after its two imports

 @returns Case that lists the stage file and the caller file and reports nothing

 @example
 ```ts
 const shapeCase = forwarderCase({ name: 'x', lines: [], },);
 ```
 */
function forwarderCase(
  {
    name,
    lines,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
  },
): ShapeCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/pin-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          ...lines,
        ],),
      },
      PIN_CALLER,
    ],
    found: [
      'imaginary/src/pin-caller.ts',
      'imaginary/src/pin-stage.ts',
    ],
    builders: [],
    spots: [],
  };
}

/**
 A stage file that exports a constant holding a builder's result on its second line.

 @param file - path of the file

 @param more - lines after the constant

 @returns File

 @example
 ```ts
 const stage = holdingFile({ file: 'imaginary/src/a.ts', more: [], },);
 ```
 */
function holdingFile(
  {
    file,
    more,
  }: {
    readonly file: string;
    readonly more: readonly string[];
  },
): SourceFile {
  return {
    file,
    text: textOf([
      CONTEST_IMPORT,
      `export const S = contestResponseFormat(${ARGUMENTS},);`,
      ...more,
    ],),
  };
}

/**
 The place the check reports for a constant that another file keeps.

 @param file - path of the file that holds the builder's call

 @param reader - path of the file that keeps the constant

 @returns Report at the builder's call

 @example
 ```ts
 const spot = keptBy({ file: 'imaginary/src/a.ts', reader: 'imaginary/src/b.ts', },);
 ```
 */
function keptBy(
  {
    file,
    reader,
  }: {
    readonly file: string;
    readonly reader: string;
  },
): BlindSpot {
  return spotAt({
    file,
    line: 2,
    position: `${CALL_ARGUMENT} in ${reader}`,
  },);
}

/**
 The place the check reports for a line of the import-equals case.

 @param line - line of the declaration

 @returns Report in the stage file

 @example
 ```ts
 const spot = vmSpotAt({ line: 1, },);
 ```
 */
function vmSpotAt({ line, }: { readonly line: number; },): BlindSpot {
  return {
    file: 'imaginary/src/equals-vm-stage.ts',
    line,
    shape: 'code-the-search-cannot-read',
    position: 'vm runs text the search cannot read',
  };
}

/**
 Cases whose inputs tell the code from a change to it.
 */
export const MUTANT_PIN_CASES: readonly ShapeCase[] = [
  forwarderCase({
    name: 'LISTS THE FILE THAT CALLS A FUNCTION WHOSE BUILDER CALL TAKES A CONSTANT MADE FROM ANOTHER CONSTANT DECLARED '
      + 'AFTER IT, since the constants made from a parameter are found until no more are',
    lines: [
      'export async function mk(args: any,) {',
      '  const pick = () => a;',
      '  const a = { ...args, };',
      `  return await gatherThroughCache({ responseFormat: contestResponseFormat(pick(),), },);`,
      '}',
    ],
  },),
  forwarderCase({
    name: 'LISTS THE FILE THAT CALLS A FUNCTION WHOSE BUILDER CALL HAS A SECOND ARGUMENT THAT READS NO PARAMETER, since '
      + 'one argument that reads a parameter is enough',
    lines: [
      'export async function mk(args: any,) {',
      '  return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(args, 0,), },);',
      '}',
    ],
  },),
  {
    name: 'REPORTS A CONSTANT A FILE EXPORTS UNDER TWO NAMES WHEN ANOTHER FILE KEEPS IT BY THE FIRST NAME',
    files: [
      holdingFile({
        file: 'imaginary/src/two-names.ts',
        more: ['export { S as R, };',],
      },),
      {
        file: 'imaginary/src/two-names-reader.ts',
        text: textOf([
          'import { S, } from \'./two-names.ts\';',
          'export const keep: unknown[] = [];',
          'keep.push(S,);',
        ],),
      },
    ],
    found: ['imaginary/src/two-names.ts',],
    builders: [],
    spots: [
      keptBy({
        file: 'imaginary/src/two-names.ts',
        reader: 'imaginary/src/two-names-reader.ts',
      },),
    ],
  },
  {
    name: 'REPORTS A CONSTANT A FILE EXPORTS WHEN ANOTHER FILE IMPORTS IT UNDER ANOTHER NAME AND BINDS THE NAME OF THE '
      + 'EXPORT AGAIN WITH A CONSTANT THAT HOLDS THE IMPORT',
    files: [
      holdingFile({
        file: 'imaginary/src/renamed.ts',
        more: [],
      },),
      {
        file: 'imaginary/src/renamed-reader.ts',
        text: textOf([
          'import { S as Y, } from \'./renamed.ts\';',
          'const S = Y;',
          'export const keep: unknown[] = [];',
          'keep.push(S,);',
        ],),
      },
    ],
    found: ['imaginary/src/renamed.ts',],
    builders: [],
    spots: [
      keptBy({
        file: 'imaginary/src/renamed.ts',
        reader: 'imaginary/src/renamed-reader.ts',
      },),
    ],
  },
  {
    name: 'ACCEPTS A SCHEMA HANDED TO THE SENDER THROUGH A NAMESPACE MEMBER AND `.call`, whose first argument is the `this` '
      + 'and whose second is the object',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/namespace-call.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import * as g from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';',
          `export const r = g.gatherThroughCache.call(null, { responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
        ],),
      },
    ],
    found: ['imaginary/src/namespace-call.ts',],
    builders: [],
    spots: [],
  },
  {
    name: 'ACCEPTS A NAME READ BEFORE THE FUNCTION THAT BINDS IT, since the read is of another thing',
    files: [
      {
        file: 'imaginary/src/read-before.ts',
        text: textOf([
          CONTEST_IMPORT,
          'const kept: unknown[] = [];',
          'export function b(s: unknown,) { kept.push(s,); }',
          `export function a() { const s = contestResponseFormat(${ARGUMENTS},); return typeof s; }`,
        ],),
      },
    ],
    found: ['imaginary/src/read-before.ts',],
    builders: [],
    spots: [],
  },
  {
    name: 'REPORTS vm BOUND BY AN IMPORT-EQUALS DECLARATION, with and without the prefix and also under export',
    files: [
      {
        file: 'imaginary/src/equals-vm-stage.ts',
        text: textOf([
          'import prefixed = require(\'node:vm\');',
          'import bare = require(\'vm\');',
          'export import exported = require(\'node:vm\');',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      vmSpotAt({ line: 1, },),
      vmSpotAt({ line: 2, },),
      vmSpotAt({ line: 3, },),
    ],
  },
];

//endregion Inputs that tell the code from a change to it
