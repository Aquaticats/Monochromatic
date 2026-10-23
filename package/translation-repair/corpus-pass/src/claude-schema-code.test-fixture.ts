//region Code of a source file, read without a parser
// What `claude-schema-sources.test-fixture.ts` reads a file's text through: the lines of code left once comments and
// import and export lists are gone, the top-level declarations among them that are functions, and whether code names
// a function. It reads the shapes the family's source is written in, not the whole language, and says which it misses.

/**
 A top-level declaration that is a function: one declared with `function` or a constant holding an arrow function.
 */
export type FunctionDeclaration = {
  /**
   The function's name.
   */
  readonly name: string;

  /**
   The declaration's lines, joined.
   */
  readonly text: string;
};

/**
 The characters a name can hold, so a name found inside a longer name is no use of it.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 The characters that open or close a string, so a name found between them is text and no use of the function.
 */
const QUOTE_CHARACTERS = '\'"`';

/**
 How many characters of a text are read for a name, which no name in the family's source outgrows.
 */
const NAME_LIMIT = 120;

/**
 What `indexOf` answers when the text it looks for is not there.
 */
const NOT_FOUND = -1;

/**
 What opens a top-level function declaration in the family's source, whatever else the line says.
 */
const FUNCTION_OPENINGS: readonly string[] = [
  'function ',
  'export function ',
  'async function ',
  'export async function ',
];

/**
 What opens a constant at the top level, which holds an arrow function when its value is one.
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
 What stands before a call whose result a function returns straight away.
 */
const RETURN_OPENINGS: readonly string[] = [
  'return ',
  '=> ',
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
 Whether a character can be part of a name.

 @param character - one character, empty at the edge of a line

 @returns Whether a name can hold it

 @example
 ```ts
 const holds = isIdentifierCharacter('a');
 // => true
 ```
 */
function isIdentifierCharacter(character: string,): boolean {
  return (character !== '') && IDENTIFIER_CHARACTERS.includes(character,);
}

/**
 The name a text starts with, empty when it starts with a character no name holds.

 @param text - text read from its start

 @returns Leading characters that a name can hold

 @example
 ```ts
 const name = leadingNameOf({ text: 'build({});', },);
 // => 'build'
 ```
 */
function leadingNameOf({ text, }: { readonly text: string; },): string {
  /**
   Where the first character no name holds stands, among the first characters of the text.
   */
  const end = Array.from(text.slice(
    0,
    NAME_LIMIT,
  ),)
    .findIndex(function endsName(character,): boolean {
      return !isIdentifierCharacter(character,);
    },);
  return (end === NOT_FOUND) ? text.slice(
    0,
    NAME_LIMIT,
  ) : text.slice(
    0,
    end,
  );
}

/**
 The positions of the lines that satisfy a test.

 @param lines - lines searched

 @param test - what a line must satisfy

 @returns Positions, in order

 @example
 ```ts
 const positions = positionsWhere({ lines: ['a', 'b', 'a',], test: (line,) => line === 'a', },);
 // => [0, 2]
 ```
 */
function positionsWhere(
  {
    lines,
    test,
  }: {
    readonly lines: readonly string[];
    readonly test: (line: string,) => boolean;
  },
): readonly number[] {
  return lines.flatMap(function positionOf(
    line,
    index,
  ): readonly number[] {
    return test(line,) ? [index,] : [];
  },);
}

/**
 The lines that a kind of span covers, each from an opening line through the first line at or after it that ends the
 span, or through the end of the file when none does.

 @param lines - the file's lines

 @param openers - positions of the lines that open a span

 @param ends - whether a line ends a span, the opening line included

 @returns Positions of the covered lines

 @example
 ```ts
 const ends = (line: string,): boolean => line.includes('*\/',);
 const covered = coveredLinesOf({ lines: ['/*', 'a', '*\/', 'b',], openers: [0,], ends, },);
 // => Set { 0, 1, 2 }
 ```
 */
function coveredLinesOf(
  {
    lines,
    openers,
    ends,
  }: {
    readonly lines: readonly string[];
    readonly openers: readonly number[];
    readonly ends: (line: string,) => boolean;
  },
): ReadonlySet<number> {
  /**
   Lines that end a span.
   */
  const closers = positionsWhere({
    lines,
    test: ends,
  },);
  return new Set(openers.flatMap(function coveredFrom(opener,): readonly number[] {
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
  },),);
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
  const commented = coveredLinesOf({
    lines,
    openers: positionsWhere({
      lines,
      test: opensBlock,
    },),
    ends: closesBlock,
  },);
  /**
   Lines inside some import or export list.
   */
  const listed = coveredLinesOf({
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
 The name of the constant a declaration holds an arrow function in, empty when it holds anything else.

 The value must start as an arrow function does: a parameter list, `async` and a parameter list, type parameters, or a
 lone parameter and the arrow. A parenthesised value that is no arrow function but holds one is read as one.

 @param lines - the declaration's lines

 @returns Name of the constant

 @example
 ```ts
 const name = arrowNameOf({ lines: ['export const build = (a: number,) => ({', '  a,', '});',], },);
 // => 'build'
 ```
 */
function arrowNameOf({ lines, }: { readonly lines: readonly string[]; },): string {
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
    return '';
  /**
   The declaration after its opening.
   */
  const afterOpening = lines.join('\n',)
    .slice(opening.length,);
  /**
   The constant's name.
   */
  const name = leadingNameOf({ text: afterOpening, },);
  /**
   Where the value starts: after the assignment, the type of the constant skipped.
   */
  const assignment = afterOpening.indexOf(' = ',);
  if ((name === '') || (assignment === NOT_FOUND))
    return '';
  /**
   The value the constant is assigned.
   */
  const value = afterOpening.slice(assignment + ' = '.length,)
    .trimStart();
  /**
   A lone parameter at the start of the value, empty when the value starts with none.
   */
  const parameter = leadingNameOf({ text: value, },);
  /**
   What follows that parameter.
   */
  const afterParameter = value.slice(parameter.length,)
    .trimStart();
  /**
   Whether the value starts the way an arrow function does.
   */
  const startsAsArrow = value.startsWith('(',)
    || value.startsWith('async (',)
    || value.startsWith('<',)
    || ((parameter !== '') && afterParameter.startsWith('=>',));
  return (startsAsArrow && value.includes('=>',)) ? name : '';
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
 The top-level declarations of a file's code that are functions, each with the lines up to the next top-level statement.

 @param code - file's lines of code

 @returns One entry per function declaration or arrow-function constant, in order

 @example
 ```ts
 const declarations = functionDeclarationsOf({ code: ['export function build() {', '  return 1;', '}',], },);
 // => [{ name: 'build', text: 'export function build() {\n  return 1;\n}', }]
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
    /**
     The declaration's lines: from its start up to the next statement or the end of the file.
     */
    const lines = code.slice(
      start,
      starts[index + 1] ?? code.length,
    );
    /**
     The name of the function the declaration holds, empty when it holds none.
     */
    const name = functionNameOpenedBy({ line: lines[0] ?? '', },) || arrowNameOf({ lines, },);
    return (name === '')
      ? []
      : [
        {
          name,
          text: lines.join('\n',),
        },
      ];
  },);
}

/**
 Whether a function's text returns the result of calling another function straight away: `return name(`, `=> name(`, and
 the same with type arguments.

 @param text - the function declaration's text

 @param name - the called function's name

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
  return RETURN_OPENINGS.some(function startsReturn(opening,): boolean {
    return CALL_FOLLOWERS.some(function followsName(follower,): boolean {
      return text.includes(`${opening}${name}${follower}`,);
    },);
  },);
}

/**
 Whether code uses a function by its name: a call, a call that names type arguments, or the function handed over by
 reference. The name must stand alone, so a longer name that holds it, a property of the same name, a string and a
 `typeof` are no use.

 @param code - lines of code

 @param name - the function's name

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
       Whether a character on one side makes the occurrence part of a longer name, a property or a string.
       */
      const insideSomethingElse = isIdentifierCharacter(previous,)
        || isIdentifierCharacter(next,)
        || (previous === '.')
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
