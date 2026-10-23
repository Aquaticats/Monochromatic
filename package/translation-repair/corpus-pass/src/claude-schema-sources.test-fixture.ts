import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

//region Sources that declare an answer schema
// What `claude-schemas.unit.test.ts` reads to keep the list of schema producers honest: the family's own source files,
// the files among them that write a `json_schema` response format, and the files that call a function which writes one.

/**
 One source file of the family.
 */
export type SourceFile = {
  /**
   Path under `package/translation-repair`, as the producer list names files.
   */
  readonly file: string;

  /**
   The file's text.
   */
  readonly text: string;
};

/**
 A function that builds an answer schema when called, with the file that declares it.
 */
export type SchemaBuilder = {
  /**
   The function's name.
   */
  readonly name: string;

  /**
   Path of the file that declares it.
   */
  readonly file: string;
};

/**
 Directory holding the family's packages.
 */
const FAMILY_ROOT = join(
  import.meta.dirname,
  '..',
  '..',
);

/**
 The list of schema producers, which calls the builders with sample labels to read the schemas they make and so is no
 stage that builds one.
 */
const PRODUCER_LIST_FILE = 'corpus-pass/src/claude-schema-producers.ts';

/**
 The characters a name can hold, so a name found inside a longer name is no call.
 */
const IDENTIFIER_CHARACTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_$';

/**
 What `indexOf` answers when the line it looks for is not there.
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
 Every non-test source file of the family, with its text.

 @returns One entry per file under a package's `src`, tests and fixtures left out

 @example
 ```ts
 const sources = await familySources();
 ```
 */
export async function familySources(): Promise<readonly SourceFile[]> {
  /**
   Files read so far.
   */
  const found: SourceFile[] = [];
  for await (
    const file of glob(
      '*/src/**/*.ts',
      { cwd: FAMILY_ROOT, },
    )
  ) {
    if (file.includes('node_modules',) || file.includes('.test',))
      continue;
    /**
     The file's text.
     */
    const text = await readFile(
      join(
        FAMILY_ROOT,
        file,
      ),
      'utf8',
    );
    found.push({
      file,
      text,
    },);
  }
  return found;
}

/**
 A file's code with the comments that sit on their own lines taken out, so a call shown in documentation is no call.

 @param text - the file's text

 @returns Lines outside block comments and line comments that start a line

 @example
 ```ts
 const lines = codeLinesOf({ text: '// contestResponseFormat(\nexport const A = 1;\n', },);
 // => ['export const A = 1;', '']
 ```
 */
function codeLinesOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Whether the scan is inside a block comment.
   */
  const block = { open: false, };
  return text
    .split('\n',)
    .filter(function isCode(line,): boolean {
      /**
       The line without its indentation.
       */
      const trimmed = line.trimStart();
      if (block.open) {
        block.open = !trimmed.includes('*/',);
        return false;
      }
      if (trimmed.startsWith('/*',)) {
        block.open = !trimmed.includes('*/',);
        return false;
      }
      return !trimmed.startsWith('//',);
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
 Whether code calls a function: its name, not part of a longer name, then an opening parenthesis.

 @param code - lines of code

 @param name - the function's name

 @returns Whether some occurrence of the name is a call

 @example
 ```ts
 const calls = callsFunction({ code: ['const a = build({});',], name: 'build', },);
 // => true
 ```
 */
function callsFunction(
  {
    code,
    name,
  }: {
    readonly code: readonly string[];
    readonly name: string;
  },
): boolean {
  return code.some(function lineCalls(line,): boolean {
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
    return beforeOccurrences.some(function occurrenceIsCall(
      before,
      index,
    ): boolean {
      /**
       The character before the occurrence, empty at the start of the line.
       */
      const previous = before.at(-1,) ?? '';
      /**
       What follows the occurrence, without its indentation.
       */
      const after = (pieces[index + 1] ?? '').trimStart();
      /**
       Whether the name stands alone and is followed by an opening parenthesis.
       */
      const standsAlone = (previous === '') || (!IDENTIFIER_CHARACTERS.includes(previous,));
      return standsAlone && after.startsWith('(',);
    },);
  },);
}

/**
 The functions that write a `json_schema` response format inside their body, so each call can make a different schema.

 A format in a constant is one schema however many files read it, so a constant is no builder.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per function, in the order of the sources

 @example
 ```ts
 const builders = buildersOf({ sources: await familySources(), notStageSchemas: new Set(), },);
 ```
 */
export function buildersOf(
  {
    sources,
    notStageSchemas,
  }: {
    readonly sources: readonly SourceFile[];
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly SchemaBuilder[] {
  return sources.flatMap(function buildersInFile(source,): readonly SchemaBuilder[] {
    if (notStageSchemas.has(source.file,))
      return [];
    /**
     The file's code.
     */
    const code = codeLinesOf({ text: source.text, },);
    return code.flatMap(function builderOpenedAt(
      line,
      index,
    ): readonly SchemaBuilder[] {
      /**
       The function the line opens, empty when it opens none.
       */
      const name = functionNameOpenedBy({ line, },);
      if (name === '')
        return [];
      /**
       Where the declaration ends: the next line that is a closing brace and nothing else.
       */
      const end = code.indexOf(
        '}',
        index,
      );
      /**
       Where the slice of the declaration stops: after the closing brace, or at the end of the file when none follows.
       */
      const stop = (end === NOT_FOUND) ? code.length : (end + 1);
      /**
       The declaration's lines.
       */
      const body = code.slice(
        index,
        stop,
      );
      return body.some(function writesFormat(bodyLine,): boolean {
        return bodyLine.includes('type: \'json_schema\'',);
      },)
        ? [
          {
            name,
            file: source.file,
          },
        ]
        : [];
    },);
  },);
}

/**
 The files that call a builder, the one that declares it and the producer list left out.

 @param sources - files searched

 @param builder - the builder whose calls are looked for

 @returns Paths, in the order of the sources

 @example
 ```ts
 const callers = callersOf({ sources, builder: { name: 'contestResponseFormat', file: 'a.ts', }, },);
 ```
 */
export function callersOf(
  {
    sources,
    builder,
  }: {
    readonly sources: readonly SourceFile[];
    readonly builder: SchemaBuilder;
  },
): readonly string[] {
  return sources
    .filter(function callsBuilder(source,): boolean {
      return (source.file !== builder.file)
        && (source.file !== PRODUCER_LIST_FILE)
        && callsFunction({
          code: codeLinesOf({ text: source.text, },),
          name: builder.name,
        },);
    },)
    .map(function pathOf(source,): string {
      return source.file;
    },);
}

/**
 The files that write a `json_schema` response format, or call a builder of one.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns Paths, in the order of the sources

 @example
 ```ts
 const declaring = declaringFilesOf({ sources: await familySources(), notStageSchemas: new Set(), },);
 ```
 */
export function declaringFilesOf(
  {
    sources,
    notStageSchemas,
  }: {
    readonly sources: readonly SourceFile[];
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly string[] {
  /**
   Builders found in the sources.
   */
  const builders = buildersOf({
    sources,
    notStageSchemas,
  },);
  /**
   Files that call some builder.
   */
  const calling = new Set(
    builders.flatMap(function callersOfBuilder(builder,): readonly string[] {
      return callersOf({
        sources,
        builder,
      },);
    },),
  );
  return sources
    .filter(function writesOrCalls(source,): boolean {
      /**
       The file's path and text.
       */
      const {
        file,
        text,
      } = source;
      /**
       Whether the file writes the format itself.
       */
      const writes = text.includes('type: \'json_schema\'',);
      return writes || calling.has(file,);
    },)
    .map(function pathOf(source,): string {
      return source.file;
    },);
}

//endregion Sources that declare an answer schema
