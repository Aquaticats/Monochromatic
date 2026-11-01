import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region The place among a sender's arguments that it reads
// The case of the third total for an object that is not where the sender reads it. The sender reads the first
// argument; through `.call` it reads the second, since the first is the `this` it is called with; through `.apply` and
// `.bind` it reads no argument that the call writes, since the first is the `this` and the rest is a list or a partial
// application. An object at any other place is the argument of a call that is no sender and is reported like one. The
// object under parentheses, `as` and `satisfies` stays the same object, so these marks change nothing.

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
 The cases of the place a sender reads its object at.
 */
export const SENDER_POSITION_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT IN AN OBJECT THAT IS NOT WHERE THE SENDER READS IT: the second argument of the '
      + 'sender, the `this` slot of `.call`, the arguments of `.apply` and `.bind`, and accepts the first argument, '
      + 'with the marks that leave an object the same, and the second argument of `.call`',
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
        file: 'imaginary/src/positioned-sender-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function secondArgument() {',
          `  return await gatherThroughCache({} as never, ${SENT_OBJECT} as never);`,
          '}',
          '',
          'export async function thisSlot() {',
          `  return await gatherThroughCache.call(${SENT_OBJECT} as never, undefined as never);`,
          '}',
          '',
          'export async function applied() {',
          `  return await gatherThroughCache.apply(undefined, [${SENT_OBJECT},] as never);`,
          '}',
          '',
          'export async function appliedThis() {',
          `  return await gatherThroughCache.apply(${SENT_OBJECT} as never, [] as never);`,
          '}',
          '',
          'export async function bound() {',
          `  return gatherThroughCache.bind(undefined, ${SENT_OBJECT} as never);`,
          '}',
          '',
          'export async function firstArgument() {',
          `  return await gatherThroughCache(${SENT_OBJECT}, 1 as never,);`,
          '}',
          '',
          'export async function firstArgumentMarked() {',
          `  return await (gatherThroughCache as any)((${SENT_OBJECT} satisfies object) as never, 1,);`,
          '}',
          '',
          'export async function calledSecond() {',
          `  return await gatherThroughCache.call(undefined, ${SENT_OBJECT},);`,
          '}',
          '',
          'export async function calledSecondMarked() {',
          `  return await gatherThroughCache.call(undefined, (${SENT_OBJECT} as never),);`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/positioned-sender-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/positioned-sender-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/positioned-sender-stage.ts',
        line: 9,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/positioned-sender-stage.ts',
        line: 13,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/positioned-sender-stage.ts',
        line: 17,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/positioned-sender-stage.ts',
        line: 21,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion The place among a sender's arguments that it reads
