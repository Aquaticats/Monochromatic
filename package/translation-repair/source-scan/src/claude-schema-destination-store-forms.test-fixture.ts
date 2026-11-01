import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A builder's result stored, or held by a constant that hands it on
// The cases of the third total that follow the value into a store (a member of a local, a field, `this`, a computed
// member) and into a constant whose reference hands it on (as a list, as a member or inside an object). The calls and
// the senders are in `claude-schema-destination-forms.test-fixture.ts` and
// `claude-schema-destination-sender-forms.test-fixture.ts`.

/**
 Cases of a value that is stored in a member or a field, or held by a constant that hands it on.
 */
export const DESTINATION_STORE_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT STORED IN A MEMBER OF A LOCAL, IN A FIELD, IN this OR IN A COMPUTED MEMBER, '
      + 'and the check reports each call',
    files: [
      {
        file: 'imaginary/src/stored-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function boxed() {',
          '  const box = {};',
          `  box.schema = contestResponseFormat(${ARGUMENTS},);`,
          '  return box.schema;',
          '}',
          '',
          'export class Holder {',
          `  schema = contestResponseFormat(${ARGUMENTS},);`,
          `  static shared = contestResponseFormat(${ARGUMENTS},);`,
          '  constructor() {',
          `    this.other = contestResponseFormat(${ARGUMENTS},);`,
          '  }',
          '}',
          '',
          'export function keyed(key) {',
          '  const box = {};',
          `  box[key] = contestResponseFormat(${ARGUMENTS},);`,
          '  return box;',
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/stored-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/stored-stage.ts',
        line: 5,
        position: 'AssignmentExpression.right',
      },),
      spotAt({
        file: 'imaginary/src/stored-stage.ts',
        line: 10,
        position: 'PropertyDefinition.value',
      },),
      spotAt({
        file: 'imaginary/src/stored-stage.ts',
        line: 11,
        position: 'PropertyDefinition.value',
      },),
      spotAt({
        file: 'imaginary/src/stored-stage.ts',
        line: 13,
        position: 'AssignmentExpression.right',
      },),
      spotAt({
        file: 'imaginary/src/stored-stage.ts',
        line: 19,
        position: 'AssignmentExpression.right',
      },),
    ],
  },
  {
    name: 'REPORTS A BUILDER\'S RESULT HELD BY A CONSTANT WHOSE REFERENCE HANDS IT ON, in a list, as a member or in '
      + 'an object, and the check reports each call',
    files: [
      {
        file: 'imaginary/src/held-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export function listed() {',
          `  const all = [contestResponseFormat(${ARGUMENTS},),];`,
          '  register(all,);',
          '}',
          '',
          'export function picked() {',
          `  const format = contestResponseFormat(${ARGUMENTS},);`,
          '  register(format.json_schema,);',
          '}',
          '',
          'export function boxedLocal() {',
          `  const format = contestResponseFormat(${ARGUMENTS},);`,
          '  const box = { format, };',
          '  audit(box,);',
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/held-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/held-stage.ts',
        line: 4,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/held-stage.ts',
        line: 9,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/held-stage.ts',
        line: 14,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion A builder's result stored, or held by a constant that hands it on
