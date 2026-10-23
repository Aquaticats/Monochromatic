import { formatLinesOf, } from './claude-schema-builder-forms.test-fixture.ts';
import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Wrappers by opening, and wrappers of method and default builders
// What `claude-schemas.unit.test.ts` feeds the builder search to show two things the other fixtures leave open: that a
// wrapper is found whichever of the openings in the list of `claude-schema-code.test-fixture.ts` precedes its call (an
// arrow function whose body is the call with no `await` and no parenthesis, a call that names type arguments), and that
// a wrapper of a builder written as a method of an object or as a default export is followed through the name its file
// gives the builder. Each case is a few imaginary files, with the contest builder (`contestResponseFormat`) beside the
// first two, and the files and builders the search must list.

/**
 Cases of a wrapper written as an arrow function with the call as its body and no `await`, and of a wrapper that
 returns a call naming type arguments, both wrapping the contest builder.
 */
export const WRAPPER_OPENING_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A WRAPPER WRITTEN AS AN ARROW FUNCTION WHOSE BODY IS THE CALL, with no await and no parenthesis, '
      + 'and the file that calls it',
    files: [
      {
        file: 'imaginary/src/plain-arrow-wire.ts',
        text: textOf([
          'export const plainArrowResponseFormat = (schemaName: string,) =>',
          '  contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
        ],),
      },
      {
        file: 'imaginary/src/plain-arrow-gate-stage.ts',
        text: textOf(['export const PLAIN_ARROW_GATE_RESPONSE_FORMAT = plainArrowResponseFormat(\'plain_arrow\',);',],),
      },
    ],
    found: [
      'imaginary/src/plain-arrow-gate-stage.ts',
      'imaginary/src/plain-arrow-wire.ts',
    ],
    builders: ['plainArrowResponseFormat',],
  },
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS A CALL NAMING A TYPE ARGUMENT, and the file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/typed-wire.ts',
        text: textOf([
          'export function typedResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat<string>({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/typed-gate-stage.ts',
        text: textOf(['export const TYPED_GATE_RESPONSE_FORMAT = typedResponseFormat(\'typed_gate\',);',],),
      },
    ],
    found: [
      'imaginary/src/typed-gate-stage.ts',
      'imaginary/src/typed-wire.ts',
    ],
    builders: ['typedResponseFormat',],
  },
];

/**
 Cases of a wrapper of a builder written as a method of an object, which the wrapper calls as `object.member`, and of
 a builder exported as the default, which the wrapper calls by the name its file gives the default import.
 */
export const WRAPPED_BUILDER_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A WRAPPER OF A BUILDER WRITTEN AS A METHOD OF AN OBJECT, called as object.member, and the file that '
      + 'calls the wrapper',
    files: [
      {
        file: 'imaginary/src/kit-wire.ts',
        text: textOf([
          'export const formatKit = {',
          '  kitResponseFormat(moods: readonly string[],) {',
          '    return {',
          ...formatLinesOf({ name: 'kit_mood', },)
            .map(function deeper(line,): string {
              return `    ${line}`;
            },),
          '    };',
          '  },',
          '};',
        ],),
      },
      {
        file: 'imaginary/src/kit-wrapper-wire.ts',
        text: textOf([
          'import { formatKit, } from \'./kit-wire.ts\';',
          '',
          'export function wrappedKitResponseFormat(moods: readonly string[],) {',
          '  return formatKit.kitResponseFormat(moods,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/kit-gate-stage.ts',
        text: textOf(['export const KIT_GATE_RESPONSE_FORMAT = wrappedKitResponseFormat([\'wags\',],);',],),
      },
    ],
    found: [
      'imaginary/src/kit-gate-stage.ts',
      'imaginary/src/kit-wire.ts',
      'imaginary/src/kit-wrapper-wire.ts',
    ],
    builders: [
      'formatKit.kitResponseFormat',
      'wrappedKitResponseFormat',
    ],
  },
  {
    name: 'FOLLOWS A WRAPPER OF A BUILDER EXPORTED AS THE DEFAULT, called by the name its file gives the import, and '
      + 'the file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/default-kit-wire.ts',
        text: textOf([
          'export default function defaultKitResponseFormat(moods: readonly string[],) {',
          '  return {',
          ...formatLinesOf({ name: 'default_kit_mood', },),
          '  };',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/default-kit-wrapper-wire.ts',
        text: textOf([
          'import makeKitFormat from \'./default-kit-wire.ts\';',
          '',
          'export function wrappedDefaultKitResponseFormat(moods: readonly string[],) {',
          '  return makeKitFormat(moods,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/default-kit-gate-stage.ts',
        text: textOf([
          'export const DEFAULT_KIT_GATE_RESPONSE_FORMAT = wrappedDefaultKitResponseFormat([\'wags\',],);',
        ],),
      },
    ],
    found: [
      'imaginary/src/default-kit-gate-stage.ts',
      'imaginary/src/default-kit-wire.ts',
      'imaginary/src/default-kit-wrapper-wire.ts',
    ],
    builders: [
      'defaultKitResponseFormat',
      'wrappedDefaultKitResponseFormat',
    ],
  },
];

//endregion Wrappers by opening, and wrappers of method and default builders
