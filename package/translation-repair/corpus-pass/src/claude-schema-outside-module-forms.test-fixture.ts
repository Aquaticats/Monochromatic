import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A member of a module outside the sources, named like a builder
// A namespace import, or an awaited load, of a specifier that names no file of the sources has members that the search
// reads as free names. A member named like a builder is read as the builder where it is called; where it is stored, handed
// to a call or put in an array, the check reports it, as it does for the named import of the same name. The cases give
// the module a node specifier, which names no file of the sources and which the check can resolve.

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'outside\', verdicts: [], asksArchive: false, }';

/**
 The place the check reports for a builder that goes where the search does not read it.

 @param file - path of the imaginary file

 @param position - where the builder sits

 @returns The report, on the second line of the file

 @example
 ```ts
 const spot = spotAt({ file: 'imaginary/src/a.ts', position: 'Property.value', },);
 ```
 */
function spotAt(
  {
    file,
    position,
  }: {
    readonly file: string;
    readonly position: string;
  },
): BlindSpot {
  return {
    file,
    line: 2,
    shape: 'builder-outside-a-read-position',
    position,
  };
}

/**
 Cases of a member of a module outside the sources.
 */
export const OUTSIDE_MODULE_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A MEMBER OF A NAMESPACE OR OF AN AWAITED LOAD OUTSIDE THE SOURCES, NAMED LIKE A BUILDER, WHEN IT IS '
      + 'STORED IN AN OBJECT, HANDED TO A CALL OR PUT IN AN ARRAY, and reads it where it is called, as the check '
      + 'reports the named import of the same name',
    files: [
      {
        file: 'imaginary/src/outside-stored-stage.ts',
        text: textOf([
          'import * as lane from \'node:path\';',
          'export const kit = { make: lane.contestResponseFormat, };',
        ],),
      },
      {
        file: 'imaginary/src/outside-handed-stage.ts',
        text: textOf([
          'import * as lane from \'node:path\';',
          'register(lane.contestResponseFormat,);',
        ],),
      },
      {
        file: 'imaginary/src/outside-awaited-stage.ts',
        text: textOf([
          'const lane = await import(\'node:path\');',
          'export const kit = { make: lane.contestResponseFormat, };',
        ],),
      },
      {
        file: 'imaginary/src/outside-listed-stage.ts',
        text: textOf([
          'import * as lane from \'node:path\';',
          'export const all = [lane.contestResponseFormat,];',
        ],),
      },
      {
        file: 'imaginary/src/outside-named-stage.ts',
        text: textOf([
          'import { contestResponseFormat, } from \'node:path\';',
          'export const kit = { make: contestResponseFormat, };',
        ],),
      },
      {
        file: 'imaginary/src/outside-called-stage.ts',
        text: textOf([
          'import * as lane from \'node:path\';',
          `export const CALLED = lane.contestResponseFormat(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/outside-awaited-stage.ts',
      'imaginary/src/outside-called-stage.ts',
      'imaginary/src/outside-handed-stage.ts',
      'imaginary/src/outside-listed-stage.ts',
      'imaginary/src/outside-named-stage.ts',
      'imaginary/src/outside-stored-stage.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/outside-stored-stage.ts',
        position: 'Property.value',
      },),
      spotAt({
        file: 'imaginary/src/outside-handed-stage.ts',
        position: 'CallExpression.arguments',
      },),
      spotAt({
        file: 'imaginary/src/outside-awaited-stage.ts',
        position: 'Property.value',
      },),
      spotAt({
        file: 'imaginary/src/outside-listed-stage.ts',
        position: 'ArrayExpression.elements',
      },),
      spotAt({
        file: 'imaginary/src/outside-named-stage.ts',
        position: 'Property.value',
      },),
    ],
  },
];

//endregion A member of a module outside the sources, named like a builder
