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
import type { Family, } from './claude-schema-resolve.test-fixture.ts';
import {
  keyOf,
  standsForKnown,
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
// that holds one in an object a constant holds. A static method, a getter, a member with a computed name, a member of
// an object nested in that object and an anonymous default export declare no function.
//
// A wrapper is a function that returns the call of a builder, however the call is wrapped: awaited, in parentheses,
// with type arguments, under a type mark or after an optional chain, as the body of an arrow function or in a
// `return`. A function that returns the call of a wrapper is a wrapper, until no function is added. A function that
// returns more than the call (a promise chain, a fallback, a conditional, an object, an array) or that calls without
// returning the result is no wrapper.
//
// A use is a name, or a chain of members, that sits where a value does: a call, also with type arguments, optional or
// as a tagged template, a function handed over by reference, a spread. A key, a member name, a longer name, a name
// under `typeof` or in a type is no use.
//
// A use is followed through an import, an alias, a namespace import and a member taken out by destructuring, through a
// dynamic `import()` that is awaited or a `require` with a literal path, and through a module that exports the builder
// again by name, under another name, with `export *`, as a namespace or as the default, however the default is written.
// A name that a declaration inside a function or a block binds to a loaded module or to another name is read as the
// file's own. The first binding of a name in the text is the one kept, and a binding at the top of the file wins over
// one below it. A member read off a load in place, as in `(await import(path)).name(...)` and
// `require(path).name(...)`, is followed like one read off a name that holds the module. An `import()` that is not
// awaited yields a promise and is followed to no name. `.call`, `.apply` and `.bind` read off a builder are the
// builder. A member named `call` of an object the file declares is that member. Two files that export everything of
// each other, and two constants that are each other's value, end the search and name nothing. An `import()` or a
// `require` with a path that is no literal, or that joins two literals, is not followed.
//
// A name the file neither declares nor imports from a file of the sources is read as the builder of that name. So is a
// name imported from a specifier that names no file of the sources, and so is a parameter that shares the builder's
// name; the scan over-reads there. A function of the builder's name that the file declares, or imports from another
// file of the sources, is that function and not the builder.
//
// The scan over-reads where a class or a constant that is no function has the builder's name. It over-reads where a
// constant, a function or a `catch` parameter inside a function has it. It over-reads where a function has a second
// `return` that holds the call, and reads that function as a wrapper. It over-reads where a constant holds
// `items.map((x) => build(x))`, and reads the constant as a wrapper.
//
// The file that declares a builder and the producer list are not its users. A file is listed when it holds a format,
// names the type of one or uses a builder.
//
// The search reads exactly these positions, and the check `blindSpotsOf`, in
// `claude-schema-blind-spots.test-fixture.ts`, reports every other reference to a module that carries a builder or to a
// binding that comes from one. The check reads every non-test source of the family; the real sources hold no such
// reference. It makes three totals.
//
// The first total is the module-load boundary. A string that names a module that carries a builder (one that declares
// a builder, or exports one by its own names, by `export *`, as a namespace or as an object with builder members) is
// read as the source of an import or of an export from, as the one literal path of an `import()` or a `require`, or
// inside a type. Any other such string is reported (`module-path-outside-a-load`): a literal handed to
// `createRequire(...)`, `require.resolve`, `import.meta.resolve` or `new URL`, held by a constant, or named by an
// import-equals declaration. An `import()` or a `require` whose path is not one literal is reported
// (`computed-load-path`). A load of such a module with a literal path is followed when it is awaited or required and
// held by a constant or a pattern the search binds name by name, or read in place with a member; any other load is
// reported (`load-in-unfollowed-form`): an un-awaited one, a rest or a computed key or a nested pattern, a load that
// is stored, handed to a call, chained with `.then` or dropped.
//
// The second total is every chain of names that stands for a builder, or for a module or object that carries one. A
// builder is read as the callee of a call, of `new` or of a tagged template, after any of `.call`, `.apply` and
// `.bind`, and as the value of a constant the search binds to it, or an alias of one. Every other place that holds one
// is reported (`builder-outside-a-read-position`), with the kind of node and the field: a class field, an argument, a
// default, an array, a conditional, a logical operand, a sequence, a getter, an assignment, a member that is no call
// method (`.name`, `.length`, `.toString`) or a member computed from a name. A module or an object that carries a
// builder is read as the object of a member the search reads, or as the value of a constant or a pattern that the
// search binds; used as a whole (a computed member, an argument, a property, a spread, a rest, a default import that
// the module does not export) it is reported (`module-binding-outside-a-read-position`). A namespace, an enum, a
// parameter property, an export assignment or an import-equals declaration that holds either is walked and reported
// (`builder-in-typescript-value-construct`). A function that holds a format and that no declared function holds, as a
// static method, a getter or an anonymous default export does, is reported (`format-in-undeclared-function`); the
// check leaves that one out for the four files the unit test names as writing the format without building a stage's
// answer schema (`NOT_STAGE_SCHEMAS`: the model health probe of `bench` and `anthropic-tool.ts`, `chat-contract.ts`
// and `claude-body.ts` of `provider`).
//
// The third total is where the value of a builder's call goes, read by `claude-schema-blind-flow.test-fixture.ts`.
// The value is followed through objects, arrays, spreads, fallbacks, conditionals, sequences, awaits, members,
// methods, locals, patterns, defaults and names assigned, and it must end in one of three places. The first is a
// `return`, an arrow function's body or a `yield` of a function the search lists as a builder; in any other function
// the value leaves it, and the function is reported (`builder-result-returned-outside-a-wrapper`). The second is the
// property of the object argument of a sender, a function named by module and by parameter in
// `claude-schema-blind-senders.test-fixture.ts`. The third is a constant, a pattern, a default or a name assigned,
// whose every reference ends in one of the three in turn. A constant at the top level is the place a schema is
// declared, and the file that holds it is listed, so what reads it is not followed. Any other place that takes the
// value is reported (`builder-result-outside-a-destination`) with the kind of node and the field: an argument of a
// call that is no sender, an argument of `new`, a store into a member, a field or `this`, a pattern an assignment
// takes the value apart into, a thrown value, a loop that iterates it. A place that reads the value and gives back
// nothing the schema could travel in ends the walk without a report: the test of a conditional or a loop, a computed
// key, an operand of an operator, a substitution in a template, a statement that drops it, an element of a sequence
// that is not the last, and the target of an assignment.
//
// The check is a net, not a second search: it reports every place it does not understand, whether or not a producer
// hides there, and a place it reports is answered by reading the place in the search or by naming the place in the
// test.

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

 @returns Module

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

 @returns Modules, by path

 @throws {@link Error} when a file does not parse, since a file a search cannot read hides what it holds

 @example
 ```ts
 const family = familyOf({ sources: await familySources(), },);
 ```
 */
export function familyOf({ sources, }: { readonly sources: readonly SourceFile[]; },): Family {
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

 @param family - modules of the sources

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

 @param family - modules of the sources

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

 @param family - modules of the sources

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
          return standsForKnown({
            family,
            file: candidate.file,
            path,
            known: {
              keys: knownKeys,
              names: knownNames,
            },
          },);
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
 The builders already found, by the modules that were searched and the files named as no stage schema, so the suite
 searches the family's builders once however many questions it asks of them.
 */
const FOUND_BUILDERS = new Map<string, readonly SchemaBuilder[]>();

/**
 Numbers given to the modules, in the order the key of a search first meets them.
 */
const MODULE_NUMBERS = new Map<ModuleFacts, number>();

/**
 The key of a search: the modules searched, in order, and the files named as no stage schema.

 @param family - modules of the sources

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns A string that two searches share when they read the same modules, in the same order, with the same names

 @example
 ```ts
 const key = searchKeyOf({ family, notStageSchemas: new Set(), },);
 ```
 */
function searchKeyOf(
  {
    family,
    notStageSchemas,
  }: {
    readonly family: Family;
    readonly notStageSchemas: ReadonlySet<string>;
  },
): string {
  return [
    ...[...family.entries(),].map(function numberedModule([file, facts,],): string {
      if (!MODULE_NUMBERS.has(facts,)) {
        MODULE_NUMBERS.set(
          facts,
          MODULE_NUMBERS.size,
        );
      }
      return `${file}=${String(MODULE_NUMBERS.get(facts,),)}`;
    },),
    '--',
    ...[...notStageSchemas,].toSorted(),
  ].join('\n',);
}

/**
 The builders of a family that is read already.

 @param family - modules of the sources

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per builder, in the order of the sources, the wrappers after the functions they wrap; the same
 array for the same modules and names

 @example
 ```ts
 const builders = buildersIn({ family, notStageSchemas: new Set(), },);
 ```
 */
export function buildersIn(
  {
    family,
    notStageSchemas,
  }: {
    readonly family: Family;
    readonly notStageSchemas: ReadonlySet<string>;
  },
): readonly SchemaBuilder[] {
  /**
   What this search and an earlier one share when they read the same modules.
   */
  const key = searchKeyOf({
    family,
    notStageSchemas,
  },);
  /**
   The builders an earlier search found.
   */
  const found = FOUND_BUILDERS.get(key,);
  if (found !== undefined)
    return found;
  /**
   The builders this search finds.
   */
  const builders = searchBuildersIn({
    family,
    notStageSchemas,
  },);
  FOUND_BUILDERS.set(
    key,
    builders,
  );
  return builders;
}

/**
 The builders of a family that is read already, searched afresh.

 @param family - modules of the sources

 @param notStageSchemas - paths of files that write the format without building a stage's answer schema

 @returns One entry per builder, in the order of the sources, the wrappers after the functions they wrap

 @example
 ```ts
 const builders = searchBuildersIn({ family, notStageSchemas: new Set(), },);
 ```
 */
function searchBuildersIn(
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
