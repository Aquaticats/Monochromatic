import { memberFunctionsOf, } from './claude-schema-member.test-fixture.ts';
import {
  isIdentifierCharacter,
  leadingNameOf,
  NOT_FOUND,
  positionsWhere,
  startsAsFunctionValue,
} from './claude-schema-text.test-fixture.ts';

//region Code of a source file, read without a parser
// What `claude-schema-sources.test-fixture.ts` reads a file's text through: the lines of code left once comments and
// import and export lists are gone, the top-level declarations among them that are functions, and whether code names
// a function. It reads the shapes the family's source is written in, not the whole language.
//
// Functions read. A function is one declared at the top level (`function`, `async function`, `export default
// function`, `export default async function`), one a top-level constant holds (an arrow function with or without
// parentheses around its parameter or type parameters before them, a function expression, or a call whose first
// argument is one, as `memoize((a) => ...)` writes it), or a member of an object literal a top-level constant holds,
// one level in, that is a method or a property holding a function; that member is named `object.member`
// (`claude-schema-member.test-fixture.ts`).
//
// Uses read. A use of a function is a name that stands alone: a call, a call that names type arguments, the name handed
// over by reference, and the name behind a spread (`...name(`). A name that is part of a longer name, a property of
// some other object, standing against a quote (a string that holds only the name, or starts or ends with it), or
// after `typeof` is no use. A name inside a longer string (`'call build() here'`) and a name in a comment that follows
// code on its line are read as a use: the scan drops only the comments that start a line, so it over-reads there, which
// a case holds. The names a file knows a function by (an import alias, a namespace import, a default import) are in
// `claude-schema-import.test-fixture.ts`.
//
// Wrappers read. A wrapper returns the call at once: after `return`, after `return await`, or as the body of an arrow
// function, with the call in parentheses or on the next line. The reading stops there on purpose: a function that
// calls a builder and returns something else (a request it sends, a value it computes from the schema) is no
// builder, and reading every function that merely calls one would make the producer list demand every caller of every
// stage function. The three functions that ask the page consistency questions (`pass-consistency-ask.ts`,
// `pass-consistency-carry-ask.ts`, `pass-consistency-run-off.ts`) are such functions: they pass a builder's result to a
// request and return something else. A wrapper is found for a builder that is a function, a method of an object (called
// as `object.member`) or a default export (called by the name the wrapper's file gives the import), and each has a
// case. A wrapper of a builder written some other way (an arrow function, a function expression) goes through the same
// rule and has no case of its own.
//
// Not read. A function declared inside another function or in a class; a wrapper that returns the call through a
// local variable, a conditional expression, an object or array it builds, or a promise chain; an optional call
// (`maybe?.build?.()`); a function assigned to a second name (`const make = build;`) or taken out of an object
// (`const { build } = kit;`), whose second name is never followed; a function re-exported under another name
// (`export { build as make } from`); an anonymous default export, or a default export written as `export default
// build;`; a dynamic `import()` or `require`; a default import through a module whose file name is not the declaring
// file's (`import make from './index.ts'` for the default export of `default-wire.ts`, as a barrel is imported), whose
// local name is never followed; an object member nested deeper than one level, an accessor (a getter or a setter), a
// member with a computed name, and a member on the line that opens the object (`claude-schema-member.test-fixture.ts`).
// A search of the family's non-test source on 2026-10-23 (`rg` for the names of the three shared builders)
// finds them only as plain calls by name, as entries of import and export lists, and in documentation, so none of
// these shapes is in use for a shared builder today. A builder written one of these ways later would be missing from
// the producer list without a case seeing it.

/**
 A top-level declaration that is a function: one declared with `function`, a constant holding a function, or a member of
 an object a constant holds.
 */
export type FunctionDeclaration = {
  /**
   The function's name, `object.member` for the member of an object.
   */
  readonly name: string;

  /**
   The declaration's lines, joined.
   */
  readonly text: string;

  /**
   Whether the declaration is the default export of its file, which another file imports under a name of its own.
   */
  readonly exportedAsDefault: boolean;
};

/**
 What the declaration of a constant holds.
 */
type ConstantDeclaration = {
  /**
   The constant's name, empty when the lines declare none.
   */
  readonly name: string;

  /**
   The value the constant is assigned, empty when the lines declare none.
   */
  readonly value: string;
};

/**
 The characters that open or close a string, so a name standing against one of them is text and no use of the function.
 */
const QUOTE_CHARACTERS = '\'"`';

/**
 What opens a top-level function declaration in the family's source, whatever else the line says.
 */
const FUNCTION_OPENINGS: readonly string[] = [
  'function ',
  'export function ',
  'async function ',
  'export async function ',
  'export default function ',
  'export default async function ',
];

/**
 What opens a function declaration that is the default export of its file.
 */
const DEFAULT_OPENING = 'export default ';

/**
 What opens a constant at the top level, which holds a function when its value starts as one does.
 */
const CONSTANT_OPENINGS: readonly string[] = [
  'export const ',
  'const ',
];

/**
 What opens an import or an export list at the top level: the statements that name a function without using it.
 */
const LIST_STATEMENT_OPENINGS: readonly string[] = [
  'import ',
  'export {',
  'export type {',
  'export * ',
  'export type * ',
];

/**
 What names the module an import or an export list comes from.
 */
const MODULE_MARKERS: readonly string[] = [
  'from \'',
  'from "',
];

/**
 What stands before a call whose result a function returns straight away, once the text is read as one line with the
 space after each opening parenthesis taken out.
 */
const RETURN_OPENINGS: readonly string[] = [
  'return ',
  'return await ',
  'return (',
  '=> ',
  '=> await ',
  '=> (',
];

/**
 What follows the name of a function called: its arguments, or the type arguments before them.
 */
const CALL_FOLLOWERS: readonly string[] = [
  '(',
  '<',
];

/**
 The lines that a kind of span covers, each from an opening line through the first line at or after it that ends the
 span, or through the end of the file when none does.

 @param lines - file's lines

 @param openers - positions of the lines that open a span

 @param ends - whether a line ends a span, the opening line included

 @returns One list of positions per span, in order

 @example
 ```ts
 const ends = (line: string,): boolean => line.includes('*\/',);
 const spans = spansOf({ lines: ['/*', 'a', '*\/', 'b',], openers: [0,], ends, },);
 // => [[0, 1, 2]]
 ```
 */
function spansOf(
  {
    lines,
    openers,
    ends,
  }: {
    readonly lines: readonly string[];
    readonly openers: readonly number[];
    readonly ends: (line: string,) => boolean;
  },
): readonly (readonly number[])[] {
  /**
   Lines that end a span.
   */
  const closers = positionsWhere({
    lines,
    test: ends,
  },);
  return openers.map(function spanFrom(opener,): readonly number[] {
    /**
     Where the span ends: the first ending line at or after its opener, the last line when none follows.
     */
    const end = closers.find(function closesAfter(closer,): boolean {
      return closer >= opener;
    },) ?? (lines.length - 1);
    return Array.from(
      { length: (end - opener) + 1, },
      function lineOf(
        _,
        offset,
      ): number {
        return opener + offset;
      },
    );
  },);
}

/**
 Whether a line opens a block comment: its first characters are the opening mark.

 @param line - one line of a file

 @returns Whether a block comment starts the line

 @example
 ```ts
 const opens = opensBlock('  /* Builds the question.');
 // => true
 ```
 */
function opensBlock(line: string,): boolean {
  /**
   The line without its indentation.
   */
  const trimmed = line.trimStart();
  return trimmed.startsWith('/*',);
}

/**
 Whether a line holds the closing mark of a block comment, wherever it stands in the line.

 @param line - one line of a file

 @returns Whether the closing mark is in the line

 @example
 ```ts
 const closes = closesBlock(' end of the block *\/');
 // => true
 ```
 */
function closesBlock(line: string,): boolean {
  return line.includes('*/',);
}

/**
 Whether a line opens an import or an export list, at the start of the line.

 @param line - one line of a file

 @returns Whether the statement starts there

 @example
 ```ts
 const opens = opensListStatement('import { build, } from \'./build.ts\';');
 // => true
 ```
 */
function opensListStatement(line: string,): boolean {
  return LIST_STATEMENT_OPENINGS.some(function startsLine(opening,): boolean {
    return line.startsWith(opening,);
  },);
}

/**
 Whether a line ends an import or an export list: it names the module the list comes from, or it ends the statement.

 @param line - one line of a file

 @returns Whether the statement ends there

 @example
 ```ts
 const ends = endsListStatement('} from \'./build.ts\';');
 // => true
 ```
 */
function endsListStatement(line: string,): boolean {
  /**
   The line without trailing blanks.
   */
  const trimmed = line.trimEnd();
  /**
   Whether the line names the module the list comes from.
   */
  const namesModule = MODULE_MARKERS.some(function inLine(marker,): boolean {
    return line.includes(marker,);
  },);
  return namesModule || trimmed.endsWith(';',);
}

/**
 The positions of the lines inside some block comment, the opening and the closing line included.

 @param lines - file's lines

 @returns Positions, as a set

 @example
 ```ts
 const commented = commentedLinesOf({ lines: ['/*', 'a', '*\/', 'b',], },);
 // => Set { 0, 1, 2 }
 ```
 */
function commentedLinesOf({ lines, }: { readonly lines: readonly string[]; },): ReadonlySet<number> {
  /**
   One list of positions per block comment.
   */
  const spans = spansOf({
    lines,
    openers: positionsWhere({
      lines,
      test: opensBlock,
    },),
    ends: closesBlock,
  },);
  return new Set(spans.flat(),);
}

/**
 The import and export lists of a file, each as the lines from the one that opens it, at the start of the line and
 outside a block comment, through the first line that names its module or ends the statement.

 @param lines - file's lines

 @param commented - positions of the lines inside some block comment

 @returns One list of positions per statement, in order

 @example
 ```ts
 const spans = listSpansOf({ lines: ['import { a, } from \'./a.ts\';', 'b',], commented: new Set(), },);
 // => [[0]]
 ```
 */
function listSpansOf(
  {
    lines,
    commented,
  }: {
    readonly lines: readonly string[];
    readonly commented: ReadonlySet<number>;
  },
): readonly (readonly number[])[] {
  return spansOf({
    lines,
    openers: positionsWhere({
      lines,
      test: opensListStatement,
    },)
      .filter(function isCode(position,): boolean {
        return !commented.has(position,);
      },),
    ends: endsListStatement,
  },);
}

/**
 A file's code: the lines left once block comments, line comments that start a line, and import and export lists are
 taken out, so a name shown in documentation or only imported is no use of a function.

 A block comment covers the lines from the one that opens it, as its first characters, through the first line at or
 after that one holding the closing mark. An import or export list covers the lines from the one that opens it, at the
 start of the line and outside a block comment, through the first line that names its module or ends the statement.

 @param text - file's text

 @returns Lines of code, blank lines kept

 @example
 ```ts
 const lines = codeLinesOf({ text: '// contestResponseFormat(\nexport const A = 1;\n', },);
 // => ['export const A = 1;', '']
 ```
 */
export function codeLinesOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   The text's lines.
   */
  const lines = text.split('\n',);
  /**
   Lines inside some block comment, the opening and the closing line included.
   */
  const commented = commentedLinesOf({ lines, },);
  /**
   One list of positions per import or export list.
   */
  const listSpans = listSpansOf({
    lines,
    commented,
  },);
  /**
   Lines inside some import or export list.
   */
  const listed = new Set(listSpans.flat(),);
  return lines.filter(function isCode(
    line,
    index,
  ): boolean {
    /**
     The line without its indentation.
     */
    const trimmed = line.trimStart();
    /**
     Whether the line sits inside a block comment or an import or export list.
     */
    const covered = commented.has(index,) || listed.has(index,);
    return (!covered) && (!trimmed.startsWith('//',));
  },);
}

/**
 The import and export lists of a file, each as its lines joined, so the names a file brings in under another name can
 be read.

 @param text - file's text

 @returns One text per statement, in order

 @example
 ```ts
 const statements = listStatementsOf({ text: 'import { a as b, } from \'./a.ts\';\nconst c = 1;\n', },);
 // => ['import { a as b, } from \'./a.ts\';']
 ```
 */
export function listStatementsOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   The text's lines.
   */
  const lines = text.split('\n',);
  return listSpansOf({
    lines,
    commented: commentedLinesOf({ lines, },),
  },)
    .map(function statementOf(span,): string {
      return span.map(function lineAt(position,): string {
        return lines[position] ?? '';
      },)
        .join('\n',);
    },);
}

/**
 The name of the function a line opens, empty when the line opens none.

 @param line - one line of code

 @returns Name, without type parameters or parameters

 @example
 ```ts
 const name = functionNameOpenedBy({ line: 'export function contestResponseFormat(', },);
 // => 'contestResponseFormat'
 ```
 */
function functionNameOpenedBy({ line, }: { readonly line: string; },): string {
  /**
   The opening the line starts with, absent when it starts with none.
   */
  const opening = FUNCTION_OPENINGS.find(function startsLine(candidate,): boolean {
    return line.startsWith(candidate,);
  },);
  if (opening === undefined)
    return '';
  /**
   What follows the opening, up to the end of the line.
   */
  const afterOpening = line.slice(opening.length,);
  /**
   The name and what follows it up to the first parameter list.
   */
  const [beforeParameters = '',] = afterOpening.split('(',);
  /**
   The name alone, cut before type parameters.
   */
  const [name = '',] = beforeParameters.split('<',);
  return name;
}

/**
 The constant a declaration holds, with the value it is assigned.

 @param lines - declaration's lines

 @returns Name and value of the constant, both empty when the lines declare none

 @example
 ```ts
 const constant = constantOf({ lines: ['export const build = (a: number,) => ({', '  a,', '});',], },);
 // => { name: 'build', value: '(a: number,) => ({\n  a,\n});' }
 ```
 */
function constantOf({ lines, }: { readonly lines: readonly string[]; },): ConstantDeclaration {
  /**
   The first line of the declaration.
   */
  const [first = '',] = lines;
  /**
   The opening the first line starts with, absent when it starts with none.
   */
  const opening = CONSTANT_OPENINGS.find(function startsLine(candidate,): boolean {
    return first.startsWith(candidate,);
  },);
  if (opening === undefined)
    return {
      name: '',
      value: '',
    };
  /**
   The declaration after its opening.
   */
  const afterOpening = lines.join('\n',)
    .slice(opening.length,);
  /**
   Where the value starts: after the assignment, the type of the constant skipped.
   */
  const assignment = afterOpening.indexOf(' = ',);
  return (assignment === NOT_FOUND)
    ? {
      name: '',
      value: '',
    }
    : {
      name: leadingNameOf({ text: afterOpening, },),
      value: afterOpening.slice(assignment + ' = '.length,)
        .trimStart(),
    };
}

/**
 Whether a line starts a top-level statement: its first character is one a name holds.

 @param line - one line of code

 @returns Whether a statement starts there

 @example
 ```ts
 const starts = startsStatement('export const A = 1;');
 // => true
 ```
 */
function startsStatement(line: string,): boolean {
  return isIdentifierCharacter(line.at(0,) ?? '',);
}

/**
 The functions one top-level declaration holds: the function it declares, the function a constant holds, or the
 functions among the members of an object a constant holds.

 @param lines - declaration's lines

 @returns Zero or more functions

 @example
 ```ts
 const declared = functionsDeclaredBy({ lines: ['export function build() {', '  return 1;', '}',], },);
 // => [{ name: 'build', text: 'export function build() {\n  return 1;\n}', exportedAsDefault: false, }]
 ```
 */
function functionsDeclaredBy({ lines, }: { readonly lines: readonly string[]; },): readonly FunctionDeclaration[] {
  /**
   The first line of the declaration.
   */
  const [first = '',] = lines;
  /**
   The name and value of the constant the declaration holds, if it holds one.
   */
  const {
    name: constantName,
    value,
  } = constantOf({ lines, },);
  /**
   The name of the function the declaration declares or holds, empty when it holds none.
   */
  const name = functionNameOpenedBy({ line: first, },)
    || (startsAsFunctionValue({ value, },) ? constantName : '');
  if (name !== '')
    return [
      {
        name,
        text: lines.join('\n',),
        exportedAsDefault: first.startsWith(DEFAULT_OPENING,),
      },
    ];
  return value.startsWith('{',)
    ? memberFunctionsOf({
      owner: constantName,
      lines,
    },)
      .map(function declarationOf(member,): FunctionDeclaration {
        return {
          ...member,
          exportedAsDefault: false,
        };
      },)
    : [];
}

/**
 The top-level declarations of a file's code that are functions, each with the lines up to the next top-level statement.

 @param code - file's lines of code

 @returns One entry per function declaration, function constant or function member of an object constant, in order

 @example
 ```ts
 const declarations = functionDeclarationsOf({ code: ['export function build() {', '  return 1;', '}',], },);
 // => [{ name: 'build', text: 'export function build() {\n  return 1;\n}', exportedAsDefault: false, }]
 ```
 */
export function functionDeclarationsOf(
  { code, }: { readonly code: readonly string[]; },
): readonly FunctionDeclaration[] {
  /**
   Positions of the lines that start a top-level statement.
   */
  const starts = positionsWhere({
    lines: code,
    test: startsStatement,
  },);
  return starts.flatMap(function declarationFrom(
    start,
    index,
  ): readonly FunctionDeclaration[] {
    return functionsDeclaredBy({
      lines: code.slice(
        start,
        starts[index + 1] ?? code.length,
      ),
    },);
  },);
}

/**
 Whether a function's text returns the result of calling another function straight away: `return name(`,
 `return await name(` or `=> name(`, the same with the call in parentheses or on the next line, and the same with type
 arguments.

 @param text - function declaration's text

 @param name - called function's name

 @returns Whether the text returns a call of it

 @example
 ```ts
 const wraps = returnsCallOf({ text: 'function a() {\n  return build({});\n}', name: 'build', },);
 // => true
 ```
 */
export function returnsCallOf(
  {
    text,
    name,
  }: {
    readonly text: string;
    readonly name: string;
  },
): boolean {
  /**
   The text on one line, the space after each opening parenthesis taken out.
   */
  const flat = text.split('\n',)
    .map(function trimmedLine(line,): string {
      return line.trim();
    },)
    .join(' ',)
    .split('( ',)
    .join('(',);
  return RETURN_OPENINGS.some(function startsReturn(opening,): boolean {
    return CALL_FOLLOWERS.some(function followsName(follower,): boolean {
      return flat.includes(`${opening}${name}${follower}`,);
    },);
  },);
}

/**
 Whether code uses a function by its name: a call, a call that names type arguments, the function handed over by
 reference, or the name behind a spread. The name must stand alone, so a longer name that holds it, a property of the
 same name, a name against a quote and a `typeof` are no use. A name inside a longer string is read as a use.

 @param code - lines of code

 @param name - function's name, `object.member` for the member of an object

 @returns Whether some occurrence of the name is a use

 @example
 ```ts
 const uses = usesFunction({ code: ['const a = [1].map(build);',], name: 'build', },);
 // => true
 ```
 */
export function usesFunction(
  {
    code,
    name,
  }: {
    readonly code: readonly string[];
    readonly name: string;
  },
): boolean {
  return code.some(function lineUses(line,): boolean {
    /**
     The line cut at every occurrence of the name.
     */
    const pieces = line.split(name,);
    /**
     The pieces that have an occurrence of the name after them.
     */
    const beforeOccurrences = pieces.slice(
      0,
      -1,
    );
    return beforeOccurrences.some(function occurrenceIsUse(
      before,
      index,
    ): boolean {
      /**
       The character before the occurrence, empty at the start of the line.
       */
      const previous = before.at(-1,) ?? '';
      /**
       The character after the occurrence, empty at the end of the line.
       */
      const next = (pieces[index + 1] ?? '').at(0,) ?? '';
      /**
       Whether the dots before the occurrence are a spread's.
       */
      const spreads = before.endsWith('...',);
      /**
       Whether a dot before the occurrence reads a property of some object: it is no spread.
       */
      const readsProperty = (previous === '.') && (!spreads);
      /**
       Whether a character on one side makes the occurrence part of a longer name, a property or a string.
       */
      const insideSomethingElse = isIdentifierCharacter(previous,)
        || isIdentifierCharacter(next,)
        || readsProperty
        || ((previous !== '') && QUOTE_CHARACTERS.includes(previous,))
        || ((next !== '') && QUOTE_CHARACTERS.includes(next,));
      /**
       Whether the occurrence follows `typeof`, which names a type and runs nothing.
       */
      const namesType = before.trimEnd()
        .endsWith('typeof',);
      return (!insideSomethingElse) && (!namesType);
    },);
  },);
}

//endregion Code of a source file, read without a parser
