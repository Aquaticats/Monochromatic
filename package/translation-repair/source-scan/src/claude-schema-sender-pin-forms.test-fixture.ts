import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Branches of the sender check that the other cases leave unpinned
// The mutants of round 17 that survived the cases of the round: a name bound by a `switch` case, by a class expression
// and by a parameter property shadows the sender; a `var` of a nested function and a `let` of a block do not; an object
// given to `.bind` in the `this` slot is not the sender's object; and a template literal in the tag position of a
// tagged template is a plain template. Each case passes against the check as it stood, and fails under the mutant that
// names it.

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
 The character that opens a substitution in a template, which an imaginary file's text spells out beside a brace.
 */
const DOLLAR = '$';

/**
 The cases of the branches of the sender check that no other case pins.
 */
export const SENDER_PIN_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A NAME A SWITCH CASE, A CLASS EXPRESSION OR A PARAMETER PROPERTY '
      + 'BINDS AND TO AN OBJECT GIVEN TO .bind IN THE this SLOT, and accepts the sender called where a nested '
      + 'function\'s var or a block\'s let binds its name, and a template literal in the tag position of a tagged '
      + 'template',
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
        file: 'imaginary/src/pinned-sender-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export function boundFirst() {',
          `  return gatherThroughCache.bind(${SENT_OBJECT} as never);`,
          '}',
          '',
          'export function bySwitch(tag: number,) {',
          '  switch (tag) {',
          '    case 1:',
          '      const gatherThroughCache = (sent: object,) => sent;',
          `      return gatherThroughCache(${SENT_OBJECT},);`,
          '    default:',
          '      return tag;',
          '  }',
          '}',
          '',
          'export function byClassExpression() {',
          '  return class gatherThroughCache {',
          '    static run() {',
          `      return gatherThroughCache(${SENT_OBJECT},);`,
          '    }',
          '  };',
          '}',
          '',
          'export class Keeper {',
          '  constructor(private readonly gatherThroughCache: (sent: object,) => unknown,) {',
          `    gatherThroughCache(${SENT_OBJECT},);`,
          '  }',
          '}',
          '',
          'export async function nestedVar() {',
          '  const inner = () => {',
          '    var gatherThroughCache = 1;',
          '    return gatherThroughCache;',
          '  };',
          '  void inner;',
          `  return await gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export async function blockLet(isFirst: boolean,) {',
          '  if (isFirst) {',
          '    let gatherThroughCache = 1;',
          '    void gatherThroughCache;',
          '  }',
          `  return await gatherThroughCache(${SENT_OBJECT},);`,
          '}',
          '',
          'export function tagTemplate() {',
          `  return \`${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`\`kept\`;`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/pinned-sender-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/pinned-sender-stage.ts',
        line: 5,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/pinned-sender-stage.ts',
        line: 12,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/pinned-sender-stage.ts',
        line: 21,
        position: CALL_ARGUMENT,
      },),
      spotAt({
        file: 'imaginary/src/pinned-sender-stage.ts',
        line: 28,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion Branches of the sender check that the other cases leave unpinned
