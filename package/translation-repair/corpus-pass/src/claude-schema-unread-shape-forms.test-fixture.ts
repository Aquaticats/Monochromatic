import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Shapes the schema search does not read, one file each
// What `claude-schemas.unit.test.ts` feeds the builder search and the check behind it (`blindSpotsOf`) to show shapes the
// search leaves unread: a load whose path is no literal, a load in a form no binding follows, a builder stored as the
// value of an object, a builder handed to a call that a constant holds, a builder's result returned through a local,
// and a builder called in a namespace, an enum or a parameter property. In each case the files the search lists show
// what it reads of the shape (the file that names the builder, not the file that calls what the shape holds), and the
// spots show where the check reports it, by file and line.

/**
 One shape the search leaves unread, with the files that show it, what the search lists, and where the check reports it.
 */
export type ShapeCase = FormCase & {
  /**
   The places the check reports in the imaginary files, in the order of the files and then of the lines.
   */
  readonly spots: readonly BlindSpot[];
};

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const CONTEST_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The character that opens a substitution in a template, which an imaginary file's text spells out beside a brace.
 */
const DOLLAR = '$';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const ARGUMENTS = '{ schemaName: \'shape\', verdicts: [], asksArchive: false, }';

/**
 Cases of a shape the search leaves unread, with the places the check reports.
 */
export const UNREAD_SHAPE_CASES: readonly ShapeCase[] = [
  {
    name: 'LEAVES UNREAD A LOAD WHOSE PATH IS A NAME, A JOINED PATH OR A TEMPLATE WITH A SUBSTITUTION, and the check '
      + 'reports each',
    files: [
      {
        file: 'imaginary/src/blind-computed-path.ts',
        text: textOf([
          `const path = '${CONTEST_SPECIFIER}';`,
          'const contest = await import(path);',
          'const required = require(\'../../lane-contest/\' + \'src/contest-ballot-wire.ts\');',
          `const templated = await import(\`${DOLLAR}{path}\`);`,
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/blind-computed-path.ts',
        line: 2,
        shape: 'computed-load-path',
      },
      {
        file: 'imaginary/src/blind-computed-path.ts',
        line: 3,
        shape: 'computed-load-path',
      },
      {
        file: 'imaginary/src/blind-computed-path.ts',
        line: 4,
        shape: 'computed-load-path',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD A LOAD THAT IS HELD UN-AWAITED, CHAINED WITH .then OR HANDED TO A CALL, and the check reports '
      + 'each',
    files: [
      {
        file: 'imaginary/src/blind-load-forms.ts',
        text: textOf([
          `const pending = import('${CONTEST_SPECIFIER}');`,
          '',
          'export function viaThen() {',
          `  return import('${CONTEST_SPECIFIER}')`,
          `    .then((contest,) => contest.contestResponseFormat(${ARGUMENTS},),);`,
          '}',
          '',
          'export function viaArgument() {',
          `  return send(require('${CONTEST_SPECIFIER}'),);`,
          '}',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/blind-load-forms.ts',
        line: 1,
        shape: 'load-in-unfollowed-form',
      },
      {
        file: 'imaginary/src/blind-load-forms.ts',
        line: 4,
        shape: 'load-in-unfollowed-form',
      },
      {
        file: 'imaginary/src/blind-load-forms.ts',
        line: 9,
        shape: 'load-in-unfollowed-form',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD THE CALL OF A BUILDER STORED AS THE VALUE OF AN OBJECT, so the file that calls it through the '
      + 'object is not listed, and the check reports the place that stores it',
    files: [
      {
        file: 'imaginary/src/blind-object-stage.ts',
        text: textOf([
          'export const kit = { make: contestResponseFormat, };',
          'export const short = { contestResponseFormat, };',
        ],),
      },
      {
        file: 'imaginary/src/blind-object-caller.ts',
        text: textOf([
          'import { kit, } from \'./blind-object-stage.ts\';',
          '',
          `export const CALLED = kit.make(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: ['imaginary/src/blind-object-stage.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/blind-object-stage.ts',
        line: 1,
        shape: 'builder-stored-in-object',
      },
      {
        file: 'imaginary/src/blind-object-stage.ts',
        line: 2,
        shape: 'builder-stored-in-object',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD THE CALL OF A BUILDER HANDED TO A CALL THAT A CONSTANT HOLDS, as in `const memo = '
      + 'memoize(build)`, so the file that calls the constant is not listed, and the check reports the place',
    files: [
      {
        file: 'imaginary/src/blind-memoize-stage.ts',
        text: textOf([
          'export const memoized = memoize(contestResponseFormat);',
          'export const MAPPED = labels.map(contestResponseFormat);',
        ],),
      },
      {
        file: 'imaginary/src/blind-memoize-caller.ts',
        text: textOf([
          'import { memoized, } from \'./blind-memoize-stage.ts\';',
          '',
          `export const CALLED = memoized(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: ['imaginary/src/blind-memoize-stage.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/blind-memoize-stage.ts',
        line: 1,
        shape: 'builder-handed-to-a-call-held-by-a-constant',
      },
      {
        file: 'imaginary/src/blind-memoize-stage.ts',
        line: 2,
        shape: 'builder-handed-to-a-call-held-by-a-constant',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD A FUNCTION THAT RETURNS A BUILDER\'S RESULT THROUGH A LOCAL, so it is no wrapper and the file '
      + 'that calls it is not listed, and the check reports the return',
    files: [
      {
        file: 'imaginary/src/blind-local-wire.ts',
        text: textOf([
          'export function throughLocal(schemaName: string,) {',
          '  const format = contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '  return format;',
          '}',
          '',
          'export async function throughAwaitedLocal(schemaName: string,) {',
          '  const format = await contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '  return format;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/blind-local-caller.ts',
        text: textOf([
          'export const CALLED = throughLocal(\'local_gate\',);',
          'export const AWAITED = await throughAwaitedLocal(\'awaited_local_gate\',);',
        ],),
      },
    ],
    found: ['imaginary/src/blind-local-wire.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/blind-local-wire.ts',
        line: 3,
        shape: 'result-returned-through-a-local',
      },
      {
        file: 'imaginary/src/blind-local-wire.ts',
        line: 8,
        shape: 'result-returned-through-a-local',
      },
    ],
  },
  {
    name: 'LEAVES UNREAD A BUILDER CALLED IN A NAMESPACE, AN ENUM INITIALIZER OR A PARAMETER PROPERTY, and the check '
      + 'reports each',
    files: [
      {
        file: 'imaginary/src/blind-typescript-forms.ts',
        text: textOf([
          'export namespace Formats {',
          `  export const FORMAT = contestResponseFormat(${ARGUMENTS},);`,
          '}',
          '',
          'export enum Kinds {',
          `  Gate = contestResponseFormat(${ARGUMENTS},) as unknown as number,`,
          '}',
          '',
          'export class Holder {',
          `  constructor(private readonly format = contestResponseFormat(${ARGUMENTS},),) {}`,
          '}',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/blind-typescript-forms.ts',
        line: 1,
        shape: 'builder-in-namespace-enum-or-parameter-property',
      },
      {
        file: 'imaginary/src/blind-typescript-forms.ts',
        line: 5,
        shape: 'builder-in-namespace-enum-or-parameter-property',
      },
      {
        file: 'imaginary/src/blind-typescript-forms.ts',
        line: 10,
        shape: 'builder-in-namespace-enum-or-parameter-property',
      },
    ],
  },
];

//endregion Shapes the schema search does not read, one file each
