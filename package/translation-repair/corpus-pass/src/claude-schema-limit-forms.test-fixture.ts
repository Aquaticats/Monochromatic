import {
  formatLinesOf,
  methodFormatLinesOf,
  nestedMethodFormatLinesOf,
} from './claude-schema-builder-forms.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Builders the search does not read
// What `claude-schemas.unit.test.ts` feeds the builder search to pin the builders it does not read, so the header of
// `claude-schema-sources.test-fixture.ts` states a limit that a case holds: a static method of a class, a getter, a
// member with a computed name, a member of an object nested one level deeper than the first, and an anonymous default
// export. In each case the file that writes the format is listed, since it holds a format, and the file that calls the
// builder is not, since the builder is not read.

/**
 Cases of a builder written in a form the search does not read, and the file that calls it.
 */
export const UNREAD_BUILDER_CASES: readonly FormCase[] = [
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
  },
];

//endregion Builders the search does not read
