import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';

//region Forms in which a file reaches a builder of an answer schema
// What `claude-schemas.unit.test.ts` feeds the builder search to show that it follows a builder whatever the file does
// with it: each case is a few imaginary files that reach the contest builder (`contestResponseFormat`, which the case
// is given beside them) in one form, and the files the search must find. The forms in which a builder is written are in
// `claude-schema-builder-forms.test-fixture.ts`.

/**
 One form of reaching or writing a builder, with the imaginary files that show it and what the search must find in them.
 */
export type FormCase = {
  /**
   Name of the form, which starts the name of its case.
   */
  readonly name: string;

  /**
   Imaginary files, beside the file that declares the contest builder.
   */
  readonly files: readonly SourceFile[];

  /**
   Paths of the imaginary files the search lists as declaring or using an answer schema, in sorted order.
   */
  readonly found: readonly string[];

  /**
   Names of the imaginary builders the search lists, in sorted order.
   */
  readonly builders: readonly string[];
};

/**
 The text of an imaginary file.

 @param lines - lines of the file

 @returns Lines, each ended by a newline

 @example
 ```ts
 const text = textOf(['export const A = 1;',]);
 // => 'export const A = 1;\n'
 ```
 */
export function textOf(lines: readonly string[],): string {
  return `${lines.join('\n',)}\n`;
}

/**
 Lines indented further.

 @param lines - lines of an imaginary file

 @param spaces - how many spaces to put before each line

 @returns The lines, each with the spaces before it

 @example
 ```ts
 const deeper = indentedBy({ lines: ['a',], spaces: 2, },);
 // => ['  a']
 ```
 */
export function indentedBy(
  {
    lines,
    spaces,
  }: {
    readonly lines: readonly string[];
    readonly spaces: number;
  },
): readonly string[] {
  return lines.map(function indented(line,): string {
    return `${' '.repeat(spaces,)}${line}`;
  },);
}

/**
 A line that calls the contest builder by the name a file knows it by, and keeps the result in a constant.

 @param name - name by which the file calls the builder, `object.member` for a member of an object

 @returns One declaration of a constant, named after the callee

 @example
 ```ts
 const line = callOf('formatOf');
 // => 'export const FORMATOF_RESPONSE_FORMAT = formatOf({ schemaName: \'called\', verdicts: [], asksArchive: false, },);'
 ```
 */
export function callOf(name: string,): string {
  /**
   The callee as a constant's name.
   */
  const constant = name.replaceAll(
    '.',
    '_',
  )
    .toUpperCase();
  return `export const ${constant}_RESPONSE_FORMAT = ${name}({ schemaName: 'called', verdicts: [], `
    + 'asksArchive: false, },);';
}

/**
 The import of the contest builder, as a file under `imaginary/src` writes it, and the line that ends it.
 */
const CONTEST_MODULE = '} from \'../../lane-contest/src/contest-ballot-wire.ts\';';

/**
 Cases in which a file calls the contest builder in a form the search read as no call: a type argument, a reference, a
 spread of the result, a call through a namespace import, a call through an import alias, and a wrapper that reaches the
 builder through an import alias.
 */
export const CALL_FORM_CASES: readonly FormCase[] = [
  {
    name: 'FOLLOWS A CALL THAT NAMES A TYPE ARGUMENT, as in `contestResponseFormat<string>(...)`',
    files: [
      {
        file: 'imaginary/src/generic-gate-stage.ts',
        text: textOf([
          'export const GENERIC_GATE_RESPONSE_FORMAT = contestResponseFormat<string>({',
          '  schemaName: \'generic_gate\',',
          '  verdicts: [],',
          '  asksArchive: false,',
          '},);',
        ],),
      },
    ],
    found: ['imaginary/src/generic-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A BUILDER HANDED OVER BY REFERENCE, as the function a `map` calls',
    files: [
      {
        file: 'imaginary/src/reference-gate-stage.ts',
        text: textOf([
          'export const FORMATS = [',
          '  { schemaName: \'a\', verdicts: [], asksArchive: false, },',
          '].map(contestResponseFormat);',
        ],),
      },
    ],
    found: ['imaginary/src/reference-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A SPREAD OF A BUILDER\'S RESULT, which the property rule read as a member of some object',
    files: [
      {
        file: 'imaginary/src/spread-gate-stage.ts',
        text: textOf([
          'export const SPREAD_GATE_RESPONSE_FORMAT = {',
          '  ...contestResponseFormat({ schemaName: \'spread_gate\', verdicts: [], asksArchive: false, },),',
          '  strict: true,',
          '};',
        ],),
      },
    ],
    found: ['imaginary/src/spread-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A CALL THROUGH A NAMESPACE IMPORT, as in `contest.contestResponseFormat(...)`',
    files: [
      {
        file: 'imaginary/src/namespace-gate-stage.ts',
        text: textOf([
          'import * as contest from \'../../lane-contest/src/contest-ballot-wire.ts\';',
          '',
          'export const NAMESPACE_GATE_RESPONSE_FORMAT = contest.contestResponseFormat({',
          '  schemaName: \'namespace_gate\',',
          '  verdicts: [],',
          '  asksArchive: false,',
          '},);',
        ],),
      },
    ],
    found: ['imaginary/src/namespace-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A CALL THROUGH AN IMPORT ALIAS, as in `import { contestResponseFormat as formatOf }`',
    files: [
      {
        file: 'imaginary/src/alias-gate-stage.ts',
        text: textOf([
          'import {',
          '  contestResponseFormat as formatOf,',
          CONTEST_MODULE,
          '',
          'export const ALIAS_GATE_RESPONSE_FORMAT = formatOf({',
          '  schemaName: \'alias_gate\',',
          '  verdicts: [],',
          '  asksArchive: false,',
          '},);',
        ],),
      },
    ],
    found: ['imaginary/src/alias-gate-stage.ts',],
    builders: [],
  },
  {
    name: 'FOLLOWS A WRAPPER THAT RETURNS THE CALL OF AN IMPORT ALIAS, and the file that calls the wrapper',
    files: [
      {
        file: 'imaginary/src/aliased-wrapper-wire.ts',
        text: textOf([
          'import {',
          '  contestResponseFormat as formatOf,',
          CONTEST_MODULE,
          '',
          'export function aliasedWrapperResponseFormat(schemaName: string,) {',
          '  return formatOf({ schemaName, verdicts: [], asksArchive: false, },);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/aliased-wrapper-gate-stage.ts',
        text: textOf([
          'export const ALIASED_WRAPPER_GATE_RESPONSE_FORMAT = aliasedWrapperResponseFormat(\'aliased_gate\',);',
        ],),
      },
    ],
    found: [
      'imaginary/src/aliased-wrapper-gate-stage.ts',
      'imaginary/src/aliased-wrapper-wire.ts',
    ],
    builders: ['aliasedWrapperResponseFormat',],
  },
];

//endregion Forms in which a file reaches a builder of an answer schema
