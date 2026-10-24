import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Functions that are no wrapper of a builder, and wrappers with a type mark around the call
// What `claude-schemas.unit.test.ts` feeds the builder search to show which functions are a wrapper: one that returns a
// builder's call at once, whatever type mark is written around it (`as`, `satisfies`, `!`, `<T>x`) and an optional
// call; and which are not: one that returns the call with something after it (a promise chain, a fallback), inside
// another expression (a conditional, an object, an array, a promise), or that only calls the builder in an inner
// function. Each case is a few imaginary files, with the contest builder (`contestResponseFormat`) beside them, and
// the files and builders the search must list.

/**
 Cases of a function that mentions a builder and returns something else, or returns its result inside another
 expression, and so is no wrapper.
 */
export const NOT_A_WRAPPER_CASES: readonly FormCase[] = [
  {
    name: 'NAMES NO WRAPPER IN A FUNCTION THAT RETURNS A BUILDER\'S RESULT WITH A PROMISE CHAIN AFTER IT, as in '
      + '`return build(a).then(...)`',
    files: [
      {
        file: 'imaginary/src/chained-wire.ts',
        text: textOf([
          'export function chainedResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },)',
          '    .then((value,) => value,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/chained-caller.ts',
        text: textOf(['export const CHAINED = chainedResponseFormat(\'chained\',);',],),
      },
    ],
    found: ['imaginary/src/chained-wire.ts',],
    builders: [],
  },
  {
    name: 'NAMES NO WRAPPER IN A FUNCTION THAT RETURNS A BUILDER\'S RESULT WITH A FALLBACK AFTER IT, as in '
      + '`return build(a) ?? {}`',
    files: [
      {
        file: 'imaginary/src/coalesced-wire.ts',
        text: textOf([
          'export function coalescedResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },) ?? {};',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/coalesced-caller.ts',
        text: textOf(['export const COALESCED = coalescedResponseFormat(\'coalesced\',);',],),
      },
    ],
    found: ['imaginary/src/coalesced-wire.ts',],
    builders: [],
  },
  {
    name: 'NAMES NO WRAPPER IN A FUNCTION THAT CALLS A BUILDER IN AN ARROW FUNCTION IT HANDS TO map AND RETURNS A '
      + 'COUNT, so the file that calls it is not demanded in the producer list',
    files: [
      {
        file: 'imaginary/src/counting-wire.ts',
        text: textOf([
          'export function countFormats(items: readonly string[],) {',
          '  const formats = items.map((item,) => contestResponseFormat({',
          '    schemaName: item,',
          '    verdicts: [],',
          '    asksArchive: false,',
          '  },),);',
          '  return formats.length;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/counting-caller.ts',
        text: textOf(['export const COUNT = countFormats([\'a\', \'b\',],);',],),
      },
    ],
    found: ['imaginary/src/counting-wire.ts',],
    builders: [],
  },
  {
    name: 'NAMES NO WRAPPER IN A FUNCTION THAT RETURNS A BUILDER\'S RESULT IN A CONDITIONAL, AN OBJECT, AN ARRAY OR A '
      + 'PROMISE, so the files that call them are not demanded in the producer list',
    files: [
      {
        file: 'imaginary/src/conditional-wire.ts',
        text: textOf([
          'export function conditionalResponseFormat(on: boolean,) {',
          '  return on',
          '    ? contestResponseFormat({ schemaName: \'on\', verdicts: [], asksArchive: false, },)',
          '    : undefined;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/conditional-caller.ts',
        text: textOf(['export const CONDITIONAL = conditionalResponseFormat(true,);',],),
      },
      {
        file: 'imaginary/src/object-wire.ts',
        text: textOf([
          'export function objectResponseFormat() {',
          '  return {',
          '    format: contestResponseFormat({ schemaName: \'in_object\', verdicts: [], asksArchive: false, },),',
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/object-caller.ts',
        text: textOf(['export const OBJECT = objectResponseFormat();',],),
      },
      {
        file: 'imaginary/src/array-wire.ts',
        text: textOf([
          'export function arrayResponseFormat() {',
          '  return [contestResponseFormat({ schemaName: \'in_array\', verdicts: [], asksArchive: false, },),];',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/array-caller.ts',
        text: textOf(['export const ARRAY = arrayResponseFormat();',],),
      },
      {
        file: 'imaginary/src/promise-wire.ts',
        text: textOf([
          'export function promiseResponseFormat() {',
          '  return Promise.resolve(contestResponseFormat({',
          '    schemaName: \'in_promise\',',
          '    verdicts: [],',
          '    asksArchive: false,',
          '  },),);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/promise-caller.ts',
        text: textOf(['export const PROMISE = promiseResponseFormat();',],),
      },
    ],
    found: [
      'imaginary/src/array-wire.ts',
      'imaginary/src/conditional-wire.ts',
      'imaginary/src/object-wire.ts',
      'imaginary/src/promise-wire.ts',
    ],
    builders: [],
  },
  {
    name: 'NAMES NO WRAPPER IN A FUNCTION THAT CALLS A BUILDER IN A STATEMENT OF ITS OWN AND RETURNS SOMETHING ELSE, so '
      + 'the file that calls it is not demanded in the producer list',
    files: [
      {
        file: 'imaginary/src/statement-wire.ts',
        text: textOf([
          'export function statementResponseFormat(schemaName: string,) {',
          '  contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '  return 1;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/statement-caller.ts',
        text: textOf(['export const STATEMENT = statementResponseFormat(\'statement\',);',],),
      },
    ],
    found: ['imaginary/src/statement-wire.ts',],
    builders: [],
  },
];

/**
 The case of wrappers that return a builder's call with a type mark or an optional chain around it.
 */
export const TYPE_MARK_WRAPPER_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS THE CALL WITH A TYPE ASSERTION, A satisfies, A NON-NULL MARK OR AN '
      + 'OPTIONAL CHAIN AROUND IT, and the files that call the wrappers',
    files: [
      {
        file: 'imaginary/src/asserted-wire.ts',
        text: textOf([
          'export function asResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },) as unknown;',
          '}',
          '',
          'export function satisfiesResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },) satisfies object;',
          '}',
          '',
          'export function nonNullResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },)!;',
          '}',
          '',
          'export function angleResponseFormat(schemaName: string,) {',
          '  return <object>contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
          '',
          'export function optionalResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat?.({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/asserted-gate-stage.ts',
        text: textOf([
          'export const AS_RESPONSE_FORMAT = asResponseFormat(\'as\',);',
          'export const SATISFIES_RESPONSE_FORMAT = satisfiesResponseFormat(\'satisfies\',);',
          'export const NON_NULL_RESPONSE_FORMAT = nonNullResponseFormat(\'non_null\',);',
          'export const ANGLE_RESPONSE_FORMAT = angleResponseFormat(\'angle\',);',
          'export const OPTIONAL_RESPONSE_FORMAT = optionalResponseFormat(\'optional\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/asserted-gate-stage.ts',
      'imaginary/src/asserted-wire.ts',
    ],
    builders: [
      'angleResponseFormat',
      'asResponseFormat',
      'nonNullResponseFormat',
      'optionalResponseFormat',
      'satisfiesResponseFormat',
    ],
  },
];

//endregion Functions that are no wrapper of a builder, and wrappers with a type mark around the call
