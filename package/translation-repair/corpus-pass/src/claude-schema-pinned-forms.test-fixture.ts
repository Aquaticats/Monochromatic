import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Reads of the walk that no other case pins
// Two reads of the walk of a builder call's value that a change could alter without any other case failing. The first
// is the callee of a call of a sender: only a chain of names that resolves to the sender is read as it, so a callee
// that is a comma expression, the result of a call, a conditional or a member of a call is no sender, and the object it
// takes is reported like the argument of any call. The second is the function a report names when the value leaves by
// more than one: the first in the order of the source, whatever its kind.

/**
 The import of the sender, as a specifier written in a file under `imaginary/src`.
 */
const SENDER_IMPORT =
  'import { gatherThroughCache, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 The object the sender is called with in each function of the case.
 */
const SENT_OBJECT = `{ responseFormat: contestResponseFormat(${ARGUMENTS},), }`;

/**
 The shape a report gives for a value a function that is no wrapper returns.
 */
const RETURNED = 'builder-result-returned-outside-a-wrapper';

/**
 The cases of the reads that no other case pins.
 */
export const PINNED_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT IN AN OBJECT HANDED TO A CALL WHOSE CALLEE IS NO CHAIN OF NAMES: a comma '
      + 'expression, the result of a call, a conditional and a member of a call, and in an object that is itself the '
      + 'callee of a call, which is no call of a sender and is reported as the value a function returns',
    files: [
      {
        file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
        text: textOf([
          'export async function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/callee-forms-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function comma() {',
          `  return await (0, gatherThroughCache)(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function resulted(pick: () => typeof gatherThroughCache,) {',
          `  return await pick()(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function conditional(isFirst: boolean, other: typeof gatherThroughCache,) {',
          `  return await (isFirst ? gatherThroughCache : other)(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function memberOfCall(senders: () => { gatherThroughCache: typeof gatherThroughCache },) {',
          `  return await senders().gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function plainCall() {',
          `  return await gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function objectCallee() {',
          `  return await (${SENT_OBJECT} as any)(1,);`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/callee-forms-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/callee-forms-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/callee-forms-stage.ts',
        line: 9,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/callee-forms-stage.ts',
        line: 13,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/callee-forms-stage.ts',
        line: 17,
        position: CALL_ARGUMENT,
      },),
      {
        file: 'imaginary/src/callee-forms-stage.ts',
        line: 25,
        shape: RETURNED,
        position: 'FunctionDeclaration returns it',
      },
    ],
  },
  {
    name: 'NAMES THE FIRST FUNCTION A BUILDER\'S RESULT LEAVES BY, in the order of the source, when a function returns '
      + 'it and an arrow returns it too, and when an arrow returns it first',
    files: [
      {
        file: 'imaginary/src/returns-first-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'export function declaredFirst(args: any,) {',
          `  const schema = contestResponseFormat(${ARGUMENTS},);`,
          '  if (args) {',
          '    return schema;',
          '  }',
          '  return () => schema;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/returns-arrow-first-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'export function arrowFirst(args: any,) {',
          `  const schema = contestResponseFormat(${ARGUMENTS},);`,
          '  const handed = () => schema;',
          '  if (args) {',
          '    return handed;',
          '  }',
          '  return schema;',
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/returns-arrow-first-stage.ts',
      'imaginary/src/returns-first-stage.ts',
    ],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/returns-first-stage.ts',
        line: 3,
        shape: RETURNED,
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/returns-arrow-first-stage.ts',
        line: 3,
        shape: RETURNED,
        position: 'ArrowFunctionExpression returns it',
      },
    ],
  },
];

//endregion Reads of the walk that no other case pins
