import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  CALL_ARGUMENT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import { declaringFile, } from './claude-schema-constant-reader-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A file that reads a constant through `import x = require(path)`
// `import a = require('./a.ts')` binds the module as a whole, as `import * as a from './a.ts'` does, for a file that
// uses the module's `export =` form. A constant the module exports is read through `a.NAME`, or through `a` as a
// whole, and the walk across files follows both: a read that hands the value to a call that is no sender is reported at
// the builder's call, naming the file of the reader, as it is for a namespace import. A member of the module that is
// not the constant reads nothing of it.

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
 Cases of a file that reads a constant through an import-equals declaration.
 */
export const IMPORT_EQUALS_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A CONSTANT THAT A FILE EXPORTS WHEN ANOTHER FILE HANDS IT TO A CALL THAT IS NO SENDER THROUGH '
      + '`import a = require(path)`, as a member of the module and as the module whole, and names the file',
    files: [
      declaringFile({
        file: 'imaginary/src/equals-member-format.ts',
        name: 'MEMBER_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/equals-member-reader.ts',
        text: textOf([
          'import formats = require(\'./equals-member-format.ts\');',
          'export const kept: unknown[] = [];',
          'kept.push(formats.MEMBER_FORMAT,);',
        ],),
      },
      declaringFile({
        file: 'imaginary/src/equals-whole-format.ts',
        name: 'WHOLE_FORMAT',
        more: [],
      },),
      {
        file: 'imaginary/src/equals-whole-reader.ts',
        text: textOf([
          'export import formats = require(\'./equals-whole-format.ts\');',
          'export const kept = Object.freeze(formats,);',
        ],),
      },
    ],
    found: [
      'imaginary/src/equals-member-format.ts',
      'imaginary/src/equals-whole-format.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/equals-member-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/equals-member-reader.ts', },),
      },),
      spotAt({
        file: 'imaginary/src/equals-whole-format.ts',
        line: 2,
        position: handedOnIn({ reader: 'imaginary/src/equals-whole-reader.ts', },),
      },),
    ],
  },
  {
    name: 'ACCEPTS A MEMBER OF A MODULE BOUND WITH `import a = require(path)` THAT IS NOT THE CONSTANT, since the read '
      + 'takes nothing of the value the constant holds',
    files: [
      declaringFile({
        file: 'imaginary/src/equals-other-format.ts',
        name: 'OTHER_FORMAT',
        more: ['export const OTHER_COUNT = 1;',],
      },),
      {
        file: 'imaginary/src/equals-other-reader.ts',
        text: textOf([
          'import formats = require(\'./equals-other-format.ts\');',
          'export const kept: unknown[] = [];',
          'kept.push(formats.OTHER_COUNT,);',
        ],),
      },
    ],
    found: ['imaginary/src/equals-other-format.ts',],
    builders: [],
    spots: [],
  },
];

//endregion A file that reads a constant through `import x = require(path)`
