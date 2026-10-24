import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A name the sender is called by that a scope of the file binds
// The case of the third total for a callee that looks like the sender and is not: the file imports the sender, and a
// function of the file binds the same name again, as a parameter, a constant, a function declaration or expression, a
// class, a loop variable, a `var` of a block or a `catch` binding. The call there is a call of that binding, which the
// list of senders does not name, so the object it takes is reported like the argument of any call. A call of the
// import in a function that binds nothing, and a call in a function whose parameter has another name, stay accepted.

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
 The case of a sender's name bound by a scope of the file, and of the calls that stay accepted.
 */
export const SENDER_SHADOW_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CALL OF A NAME THE SENDER IS IMPORTED BY WHEN A SCOPE OF THE FILE '
      + 'BINDS THAT NAME AGAIN, as a parameter, a constant, a function declaration, a named function expression, a '
      + 'local class declaration, a loop variable, a `var` of a block or a `catch` binding, and a member of a class '
      + 'called through this, and accepts the import called in a function that binds nothing and in one whose '
      + 'parameter has another name',
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
        file: 'imaginary/src/shadowed-sender-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function byParameter(gatherThroughCache: (sent: object,) => unknown,) {',
          `  return await gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export function byConstant() {',
          '  const kept: unknown[] = [];',
          '  const gatherThroughCache = (sent: object,) => kept.push(sent,);',
          `  gatherThroughCache(${SENT_OBJECT},);`,
          '  return kept;',
          '}',
          '',
          'export function byFunction() {',
          '  function gatherThroughCache(sent: object,) {',
          '    return sent;',
          '  }',
          `  return gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export function byExpression() {',
          '  return function gatherThroughCache(sent: object,): unknown {',
          `    return [sent, gatherThroughCache(${SENT_OBJECT},),];`,
          '  };',
          '}',
          '',
          'export function byClass() {',
          '  class gatherThroughCache {',
          '    static sent = 0;',
          '  }',
          `  return gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export function byLoop(senders: readonly ((sent: object,) => unknown)[],) {',
          '  for (const gatherThroughCache of senders) {',
          `    gatherThroughCache(${SENT_OBJECT},);`,
          '  }',
          '}',
          '',
          'export function byVar(isFirst: boolean,) {',
          '  if (isFirst) {',
          '    var gatherThroughCache = (sent: object,) => sent;',
          '  }',
          `  return gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export function byCatch() {',
          '  try {',
          '    throw new Error(\'stop\');',
          '  } catch (gatherThroughCache: any) {',
          `    return gatherThroughCache(${SENT_OBJECT},);`,
          '  }',
          '}',
          '',
          'export async function bindsNothing() {',
          `  return await gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function otherParameter(client: unknown,) {',
          `  return await gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export class Holder {',
          '  gatherThroughCache(sent: object,) {',
          '    return sent;',
          '  }',
          '',
          '  run() {',
          `    return this.gatherThroughCache(${SENT_OBJECT},);`,
          '  }',
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/shadowed-sender-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 11,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 19,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 24,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 32,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 37,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 45,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 52,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/shadowed-sender-stage.ts',
        line: 70,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion A name the sender is called by that a scope of the file binds
