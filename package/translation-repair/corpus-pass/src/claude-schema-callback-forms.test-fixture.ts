import type { BlindSpot, } from './claude-schema-blind-spots.test-fixture.ts';
import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CALL_ARGUMENT,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A value handed to a callback
// The third total follows a builder's value through the array, the object and the constant it is put in, and through
// the members read off it and the methods called on it. A method that calls a function with the elements of the array
// it is called on (`forEach`, `map`, `filter`, `flatMap`, `find`, `some`, `every` and `reduce`) hands the value to the
// parameters of that function, so the walk goes on from the parameters the callback binds: the element, and the array
// itself as the last parameter; for `reduce` the accumulator, the element and the array. A callback that is a function
// written there, or a name of a function the file declares, is read. A callback that is any other expression (a
// parameter, a member, a computed member, a name the file imports or declares as a type only) is code the search
// cannot read, and the check reports it where the builder's call is (the cases of it are in
// `claude-schema-callback-unread-forms.test-fixture.ts`). A method that takes a callback and is not one of the eight
// ends the walk as any other method called on the value does.

/**
 The methods that call their callback with the elements of the array they are called on.
 */
const ELEMENT_METHODS: readonly string[] = [
  'every',
  'filter',
  'find',
  'flatMap',
  'forEach',
  'map',
  'some',
];

/**
 The lines a case writes a builder's call on, counted from one, found as the lines of the case with a call.

 @param lines - lines of the stage file after its import

 @returns Lines of the stage file

 @example
 ```ts
 const found = builderLinesIn({ lines: ['const a = 1;', 'const b = contestResponseFormat(),'], });
 // => [3]
 ```
 */
function builderLinesIn({ lines, }: { readonly lines: readonly string[]; },): readonly number[] {
  return lines.flatMap(function lineOfCall(
    line,
    index,
  ): readonly number[] {
    return line.includes('contestResponseFormat(',) ? [index + 2,] : [];
  },);
}

/**
 A file of the case, with the import of the builder on its first line.

 @param file - path of the file

 @param lines - lines after the import

 @returns File

 @example
 ```ts
 const stage = stageFile({ file: 'imaginary/src/callback-stage.ts', lines: [], },);
 ```
 */
export function stageFile(
  {
    file,
    lines,
  }: {
    readonly file: string;
    readonly lines: readonly string[];
  },
): SourceFile {
  return {
    file,
    text: textOf([
      CONTEST_IMPORT,
      ...lines,
    ],),
  };
}

/**
 The lines of a stage file that hands a one-element array of the builder's value to a method with a callback that keeps
 its first parameter.

 @param method - method called on the array

 @returns Lines after the import

 @example
 ```ts
 const lines = keepingLines({ method: 'forEach', },);
 ```
 */
function keepingLines({ method, }: { readonly method: string; },): readonly string[] {
  return [
    'const kept: unknown[] = [];',
    'export function mk() {',
    `  [contestResponseFormat(${ARGUMENTS},),].${method}((s,) => { kept.push(s,); },);`,
    '}',
  ];
}

/**
 A case with one stage file, whose every builder's call is reported where the value goes.

 @param name - name of the case

 @param lines - lines of the stage file after its import

 @param position - where the value goes

 @returns Case that lists the stage file

 @example
 ```ts
 const shapeCase = reportingCase({ name: 'x', lines: [], position: CALL_ARGUMENT, },);
 ```
 */
export function reportingCase(
  {
    name,
    lines,
    position,
  }: {
    readonly name: string;
    readonly lines: readonly string[];
    readonly position: string;
  },
): ShapeCase {
  /**
   Path of the stage file every one-file case writes.
   */
  const file = 'imaginary/src/callback-stage.ts';
  return {
    name,
    files: [stageFile({
      file,
      lines,
    },),],
    found: [file,],
    builders: [],
    spots: builderLinesIn({ lines, },)
      .map(function spotOfLine(line,): BlindSpot {
        return spotAt({
          file,
          line,
          position,
        },);
      },),
  };
}

/**
 Cases of a builder's value handed to a callback.
 */
export const CALLBACK_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE METHOD HANDS IT TO A CALLBACK THAT KEEPS IT, for every, '
      + 'filter, find, flatMap, forEach, map and some, since the callback\'s parameter is where the value goes on',
    files: ELEMENT_METHODS.map(function fileOf(method,): SourceFile {
      return stageFile({
        file: `imaginary/src/callback-${method}-stage.ts`,
        lines: keepingLines({ method, },),
      },);
    },),
    found: ELEMENT_METHODS.map(function pathOf(method,): string {
      return `imaginary/src/callback-${method}-stage.ts`;
    },),
    builders: [],
    spots: ELEMENT_METHODS.map(function spotOf(method,): BlindSpot {
      return spotAt({
        file: `imaginary/src/callback-${method}-stage.ts`,
        line: 4,
        position: CALL_ARGUMENT,
      },);
    },),
  },
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY THAT REDUCE HANDS TO A CALLBACK THAT KEEPS THE ELEMENT',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].reduce((total, s,) => { kept.push(s,); return total; }, 0,);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY THAT REDUCE HANDS TO A CALLBACK THAT KEEPS THE ACCUMULATOR, which '
      + 'is the first element when the call gives no start value',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].reduce((total, s,) => { kept.push(total,); return 0; },);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE CALLBACK KEEPS THE ARRAY IT IS CALLED WITH, as the last '
      + 'parameter, through a rest parameter, or the element through a pattern',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach((s, index, all,) => { kept.push(all,); },);`,
      '}',
      'export function rest() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach((...all) => { kept.push(all,); },);`,
      '}',
      'export function pattern() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach(({ kind, }) => { kept.push(kind,); },);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT READ OFF A MEMBER OF THE VALUE WHOSE METHOD HANDS IT TO A CALLBACK THAT KEEPS '
      + 'IT',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  (contestResponseFormat(${ARGUMENTS},) as any).items.forEach((s: unknown,) => kept.push(s,),);`,
      '}',
      'export function mapped() {',
      `  (contestResponseFormat(${ARGUMENTS},) as any).items.map((s: unknown,) => kept.push(s,),);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  {
    name: 'REPORTS A BUILDER\'S RESULT IN A CONSTANT ARRAY WHOSE METHOD HANDS IT TO A CALLBACK THAT KEEPS IT, in the '
      + 'file that declares the constant, in a file that imports the constant, and in a constant inside a function',
    files: [
      stageFile({
        file: 'imaginary/src/callback-list.ts',
        lines: [
          `export const LIST = [contestResponseFormat(${ARGUMENTS},),];`,
          'const kept: unknown[] = [];',
          'LIST.forEach((s,) => kept.push(s,),);',
        ],
      },),
      {
        file: 'imaginary/src/callback-reader.ts',
        text: textOf([
          'import { LIST, } from \'./callback-list.ts\';',
          'const kept: unknown[] = [];',
          'LIST.map((s,) => kept.push(s,),);',
        ],),
      },
      stageFile({
        file: 'imaginary/src/callback-local.ts',
        lines: [
          'const kept: unknown[] = [];',
          'export function mk() {',
          `  const list = [contestResponseFormat(${ARGUMENTS},),];`,
          '  list.forEach((s,) => kept.push(s,),);',
          '}',
        ],
      },),
    ],
    found: [
      'imaginary/src/callback-list.ts',
      'imaginary/src/callback-local.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/callback-list.ts',
        line: 2,
        position: `${CALL_ARGUMENT} in imaginary/src/callback-reader.ts, ${CALL_ARGUMENT}`,
      },),
      spotAt({
        file: 'imaginary/src/callback-local.ts',
        line: 4,
        position: CALL_ARGUMENT,
      },),
    ],
  },
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE METHODS ARE CHAINED, the filter by `Boolean` then the '
      + 'callback that keeps it',
    lines: [
      'const kept: unknown[] = [];',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].filter(Boolean,).forEach((v,) => kept.push(v,),);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT IN AN ARRAY WHOSE CALLBACK STORES IT IN A MEMBER OF THE GLOBAL OBJECT',
    lines: [
      'export function stored() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach((s,) => { (globalThis as any).kept = s; },);`,
      '}',
    ],
    position: 'AssignmentExpression.right',
  },),
  reportingCase({
    name: 'REPORTS A BUILDER\'S RESULT HANDED TO A CALLBACK THAT IS A NAME OF A FUNCTION THE FILE DECLARES AND THAT '
      + 'KEEPS ITS PARAMETER, as a function declaration and as a constant arrow',
    lines: [
      'const kept: unknown[] = [];',
      'function keep(s: unknown,) {',
      '  kept.push(s,);',
      '}',
      'const keepAgain = (s: unknown,) => { kept.push(s,); };',
      'export function mk() {',
      `  [contestResponseFormat(${ARGUMENTS},),].forEach(keep,);`,
      `  [contestResponseFormat(${ARGUMENTS},),].forEach(keepAgain,);`,
      '}',
    ],
    position: CALL_ARGUMENT,
  },),
];

//endregion A value handed to a callback
