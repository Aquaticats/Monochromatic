import {
  type FormCase,
  textOf,
} from './claude-schema-call-forms.test-fixture.ts';

//region Wrappers that return the call in a nested form, and chains of wrappers
// What `claude-schemas.unit.test.ts` feeds the builder search to show that a wrapper is found when its call sits in
// parentheses inside an `await`, and that the search for wrappers goes on through a chain of three. Each case is a few
// imaginary files, with the contest builder (`contestResponseFormat`) beside them, and the files and builders the
// search must list.

/**
 Cases of a wrapper whose call sits in parentheses inside an `await`, and of a chain of three wrappers.
 */
export const WRAPPER_NESTING_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS AN AWAITED CALL IN PARENTHESES, as in `return await (build(...))`, and the '
      + 'file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/awaited-parenthesised-wire.ts',
        text: textOf([
          'export async function awaitedParenthesisedResponseFormat(schemaName: string,) {',
          '  return await (contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },));',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/awaited-parenthesised-gate-stage.ts',
        text: textOf([
          'export const AWAITED_PARENTHESISED_GATE_RESPONSE_FORMAT = await awaitedParenthesisedResponseFormat(',
          '  \'awaited_parenthesised\',',
          ');',
        ],),
      },
    ],
    found: [
      'imaginary/src/awaited-parenthesised-gate-stage.ts',
      'imaginary/src/awaited-parenthesised-wire.ts',
    ],
    builders: ['awaitedParenthesisedResponseFormat',],
  },
  {
    name: 'FOLLOWS A CHAIN OF THREE WRAPPERS, each returning the call of the one before it, and the file that calls '
      + 'the last, so the search for wrappers is not cut short',
    files: [
      {
        file: 'imaginary/src/chain-wire.ts',
        text: textOf([
          'export function firstLinkResponseFormat(schemaName: string,) {',
          '  return contestResponseFormat({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
          '',
          'export function secondLinkResponseFormat(schemaName: string,) {',
          '  return firstLinkResponseFormat(schemaName,);',
          '}',
          '',
          'export function thirdLinkResponseFormat(schemaName: string,) {',
          '  return secondLinkResponseFormat(schemaName,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/chain-gate-stage.ts',
        text: textOf(['export const CHAIN_GATE_RESPONSE_FORMAT = thirdLinkResponseFormat(\'chain_gate\',);',],),
      },
    ],
    found: [
      'imaginary/src/chain-gate-stage.ts',
      'imaginary/src/chain-wire.ts',
    ],
    builders: [
      'firstLinkResponseFormat',
      'secondLinkResponseFormat',
      'thirdLinkResponseFormat',
    ],
  },
];

//endregion Wrappers that return the call in a nested form, and chains of wrappers
