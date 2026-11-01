import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A member written as a string, as a template with no substitution, and as a template with one
// A member of a module or of a function is read as a name when it is written as a name or as a string literal in
// brackets, and a template literal with no substitution is the same string, so it names the member the same way. The
// call of the sender through such a member is a send, and the `call` method of the sender written so is the `call`
// method. A template with a substitution names nothing the search can read, so the call through it is read as the
// call of any function, which takes the schema as an argument and is reported.

/**
 The character that opens a substitution in a template, which a line of an imaginary file spells out beside a brace.
 */
const DOLLAR = '$';

/**
 The line of the stage file that holds the builder's call in the case whose member is a template with a substitution.
 */
const SUBSTITUTION_CALL_LINE = 5;

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/key-stage.ts';

/**
 The import of the sender's module as a namespace.
 */
const NAMESPACE_IMPORT = 'import * as sender from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 A case whose stage file imports the sender, then sends a schema in the form the lines say.

 @param name - name of the case

 @param imports - import of the sender

 @param lines - lines of the stage file after the two imports

 @param reported - lines of the builder's call that the check reports, none for a case that reports nothing

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = keyCase({ name: 'x', imports: SENDER_IMPORT, lines: [], reported: [], },);
 ```
 */
function keyCase(
  {
    name,
    imports,
    lines,
    reported,
  }: {
    readonly name: string;
    readonly imports: string;
    readonly lines: readonly string[];
    readonly reported: readonly number[];
  },
): ShapeCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: STAGE,
        text: textOf([
          CONTEST_IMPORT,
          imports,
          ...lines,
        ],),
      },
    ],
    found: [STAGE,],
    builders: [],
    spots: reported.map(function spotOfLine(line,): BlindSpot {
      return spotAt({
        file: STAGE,
        line,
        position: CALL_ARGUMENT,
      },);
    },),
  };
}

/**
 Cases of a member written as a string, as a template with no substitution, and as a template with one.
 */
export const MEMBER_KEY_CASES: readonly ShapeCase[] = [
  keyCase({
    name: 'ACCEPTS A SEND THROUGH A MEMBER OF A NAMESPACE IMPORT WRITTEN AS A STRING IN BRACKETS',
    imports: NAMESPACE_IMPORT,
    lines: [
      `export const sent = sender['gatherThroughCache']({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`,
    ],
    reported: [],
  },),
  keyCase({
    name: 'ACCEPTS A SEND THROUGH A MEMBER OF A NAMESPACE IMPORT WRITTEN AS A TEMPLATE WITH NO SUBSTITUTION',
    imports: NAMESPACE_IMPORT,
    lines: [
      'export const sent = sender[`gatherThroughCache`](',
      `  { responseFormat: contestResponseFormat(${ARGUMENTS},), },`,
      ');',
    ],
    reported: [],
  },),
  keyCase({
    name: 'ACCEPTS A SEND THROUGH THE call METHOD OF THE SENDER WRITTEN AS A TEMPLATE WITH NO SUBSTITUTION',
    imports: SENDER_IMPORT,
    lines: [
      'export const sent = (gatherThroughCache as any)[`call`](',
      '  null,',
      `  { responseFormat: contestResponseFormat(${ARGUMENTS},), },`,
      ');',
    ],
    reported: [],
  },),
  keyCase({
    name: 'REPORTS A SEND THROUGH A MEMBER OF A NAMESPACE IMPORT WRITTEN AS A TEMPLATE WITH A SUBSTITUTION, which '
      + 'names no member the search can read',
    imports: NAMESPACE_IMPORT,
    lines: [
      'const part = \'Through\' as const;',
      `export const sent = (sender as any)[\`gather${DOLLAR}{part}Cache\`](`,
      `  { responseFormat: contestResponseFormat(${ARGUMENTS},), },`,
      ');',
    ],
    reported: [SUBSTITUTION_CALL_LINE,],
  },),
];

//endregion A member written as a string, as a template with no substitution, and as a template with one
