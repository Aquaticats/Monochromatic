import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';
import { keyOf, } from './claude-schema-usage.test-fixture.ts';

//region The files of the cases of a forwarder written as a member of an object literal
// A function written as a property or a method of an object literal that hands a sender a schema made from its own
// parameters is a forwarder named `object.member`, and the files that call it by that name are listed. A file that
// reads the object or the member where it is no callee can call it where the listing does not look, so the object is
// a carrier as an object that holds a builder is: the member held as a value, the object handed on whole, a member
// read by a computed key and a member taken out by a pattern are reported where they sit. A member that nothing
// reads is silent, since no file can call it without being listed or reported, and the case that states it is a pin.
// This file holds what the cases share: the stage file with the member in the form a case names, and the reports.

/**
 The file that declares the object in every case.
 */
const STAGE = 'imaginary/src/object-stage.ts';

/**
 The file that reads the object.
 */
export const USER = 'imaginary/src/object-user.ts';

/**
 The file that calls what the first file holds.
 */
export const THIRD = 'imaginary/src/object-third.ts';

/**
 The line that brings the object in.
 */
export const IMPORT_OBJECT = 'import { o, } from \'./object-stage.ts\';';

/**
 The type of a list of the arguments of the contest builder, as the stage file writes it.
 */
const LIST = 'readonly Parameters<typeof contestResponseFormat>[0][]';

/**
 The send of one schema made from the name `a`.
 */
const SEND = 'gatherThroughCache({ responseFormat: contestResponseFormat(a,), },)';

/**
 The arguments of a call of the member, as a file writes them.
 */
export const CALL_ARGUMENTS = '[{ schemaName: \'object\', verdicts: [], asksArchive: false, },]';

/**
 The forms of the member that the stage file holds.
 */
export type ObjectMember = 'arrow' | 'direct' | 'each' | 'expression' | 'for' | 'method';

/**
 The lines of the stage file after its imports, by the form of the member.
 */
const MEMBERS: Readonly<Record<ObjectMember, readonly string[]>> = {
  arrow: [`export const o = { run: (list: ${LIST},) => list.map((a,) => ${SEND},), };`,],
  direct: [`export const o = { run(a: Parameters<typeof contestResponseFormat>[0],) { return ${SEND}; }, };`,],
  each: [`export const o = { run(list: ${LIST},) { list.forEach((a,) => { void ${SEND}; },); }, };`,],
  expression: [`export const o = { run: function (list: ${LIST},) { return list.map((a,) => ${SEND},); }, };`,],
  for: [`export const o = { run(list: ${LIST},) { for (const a of list) { void ${SEND}; } }, };`,],
  method: [`export const o = { run(list: ${LIST},) { return list.map((a,) => ${SEND},); }, };`,],
};

/**
 The report for the member `o.run` held where the search does not read it.

 @param file - file that holds it

 @param line - its line

 @returns Report

 @example
 ```ts
 const spot = heldAt({ file: USER, line: 2, },);
 ```
 */
export function heldAt(
  {
    file,
    line,
  }: {
    readonly file: string;
    readonly line: number;
  },
): BlindSpot {
  return {
    file,
    line,
    shape: 'forwarder-outside-a-read-position',
    position: 'ArrayExpression.elements',
    held: keyOf({
      file: STAGE,
      name: 'o.run',
    },),
  };
}

/**
 The report for the object read where the search does not look.

 @param file - file that reads it

 @param line - line of the place

 @param position - kind of the node that holds the object and the field

 @returns Report

 @example
 ```ts
 const spot = objectAt({ file: USER, line: 2, position: 'CallExpression.arguments', },);
 ```
 */
export function objectAt(
  {
    file,
    line,
    position,
  }: {
    readonly file: string;
    readonly line: number;
    readonly position: string;
  },
): BlindSpot {
  return {
    file,
    line,
    shape: 'module-binding-outside-a-read-position',
    position,
  };
}

/**
 A case whose stage file declares the object with one member, and whose other files read it as the lines say.

 @param name - name of the case

 @param member - form of the member

 @param users - files besides the stage file, each with its lines

 @param listed - files besides the stage file that the search lists

 @param spots - reports the check gives

 @returns Case

 @example
 ```ts
 const shapeCase = objectCase({ name: 'x', member: 'method', users: [], listed: [], spots: [], },);
 ```
 */
export function objectCase(
  {
    name,
    member,
    users,
    listed,
    spots,
  }: {
    readonly name: string;
    readonly member: ObjectMember;
    readonly users: readonly {
      readonly file: string;
      readonly lines: readonly string[];
    }[];
    readonly listed: readonly string[];
    readonly spots: readonly BlindSpot[];
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
          SENDER_IMPORT,
          ...MEMBERS[member],
        ],),
      },
      ...users.map(function sourceOf({
        file,
        lines,
      },) {
        return {
          file,
          text: textOf(lines,),
        };
      },),
    ],
    found: [
      STAGE,
      ...listed,
    ].toSorted(),
    builders: [],
    spots,
  };
}

//endregion The files of the cases of a forwarder written as a member of an object literal
