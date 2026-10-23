import { listStatementsOf, } from './claude-schema-code.test-fixture.ts';
import { NOT_FOUND, } from './claude-schema-text.test-fixture.ts';

//region Names a file knows a function by
// What the builder search reads off the import lists of a file, so a function imported under another name, through a
// namespace or as the default export is still found: `import { name as local }`, `import * as ns` and `import local
// from`. A type-only import brings in no function, a side-effect import no name, and an export list (`export { name as
// other } from`) is not followed.

/**
 One name a file imports a function by.
 */
type NamedImport = {
  /**
   The name the module exports it by.
   */
  readonly imported: string;

  /**
   The name the file uses it by.
   */
  readonly local: string;
};

/**
 What a file imports, as the names it uses.
 */
export type ImportBindings = {
  /**
   Names imported by name, with an alias or without.
   */
  readonly named: readonly NamedImport[];

  /**
   Names given to the default export of a module, with the module's specifier.
   */
  readonly defaults: readonly {
    readonly local: string;
    readonly specifier: string;
  }[];

  /**
   Names given to a module as a whole, by which its members are read as `name.member`.
   */
  readonly namespaces: readonly string[];
};

/**
 A function that can be imported: its name and the file that declares it.
 */
type ImportedFunction = {
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
 The text between the quotes that follow a statement's `from`.

 @param statement - import statement's text

 @returns Module specifier, empty when the statement names none

 @example
 ```ts
 const specifier = specifierOf({ statement: 'import a from \'./a.ts\';', },);
 // => './a.ts'
 ```
 */
function specifierOf({ statement, }: { readonly statement: string; },): string {
  /**
   Text after the last `from`.
   */
  const afterFrom = statement.slice(statement.lastIndexOf(' from ',) + ' from '.length,)
    .trim();
  /**
   The quote that opens the specifier.
   */
  const quote = afterFrom.at(0,) ?? '';
  /**
   Where the closing quote stands.
   */
  const end = afterFrom.indexOf(
    quote,
    1,
  );
  return ((quote === '') || (end === NOT_FOUND)) ? '' : afterFrom.slice(
    1,
    end,
  );
}

/**
 The names an import statement brings in.

 @param statement - import statement's text

 @returns Names, empty for a type-only import, a side-effect import and an export list

 @example
 ```ts
 const bindings = bindingsOfStatement({ statement: 'import a, { b as c, } from \'./a.ts\';', },);
 // => { named: [{ imported: 'b', local: 'c', }], defaults: [{ local: 'a', specifier: './a.ts', }], namespaces: [] }
 ```
 */
function bindingsOfStatement({ statement, }: { readonly statement: string; },): ImportBindings {
  /**
   What an import that brings in nothing looks like.
   */
  const nothing: ImportBindings = {
    named: [],
    defaults: [],
    namespaces: [],
  };
  /**
   The statement without blanks at its ends.
   */
  const trimmedStatement = statement.trim();
  /**
   The statement after `import`.
   */
  const afterImport = trimmedStatement.slice('import '.length,);
  /**
   Where the clause of names ends.
   */
  const clauseEnd = afterImport.lastIndexOf(' from ',);
  /**
   Whether the statement is an import, which an export list is not.
   */
  const isImport = trimmedStatement.startsWith('import ',);
  /**
   Whether the import brings in types only, which no call uses.
   */
  const isTypeOnly = afterImport.startsWith('type ',);
  /**
   Whether the statement brings in no name a call could use.
   */
  const bringsNothing = (!isImport)
    || isTypeOnly
    || (clauseEnd === NOT_FOUND);
  if (bringsNothing)
    return nothing;
  /**
   The clause of names: a default name, a namespace, a list in braces, or a default name and one of the others.
   */
  const clause = afterImport.slice(
    0,
    clauseEnd,
  );
  /**
   The specifier of the module.
   */
  const specifier = specifierOf({ statement, },);
  /**
   Names in braces, one entry each, empty when the clause has no braces.
   */
  const members = clause.includes('{',)
    ? clause.slice(
      clause.indexOf('{',) + 1,
      clause.indexOf('}',),
    )
      .split(',',)
      .map(function trimmed(member,): string {
        return member.trim();
      },)
      .filter(function holdsName(member,): boolean {
        return (member !== '') && (!member.startsWith('type ',));
      },)
    : [];
  /**
   The clause before its braces, all of it when it has none.
   */
  const [beforeBraces = '',] = clause.split('{',);
  /**
   Names before the braces: a default name or a namespace, or both.
   */
  const outside = beforeBraces.split(',',)
    .map(function trimmed(name,): string {
      return name.trim();
    },)
    .filter(function holdsName(name,): boolean {
      return name !== '';
    },);
  /**
   Names in braces as imported name and local name.
   */
  const pairs = members.map(function pairOf(member,): NamedImport {
    /**
     The member's names: the name the module exports it by, and the alias after `as` when there is one.
     */
    const [imported = '', local = imported,] = member.split(' as ',)
      .map(function trimmed(name,): string {
        return name.trim();
      },);
    return {
      imported,
      local,
    };
  },);
  return {
    named: pairs.filter(function isNotDefault({ imported, },): boolean {
      return imported !== 'default';
    },),
    defaults: [
      ...pairs
        .filter(function isDefault({ imported, },): boolean {
          return imported === 'default';
        },)
        .map(function defaultOf({ local, },) {
          return {
            local,
            specifier,
          };
        },),
      ...outside
        .filter(function isDefaultName(name,): boolean {
          return !name.startsWith('* as ',);
        },)
        .map(function defaultOf(local,) {
          return {
            local,
            specifier,
          };
        },),
    ],
    namespaces: outside
      .filter(function isNamespace(name,): boolean {
        return name.startsWith('* as ',);
      },)
      .map(function nameOf(name,): string {
        return name.slice('* as '.length,)
          .trim();
      },),
  };
}

/**
 What a file imports.

 @param text - file's text

 @returns Names the file imports functions by

 @example
 ```ts
 const imports = importsOf({ text: 'import { build as make, } from \'./a.ts\';\n', },);
 // => { named: [{ imported: 'build', local: 'make', }], defaults: [], namespaces: [] }
 ```
 */
export function importsOf({ text, }: { readonly text: string; },): ImportBindings {
  /**
   What each import statement brings in.
   */
  const statements = listStatementsOf({ text, },)
    .map(function bindingsOf(statement,): ImportBindings {
      return bindingsOfStatement({ statement, },);
    },);
  return {
    named: statements.flatMap(function namedOf({ named, },): readonly NamedImport[] {
      return named;
    },),
    defaults: statements.flatMap(function defaultsOf({ defaults, },) {
      return defaults;
    },),
    namespaces: statements.flatMap(function namespacesOf({ namespaces, },): readonly string[] {
      return namespaces;
    },),
  };
}

/**
 The name of a file in a path, without its directory and its extension.

 @param path - path or module specifier

 @returns Last segment before its extension

 @example
 ```ts
 const base = baseNameOf({ path: '../a/default-wire.ts', },);
 // => 'default-wire'
 ```
 */
function baseNameOf({ path, }: { readonly path: string; },): string {
  /**
   The segments of the path.
   */
  const segments = path.split('/',);
  /**
   The last segment of the path.
   */
  const segment = segments.at(-1,) ?? '';
  /**
   Where the extension starts.
   */
  const dot = segment.lastIndexOf('.',);
  return (dot > 0) ? segment.slice(
    0,
    dot,
  ) : segment;
}

/**
 Every name by which a file can call a function: its own name, the alias it imports it under, the member of a namespace
 it imports, and, for a default export, the name it gives the default import of a module with the file's name.

 @param imported - function and the file that declares it

 @param imports - what the calling file imports

 @returns Names, each a text that stands alone where the function is called

 @example
 ```ts
 const imports = importsOf({ text: 'import { build as make, } from \'./a.ts\';\n', },);
 const names = namesOfFunctionIn({ imported: { name: 'build', file: 'a.ts', exportedAsDefault: false, }, imports, },);
 // => ['build', 'make']
 ```
 */
export function namesOfFunctionIn(
  {
    imported,
    imports,
  }: {
    readonly imported: ImportedFunction;
    readonly imports: ImportBindings;
  },
): readonly string[] {
  /**
   The function's name and file, and whether it is the default export.
   */
  const {
    name: functionName,
    file,
    exportedAsDefault,
  } = imported;
  /**
   What the file imports, by kind.
   */
  const {
    named,
    defaults,
    namespaces,
  } = imports;
  /**
   The part of the name before the first dot: the function, or the object that holds the member.
   */
  const [head = '',] = functionName.split('.',);
  /**
   The member's part of the name, empty for a function.
   */
  const member = functionName.slice(head.length,);
  return [
    ...new Set([
      functionName,
      ...named
        .filter(function importsHead({ imported: name, },): boolean {
          return name === head;
        },)
        .map(function aliased({ local, },): string {
          return `${local}${member}`;
        },),
      ...namespaces.map(function memberOf(namespace,): string {
        return `${namespace}.${functionName}`;
      },),
      ...(exportedAsDefault
        ? defaults
          .filter(function isModuleOfFile({ specifier, },): boolean {
            return baseNameOf({ path: specifier, },) === baseNameOf({ path: file, },);
          },)
          .map(function localOf({ local, },): string {
            return local;
          },)
        : []),
    ],),
  ];
}

//endregion Names a file knows a function by
