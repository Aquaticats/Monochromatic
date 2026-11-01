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

//region Names and paths that collide with what the check keeps apart
// The check marks a member computed from anything but a string with a name no source writes, and tells a module by its
// path. Both can collide with something a file writes: a string key can spell the marker, and a path can end like
// another path. A member whose key spells the marker is read as a computed member, so a call through it is a call of an
// unknown function and the schema it carries is reported (the check fails closed). A module whose path ends the path of
// a module that declares a forwarder is another module, and the namespace of the first is no carrier.

/**
 Cases of a name that spells the marker of a computed member and of a path that ends like another path.
 */
export const KEY_COLLISION_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A SENDER EXPORTED UNDER THE NAME THAT SPELLS THE MARKER OF A COMPUTED MEMBER AND CALLED BY THAT '
      + 'NAME, since a key that spells the marker is a computed member and the call is a call of an unknown function',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/marker-sender.ts',
        text: textOf([
          SENDER_IMPORT,
          String.raw`export { gatherThroughCache as "\u0000computed" };`,
        ],),
      },
      {
        file: 'imaginary/src/marker-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          'import * as ns from \'./marker-sender.ts\';',
          'export async function ask() {',
          `  return await (ns as any)["\\u0000computed"]({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
          '}',
        ],),
      },
    ],
    found: ['imaginary/src/marker-stage.ts',],
    builders: [],
    spots: [
      {
        file: 'imaginary/src/marker-stage.ts',
        line: 4,
        shape: 'builder-result-outside-a-destination',
        position: 'CallExpression.arguments',
      },
    ],
  },
  {
    name: 'ACCEPTS THE NAMESPACE OF A PLAIN MODULE WHOSE PATH ENDS THE PATH OF A MODULE THAT DECLARES A FORWARDER, '
      + 'handed on whole',
    files: [
      SENDER_FILE,
      {
        file: 'x/imaginary/src/suffix-mod.ts',
        text: textOf([
          'import { contestResponseFormat, } from \'../../../lane-contest/src/contest-ballot-wire.ts\';',
          'import { gatherThroughCache, } from \'../../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';',
          'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/suffix-mod.ts',
        text: textOf(['export const plain = 1;',],),
      },
      {
        file: 'imaginary/src/suffix-user.ts',
        text: textOf([
          'import * as ns from \'./suffix-mod.ts\';',
          'export const kept = [ns,];',
        ],),
      },
    ],
    found: [],
    builders: [],
    spots: [],
  },
];

//endregion Names and paths that collide with what the check keeps apart
