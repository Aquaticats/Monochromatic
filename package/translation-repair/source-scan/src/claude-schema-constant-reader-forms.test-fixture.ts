import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The files that read a constant a builder's value is held by
// The third total follows a constant, whose every reference must end in a sender, a wrapper or a constant that does.
// A constant that a file exports is read in the files that import it, by its name, by an alias, through a module that
// exports it again, through a namespace import or an `import()` of the file. The cases give one file for each way, each
// with one reader that hands the value to a call that is no sender, reported at the builder's call and naming the file
// of the reader. The places where the walk ends without a report are in
// `claude-schema-constant-end-forms.test-fixture.ts`.

/**
 The file that declares a constant a builder's value is held by, the builder's call on the second line.

 @param file - path of the imaginary file

 @param name - name of the constant

 @param more - lines after the constant

 @returns File

 @example
 ```ts
 const declaring = declaringFile({ file: 'imaginary/src/a.ts', name: 'A_FORMAT', more: [], },);
 ```
 */
export function declaringFile(
  {
    file,
    name,
    more,
  }: {
    readonly file: string;
    readonly name: string;
    readonly more: readonly string[];
  },
): SourceFile {
  return {
    file,
    text: textOf([
      CONTEST_IMPORT,
      `export const ${name} = contestResponseFormat(${ARGUMENTS},);`,
      ...more,
    ],),
  };
}

/**
 The place the check reports for a constant read by a file.

 @param reader - path of the file that reads it

 @returns Position that names the kind of place and the file

 @example
 ```ts
 const position = handedOnIn({ reader: 'imaginary/src/a.ts', },);
 ```
 */
function handedOnIn({ reader, }: { readonly reader: string; },): string {
  return `${CALL_ARGUMENT} in ${reader}`;
}

/**
 Cases of the files that read a constant a builder's value is held by.
 */
export const CONSTANT_READER_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A CONSTANT THAT A FILE EXPORTS WHEN ANOTHER FILE HANDS IT TO A CALL THAT IS NO SENDER, by its name, '
      + 'by an alias through a module that exports it again and through a star, through a member of a namespace '
      + 'import, through a namespace import as a whole, through an import() of the file, through a default export '
      + 'and through a module that exports an imported name, and a namespace export, and names the file',
    files: [
      declaringFile({
        file: 'imaginary/src/direct-format.ts',
        name: 'DIRECT_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/direct-reader.ts',
        text: textOf([
          'import { DIRECT_FORMAT, } from \'./direct-format.ts\';',
          'export const kept = Object.freeze(DIRECT_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/chain-format.ts',
        name: 'CHAIN_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/chain-index.ts',
        text: textOf([
          'export { CHAIN_FORMAT as RENAMED_FORMAT, } from \'./chain-format.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/chain-star.ts',
        text: textOf([
          'export * from \'./chain-index.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/chain-reader.ts',
        text: textOf([
          'import { RENAMED_FORMAT as renamed, } from \'./chain-star.ts\';',
          'export const kept = Object.freeze(renamed,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/member-format.ts',
        name: 'MEMBER_FORMAT',
        more: ['export const MEMBER_COUNT = 1;',],
      },),
      {
        file: 'imaginary/src/member-reader.ts',
        text: textOf([
          'import * as formats from \'./member-format.ts\';',
          'export const kept = Object.freeze(formats.MEMBER_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/whole-format.ts',
        name: 'WHOLE_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/whole-reader.ts',
        text: textOf([
          'import * as formats from \'./whole-format.ts\';',
          'export const kept = Object.freeze(formats,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/load-format.ts',
        name: 'LOAD_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/load-reader.ts',
        text: textOf([
          'export const kept = Object.freeze((await import(\'./load-format.ts\')).LOAD_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/nsx-format.ts',
        name: 'NSX_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/nsx-index.ts',
        text: textOf([
          'export * as formats from \'./nsx-format.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/default-format.ts',
        text: textOf([
          CONTEST_IMPORT,
          `const DEFAULT_FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          'export default DEFAULT_FORMAT;',
        ],),
      },
      {
        file: 'imaginary/src/default-reader.ts',
        text: textOf([
          'import chosen from \'./default-format.ts\';',
          'export const kept = Object.freeze(chosen,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/again-format.ts',
        name: 'AGAIN_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/again-index.ts',
        text: textOf([
          'import { AGAIN_FORMAT, } from \'./again-format.ts\';',
          'export { AGAIN_FORMAT as REEXPORTED_FORMAT, };',
        ],),
      },
      {
        file: 'imaginary/src/again-reader.ts',
        text: textOf([
          'import { REEXPORTED_FORMAT, } from \'./again-index.ts\';',
          'export const kept = Object.freeze(REEXPORTED_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/list-format.ts',
        name: 'LIST_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/list-producers.ts',
        text: textOf([
          'import { LIST_FORMAT, } from \'./list-format.ts\';',
          'export const LIST_PRODUCERS = [{ name: \'list\', formats: [LIST_FORMAT,], },];',
        ],),
      },
      {
        file: 'imaginary/src/list-reader.ts',
        text: textOf([
          'import { LIST_PRODUCERS, } from \'./list-producers.ts\';',
          'export const kept = Object.freeze(LIST_PRODUCERS,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/again-format.ts',
      'imaginary/src/chain-format.ts',
      'imaginary/src/default-format.ts',
      'imaginary/src/direct-format.ts',
      'imaginary/src/list-format.ts',
      'imaginary/src/load-format.ts',
      'imaginary/src/member-format.ts',
      'imaginary/src/nsx-format.ts',
      'imaginary/src/whole-format.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/direct-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/direct-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/chain-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/chain-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/member-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/member-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/whole-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/whole-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/load-format.ts',
        line: 2,
        position: 'import() or require of imaginary/src/load-format.ts in imaginary/src/load-reader.ts',
      },),
      spotAt({
        file: 'imaginary/src/nsx-format.ts',
        line: 2,
        position: 'namespace export of imaginary/src/nsx-format.ts in imaginary/src/nsx-index.ts',
      },),
      spotAt({
        file: 'imaginary/src/default-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/default-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/again-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/again-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/list-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/list-reader.ts', },),
      },),
    ],
  },
];

//endregion The files that read a constant a builder's value is held by
