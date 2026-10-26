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

//region A name that holds the sender and is assigned again
// A name that holds the sender (`let g = gatherThroughCache;`) is a way to call it only while the name keeps that
// value. A name that is assigned again anywhere in the file, by an assignment, a pattern that an assignment takes apart
// or a loop head, can hold any function when it is called, so the call of it is no call of the sender and the object it
// takes is reported like the argument of any call. A name that is never assigned again, whether `let`, `var` or
// `const`, and a name of the same kind that holds something else, stay as they were.

/**
 The path of the stage file of every case.
 */
const STAGE = 'imaginary/src/alias-stage.ts';

/**
 What to add to the number of lines of a case to get the line of the call that ends the stage file: the two imports
 before the lines, and the call itself.
 */
const LINE_OF_CALL_AFTER_CASE_LINES = 3;

/**
 The call of the alias with a schema made in place, which every case ends with.
 */
const CALL_OF_ALIAS = `export const r = g({ responseFormat: contestResponseFormat(${ARGUMENTS},), },);`;

/**
 A case whose stage file declares an alias of the sender and a call of it.

 @param name - name of the case

 @param lines - lines of the stage file between the imports and the call of the alias

 @param isReported - whether the call of the alias is reported at the builder's call

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = aliasCase({ name: 'x', lines: [], isReported: true, },);
 ```
 */
function aliasCase(
  {
    name,
    lines,
    isReported,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly isReported: boolean;
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
          ...lines,
          CALL_OF_ALIAS,
        ],),
      },
    ],
    found: [STAGE,],
    builders: [],
    spots: isReported
      ? [
        spotAt({
          file: STAGE,
          line: lines.length + LINE_OF_CALL_AFTER_CASE_LINES,
          position: CALL_ARGUMENT,
        },),
      ]
      : [],
  };
}

/**
 Cases of a name that holds the sender and is assigned again, and of the names that stay accepted.
 */
export const SENDER_ALIAS_CASES: readonly ShapeCase[] = [
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN BEFORE THE CALL, with let',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      'g = other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED IN ANOTHER FUNCTION, with var',
    lines: [
      'var g: any = gatherThroughCache;',
      'export function swap() { g = (o: object,) => o; }',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED BY A PATTERN, in an array pattern '
      + 'and in an object pattern',
    lines: [
      'let g: any = gatherThroughCache;',
      '[g,] = [(o: object,) => o,];',
      '({ g, } = { g: (o: object,) => o, });',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS THE HEAD OF A LOOP',
    lines: [
      'let g: any = gatherThroughCache;',
      'for (g of [(o: object,) => o,]) {',
      '  void g;',
      '}',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS THE HEAD OF A FOR-IN LOOP',
    lines: [
      'let g: any = gatherThroughCache;',
      'for (g in { key: 1, }) {',
      '  void g;',
      '}',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN UNDER A TYPE ASSERTION',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      '(g as any) = other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN UNDER A NON-NULL ASSERTION',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      '(g!) = other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN UNDER A SATISFIES MARK',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      '(g satisfies any) = other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN UNDER AN ANGLE BRACKET '
      + 'ASSERTION',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      '(<any>g) = other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN UNDER A LOGICAL ASSIGNMENT',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      'g ||= other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'REPORTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS ASSIGNED AGAIN UNDER A NULLISH ASSIGNMENT',
    lines: [
      'const other = (o: object,) => o;',
      'let g: any = gatherThroughCache;',
      'g ??= other;',
    ],
    isReported: true,
  },),
  aliasCase({
    name: 'ACCEPTS A SCHEMA HANDED TO A NAME THAT HOLDS THE SENDER AND IS NEVER ASSIGNED AGAIN, with let, and a name '
      + 'of another alias that is assigned again',
    lines: [
      'let g: any = gatherThroughCache;',
      'let h: any = (o: object,) => o;',
      'h = 1;',
    ],
    isReported: false,
  },),
];

//endregion A name that holds the sender and is assigned again
