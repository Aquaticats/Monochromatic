import type { ModuleFacts, } from './claude-schema-facts.test-fixture.ts';
import {
  exportStep,
  starStep,
} from './claude-schema-resolve-export-steps.test-fixture.ts';
import {
  memberStep,
  nameStep,
  pathStep,
  type Query,
} from './claude-schema-resolve-steps.test-fixture.ts';

//region Which function a name stands for
// What `claude-schema-sources.test-fixture.ts` asks to tell whether a file uses a builder: for a name or a chain of
// members written in one file, the function it stands for, found by following the file's imports, aliases, destructured
// members and dynamic imports to the module that declares it, and through the modules that export it again. Modules are
// the files handed in; a specifier that names none of them (a package outside the family, a path nobody supplied)
// leaves the name to be read as the function of that name.

/**
 What a name or a chain of members stands for.
 */
export type Resolved =
  | {
    /**
     A function a module declares, or a member of an object it declares as `object.member`.
     */
    readonly kind: 'function';

    /**
     Path of the module.
     */
    readonly file: string;

    /**
     The function's name.
     */
    readonly name: string;
  }
    | {
    /**
     An object a module declares, whose members are functions.
     */
      readonly kind: 'object';

    /**
     Path of the module.
     */
      readonly file: string;

    /**
     The object's name.
     */
      readonly name: string;
    }
    | {
    /**
     A module of the sources as a whole.
     */
      readonly kind: 'namespace';

    /**
     Path of the module.
     */
      readonly file: string;
    }
    | {
    /**
     A name the file neither declares nor imports from a module of the sources, read as the function of that name.
     */
      readonly kind: 'free';

    /**
     The name, a chain of members joined by dots.
     */
      readonly name: string;
    }
    | {
    /**
     A module outside the sources as a whole, whose members are read as free names.
     */
      readonly kind: 'outside';
    }
    | {
    /**
     Something no function is read through: a value that is none, a member of a function.
     */
      readonly kind: 'opaque';
    }
    | {
    /**
     What a module exports under a name it does not export.
     */
      readonly kind: 'none';
    };

/**
 The modules of the sources, by path.
 */
export type Family = ReadonlyMap<string, ModuleFacts>;

/**
 Answers a question about the family by the step that reads it, which gets this function to ask the questions it leads
 to.

 @param family - modules of the sources

 @param seen - bindings and exports already followed

 @param query - question

 @returns What the question's subject stands for

 @example
 ```ts
 const resolved = resolveQuery({ family, seen: new Set<string>(), query: { kind: 'export', file, name: 'build', }, },);
 ```
 */
function resolveQuery(
  {
    family,
    seen,
    query,
  }: {
    readonly family: Family;
    readonly seen: ReadonlySet<string>;
    readonly query: Query;
  },
): Resolved {
  if (query.kind === 'member') {
    return memberStep({
      family,
      of: query.of,
      member: query.member,
      seen,
      resolve: resolveQuery,
    },);
  }
  if (query.kind === 'name') {
    return nameStep({
      family,
      file: query.file,
      name: query.name,
      seen,
      resolve: resolveQuery,
    },);
  }
  if (query.kind === 'path') {
    return pathStep({
      family,
      file: query.file,
      path: query.path,
      seen,
      resolve: resolveQuery,
    },);
  }
  if (query.kind === 'export') {
    return exportStep({
      family,
      file: query.file,
      name: query.name,
      seen,
      resolve: resolveQuery,
    },);
  }
  return starStep({
    family,
    file: query.file,
    name: query.name,
    seen,
    resolve: resolveQuery,
  },);
}

/**
 What a chain of names written in a module stands for.

 @param family - modules of the sources

 @param file - path of the module that writes the chain

 @param path - names in order, the first of which is bound at the top level of the module or free

 @param seen - bindings already followed

 @returns What the chain stands for

 @example
 ```ts
 const resolved = resolvePath({ family, file, path: ['kit', 'build',], seen: new Set<string>(), },);
 ```
 */
export function resolvePath(
  {
    family,
    file,
    path,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly path: readonly string[];
    readonly seen: ReadonlySet<string>;
  },
): Resolved {
  return resolveQuery({
    family,
    seen,
    query: {
      kind: 'path',
      file,
      path,
    },
  },);
}

/**
 What a module exports under a name, followed through the modules it exports from.

 @param family - modules of the sources

 @param file - path of the module

 @param name - name it exports, or `default`

 @param seen - exports already followed, so a module that exports itself ends

 @returns What the export stands for, `none` when the module exports no such name

 @example
 ```ts
 const resolved = resolveExport({ family, file: 'a/src/index.ts', name: 'build', seen: new Set<string>(), },);
 ```
 */
export function resolveExport(
  {
    family,
    file,
    name,
    seen,
  }: {
    readonly family: Family;
    readonly file: string;
    readonly name: string;
    readonly seen: ReadonlySet<string>;
  },
): Resolved {
  return resolveQuery({
    family,
    seen,
    query: {
      kind: 'export',
      file,
      name,
    },
  },);
}

//endregion Which function a name stands for
