import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CONTEST_IMPORT,
} from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A module that exports a value with `export =` and a file that brings it in with `import x = require(path)`
// `export = name` gives the module the value that the name stands for, and `import x = require(path)` of such a module
// binds that value, not the module as a whole. A function exported that way is the function its callers call, so the
// files that call it are listed. A module with no `export =` is bound as a whole, which
// `claude-schema-import-equals-forms.test-fixture.ts` states. A constant exported that way is reported where `export =`
// takes it, as any other store of a builder's result is.

/**
 Cases of a module that exports a value with `export =`.
 */
export const EXPORT_ASSIGNMENT_CASES: readonly ShapeCase[] = [
  {
    name: 'LISTS THE FILE THAT CALLS A FORWARDER A FILE EXPORTS WITH `export =` AND BRINGS IN WITH '
      + '`import x = require(path)`, since the name the caller binds stands for the forwarder',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/assignment-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
          'export = mk;',
        ],),
      },
      {
        file: 'imaginary/src/assignment-caller.ts',
        text: textOf([
          'import mk = require(\'./assignment-stage.ts\');',
          `export const SENT = mk(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/assignment-caller.ts',
      'imaginary/src/assignment-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'LISTS THE FILE THAT CALLS A FORWARDER A FILE EXPORTS WITH `export =` AND BRINGS IN AS ITS DEFAULT, since a '
      + 'module with an `export =` has no other value to give a default import',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/assignment-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
          'export = mk;',
        ],),
      },
      {
        file: 'imaginary/src/assignment-caller.ts',
        text: textOf([
          'import made from \'./assignment-stage.ts\';',
          `export const SENT = made(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/assignment-caller.ts',
      'imaginary/src/assignment-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'LISTS THE FILE THAT CALLS A FORWARDER A FILE EXPORTS WITH `export =` AND BRINGS IN AS A NAMESPACE, which a '
      + 'module with an `export =` gives as the value itself when it is imported without the interop of a default',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/assignment-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
          'export = mk;',
        ],),
      },
      {
        file: 'imaginary/src/assignment-caller.ts',
        text: textOf([
          'import * as made from \'./assignment-stage.ts\';',
          `export const SENT = made(${ARGUMENTS},);`,
        ],),
      },
    ],
    found: [
      'imaginary/src/assignment-caller.ts',
      'imaginary/src/assignment-stage.ts',
    ],
    builders: [],
    spots: [],
  },
];

//endregion A module that exports a value with `export =` and a file that brings it in with `import x = require(path)`
