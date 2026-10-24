import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A function that hands a sender a builder call made from its own parameters
// A function that makes a schema from what its caller passes and hands it to a sender is a forwarder: the files that
// call it supply the arguments of the schema, so the search lists them, as it lists the callers of a wrapper that
// returns the value. The forwarder is not a builder: what it gives back is the sender's answer, so a file that uses
// that answer is listed and not reported.

/**
 The import of the sender the cases use, as a specifier written in a file under `imaginary/src`.
 */
const SENDER_IMPORT =
  'import { gatherThroughCache, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 The sender the cases use, as the family declares it.
 */
const SENDER_FILE = {
  file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
  text: textOf([
    'export async function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
    '  return responseFormat;',
    '}',
  ],),
};

/**
 A file that calls the forwarder the case declares in `imaginary/src/forward-stage.ts` and uses the answer in a place
 that is reported when the value is a schema.
 */
const CALLER_FILE = {
  file: 'imaginary/src/forward-caller.ts',
  text: textOf([
    'import { ask, } from \'./forward-stage.ts\';',
    'export async function run() {',
    '  const reply = await ask({ schemaName: \'mine\', verdicts: [], asksArchive: false, },);',
    '  return Object.freeze(reply,);',
    '}',
  ],),
};

/**
 Cases of a function that hands a sender a builder call whose arguments come from its own parameters.
 */
export const FORWARDER_CASES: readonly ShapeCase[] = [
  {
    name: 'LISTS THE FILE THAT CALLS A FUNCTION WHOSE BUILDER CALL TAKES ITS OWN PARAMETER and hands the value to a '
      + 'sender, and does not report the file for the answer it uses',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/forward-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function ask(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      CALLER_FILE,
    ],
    found: [
      'imaginary/src/forward-caller.ts',
      'imaginary/src/forward-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'LISTS THE FILE THAT CALLS A FUNCTION WHOSE BUILDER CALL TAKES A NAME MADE FROM ITS PARAMETER, a constant '
      + 'that holds the schema and a constant that holds the arguments',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/forward-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function ask(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  const labels = args.verdicts;',
          '  const schema = contestResponseFormat({ ...args, verdicts: labels, },);',
          '  return await gatherThroughCache({ responseFormat: schema, },);',
          '}',
        ],),
      },
      CALLER_FILE,
    ],
    found: [
      'imaginary/src/forward-caller.ts',
      'imaginary/src/forward-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'DOES NOT LIST THE FILE THAT CALLS A FUNCTION WHOSE BUILDER CALL TAKES NOTHING OF ITS PARAMETERS, since the '
      + 'file that makes the schema is the one the search lists',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/forward-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function ask(question: string,) {',
          '  return await gatherThroughCache({',
          '    responseFormat: contestResponseFormat({ schemaName: \'fixed\', verdicts: [], asksArchive: false, },),',
          '    question,',
          '  },);',
          '}',
        ],),
      },
      CALLER_FILE,
    ],
    found: ['imaginary/src/forward-stage.ts',],
    builders: [],
    spots: [],
  },
  {
    name: 'DOES NOT LIST THE FILE THAT CALLS A FUNCTION WHOSE SENDER IS A NAME ITS OWN PARAMETER BINDS, since the call '
      + 'is no call of the sender',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/forward-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'export async function ask(',
          '  args: Parameters<typeof contestResponseFormat>[0],',
          '  gatherThroughCache: (input: object,) => Promise<unknown>,',
          ') {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      CALLER_FILE,
    ],
    found: ['imaginary/src/forward-stage.ts',],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/forward-stage.ts',
        line: 6,
        position: CALL_ARGUMENT,
      },),
    ],
  },
];

//endregion A function that hands a sender a builder call made from its own parameters
