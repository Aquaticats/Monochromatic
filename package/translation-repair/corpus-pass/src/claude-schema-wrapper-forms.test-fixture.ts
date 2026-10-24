import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Forms in which a function returns the result of a builder of an answer schema
// What `claude-schemas.unit.test.ts` feeds the builder search to show that it finds a wrapper of a builder whatever
// way the wrapper returns the call, and that it names nothing in the files that only look like a builder or a use.
// Each case is a few imaginary files, with the contest builder (`contestResponseFormat`) beside them, and the files
// and builders the search must list.

/**
 Cases in which a function returns the result of a builder, in the forms the search reads, and a file calls it.
 */
export const WRAPPER_FORM_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS A BUILDER\'S CALL, and the file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/wrapped-wire.ts',
        text: textOf([
          'export function wrappedResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/wrapped-gate-stage.ts',
        text: textOf(['export const WRAPPED_GATE_RESPONSE_FORMAT = wrappedResponseFormat(\'wrapped_gate\',);',],),
      },
    ],
    found: [
      'imaginary/src/wrapped-gate-stage.ts',
      'imaginary/src/wrapped-wire.ts',
    ],
    builders: ['wrappedResponseFormat',],
  },
  {
    name: 'FOLLOWS A WRAPPER OF A WRAPPER, and the file that calls the outer one, so the search for wrappers goes on '
      + 'until a pass adds none',
    files: [
      {
        file: 'imaginary/src/wrapped-wire.ts',
        text: textOf([
          'export function wrappedResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/doubly-wire.ts',
        text: textOf([
          'export function doublyWrappedResponseFormat(schemaName: string,) {',
          '  return wrappedResponseFormat(schemaName,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/doubly-gate-stage.ts',
        text: textOf(['export const DOUBLY_GATE_RESPONSE_FORMAT = doublyWrappedResponseFormat(\'doubly_gate\',);',],),
      },
    ],
    found: [
      'imaginary/src/doubly-gate-stage.ts',
      'imaginary/src/doubly-wire.ts',
      'imaginary/src/wrapped-wire.ts',
    ],
    builders: [
      'doublyWrappedResponseFormat',
      'wrappedResponseFormat',
    ],
  },
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS AN AWAITED CALL, and the file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/awaited-wire.ts',
        text: textOf([
          'export async function awaitedResponseFormat(schemaName: string,) {',
          '  return await contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/awaited-gate-stage.ts',
        text: textOf(['export const AWAITED_GATE_RESPONSE_FORMAT = await awaitedResponseFormat(\'awaited_gate\',);',],),
      },
    ],
    found: [
      'imaginary/src/awaited-gate-stage.ts',
      'imaginary/src/awaited-wire.ts',
    ],
    builders: ['awaitedResponseFormat',],
  },
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS A PARENTHESISED CALL ACROSS LINES, and the file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/parenthesised-wire.ts',
        text: textOf([
          'export function parenthesisedResponseFormat(schemaName: string,) {',
          '  return (',
          '    contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },)',
          '  );',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/parenthesised-gate-stage.ts',
        text: textOf([
          'export const PARENTHESISED_GATE_RESPONSE_FORMAT = parenthesisedResponseFormat(\'parenthesised_gate\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/parenthesised-gate-stage.ts',
      'imaginary/src/parenthesised-wire.ts',
    ],
    builders: ['parenthesisedResponseFormat',],
  },
  {
    name: 'FOLLOWS A WRAPPER WRITTEN AS AN ASYNC ARROW FUNCTION THAT AWAITS THE CALL, and the file that calls '
      + 'the wrapper',
    files: [
      {
        file: 'imaginary/src/arrow-awaited-wire.ts',
        text: textOf([
          'export const arrowAwaitedResponseFormat = async (schemaName: string,) =>',
          '  await contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
        ],),
      },
      {
        file: 'imaginary/src/arrow-awaited-gate-stage.ts',
        text: textOf([
          'export const ARROW_AWAITED_GATE_RESPONSE_FORMAT = await arrowAwaitedResponseFormat(',
          '  \'arrow_awaited_gate\',',
          ');',
        ],),
      },
    ],
    found: [
      'imaginary/src/arrow-awaited-gate-stage.ts',
      'imaginary/src/arrow-awaited-wire.ts',
    ],
    builders: ['arrowAwaitedResponseFormat',],
  },
  {
    name: 'FOLLOWS A WRAPPER WRITTEN AS AN ARROW FUNCTION WHOSE BODY IS A PARENTHESISED CALL, and the file that '
      + 'calls it',
    files: [
      {
        file: 'imaginary/src/arrow-parenthesised-wire.ts',
        text: textOf([
          'export const arrowParenthesisedResponseFormat = (schemaName: string,) => (',
          '  contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },)',
          ');',
        ],),
      },
      {
        file: 'imaginary/src/arrow-parenthesised-gate-stage.ts',
        text: textOf([
          'export const ARROW_PARENTHESISED_GATE_RESPONSE_FORMAT = arrowParenthesisedResponseFormat(',
          '  \'parenthesised\',',
          ');',
        ],),
      },
    ],
    found: [
      'imaginary/src/arrow-parenthesised-gate-stage.ts',
      'imaginary/src/arrow-parenthesised-wire.ts',
    ],
    builders: ['arrowParenthesisedResponseFormat',],
  },
];

/**
 Cases in which the files look like a builder or a use of one and are none: an import, a re-export, a type, an alias
 that is never called, a property of the same name read off some other object, a name against a quote, and a function
 that calls a builder without returning its result, whose caller therefore builds no schema. The last case is the
 reverse: a name inside a longer string or in a comment after code, which the search reads as a use on purpose.
 */
export const NO_BUILDER_CASES: readonly FormCase[] = [
  {
    name: 'NAMES NOTHING IN A FILE THAT ONLY IMPORTS, RE-EXPORTS OR NAMES THE TYPE OF A BUILDER, or reads a '
      + 'property of the same name off some other object',
    files: [
      {
        file: 'imaginary/src/import-only.ts',
        text: textOf([
          'import {',
          '  contestResponseFormat,',
          '} from \'../../lane-contest/src/contest-ballot-wire.ts\';',
          '',
          'export const NOTHING = 2;',
        ],),
      },
      {
        file: 'imaginary/src/alias-import-only.ts',
        text: textOf([
          'import {',
          '  contestResponseFormat as formatOf,',
          '} from \'../../lane-contest/src/contest-ballot-wire.ts\';',
          '',
          'export const NOTHING_ELSE = 3;',
        ],),
      },
      {
        file: 'imaginary/src/re-export.ts',
        text: textOf([
          'export {',
          '  contestResponseFormat,',
          '} from \'../../lane-contest/src/contest-ballot-wire.ts\';',
        ],),
      },
      {
        file: 'imaginary/src/type-only.ts',
        text: textOf(['export type Maker = typeof contestResponseFormat;',],),
      },
      {
        file: 'imaginary/src/property-only.ts',
        text: textOf(['export const READ = someOtherObject.contestResponseFormat;',],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES NO WRAPPER IN A FUNCTION THAT CALLS A BUILDER WITHOUT RETURNING ITS RESULT, so the file that calls '
      + 'that function is not demanded in the producer list',
    files: [
      {
        file: 'imaginary/src/asking-wire.ts',
        text: textOf([
          'export function askWithFormat(schemaName: string,) {',
          '  const format = contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '  return send(format,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/asking-caller.ts',
        text: textOf(['export const ASKED = askWithFormat(\'asked\',);',],),
      },
    ],
    found: ['imaginary/src/asking-wire.ts',],
    builders: [],
  },
  {
    name: 'NAMES NOTHING IN A STRING THAT HOLDS ONLY A BUILDER\'S NAME OR STARTS OR ENDS WITH IT, because a quote '
      + 'stands against the name',
    files: [
      {
        file: 'imaginary/src/quoted-name.ts',
        text: textOf([
          'export const ONLY = \'contestResponseFormat\';',
          'export const STARTS = \'contestResponseFormat is the builder\';',
          'export const ENDS = \'the builder is contestResponseFormat\';',
          'export const TEMPLATE = `contestResponseFormat`;',
        ],),
      },
    ],
    found: [],
    builders: [],
  },
  {
    name: 'NAMES NOTHING IN A LONGER STRING OR IN A COMMENT THAT TRAILS CODE, because the search reads the syntax '
      + 'tree, where a string is one value and a comment is not a node',
    files: [
      {
        file: 'imaginary/src/message-name.ts',
        text: textOf(['export const MESSAGE = \'call contestResponseFormat() here\';',],),
      },
      {
        file: 'imaginary/src/trailing-comment-name.ts',
        text: textOf(['export const NOTHING = 4; // contestResponseFormat(...) is the builder',],),
      },
    ],
    found: [],
    builders: [],
  },
];

//endregion Forms in which a function returns the result of a builder of an answer schema
