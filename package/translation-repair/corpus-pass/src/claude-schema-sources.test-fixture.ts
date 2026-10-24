import {
  glob,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { parseSource, } from '@monochromatic-dev/translation-repair-source-scan/ts/source-scan.test-fixture.ts';

import type {
  DeclaredFunction,
  ModuleFacts,
} from './claude-schema-facts.test-fixture.ts';
import { moduleFactsOf, } from './claude-schema-module.test-fixture.ts';
import {
  type Family,
  resolvePath,
} from './claude-schema-resolve.test-fixture.ts';
import {
  keyOf,
  NOTHING_FOLLOWED,
  usagesIn,
  usersIn,
} from './claude-schema-usage.test-fixture.ts';
import {
  holdsFormatValue,
  returnedCalleesOf,
} from './claude-schema-tree.test-fixture.ts';

//region Sources that declare an answer schema
// What `claude-schemas.unit.test.ts` reads to keep the list of schema producers honest: the family's own source files,
// the files among them that write a `json_schema` response format, and the files that use a function which writes one.
//
// Every file is parsed into a syntax tree, so a string, a template or a comment is never read as code, and a type is
// never read as a value.
//
// A builder is a function whose body holds an object literal with the property `type: 'json_schema'`. The key may be
// plain, in quotes or in brackets, and the value may be in either kind of quote or in a template with no substitution.
// The text of a format in a comment, in a string, under another key or in an array makes no format. A file that names
// the type of a format is listed though it holds no format. A constant that holds a format is no builder, but the file
// that holds it is listed. A function written inside a builder is no builder of its own.
//
// A function is declared by `function` or `async function`, exported or not, by a constant that holds an arrow
// function, a function expression or a call that takes one first, and, as `object.member`, by a method or a property
// that holds one in an object a constant holds. A static method, a getter, a member with a computed name, a member of an
// object nested in that object and an anonymous default export declare no function.
//
// A wrapper is a function that returns the call of a builder, however the call is wrapped: awaited, in parentheses, with
// type arguments, under a type mark or after an optional chain, as the body of an arrow function or in a `return`. A
// function that returns the call of a wrapper is a wrapper, until no function is added. A function that returns more
// than the call (a promise chain, a fallback, a conditional, an object, an array) or that calls without returning the
// result is no wrapper.
//
// A use is a name, or a chain of members, that sits where a value does: a call, also with type arguments, optional or
// as a tagged template, a function handed over by reference, a spread. A key, a member name, a longer name, a name under
// `typeof` or in a type is no use.
//
// A use is followed through an import, an alias, a namespace import and a member taken out by destructuring, through a
// dynamic `import()` or a `require` with a literal path, and through a module that exports the builder again by name,
// under another name, with `export *`, as a namespace or as the default, however the default is written. An `import()`
// or a `require` with a path that is no literal is not followed.
//
// A name the file neither declares nor imports from a file of the sources is read as the builder of that name. So is a
// name imported from a specifier that names no file of the sources, and so is a parameter that shares the builder's
// name; the scan over-reads there. A function of the builder's name that the file declares, or imports from another file
// of the sources, is that function and not the builder.
//
// The file that declares a builder and the producer list are not its users. A file is listed when it holds a format,
// names the type of one or uses a builder.

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
   The function's name, `object.member` for the member of an object.
   */
  readonly name: string;

  /**
   Path of the file that declares it.
   */
  readonly file: string;

  /**
   Whether it is the default export of that file.
   */
  readonly exportedAsDefault: boolean;
};

/**
 A function a searched file declares, with the file that declares it.
 */
type Candidate = DeclaredFunction & {
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
 The modules already read, by path and text, so the same file read for another question is not parsed again.
 */
const READ_MODULES = new Map<string, ModuleFacts>();

/**
 Reads one source file, or takes the module an earlier read of the same text made.

 @param file - path of the file

 @param text - text of the file

 @returns The module

 @throws {@link Error} when the file does not parse, since a file a search cannot read hides what it holds

 @example
 ```ts
 const facts = moduleOf({ file: 'a/src/a.ts', text: 'export const a = 1;', },);
 ```
 */
function moduleOf({
  file,
  text,
}: {
  readonly file: string;
  readonly text: string
},): ModuleFacts {
  /**
   The path and the text as one key.
   */
  const key = `${file}\n${text}`;
  /**
   The module an earlier read made.
   */
  const read = READ_MODULES.get(key,);
  if (read !== undefined)
    return read;
  /**
   The module this read makes.
   */
  const facts = moduleFactsOf({
    file,
    program: parseSource({
      file: {
        path: file,
        text,
        isTest: false,
      },
    },)
      .program,
  },);
  READ_MODULES.set(
    key,
    facts,
  );
  return facts;
}

/**
 Reads each source file once.

 @param sources - files read

 @returns The modules, by path

 @throws {@link Error} when a file does not parse, since a file a search cannot read hides what it holds

 @example
 ```ts
 const family = familyOf({ sources: await familySources(), },);
 ```
 */
function familyOf({ sources, }: { readonly sources: readonly SourceFile[]; },): Family {
  return new Map(sources.map(function entryOf(source,): readonly [
    string,
    ModuleFacts,
  ] {
    return [
      source.file,
      moduleOf({
        file: source.file,
        text: source.text,
      },),
    ];
  },),);
}

/**
 A declared function as a builder.

 @param family - the modules of the sources

 @param candidate - function and the file that declares it

 @returns Builder with the three fields and no other

 @example
 ```ts
 const builder = builderOf({ family, candidate, },);
 ```
 */
function builderOf({
  family,
  candidate,
}: {
  readonly family: Family;
  readonly candidate: Candidate
},): SchemaBuilder {
  /**
   What the module that declares the function exports as its default.
   */
  const defaultExport = family.get(candidate.file,)
    ?.exports
    .get('default',);
  return {
    name: candidate.name,
    file: candidate.file,
    exportedAsDefault: (defaultExport?.kind === 'local') && (defaultExport.name === candidate.name),
  };
}

/**
 Every function the searched files declare.

 @param family - the modules of the sources

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per function, in the order of the sources

 @example
 ```ts
 const candidates = candidatesIn({ family, notStageSchemas: new Set(), },);
 ```
 */
function candidatesIn(
  {
    family,
    notStageSchemas,
  }: {
    readonly family: Family;
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly Candidate[] {
  return [...family.values(),]
    .filter(function isSearched({ file, },): boolean {
      return !notStageSchemas.has(file,);
    },)
    .flatMap(function candidatesOf({
      file,
      declared,
    },): readonly Candidate[] {
      return declared.map(function withFile(declaration,): Candidate {
        return {
          ...declaration,
          file,
        };
      },);
    },);
}

/**
 The builders and every function that returns the call of one of them, and so on until no function is added.

 @param family - the modules of the sources

 @param candidates - every function the searched files declare

 @param known - builders found so far

 @returns Known builders followed by the wrappers found, in the order of the declarations

 @example
 ```ts
 const builders = withWrappers({ family, candidates, known: [], },);
 ```
 */
function withWrappers(
  {
    family,
    candidates,
    known,
  }: {
    readonly family: Family;
    readonly candidates: readonly Candidate[];
    readonly known: readonly Candidate[];
  },
): readonly Candidate[] {
  /**
   Keys of the known builders and wrappers.
   */
  const knownKeys = new Set(known.map(function keyOfKnown(candidate,): string {
    return keyOf(candidate,);
  },),);
  /**
   Names of the known builders and wrappers.
   */
  const knownNames = new Set(known.map(function nameOf({ name, },): string {
    return name;
  },),);
  /**
   Functions not yet known that return the call of a known builder, however the call names it.
   */
  const wrappers = candidates.filter(function isNewWrapper(candidate,): boolean {
    return (!knownKeys.has(keyOf(candidate,),))
      && returnedCalleesOf({ fn: candidate.fn, },)
        .some(function callsKnown(path,): boolean {
          /**
           What the callee stands for in the file that declares the candidate.
           */
          const callee = resolvePath({
            family,
            file: candidate.file,
            path,
            seen: NOTHING_FOLLOWED,
          },);
          return (callee.kind === 'function') ? knownKeys.has(keyOf(callee,),) : (callee.kind === 'free')
            && knownNames.has(callee.name,);
        },);
  },);
  return (wrappers.length === 0)
    ? known
    : withWrappers({
      family,
      candidates,
      known: [
        ...known,
        ...wrappers,
      ],
    },);
}

/**
 The functions that build an answer schema when called: those that write a `json_schema` response format inside their
 body, and those that return the result of such a function.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per function, in the order of the sources, the wrappers after the functions they wrap

 @throws {@link Error} when a file does not parse

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
  /**
   The modules of the sources.
   */
  const family = familyOf({ sources, },);
  return buildersIn({
    family,
    notStageSchemas,
  },);
}

/**
 The builders of a family that is read already.

 @param family - the modules of the sources

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per builder, in the order of the sources, the wrappers after the functions they wrap

 @example
 ```ts
 const builders = buildersIn({ family, notStageSchemas: new Set(), },);
 ```
 */
function buildersIn(
  {
    family,
    notStageSchemas,
  }: {
    readonly family: Family;
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly SchemaBuilder[] {
  /**
   Every function the searched files declare.
   */
  const candidates = candidatesIn({
    family,
    notStageSchemas,
  },);
  return withWrappers({
    family,
    candidates,
    known: candidates.filter(function writesFormat({ holder, },): boolean {
      return holdsFormatValue({ root: holder, },);
    },),
  },)
    .map(function builder(candidate,): SchemaBuilder {
      return builderOf({
        family,
        candidate,
      },);
    },);
}

/**
 The files that use a builder, the one that declares it and the producer list left out.

 @param sources - files searched

 @param builder - builder whose uses are looked for

 @returns Paths, in the order of the sources

 @throws {@link Error} when a file does not parse

 @example
 ```ts
 const callers = callersOf({ sources, builder: { name: 'build', file: 'a.ts', exportedAsDefault: false, }, },);
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
    usages: usagesIn({ family: familyOf({ sources, },), },),
    builder,
  },);
}

/**
 The files that write a `json_schema` response format, or use a builder of one.

 @param sources - files searched

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns Paths, in the order of the sources

 @throws {@link Error} when a file does not parse

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
  const family = familyOf({ sources, },);
  /**
   What the references of each file stand for.
   */
  const usages = usagesIn({ family, },);
  /**
   Files that use some builder.
   */
  const using = new Set(
    buildersIn({
      family,
      notStageSchemas,
    },)
      .flatMap(function usersOfBuilder(builder,): readonly string[] {
        return usersIn({
          usages,
          builder,
        },);
      },),
  );
  return [...family.values(),]
    .filter(function writesOrUses({
      file,
      holdsFormat,
    },): boolean {
      return holdsFormat || using.has(file,);
    },)
    .map(function pathOf({ file, },): string {
      return file;
    },);
}

//endregion Sources that declare an answer schema
