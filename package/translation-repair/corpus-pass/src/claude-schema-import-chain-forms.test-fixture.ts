import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { declaringFile, } from './claude-schema-constant-reader-forms.test-fixture.ts';
import { spotAt, } from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region An import-equals declaration that reads a member of a module
// `import U = A.S` binds `U` to the member `S` of the namespace `A`, as `const U = A.S` does, and a qualified name is
// no member expression. The name the declaration reads from is a reference to the namespace like any other, so a
// constant that the module exports and the declaration reads is reported where the declaration reads it, whether the
// namespace comes from `import * as A` or from `import A = require(path)`, and whether the declaration is exported or
// not. A qualified name that reads a namespace the file declares itself is no read of a module and reports nothing.

/**
 The path of the file that declares the constant in every case.
 */
const CONSTANT_FILE = 'imaginary/src/chain-a.ts';

/**
 The path of the file that reads the constant through an import-equals declaration.
 */
const READER_FILE = 'imaginary/src/chain-b.ts';

/**
 A case whose reader file reads the member `FORMAT` of the constant file through an import-equals declaration.

 @param name - name of the case

 @param lines - lines of the reader file

 @param isReported - whether the check reports the read at the constant's call

 @returns Case that lists the constant file

 @example
 ```ts
 const shapeCase = chainCase({ name: 'x', lines: [], isReported: true, },);
 ```
 */
function chainCase(
  {
    name,
    lines,
    isReported,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly isReported: boolean;
  },
): ShapeCase {
  return {
    name,
    files: [
      declaringFile({
        file: CONSTANT_FILE,
        name: 'FORMAT',
        more: [],
      },),
      {
        file: READER_FILE,
        text: textOf(lines,),
      },
    ],
    found: [CONSTANT_FILE,],
    builders: [],
    spots: isReported
      ? [
        spotAt({
          file: CONSTANT_FILE,
          line: 2,
          position: `TSQualifiedName.left in ${READER_FILE}`,
        },),
      ]
      : [],
  };
}

/**
 Cases of an import-equals declaration that reads a member.
 */
export const IMPORT_CHAIN_CASES: readonly ShapeCase[] = [
  chainCase({
    name: 'REPORTS A CONSTANT READ BY AN IMPORT-EQUALS DECLARATION OF A MEMBER OF A NAMESPACE IMPORT, since the '
      + 'qualified name is no member expression',
    lines: [
      'import * as A from \'./chain-a.ts\';',
      'import U = A.FORMAT;',
      'export const keep: unknown[] = [];',
      'keep.push(U,);',
    ],
    isReported: true,
  },),
  chainCase({
    name: 'REPORTS A CONSTANT READ BY AN IMPORT-EQUALS DECLARATION OF A MEMBER OF A REQUIRED MODULE',
    lines: [
      'import a = require(\'./chain-a.ts\');',
      'import U = a.FORMAT;',
      'export const keep: unknown[] = [];',
      'keep.push(U,);',
    ],
    isReported: true,
  },),
  chainCase({
    name: 'REPORTS A CONSTANT READ BY AN EXPORTED IMPORT-EQUALS DECLARATION OF A MEMBER, where the declaration reads '
      + 'it and not where another file reads the name it exports',
    lines: [
      'import * as A from \'./chain-a.ts\';',
      'export import T = A.FORMAT;',
    ],
    isReported: true,
  },),
  chainCase({
    name: 'ACCEPTS AN IMPORT-EQUALS DECLARATION OF A MEMBER OF A NAMESPACE THE FILE DECLARES, which reads no module',
    lines: [
      'namespace Local { export const FORMAT = 1; }',
      'import U = Local.FORMAT;',
      'export const keep: unknown[] = [];',
      'keep.push(U,);',
    ],
    isReported: false,
  },),
  chainCase({
    name: 'ACCEPTS THE LAST NAME OF A QUALIFIED NAME THAT IS ALSO THE NAME OF AN IMPORTED CONSTANT, since the last name '
      + 'of a qualified name is a member and no read of a name',
    lines: [
      'import { FORMAT, } from \'./chain-a.ts\';',
      'namespace Local { export const FORMAT = 1; }',
      'import U = Local.FORMAT;',
      'export const keep: unknown[] = [U];',
    ],
    isReported: false,
  },),
];

//endregion An import-equals declaration that reads a member of a module
