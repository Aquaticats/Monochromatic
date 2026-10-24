import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Places that take a builder's result and are not a call or a store
// The cases of the third total for the places that are neither a call, a store nor a constant: the places that carry
// the value on without the check naming them (a thrown value, a loop over it, a default export, a class that extends
// it, a pattern an assignment takes apart), which are reported, and the places that read the value and give nothing
// back that the schema could travel in, which are not.

/**
 The character that opens a substitution in a template, which an imaginary file's text spells out beside a brace.
 */
const DOLLAR = '$';

/**
 Cases of the places that take a builder's result and are not a call or a store.
 */
export const DESTINATION_OTHER_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT THROWN, ITERATED, TAKEN APART BY AN ASSIGNMENT, EXPORTED AS THE DEFAULT OR '
      + 'EXTENDED, and the check reports each call',
    files: [
      {
        file: 'imaginary/src/carried-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function thrown() {',
          `  throw contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export function iterated() {',
          `  for (const part of contestResponseFormat(${ARGUMENTS},)) {`,
          '    keep(part,);',
          '  }',
          '}',
          '',
          'export function destructured() {',
          '  let first;',
          `  [first,] = contestResponseFormat(${ARGUMENTS},);`,
          '  return first;',
          '}',
          '',
          `export default contestResponseFormat(${ARGUMENTS},);`,
          '',
          `export class Extended extends contestResponseFormat(${ARGUMENTS},) {}`,
        ],),
      },
    ],
    found: ['imaginary/src/carried-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/carried-stage.ts',
        line: 4,
        position: 'ThrowStatement.argument',
      },),
      spotAt({
        file: 'imaginary/src/carried-stage.ts',
        line: 8,
        position: 'ForOfStatement.right',
      },),
      spotAt({
        file: 'imaginary/src/carried-stage.ts',
        line: 15,
        position: 'AssignmentExpression.right',
      },),
      spotAt({
        file: 'imaginary/src/carried-stage.ts',
        line: 19,
        position: 'ExportDefaultDeclaration.declaration',
      },),
      spotAt({
        file: 'imaginary/src/carried-stage.ts',
        line: 21,
        position: 'ClassDeclaration.superClass',
      },),
    ],
  },
  {
    name: 'REPORTS NOTHING FOR A BUILDER\'S RESULT THAT IS COMPARED, TESTED BY A LOOP OR AN IF, NEGATED, PUT IN A '
      + 'TEMPLATE, DROPPED BY A STATEMENT OR THE TARGET OF AN ASSIGNMENT: the place reads the value and gives back '
      + 'nothing the schema could travel in',
    files: [
      {
        file: 'imaginary/src/read-only-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function compared() {',
          `  return contestResponseFormat(${ARGUMENTS},) === undefined;`,
          '}',
          '',
          'export function negated() {',
          `  return !contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export function tested() {',
          `  if (contestResponseFormat(${ARGUMENTS},)) {`,
          '    return 1;',
          '  }',
          `  while (contestResponseFormat(${ARGUMENTS},)) {`,
          '    return 2;',
          '  }',
          '  return 0;',
          '}',
          '',
          'export function templated() {',
          `  return \`${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`;`,
          '}',
          '',
          'export function dropped() {',
          `  contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export function targeted() {',
          `  contestResponseFormat(${ARGUMENTS},).schema = 1;`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/read-only-stage.ts',],
    builders: [],
    spots: [],
  },
];

//endregion Places that take a builder's result and are not a call or a store
