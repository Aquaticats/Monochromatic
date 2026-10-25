import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Calls of a builder whose result leaves a function the search does not know
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) to hold the part
// of the second total that follows the value a builder's call gives: it leaves a function through a `return`, an arrow
// function's body or a `yield`, whole or inside an object, an array, a spread, a fallback, a conditional, a sequence, a
// promise chain or a member read off it, or through a local, a pattern, a default parameter or a name assigned. The
// case is one imaginary file with one such place on each function, beside the file that calls through it. The calls
// the search reads, which the check reports nothing for, are in `claude-schema-whitelist-read-forms.test-fixture.ts`.

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The import of the contest builder that opens each stage file.
 */
const CONTEST_IMPORT = `import { contestResponseFormat, } from '${CONTEST_SPECIFIER}';`;

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'call\', verdicts: [], asksArchive: false, }';

/**
 Cases of a call whose value leaves a function the search does not know as a builder.
 */
export const WHITELIST_CALL_CASES: readonly ShapeCase[] = [
  {
    name: 'LEAVES UNREAD A CALL OF A BUILDER WHOSE RESULT LEAVES A FUNCTION THE SEARCH DOES NOT KNOW AS A BUILDER, '
      + 'whole, inside an object, an array, a spread, a fallback, a promise chain, a member, a conditional or a '
      + 'sequence, through a local, a pattern, a default parameter or a name assigned, by return, arrow body or yield, '
      + 'so the files that call that function are not listed, and the check reports each call',
    files: [
      {
        file: 'imaginary/src/leaving-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          'export class Kit {',
          `  made = () => contestResponseFormat(${ARGUMENTS},);`,
          '  get format() {',
          `    return contestResponseFormat(${ARGUMENTS},);`,
          '  }',
          '  static make() {',
          `    return contestResponseFormat(${ARGUMENTS},);`,
          '  }',
          '}',
          '',
          'export const kit = {',
          '  inner: {',
          '    make() {',
          `      return contestResponseFormat(${ARGUMENTS},);`,
          '    },',
          '  },',
          '};',
          '',
          `export function withDefaultCall(format = contestResponseFormat(${ARGUMENTS},),) {`,
          '  return format;',
          '}',
          '',
          'export function spreads() {',
          `  return { ...contestResponseFormat(${ARGUMENTS},), extra: 1, };`,
          '}',
          '',
          'export function inObject() {',
          `  return { format: contestResponseFormat(${ARGUMENTS},), };`,
          '}',
          '',
          'export async function awaitedObject() {',
          `  return { format: await contestResponseFormat(${ARGUMENTS},), };`,
          '}',
          '',
          'export function withFallback() {',
          `  return contestResponseFormat(${ARGUMENTS},) ?? {};`,
          '}',
          '',
          'export function chained() {',
          `  return contestResponseFormat(${ARGUMENTS},).then((format,) => format,);`,
          '}',
          '',
          'export function member() {',
          `  return contestResponseFormat(${ARGUMENTS},).json_schema;`,
          '}',
          '',
          'export function conditional(flag: boolean,) {',
          `  return flag ? contestResponseFormat(${ARGUMENTS},) : undefined;`,
          '}',
          '',
          'export function listed() {',
          `  return [contestResponseFormat(${ARGUMENTS},),];`,
          '}',
          '',
          'export function sequenced() {',
          `  return (log(), contestResponseFormat(${ARGUMENTS}));`,
          '}',
          '',
          'export function* generated() {',
          `  yield contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export function throughLocal() {',
          `  const format = contestResponseFormat(${ARGUMENTS},);`,
          '  return format;',
          '}',
          '',
          'export function throughPattern() {',
          `  const { json_schema: schema, } = contestResponseFormat(${ARGUMENTS},);`,
          '  return schema;',
          '}',
          '',
          'export function throughObject() {',
          `  const format = contestResponseFormat(${ARGUMENTS},);`,
          '  const held = { format, };',
          '  return held;',
          '}',
          '',
          'let cache;',
          '',
          'export function fill() {',
          `  cache = contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export function read() {',
          '  return cache;',
          '}',
          '',
          'export function throughArray() {',
          `  const [format,] = contestResponseFormat(${ARGUMENTS},);`,
          '  return format;',
          '}',
          '',
          'export function throughDefault() {',
          `  const { json_schema: schema = {}, } = contestResponseFormat(${ARGUMENTS},);`,
          '  return schema;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/leaving-caller.ts',
        text: textOf([
          'import { chained, } from \'./leaving-stage.ts\';',
          '',
          'export const CALLED = chained();',
        ],),
      },
    ],
    found: ['imaginary/src/leaving-stage.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 4,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'ArrowFunctionExpression returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 6,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionExpression returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 9,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionExpression returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 16,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionExpression returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 21,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 26,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 30,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 34,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 38,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 42,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 42,
        shape: 'code-the-search-cannot-read',
        position: 'callback of then cannot be read',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 46,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 50,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 54,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 58,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 62,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 66,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 71,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 76,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 84,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 92,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
      {
        file: 'imaginary/src/leaving-stage.ts',
        line: 97,
        shape: 'builder-result-returned-outside-a-wrapper',
        position: 'FunctionDeclaration returns it',
      },
    ],
  },
];

//endregion Calls of a builder whose result leaves a function the search does not know
