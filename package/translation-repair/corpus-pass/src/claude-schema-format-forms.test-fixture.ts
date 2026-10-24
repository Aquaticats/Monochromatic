import {
  formatLinesOf,
  methodFormatLinesOf,
} from './claude-schema-builder-forms.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Forms in which an answer schema's format is written
// What `claude-schemas.unit.test.ts` feeds the builder search to show what makes a file hold a format and a function a
// builder: an object literal with a property `type` whose value is the string `json_schema`, in any spelling the
// language allows, and nothing else. Each case is a few imaginary files, with the contest builder
// (`contestResponseFormat`) beside them, and the files and builders the search must list.

/**
 The lines of an imaginary builder's format with the line that names its type written another way.

 @param typeLine - the property that names the type, as the source writes it, without its indentation

 @returns Lines of an object literal, indented for a function body

 @example
 ```ts
 const lines = spelledFormatLinesOf('type: "json_schema"');
 ```
 */
function spelledFormatLinesOf(typeLine: string,): readonly string[] {
  return [
    `  ${typeLine},`,
    ...formatLinesOf({ name: 'spelled_mood', },)
      .slice(1,),
  ];
}

/**
 Cases in which a builder writes the property that makes its format with a double-quoted value, a template, a quoted
 key and a computed key, and the file that calls it.
 */
export const SPELLING_CASES: readonly FormCase[] = [
  {
    name: 'FINDS A BUILDER WHOSE FORMAT IS WRITTEN WITH A DOUBLE-QUOTED VALUE, and the file that calls it',
    files: [
      {
        file: 'imaginary/src/double-quoted-wire.ts',
        text: textOf([
          'export function doubleQuotedResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...spelledFormatLinesOf('type: "json_schema"',),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/double-quoted-gate-stage.ts',
        text: textOf([
          'export const DOUBLE_QUOTED_GATE_RESPONSE_FORMAT = doubleQuotedResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/double-quoted-gate-stage.ts',
      'imaginary/src/double-quoted-wire.ts',
    ],
    builders: ['doubleQuotedResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER WHOSE FORMAT IS WRITTEN WITH A TEMPLATE THAT HAS NO SUBSTITUTION, and the file that '
      + 'calls it',
    files: [
      {
        file: 'imaginary/src/template-wire.ts',
        text: textOf([
          'export function templateResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...spelledFormatLinesOf('type: `json_schema`',),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/template-gate-stage.ts',
        text: textOf(['export const TEMPLATE_GATE_RESPONSE_FORMAT = templateResponseFormat([\'wags\',],);',],),
      },
    ],
    found: [
      'imaginary/src/template-gate-stage.ts',
      'imaginary/src/template-wire.ts',
    ],
    builders: ['templateResponseFormat',],
  },
  {
    name: 'FINDS A BUILDER WHOSE FORMAT HAS ITS KEY IN QUOTES OR IN BRACKETS, and the file that calls it',
    files: [
      {
        file: 'imaginary/src/quoted-key-wire.ts',
        text: textOf([
          'export function quotedKeyResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...spelledFormatLinesOf('\'type\': \'json_schema\'',),
          '  };',
          '}',
          '',
          'export function computedKeyResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...spelledFormatLinesOf('[\'type\']: \'json_schema\'',),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/quoted-key-gate-stage.ts',
        text: textOf([
          'export const QUOTED_KEY_GATE_RESPONSE_FORMAT = quotedKeyResponseFormat([\'wags\',],);',
          'export const COMPUTED_KEY_GATE_RESPONSE_FORMAT = computedKeyResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/quoted-key-gate-stage.ts',
      'imaginary/src/quoted-key-wire.ts',
    ],
    builders: [
      'computedKeyResponseFormat',
      'quotedKeyResponseFormat',
    ],
  },
];

/**
 Cases of what is no format or makes no builder: the text of a format in a comment or a string, the string
 `json_schema` under another key or in an array, a type that names the format, a format held by a constant, and a
 format written in a function inside the builder.
 */
export const FORMAT_LIMIT_CASES: readonly FormCase[] = [
  {
    name: 'NAMES NOTHING IN THE TEXT OF A FORMAT IN A COMMENT OR A STRING, or in the string json_schema under '
      + 'another key or in an array',
    files: [
      {
        file: 'imaginary/src/format-in-text.ts',
        text: textOf([
          '/**',
          ' A format looks like { type: \'json_schema\', json_schema: { name: \'x\', schema: {}, }, }.',
          ' */',
          String.raw`export const SHOWN = 'type: \'json_schema\'';`,
          'export const KINDS = [\'json_schema\', \'text\',];',
          'export const OTHER = { kind: \'json_schema\', };',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES A FILE THAT HOLDS ONLY A TYPE OF THE FORMAT, as in `readonly type: \'json_schema\'`, and no function',
    files: [
      {
        file: 'imaginary/src/format-type.ts',
        text: textOf([
          'export type Envelope = {',
          '  readonly type: \'json_schema\';',
          '};',
        ],),
      },
    ],
    found: ['imaginary/src/format-type.ts',],
    builders: [],
  },
  {
    name: 'FINDS NO BUILDER IN A CONSTANT THAT HOLDS A FORMAT, so the file that reads the constant is not demanded '
      + 'in the producer list',
    files: [
      {
        file: 'imaginary/src/constant-wire.ts',
        text: textOf([
          'export const CONSTANT_RESPONSE_FORMAT = {',
          ...formatLinesOf({ name: 'constant_mood', },),
          '};',
        ],),
      },
      {
        file: 'imaginary/src/constant-reader.ts',
        text: textOf([
          'import { CONSTANT_RESPONSE_FORMAT, } from \'./constant-wire.ts\';',
          '',
          'export const COPY = CONSTANT_RESPONSE_FORMAT;',
        ],),
      },
    ],
    found: ['imaginary/src/constant-wire.ts',],
    builders: [],
  },
  {
    name: 'FINDS A BUILDER THAT WRITES ITS FORMAT IN A FUNCTION INSIDE IT, which is the builder and not the inner '
      + 'one',
    files: [
      {
        file: 'imaginary/src/inner-wire.ts',
        text: textOf([
          'export function outerResponseFormat(moods: readonly string[],) {',
          '  function innerFormat() {',
          '    return {',
          ...methodFormatLinesOf({ name: 'inner_mood', },),
          '    };',
          '  }',
          '  return innerFormat();',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/inner-gate-stage.ts',
        text: textOf(['export const INNER_GATE_RESPONSE_FORMAT = outerResponseFormat([\'wags\',],);',],),
      },
    ],
    found: [
      'imaginary/src/inner-gate-stage.ts',
      'imaginary/src/inner-wire.ts',
    ],
    builders: ['outerResponseFormat',],
  },
];

//endregion Forms in which an answer schema's format is written
