import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  codeLinesOf,
  type FunctionDeclaration,
  functionDeclarationsOf,
  returnsCallOf,
  usesFunction,
} from './claude-schema-code.test-fixture.ts';

//region Sources that declare an answer schema
// What `claude-schemas.unit.test.ts` reads to keep the list of schema producers honest: the family's own source files,
// the files among them that write a `json_schema` response format, and the files that call a function which writes one.
//
// A BUILDER IS A FUNCTION THAT WRITES THE FORMAT IN ITS BODY, or returns the result of one that does (a wrapper), so
// each call can make a different schema. It is declared at the top level, with `function` or as a constant holding an
// arrow function, and a file uses it by calling it, calling it with type arguments, or handing it over by reference
// (`claude-schema-code.test-fixture.ts` says which shapes are read). A format held in a constant is one schema however
// many files read it, and the producer list holds it once. That is why `blockPairingProtocol`
// (`slice-reading/src/block-pairing-protocol.ts`) is left out of the builders: it returns a clone of the constant
// `PAIRING_RESPONSE_FORMAT`, so every caller sends the same schema, the format text is not in its body, and its one
// caller (`pair-blocks-stage.ts`) is covered by the producer list's entry for the constant.

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
 What an answer schema's format writes, which makes a function that holds it a builder.
 */
const FORMAT_TEXT = 'type: \'json_schema\'';

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
   Every file under a package's `src`, tests and fixtures not yet left out.
   */
  const files = await Array.fromAsync(glob(
    '*/src/**/*.ts',
    { cwd: FAMILY_ROOT, },
  ),);
  return await Promise.all(
    files
      .filter(function isSource(file,): boolean {
        return !(file.includes('node_modules',) || file.includes('.test',));
      },)
      .map(async function sourceOf(file,): Promise<SourceFile> {
        return {
          file,
          text: await readFile(
            join(
              FAMILY_ROOT,
              file,
            ),
            'utf8',
          ),
        };
      },),
  );
}

/**
 A source file read once: its code, and the functions it declares.
 */
type ParsedSource = SourceFile & {
  /**
   Lines of code, once comments and import and export lists are out.
   */
  readonly code: readonly string[];

  /**
   The top-level functions the file declares.
   */
  readonly functions: readonly FunctionDeclaration[];
};

/**
 Reads each source file once.

 @param sources - files read

 @returns One entry per file, in order

 @example
 ```ts
 const parsed = parseSources({ sources: await familySources(), },);
 ```
 */
function parseSources({ sources, }: { readonly sources: readonly SourceFile[]; },): readonly ParsedSource[] {
  return sources.map(function parsedOf(source,): ParsedSource {
    /**
     The file's code.
     */
    const code = codeLinesOf({ text: source.text, },);
    return {
      ...source,
      code,
      functions: functionDeclarationsOf({ code, },),
    };
  },);
}

/**
 The functions that write a `json_schema` response format inside their body, and the wrappers of them that return a
 builder's result, so each call can make a different schema.

 @param parsed - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per function, in the order of the sources, the wrappers after the functions they wrap

 @example
 ```ts
 const builders = buildersIn({ parsed, notStageSchemas: new Set(), },);
 ```
 */
function buildersIn(
  {
    parsed,
    notStageSchemas,
  }: {
    readonly parsed: readonly ParsedSource[];
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly SchemaBuilder[] {
  /**
   Every function the searched files declare, with its file.
   */
  const declared = parsed
    .filter(function isSearched(source,): boolean {
      return !notStageSchemas.has(source.file,);
    },)
    .flatMap(function functionsOf(source,): readonly (FunctionDeclaration & { readonly file: string; })[] {
      return source.functions
        .map(function withFile(declaration,) {
          return {
            ...declaration,
            file: source.file,
          };
        },);
    },);
  return withWrappers({
    declared,
    known: declared
      .filter(function writesFormat({ text, },): boolean {
        return text.includes(FORMAT_TEXT,);
      },)
      .map(function builderOf(
        {
          name,
          file,
        },
      ): SchemaBuilder {
        return {
          name,
          file,
        };
      },),
  },);
}

/**
 The builders and every function that returns one of their results, and so on until no function is added.

 @param declared - every function the searched files declare

 @param known - builders found so far

 @returns Known builders followed by the wrappers found, in the order of the declarations

 @example
 ```ts
 const builders = withWrappers({ declared, known: [{ name: 'contestResponseFormat', file: 'a.ts', },], },);
 ```
 */
function withWrappers(
  {
    declared,
    known,
  }: {
    readonly declared: readonly (FunctionDeclaration & { readonly file: string; })[];
    readonly known: readonly SchemaBuilder[];
  },
): readonly SchemaBuilder[] {
  /**
   Functions not yet known that return the result of a known builder.
   */
  const wrappers = declared
    .filter(function isNewWrapper(declaration,): boolean {
      /**
       Whether the function is a builder already.
       */
      const isKnown = known.some(function sameFunction(builder,): boolean {
        return (builder.name === declaration.name) && (builder.file === declaration.file);
      },);
      /**
       Whether the function returns the result of a known builder.
       */
      const isWrapper = known.some(function isWrapped(builder,): boolean {
        return returnsCallOf({
          text: declaration.text,
          name: builder.name,
        },);
      },);
      return (!isKnown) && isWrapper;
    },)
    .map(function builderOf(
      {
        name,
        file,
      },
    ): SchemaBuilder {
      return {
        name,
        file,
      };
    },);
  return (wrappers.length === 0)
    ? known
    : withWrappers({
      declared,
      known: [
        ...known,
        ...wrappers,
      ],
    },);
}

/**
 The files that use a builder, the one that declares it and the producer list left out.

 @param parsed - files searched

 @param builder - builder whose uses are looked for

 @returns Paths, in the order of the sources

 @example
 ```ts
 const users = usersIn({ parsed, builder: { name: 'contestResponseFormat', file: 'a.ts', }, },);
 ```
 */
function usersIn(
  {
    parsed,
    builder,
  }: {
    readonly parsed: readonly ParsedSource[];
    readonly builder: SchemaBuilder;
  },
): readonly string[] {
  return parsed
    .filter(function usesBuilder(source,): boolean {
      return (source.file !== builder.file)
        && (source.file !== PRODUCER_LIST_FILE)
        && usesFunction({
          code: source.code,
          name: builder.name,
        },);
    },)
    .map(function pathOf(source,): string {
      return source.file;
    },);
}

/**
 The functions that build an answer schema when called: those that write a `json_schema` response format in their
 body, and those that return the result of such a function.

 A format in a constant is one schema however many files read it, so a constant is no builder. A function is read as a
 builder when it is declared at the top level of a file, with `function` or as a constant holding an arrow function.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per function, in the order of the sources, the wrappers after the functions they wrap

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
  return buildersIn({
    parsed: parseSources({ sources, },),
    notStageSchemas,
  },);
}

/**
 The files that use a builder, the one that declares it and the producer list left out. A use is a call, a call that
 names type arguments, or the builder handed over by reference.

 @param sources - files searched

 @param builder - builder whose uses are looked for

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
  return usersIn({
    parsed: parseSources({ sources, },),
    builder,
  },);
}

/**
 The files that write a `json_schema` response format, or use a builder of one.

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
   Each file read once.
   */
  const parsed = parseSources({ sources, },);
  /**
   Files that use some builder.
   */
  const using = new Set(
    buildersIn({
      parsed,
      notStageSchemas,
    },)
      .flatMap(function usersOfBuilder(builder,): readonly string[] {
        return usersIn({
          parsed,
          builder,
        },);
      },),
  );
  return parsed
    .filter(function writesOrUses(
      {
        file,
        text,
      },
    ): boolean {
      return text.includes(FORMAT_TEXT,) || using.has(file,);
    },)
    .map(function pathOf(source,): string {
      return source.file;
    },);
}

//endregion Sources that declare an answer schema
