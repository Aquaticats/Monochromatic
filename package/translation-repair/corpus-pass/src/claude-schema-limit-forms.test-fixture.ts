import {
  formatLinesOf,
  methodFormatLinesOf,
  nestedMethodFormatLinesOf,
} from './claude-schema-builder-forms.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Builders the search does not read
// What `claude-schemas.unit.test.ts` feeds the builder search to pin the builders it does not read, so the header of
// `claude-schema-sources.test-fixture.ts` states a limit that a case holds: a static method of a class, a getter, a
// member with a computed name, a member of an object nested one level deeper than the first, and an anonymous default
// export. In each case the file that writes the format is listed, since it holds a format, and the file that calls the
// builder is not, since the builder is not read. The check behind the search (`blindSpotsOf`) reports the function that
// holds the format, by file and line.

/**
 Cases of a builder written in a form the search does not read, the file that calls it, and the place of the function
 that holds the format.
 */
export const UNREAD_BUILDER_CASES: readonly ShapeCase[] = [
  {
    name: 'FINDS NO BUILDER IN A STATIC METHOD OF A CLASS, so the file that calls it is not listed',
    files: [
      {
        file: 'imaginary/src/class-wire.ts',
        text: textOf([
          'export class ClassWire {',
          '  static classResponseFormat(moods: readonly string[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'class_mood', },),
          '    };',
          '  }',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/class-gate-stage.ts',
        text: textOf([
          'import { ClassWire, } from \'./class-wire.ts\';',
          '',
          'export const CLASS_GATE_RESPONSE_FORMAT = ClassWire.classResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: ['imaginary/src/class-wire.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/class-wire.ts',
        line: 2,
        shape: 'format-in-undeclared-function',
        position: 'FunctionExpression',
      },
    ],
  },
  {
    name: 'FINDS NO BUILDER IN A GETTER OR IN A MEMBER WITH A COMPUTED NAME, so the file that reads them is not '
      + 'listed',
    files: [
      {
        file: 'imaginary/src/accessor-wire.ts',
        text: textOf([
          'export const accessorKit = {',
          '  get getterResponseFormat() {',
          '    return {',
          ...methodFormatLinesOf({ name: 'getter_mood', },),
          '    };',
          '  },',
          '  [\'computedResponseFormat\'](moods: readonly string[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'computed_mood', },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/accessor-gate-stage.ts',
        text: textOf([
          'import { accessorKit, } from \'./accessor-wire.ts\';',
          '',
          'export const GETTER_GATE_RESPONSE_FORMAT = accessorKit.getterResponseFormat;',
          'export const COMPUTED_GATE_RESPONSE_FORMAT = accessorKit.computedResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: ['imaginary/src/accessor-wire.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/accessor-wire.ts',
        line: 2,
        shape: 'format-in-undeclared-function',
        position: 'FunctionExpression',
      },
      {
        file: 'imaginary/src/accessor-wire.ts',
        line: 11,
        shape: 'format-in-undeclared-function',
        position: 'FunctionExpression',
      },
    ],
  },
  {
    name: 'FINDS NO BUILDER IN A MEMBER OF AN OBJECT NESTED INSIDE THE OBJECT A CONSTANT HOLDS, so the file that '
      + 'calls it is not listed',
    files: [
      {
        file: 'imaginary/src/nested-wire.ts',
        text: textOf([
          'export const nestedKit = {',
          '  inner: {',
          '    deepResponseFormat(moods: readonly string[],) {',
          '      return {',
          ...nestedMethodFormatLinesOf({ name: 'deep_mood', },),
          '      };',
          '    },',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/nested-gate-stage.ts',
        text: textOf([
          'import { nestedKit, } from \'./nested-wire.ts\';',
          '',
          'export const NESTED_GATE_RESPONSE_FORMAT = nestedKit.inner.deepResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: ['imaginary/src/nested-wire.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/nested-wire.ts',
        line: 3,
        shape: 'format-in-undeclared-function',
        position: 'FunctionExpression',
      },
    ],
  },
  {
    name: 'FINDS NO BUILDER IN AN ANONYMOUS DEFAULT EXPORT, so the file that imports it under a name of its own is '
      + 'not listed',
    files: [
      {
        file: 'imaginary/src/anonymous-wire.ts',
        text: textOf([
          'export default function (moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'anonymous_mood', },),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/anonymous-gate-stage.ts',
        text: textOf([
          'import makeAnonymous from \'./anonymous-wire.ts\';',
          '',
          'export const ANONYMOUS_GATE_RESPONSE_FORMAT = makeAnonymous([\'wags\',],);',
        ],),
      },
    ],
    found: ['imaginary/src/anonymous-wire.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/anonymous-wire.ts',
        line: 1,
        shape: 'format-in-undeclared-function',
        position: 'FunctionDeclaration',
      },
    ],
  },
];

/**
 The module of the contest builder, as a specifier written in a file under `imaginary/src`.
 */
const LOAD_SPECIFIER = '../../lane-contest/src/contest-ballot-wire.ts';

/**
 The arguments of a call of the contest builder, as an imaginary file writes them.
 */
const LOAD_ARGUMENTS = '{ schemaName: \'load\', verdicts: [], asksArchive: false, }';

/**
 Cases of a load that is not awaited, which yields a promise and no module.
 */
export const UNAWAITED_LOAD_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS NO NAME FROM AN import() THAT IS NOT AWAITED, held by a constant or read in place, since it yields '
      + 'a promise and no module',
    files: [
      {
        file: 'imaginary/src/unawaited-held-stage.ts',
        text: textOf([
          `const pending = import('${LOAD_SPECIFIER}');`,
          '',
          `export const UNAWAITED_HELD_RESPONSE_FORMAT = pending.contestResponseFormat(${LOAD_ARGUMENTS},);`,
        ],),
      },
      {
        file: 'imaginary/src/unawaited-in-place-stage.ts',
        text: textOf([
          `export const UNAWAITED_IN_PLACE_RESPONSE_FORMAT = import('${LOAD_SPECIFIER}')`,
          `  .contestResponseFormat(${LOAD_ARGUMENTS},);`,
        ],),
      },
    ],
    found: [],
    builders: [],
  },
];

/**
 Cases of the loads the search follows, which the check behind it reports as nothing: held by a constant or taken apart
 by a pattern, awaited or required, and read in place.
 */
export const FOLLOWED_LOAD_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS NO SHAPE IN A LOAD THE SEARCH FOLLOWS, held by a constant, taken apart or read in place, awaited '
      + 'or required',
    files: [
      {
        file: 'imaginary/src/followed-load-held-stage.ts',
        text: textOf([
          `const contest = await import('${LOAD_SPECIFIER}');`,
          `const { contestResponseFormat: lazyFormat, } = await import('${LOAD_SPECIFIER}');`,
          `const required = require('${LOAD_SPECIFIER}');`,
          '',
          `export const HELD_RESPONSE_FORMAT = contest.contestResponseFormat(${LOAD_ARGUMENTS},);`,
          `export const TAKEN_APART_RESPONSE_FORMAT = lazyFormat(${LOAD_ARGUMENTS},);`,
          `export const REQUIRED_RESPONSE_FORMAT = required.contestResponseFormat(${LOAD_ARGUMENTS},);`,
        ],),
      },
      {
        file: 'imaginary/src/followed-load-in-place-stage.ts',
        text: textOf([
          `export const IN_PLACE_RESPONSE_FORMAT = (await import('${LOAD_SPECIFIER}'))`,
          `  .contestResponseFormat(${LOAD_ARGUMENTS},);`,
          `export const REQUIRED_IN_PLACE_RESPONSE_FORMAT = require('${LOAD_SPECIFIER}')`,
          `  .contestResponseFormat(${LOAD_ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/followed-load-held-stage.ts',
      'imaginary/src/followed-load-in-place-stage.ts',
    ],
    builders: [],
    spots: [],
  },
];

//endregion Builders the search does not read
