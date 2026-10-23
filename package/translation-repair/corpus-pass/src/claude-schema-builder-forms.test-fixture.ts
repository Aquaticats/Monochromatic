import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Forms in which a builder of an answer schema is written
// What `claude-schemas.unit.test.ts` feeds the builder search to show that it finds a builder whatever way the file
// writes it. Each case is a builder and the file that calls it, as imaginary files, and the files and builders the
// search must list. The wrappers of a builder are in `claude-schema-wrapper-forms.test-fixture.ts` and
// `claude-schema-wrapped-builder-forms.test-fixture.ts`.

/**
 The body of an imaginary builder: the format with a name and one enum of moods, which the builder writes in its body.

 @param name - name of the schema the format carries

 @returns Lines of an object literal, indented for a function body

 @example
 ```ts
 const lines = formatLinesOf({ name: 'cat_mood', },);
 ```
 */
export function formatLinesOf({ name, }: { readonly name: string; },): readonly string[] {
  return [
    '  type: \'json_schema\',',
    '  json_schema: {',
    `    name: '${name}',`,
    '    schema: { type: \'object\', properties: { mood: { enum: moods, }, }, },',
    '  },',
  ];
}

/**
 Cases in which a builder is written as an arrow function, a generic arrow function, a function expression, a default
 export (also an async one), a method of an object, an arrow function handed to a call, or an arrow function with one
 bare parameter, and a file calls it.
 */
export const BUILDER_FORM_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A BUILDER WRITTEN AS AN ARROW FUNCTION, and the file that calls it',
    files: [
      {
        file: 'imaginary/src/arrow-wire.ts',
        text: textOf([
          'export const arrowResponseFormat = ({ moods, }: { readonly moods: readonly string[]; },) => ({',
          ...formatLinesOf({ name: 'arrow_mood', },),
          '});',
        ],),
      },
      {
        file: 'imaginary/src/arrow-gate-stage.ts',
        text: textOf(['export const ARROW_GATE_RESPONSE_FORMAT = arrowResponseFormat({ moods: [\'wags\',], },);',],),
      },
    ],
    found: [
      'imaginary/src/arrow-gate-stage.ts',
      'imaginary/src/arrow-wire.ts',
    ],
    builders: ['arrowResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER WRITTEN AS A FUNCTION EXPRESSION, and the file that calls it',
    files: [
      {
        file: 'imaginary/src/expression-wire.ts',
        text: textOf([
          'export const expressionResponseFormat = function ({ moods, }: { readonly moods: readonly string[]; },) {',
          '  return {',
          ...formatLinesOf({ name: 'expression_mood', },),
          '  };',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/expression-gate-stage.ts',
        text: textOf([
          'export const EXPRESSION_GATE_RESPONSE_FORMAT = expressionResponseFormat({ moods: [\'wags\',], },);',
        ],),
      },
    ],
    found: [
      'imaginary/src/expression-gate-stage.ts',
      'imaginary/src/expression-wire.ts',
    ],
    builders: ['expressionResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER EXPORTED AS THE DEFAULT, and the file that calls it by the name it imports it as',
    files: [
      {
        file: 'imaginary/src/default-wire.ts',
        text: textOf([
          'export default function defaultResponseFormat({ moods, }: { readonly moods: readonly string[]; },) {',
          '  return {',
          ...formatLinesOf({ name: 'default_mood', },),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/default-gate-stage.ts',
        text: textOf([
          'import makeFormat from \'./default-wire.ts\';',
          '',
          'export const DEFAULT_GATE_RESPONSE_FORMAT = makeFormat({ moods: [\'wags\',], },);',
        ],),
      },
    ],
    found: [
      'imaginary/src/default-gate-stage.ts',
      'imaginary/src/default-wire.ts',
    ],
    builders: ['defaultResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER WRITTEN AS A METHOD OF AN OBJECT, and the file that calls it through the object',
    files: [
      {
        file: 'imaginary/src/method-wire.ts',
        text: textOf([
          'export const schemaKit = {',
          '  methodResponseFormat({ moods, }: { readonly moods: readonly string[]; },) {',
          '    return {',
          ...formatLinesOf({ name: 'method_mood', },)
            .map(function deeper(line,): string {
              return `    ${line}`;
            },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/method-gate-stage.ts',
        text: textOf([
          'import { schemaKit, } from \'./method-wire.ts\';',
          '',
          'export const METHOD_GATE_RESPONSE_FORMAT = schemaKit.methodResponseFormat({ moods: [\'wags\',], },);',
        ],),
      },
    ],
    found: [
      'imaginary/src/method-gate-stage.ts',
      'imaginary/src/method-wire.ts',
    ],
    builders: ['schemaKit.methodResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER WRITTEN AS AN ARROW FUNCTION HANDED TO A CALL, as `memoize` wraps it',
    files: [
      {
        file: 'imaginary/src/memoized-wire.ts',
        text: textOf([
          'export const memoizedResponseFormat = memoize(({ moods, }: { readonly moods: readonly string[]; },) => ({',
          ...formatLinesOf({ name: 'memoized_mood', },),
          '}));',
        ],),
      },
      {
        file: 'imaginary/src/memoized-gate-stage.ts',
        text: textOf([
          'export const MEMOIZED_GATE_RESPONSE_FORMAT = memoizedResponseFormat({ moods: [\'wags\',], },);',
        ],),
      },
    ],
    found: [
      'imaginary/src/memoized-gate-stage.ts',
      'imaginary/src/memoized-wire.ts',
    ],
    builders: ['memoizedResponseFormat',],
  },
  {
    name: 'FINDS AN ASYNC ARROW BUILDER WHOSE ONE PARAMETER HAS NO PARENTHESES',
    files: [
      {
        file: 'imaginary/src/bare-wire.ts',
        text: textOf([
          'export const bareResponseFormat = async moods => ({',
          ...formatLinesOf({ name: 'bare_mood', },),
          '});',
        ],),
      },
      {
        file: 'imaginary/src/bare-gate-stage.ts',
        text: textOf(['export const BARE_GATE_RESPONSE_FORMAT = await bareResponseFormat([\'wags\',],);',],),
      },
    ],
    found: [
      'imaginary/src/bare-gate-stage.ts',
      'imaginary/src/bare-wire.ts',
    ],
    builders: ['bareResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER WRITTEN AS A GENERIC ARROW FUNCTION, whose value starts with its type parameters, and the '
      + 'file that calls it',
    files: [
      {
        file: 'imaginary/src/generic-arrow-wire.ts',
        text: textOf([
          'export const genericArrowResponseFormat = <Mood extends string,>(moods: readonly Mood[],) => ({',
          ...formatLinesOf({ name: 'generic_arrow_mood', },),
          '});',
        ],),
      },
      {
        file: 'imaginary/src/generic-arrow-gate-stage.ts',
        text: textOf([
          'export const GENERIC_ARROW_GATE_RESPONSE_FORMAT = genericArrowResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/generic-arrow-gate-stage.ts',
      'imaginary/src/generic-arrow-wire.ts',
    ],
    builders: ['genericArrowResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER EXPORTED AS THE DEFAULT WITH async, and the file that calls it by the name it imports it as',
    files: [
      {
        file: 'imaginary/src/async-default-wire.ts',
        text: textOf([
          'export default async function asyncDefaultResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'async_default_mood', },),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/async-default-gate-stage.ts',
        text: textOf([
          'import makeAsyncFormat from \'./async-default-wire.ts\';',
          '',
          'export const ASYNC_DEFAULT_GATE_RESPONSE_FORMAT = await makeAsyncFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/async-default-gate-stage.ts',
      'imaginary/src/async-default-wire.ts',
    ],
    builders: ['asyncDefaultResponseFormat',],
  },
];

//endregion Forms in which a builder of an answer schema is written
