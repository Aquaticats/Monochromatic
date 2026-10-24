import {
  formatLinesOf,
  methodFormatLinesOf,
  propertyFormatLinesOf,
} from './claude-schema-builder-forms.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Declarations of builders that no file exports, and members written in other ways
// What `claude-schemas.unit.test.ts` feeds the builder search for the declaration shapes the re-check of the search
// named: a builder or a wrapper that the file does not export, a member of an object written as an asynchronous
// method, as a property holding an arrow function, as a generic method, or on the line that opens the object. Each
// case is a few imaginary files, and the files and builders the search must list.

/**
 Cases of a builder and a wrapper that no file exports, of members of an object written in the other ways, and of an
 object written on one line.
 */
export const DECLARATION_SHAPE_CASES: readonly FormCase[] = [
  {
    name: 'FINDS A BUILDER DECLARED WITH function AND NO export, and the file that calls the exported function '
      + 'that returns its result',
    files: [
      {
        file: 'imaginary/src/hidden-wire.ts',
        text: textOf([
          'function hiddenResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'hidden_mood', },),
          '  };',
          '}',
          '',
          'export function exposedResponseFormat(moods: readonly string[],) {',
          '  return hiddenResponseFormat(moods,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/hidden-gate-stage.ts',
        text: textOf(['export const HIDDEN_GATE_RESPONSE_FORMAT = exposedResponseFormat([\'wags\',],);',],),
      },
    ],
    found: [
      'imaginary/src/hidden-gate-stage.ts',
      'imaginary/src/hidden-wire.ts',
    ],
    builders: [
      'exposedResponseFormat',
      'hiddenResponseFormat',
    ],
  },
  {
    name: 'FINDS A BUILDER DECLARED WITH async function AND NO export, and the file that calls the exported '
      + 'function that returns its result',
    files: [
      {
        file: 'imaginary/src/hidden-async-wire.ts',
        text: textOf([
          'async function hiddenAsyncResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'hidden_async_mood', },),
          '  };',
          '}',
          '',
          'export async function exposedAsyncResponseFormat(moods: readonly string[],) {',
          '  return await hiddenAsyncResponseFormat(moods,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/hidden-async-gate-stage.ts',
        text: textOf([
          'export const HIDDEN_ASYNC_GATE_RESPONSE_FORMAT = await exposedAsyncResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/hidden-async-gate-stage.ts',
      'imaginary/src/hidden-async-wire.ts',
    ],
    builders: [
      'exposedAsyncResponseFormat',
      'hiddenAsyncResponseFormat',
    ],
  },
  {
    name: 'FINDS A BUILDER HELD BY A CONSTANT THAT IS NOT EXPORTED, and the file that calls the exported function '
      + 'that returns its result',
    files: [
      {
        file: 'imaginary/src/hidden-constant-wire.ts',
        text: textOf([
          'const hiddenConstantResponseFormat = (moods: readonly string[],) => ({',
          ...formatLinesOf({ name: 'hidden_constant_mood', },),
          '});',
          '',
          'export const exposedConstantResponseFormat = (moods: readonly string[],) =>',
          '  hiddenConstantResponseFormat(moods,);',
        ],),
      },
      {
        file: 'imaginary/src/hidden-constant-gate-stage.ts',
        text: textOf([
          'export const HIDDEN_CONSTANT_GATE_RESPONSE_FORMAT = exposedConstantResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/hidden-constant-gate-stage.ts',
      'imaginary/src/hidden-constant-wire.ts',
    ],
    builders: [
      'exposedConstantResponseFormat',
      'hiddenConstantResponseFormat',
    ],
  },
  {
    name: 'FINDS BUILDERS WRITTEN AS AN ASYNC METHOD, A PROPERTY HOLDING AN ARROW FUNCTION AND A GENERIC METHOD of '
      + 'an object, and the file that calls them through the object',
    files: [
      {
        file: 'imaginary/src/member-forms-wire.ts',
        text: textOf([
          'export const memberKit = {',
          '  async asyncResponseFormat(moods: readonly string[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'async_member_mood', },),
          '    };',
          '  },',
          '  propertyResponseFormat: (moods: readonly string[],) => ({',
          ...propertyFormatLinesOf({ name: 'property_member_mood', },),
          '  }),',
          '  genericResponseFormat<Mood extends string,>(moods: readonly Mood[],) {',
          '    return {',
          ...methodFormatLinesOf({ name: 'generic_member_mood', },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/member-forms-gate-stage.ts',
        text: textOf([
          'import { memberKit, } from \'./member-forms-wire.ts\';',
          '',
          'export const ASYNC_MEMBER_RESPONSE_FORMAT = await memberKit.asyncResponseFormat([\'wags\',],);',
          'export const PROPERTY_MEMBER_RESPONSE_FORMAT = memberKit.propertyResponseFormat([\'wags\',],);',
          'export const GENERIC_MEMBER_RESPONSE_FORMAT = memberKit.genericResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/member-forms-gate-stage.ts',
      'imaginary/src/member-forms-wire.ts',
    ],
    builders: [
      'memberKit.asyncResponseFormat',
      'memberKit.genericResponseFormat',
      'memberKit.propertyResponseFormat',
    ],
  },
  {
    name: 'FINDS A BUILDER THAT IS A MEMBER ON THE LINE THAT OPENS THE OBJECT, as in `{ member: () => ({ ... }) }` '
      + 'written on one line, and the file that calls it',
    files: [
      {
        file: 'imaginary/src/inline-wire.ts',
        text: textOf([
          'export const inlineKit = { inlineResponseFormat: () => ({ type: \'json_schema\', json_schema: {}, }), };',
        ],),
      },
      {
        file: 'imaginary/src/inline-gate-stage.ts',
        text: textOf([
          'import { inlineKit, } from \'./inline-wire.ts\';',
          '',
          'export const INLINE_GATE_RESPONSE_FORMAT = inlineKit.inlineResponseFormat();',
        ],),
      },
    ],
    found: [
      'imaginary/src/inline-gate-stage.ts',
      'imaginary/src/inline-wire.ts',
    ],
    builders: ['inlineKit.inlineResponseFormat',],
  },
];

//endregion Declarations of builders that no file exports, and members written in other ways
