import type { ModuleFacts, } from './claude-schema-facts.test-fixture.ts';
import {
  type Family,
  type Resolved,
  resolvePath,
} from './claude-schema-resolve.test-fixture.ts';

//region What the files of the sources refer to
// What `claude-schema-sources.test-fixture.ts` asks to tell which files use a function: for each file, the functions
// its references stand for, and, for a function, the files other than its own and the producer list that refer to it.

/**
 The functions a file refers to, and the names it refers to without declaring or importing them.
 */
export type Usage = {
  /**
   Path of the file.
   */
  readonly file: string;

  /**
   Keys of the functions the file refers to, by the module that declares them.
   */
  readonly keys: ReadonlySet<string>;

  /**
   Names the file refers to that it neither declares nor imports from a module of the sources.
   */
  readonly freeNames: ReadonlySet<string>;
};

/**
 A function that can be used, by its name and the file that declares it.
 */
type Referent = {
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
 The list of schema producers, which calls the builders with sample labels to read the schemas they make and so is no
 stage that builds one.
 */
const PRODUCER_LIST_FILE = 'corpus-pass/src/claude-schema-producers.ts';

/**
 The bindings and exports already followed when a walk starts.
 */
const NOTHING_FOLLOWED: ReadonlySet<string> = new Set<string>();

/**
 Where a function sits, as one text.

 @param file - path of the file that declares the function

 @param name - function's name

 @returns Key of the function

 @example
 ```ts
 const key = keyOf({ file: 'a/src/a.ts', name: 'build', },);
 ```
 */
export function keyOf({
  file,
  name,
}: {
  readonly file: string;
  readonly name: string
},): string {
  return `${file}#${name}`;
}

/**
 The functions a search has found so far, by where each sits and by name.
 */
export type KnownFunctions = {
  /**
   Keys of the functions, by the module that declares them.
   */
  readonly keys: ReadonlySet<string>;

  /**
   Names of the functions, which a name a file neither declares nor imports is read as.
   */
  readonly names: ReadonlySet<string>;
};

/**
 Whether a chain of names written in a file stands for one of the known functions: the function the chain resolves to,
 or, for a name the file neither declares nor imports, the function of that name.

 @param family - modules of the sources

 @param file - path of the file that writes the chain

 @param path - names in order

 @param known - functions looked for

 @returns Whether the chain stands for one of them

 @example
 ```ts
 const calls = standsForKnown({ family, file, path: ['contestResponseFormat',], known, },);
 ```
 */
export function standsForKnown(
  {
    family,
    file,
    path,
    known,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly path: readonly string[];
    readonly known: KnownFunctions;
  },
): boolean {
  /**
   What the chain stands for in the file that writes it.
   */
  const callee = resolvePath({
    family,
    file,
    path,
    seen: NOTHING_FOLLOWED,
  },);
  return (callee.kind === 'function') ? known.keys
    .has(keyOf(callee,),) : (callee.kind === 'free')
      && known.names
        .has(callee.name,);
}

/**
 What a file's references stand for, once.

 @param family - modules of the sources

 @param facts - file

 @returns Functions and the free names

 @example
 ```ts
 const usage = usageOf({ family, facts, },);
 ```
 */
function usageOf({
  family,
  facts,
}: {
  readonly family: Family;
  readonly facts: ModuleFacts
},): Usage {
  /**
   What each distinct reference of the file stands for.
   */
  const resolved: readonly Resolved[] = [
    ...new Map(facts.references
      .map(function keyedPath(path,): readonly [
        string,
        readonly string[],
      ] {
        return [
          path.join('.',),
          path,
        ];
      },),)
      .values(),
  ].map(function resolveReference(path,): Resolved {
    return resolvePath({
      family,
      file: facts.file,
      path,
      seen: NOTHING_FOLLOWED,
    },);
  },);
  return {
    file: facts.file,
    keys: new Set(resolved.flatMap(function keyOfFunction(reference,): readonly string[] {
      return (reference.kind === 'function') ? [keyOf(reference,),] : [];
    },),),
    freeNames: new Set(resolved.flatMap(function nameOfFree(reference,): readonly string[] {
      return (reference.kind === 'free') ? [reference.name,] : [];
    },),),
  };
}

/**
 Whether a file's references include a function, by the module that declares it or by its name.

 @param usage - what the file's references stand for

 @param builder - function, with the file that declares it

 @returns Whether the file refers to it

 @example
 ```ts
 const refers = refersTo({ usage, builder, },);
 ```
 */
function refersTo({
  usage,
  builder,
}: {
  readonly usage: Usage;
  readonly builder: Referent
},): boolean {
  return usage.keys
    .has(keyOf(builder,),)
    || usage.freeNames
      .has(builder.name,);
}

/**
 The files that use a function, the one that declares it and the producer list left out.

 @param usages - what the references of each file stand for

 @param builder - function whose uses are looked for

 @returns Paths, in the order of the sources

 @example
 ```ts
 const users = usersIn({ usages, builder: { name: 'build', file: 'a.ts', }, },);
 ```
 */
export function usersIn(
  {
    usages,
    builder,
  }: {
    readonly usages: readonly Usage[];
    readonly builder: Referent;
  },
): readonly string[] {
  return usages
    .filter(function usesBuilder(usage,): boolean {
      return (usage.file !== builder.file) && (usage.file !== PRODUCER_LIST_FILE)
        && refersTo({
          usage,
          builder,
        },);
    },)
    .map(function pathOf({ file, },): string {
      return file;
    },);
}

/**
 What the references of every file stand for.

 @param family - modules of the sources

 @returns Usage of each file, in the order of the sources

 @example
 ```ts
 const usages = usagesIn({ family, },);
 ```
 */
export function usagesIn({ family, }: { readonly family: Family; },): readonly Usage[] {
  return [...family.values(),].map(function usageOfFile(facts,): Usage {
    return usageOf({
      family,
      facts,
    },);
  },);
}

//endregion What the files of the sources refer to
