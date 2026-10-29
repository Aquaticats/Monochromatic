/**
 Guards against a top-level declaration in a test file or test fixture that
 nothing reads (ledger B118). No configured check reports one: the
 repository's lint leaves `no-unused-vars` off and its TypeScript
 configuration leaves `noUnusedLocals` false, so a declaration a caller
 stopped reading outlives the caller, the same gap `unused-imports.unit.test.ts`
 closes for imports rather than declarations.

 WHAT COUNTS AS DECLARED. A top-level function, class, interface, type alias,
 enum, or const, let or var name (destructured names included), a function's
 overload signatures merged with its implementation under one name.

 WHAT COUNTS AS READ. An unexported declaration is read when its name stands
 as an `Identifier` somewhere else in its file in a position that reads a
 binding: never as a property or member name (`{ nap: 1 }`, `cat.nap`, a
 class or interface member, a qualified type name's last part, an enum
 member, an import or export specifier's name, a label), and never as
 another binding (a parameter, a nested declaration, a pattern's target,
 a type parameter). A name mentioned only inside a TSDoc example is not
 read either, since a comment is no node of the program. An exported
 declaration in a `*.test-fixture.ts` file must be imported or re-exported
 by name from that file by another file: `export` is a fixture's offer to
 other files, and one its own file alone reads offers nothing, so it loses
 the `export` rather than counting as read. An export in a `*.test.ts` file
 is left alone.

 FORMS THE SCAN CANNOT FOLLOW ARE FINDINGS OF THEIR OWN, so a reader the scan
 does not see can never pass silently: a default export, and a namespace
 import or a dynamic `import()` of a test file or fixture (a source that ends
 in `.test.ts` or `.test-fixture.ts`), in a test file or fixture. The
 package's tests use none of them. An import of anything else, such as a built
 entry under `dist/` or package source, reads no name of a test file or
 fixture, so there is nothing to follow and it is no finding.

 LEFT LOOSE: a nested binding that shadows a top-level name and is then read
 counts as reading the top-level declaration, and two unexported declarations
 that read only each other both count as read; telling either apart needs
 scope resolution, which the scan does not do.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each kind (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source, tests included.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  declaringModule,
  namesFamilyModule,
  readPackageSource,
} from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  literalText,
  nodesUnder,
  parentsOf,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

//region Dead top-level declaration scan
// Each test file's top-level names, read against the identifiers its program
// holds in reading positions, and each fixture's exports read against the
// names other files import from it.

/**
 Top-level declaration kinds whose name is their `id`.
 */
const NAMED_DECLARATION_TYPES: ReadonlySet<string> = new Set([
  'ClassDeclaration',
  'FunctionDeclaration',
  'TSDeclareFunction',
  'TSEnumDeclaration',
  'TSInterfaceDeclaration',
  'TSTypeAliasDeclaration',
],);

/**
 Fields of a node kind whose identifier names a property, member, label or
 specifier rather than reading a binding, by node kind; a computed key reads
 its expression, so these apply only where the node is not computed.
 */
const NAME_ONLY_FIELDS: ReadonlyMap<string, readonly string[]> = new Map([
  [
    'AccessorProperty',
    ['key',],
  ],
  [
    'BreakStatement',
    ['label',],
  ],
  [
    'ContinueStatement',
    ['label',],
  ],
  [
    'ExportSpecifier',
    [
      'local',
      'exported',
    ],
  ],
  [
    'ImportSpecifier',
    ['imported',],
  ],
  [
    'LabeledStatement',
    ['label',],
  ],
  [
    'MemberExpression',
    ['property',],
  ],
  [
    'MethodDefinition',
    ['key',],
  ],
  [
    'Property',
    ['key',],
  ],
  [
    'PropertyDefinition',
    ['key',],
  ],
  [
    'TSAbstractAccessorProperty',
    ['key',],
  ],
  [
    'TSAbstractMethodDefinition',
    ['key',],
  ],
  [
    'TSAbstractPropertyDefinition',
    ['key',],
  ],
  [
    'TSEnumMember',
    ['id',],
  ],
  [
    'TSMethodSignature',
    ['key',],
  ],
  [
    'TSPropertySignature',
    ['key',],
  ],
  [
    'TSQualifiedName',
    ['right',],
  ],
],);

/**
 Field holding a binding target's own targets, by node kind: a default's
 target, a rest element's argument, a parameter property's parameter, a
 pattern property's value, and an object or array pattern's members.
 */
const BINDING_TARGET_FIELDS: ReadonlyMap<string, string> = new Map([
  [
    'ArrayPattern',
    'elements',
  ],
  [
    'AssignmentPattern',
    'left',
  ],
  [
    'ObjectPattern',
    'properties',
  ],
  [
    'Property',
    'value',
  ],
  [
    'RestElement',
    'argument',
  ],
  [
    'TSParameterProperty',
    'parameter',
  ],
],);

/**
 Node kinds that bind parameters, values and type signatures alike.
 */
const FUNCTION_TYPES: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
  'TSCallSignatureDeclaration',
  'TSConstructSignatureDeclaration',
  'TSConstructorType',
  'TSDeclareFunction',
  'TSFunctionType',
  'TSMethodSignature',
],);

/**
 One top-level declaration a file carries.
 */
type TopLevelDeclaration = {
  /**
   Declared name.
   */
  readonly name: string;

  /**
   Whether it carries `export`, directly or through a same-file export list.
   */
  readonly exported: boolean;
};

/**
 One name a top-level statement declares.
 */
type DeclaredName = {
  /**
   Declared name.
   */
  readonly name: string;

  /**
   Whether the statement exports it directly.
   */
  readonly exported: boolean;
};

/**
 Nodes a field holds, a list flattened.

 @param value - field read, of any kind

 @returns Its nodes, none when it holds none

 @example
 ```ts
 const statements = nodesIn({ value: program.body, },);
 ```
 */
function nodesIn({ value, }: { readonly value: unknown; },): readonly TreeNode[] {
  /**
   The field as a list.
   */
  const items: readonly unknown[] = Array.isArray(value,) ? value : [value,];
  return items.filter(function isNode(item,): item is TreeNode {
    return isTreeNode(item,);
  },);
}

/**
 Every identifier a binding target declares, destructured and renamed names
 included; a pattern's property keys and defaults declare nothing.

 @param node - identifier, pattern, rest element or parameter property a
 declarator or parameter binds

 @returns Its bound identifiers, in no particular order

 @example
 ```ts
 const names = bindingNamesOf({ node: declarator.id, },);
 ```
 */
function bindingNamesOf({ node, }: { readonly node: unknown; },): readonly TreeNode[] {
  if (!isTreeNode(node,))
    return [];
  if (node.type === 'Identifier')
    return [node,];
  /**
   Field holding this node's binding targets, absent for a node binding
   nothing.
   */
  const field = BINDING_TARGET_FIELDS.get(node.type,);
  if (field === undefined)
    return [];
  return nodesIn({ value: node[field], },)
    .flatMap(function fromTarget(target,): readonly TreeNode[] {
      return bindingNamesOf({ node: target, },);
    },);
}

/**
 Names one top-level statement declares.

 @param statement - statement of a program's body

 @returns Its declared names, none for an import, an expression or an export
 list

 @example
 ```ts
 const names = declaredNamesOf({ statement, },);
 ```
 */
function declaredNamesOf({ statement, }: { readonly statement: TreeNode; },): readonly DeclaredName[] {
  /**
   Whether the statement is an `export` wrapping a declaration.
   */
  const exported = statement.type === 'ExportNamedDeclaration';
  /**
   The declaration itself, absent for an export list or a re-export.
   */
  const inner: unknown = exported ? statement.declaration : statement;
  if (!isTreeNode(inner,))
    return [];
  if (NAMED_DECLARATION_TYPES.has(inner.type,)) {
    return nodesIn({ value: inner.id, },)
      .map(function named(id,): DeclaredName {
        return {
          name: identifierName({ node: id, },),
          exported,
        };
      },);
  }
  if (inner.type !== 'VariableDeclaration')
    return [];
  return nodesIn({ value: inner.declarations, },)
    .flatMap(function fromDeclarator(declarator,): readonly TreeNode[] {
      return bindingNamesOf({ node: declarator.id, },);
    },)
    .map(function named(id,): DeclaredName {
      return {
        name: identifierName({ node: id, },),
        exported,
      };
    },);
}

/**
 Names a same-file export list (`export { nap, };`) exports.

 @param program - file's program

 @returns Local names the file's export lists name

 @example
 ```ts
 const listed = listedExportsOf({ program, },);
 ```
 */
function listedExportsOf({ program, }: { readonly program: TreeNode; },): ReadonlySet<string> {
  return new Set(nodesIn({ value: program.body, },)
    .filter(function isLocalList(statement,): boolean {
      return (statement.type === 'ExportNamedDeclaration') && (!isTreeNode(statement.declaration,))
        && (!isTreeNode(statement.source,));
    },)
    .flatMap(function specifiers(statement,): readonly TreeNode[] {
      return nodesIn({ value: statement.specifiers, },);
    },)
    .map(function localName(specifier,): string {
      return identifierName({ node: specifier.local, },);
    },),);
}

/**
 Every top-level declaration a file's program carries, one per name, a
 function's overloads merged with its implementation.

 @param program - file's program

 @returns Declarations in source order of their first statement

 @example
 ```ts
 const declarations = topLevelDeclarationsOf({ program, },);
 ```
 */
function topLevelDeclarationsOf({ program, }: { readonly program: TreeNode; },): readonly TopLevelDeclaration[] {
  /**
   Every name each statement declares, overloads repeating their name.
   */
  const declared = nodesIn({ value: program.body, },)
    .flatMap(function fromStatement(statement,): readonly DeclaredName[] {
      return declaredNamesOf({ statement, },);
    },);
  /**
   Names a same-file export list exports.
   */
  const listed = listedExportsOf({ program, },);
  return [
    ...new Set(declared.map(function nameOf(declaration,): string {
      return declaration.name;
    },),),
  ]
    .map(function merged(name,): TopLevelDeclaration {
      return {
        name,
        exported: listed.has(name,) || declared.some(function exports(declaration,): boolean {
          return (declaration.name === name) && declaration.exported;
        },),
      };
    },);
}

/**
 Every identifier a program binds rather than reads: declared names at any
 depth, parameters, pattern targets, catch parameters, type parameters and
 import names.

 @param program - file's program

 @returns Binding identifier nodes

 @example
 ```ts
 const bindings = bindingIdentifiersOf({ program, },);
 ```
 */
function bindingIdentifiersOf({ program, }: { readonly program: TreeNode; },): ReadonlySet<TreeNode> {
  return new Set(nodesUnder({ root: program, },)
    .flatMap(function bound(node,): readonly TreeNode[] {
      if (node.type === 'VariableDeclarator')
        return bindingNamesOf({ node: node.id, },);
      if (FUNCTION_TYPES.has(node.type,)) {
        return [
          ...nodesIn({ value: node.id, },),
          ...nodesIn({ value: node.params, },)
            .flatMap(function fromParameter(parameter,): readonly TreeNode[] {
              return bindingNamesOf({ node: parameter, },);
            },),
        ];
      }
      if (node.type === 'CatchClause')
        return bindingNamesOf({ node: node.param, },);
      if (node.type === 'TSTypeParameter')
        return nodesIn({ value: node.name, },);
      if ((node.type === 'ImportSpecifier') || (node.type === 'ImportDefaultSpecifier')
        || (node.type === 'ImportNamespaceSpecifier'))
        return nodesIn({ value: node.local, },);
      if (NAMED_DECLARATION_TYPES.has(node.type,) || (node.type === 'ClassExpression'))
        return nodesIn({ value: node.id, },);
      return [];
    },),);
}

/**
 Names a file reads: every identifier in a reading position, never a
 property, member, label or specifier name and never a binding.

 @param program - file's program

 @returns Names read

 @example
 ```ts
 const read = namesReadIn({ program, },);
 ```
 */
function namesReadIn({ program, }: { readonly program: TreeNode; },): ReadonlySet<string> {
  /**
   Each node's parent.
   */
  const parents = parentsOf({ program, },);
  /**
   Identifiers that bind rather than read.
   */
  const bindings = bindingIdentifiersOf({ program, },);
  return new Set(nodesUnder({ root: program, },)
    .filter(function reads(node,): boolean {
      if ((node.type !== 'Identifier') || bindings.has(node,))
        return false;
      /**
       The node holding this identifier.
       */
      const parent = parents.get(node,);
      if ((parent === undefined) || (parent.computed === true))
        return true;
      return !(NAME_ONLY_FIELDS.get(parent.type,) ?? []).some(function namesOnly(field,): boolean {
        return parent[field] === node;
      },);
    },)
    .map(function nameOf(node,): string {
      return identifierName({ node, },);
    },),);
}

/**
 Names each file's static imports and re-exports take from the family file
 that declares them, as `path#name` keys: through a relative specifier, a family package's
 `/ts/<path>` or the index of a family package, which re-exports the name from its declaring file.

 @param files - files to read

 @returns Every `path#name` some file imports or re-exports

 @example
 ```ts
 const imported = importedNames({ files, },);
 ```
 */
function importedNames({ files, }: { readonly files: readonly SourceText[]; },): ReadonlySet<string> {
  return new Set(files.flatMap(function fromFile(file,): readonly string[] {
    return nodesIn({ value: parseSource({ file, },).program.body, },)
      .filter(function namesSource(statement,): boolean {
        return ((statement.type === 'ImportDeclaration') || (statement.type === 'ExportNamedDeclaration'))
          && namesFamilyModule({ specifier: literalText({ node: statement.source, },), },);
      },)
      .flatMap(function keys(statement,): readonly string[] {
        return nodesIn({ value: statement.specifiers, },)
          .filter(function named(specifier,): boolean {
            return (specifier.type === 'ImportSpecifier') || (specifier.type === 'ExportSpecifier');
          },)
          .map(function key(specifier,): string {
            /**
             Name taken from the module the statement names.
             */
            const taken = identifierName({ node: (specifier.type === 'ImportSpecifier') ? specifier.imported : specifier.local, },);
            return `${declaringModule({
              fromPath: file.path,
              specifier: literalText({ node: statement.source, },),
              imported: taken,
            },)}#${taken}`;
          },);
      },);
  },),);
}

/**
 Whether an import source names a test file or a test fixture, the files whose names the scan tracks.

 @param specifier - text of the import source

 @returns Whether it ends in `.test.ts` or `.test-fixture.ts`

 @example
 ```ts
 const tracked = namesTestFile({ specifier: './whisker-fixture.test-fixture.ts', },); // true
 ```
 */
function namesTestFile({ specifier, }: { readonly specifier: string; },): boolean {
  return specifier.endsWith('.test.ts',) || specifier.endsWith('.test-fixture.ts',);
}

/**
 Forms in a file the scan cannot follow: a default export, and a namespace import or a dynamic `import()` of a test
 file or test fixture.

 @param file - file read

 @param program - its program

 @returns Each as `path#<form>`

 @example
 ```ts
 const unfollowed = unfollowedFormsIn({ file, program, },);
 ```
 */
function unfollowedFormsIn({ file, program, }: { readonly file: SourceText; readonly program: TreeNode; },): readonly string[] {
  return nodesUnder({ root: program, },)
    .flatMap(function form(node,): readonly string[] {
      if (node.type === 'ExportDefaultDeclaration')
        return [`${file.path}#default export`,];
      if ((node.type === 'ImportDeclaration') && namesTestFile({ specifier: literalText({ node: node.source, },), },)
        && nodesIn({ value: node.specifiers, },)
          .some(function isNamespace(specifier,): boolean {
            return specifier.type === 'ImportNamespaceSpecifier';
          },))
        return [`${file.path}#namespace import`,];
      if ((node.type === 'ImportExpression') && namesTestFile({ specifier: literalText({ node: node.source, },), },))
        return [`${file.path}#dynamic import`,];
      return [];
    },);
}

/**
 Top-level declarations in a set of test files and test fixtures that
 nothing reads, and the forms the scan cannot follow.

 @param files - package's files; package source carries no declaration
 the scan reads, but its imports count

 @returns Each as `path#name` or `path#<form>`, sorted without repeats

 @example
 ```ts
 const dead = deadTopLevelDeclarations({ files, },);
 ```
 */
function deadTopLevelDeclarations({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Every `path#name` some file imports or re-exports.
   */
  const imported = importedNames({ files, },);
  return [
    ...new Set(files
      .filter(function isTest(file,): boolean {
        return file.path.endsWith('.test.ts',) || file.path.endsWith('.test-fixture.ts',);
      },)
      .flatMap(function inFile(file,): readonly string[] {
        /**
         The file's program and the names it reads.
         */
        const { program, } = parseSource({ file, },);
        const read = namesReadIn({ program, },);
        /**
         Whether an exported declaration here must be imported by another file.
         */
        const isFixture = file.path.endsWith('.test-fixture.ts',);
        return [
          ...unfollowedFormsIn({
            file,
            program,
          },),
          ...topLevelDeclarationsOf({ program, },)
            .filter(function dead(declaration,): boolean {
              if (!declaration.exported)
                return !read.has(declaration.name,);
              return isFixture && (!imported.has(`${file.path}#${declaration.name}`,));
            },)
            .map(function located(declaration,): string {
              return `${file.path}#${declaration.name}`;
            },),
        ];
      },),),
  ].toSorted();
}

//endregion Dead top-level declaration scan

/**
 A source file for the fixture case.

 @param path - file name, whose ending decides whether the scan reads it

 @param lines - file lines

 @returns Source file

 @example
 ```ts
 const file = fixture({ path: 'cat.test.ts', lines: ['export const nap = 1;',], },);
 ```
 */
function fixture({ path, lines, }: { readonly path: string; readonly lines: readonly string[]; },): SourceText {
  return {
    path,
    text: lines.join('\n',),
    isTest: true,
  };
}

/**
 The cat fixtures: a test file holding dead declarations of each kind the
 scan reads (one named only in its own TSDoc example, one only as a property
 key, one only as a member name, one only through a shadowing parameter's
 own binding) beside live ones read through a shorthand property, `typeof`
 and a computed key; a fixture exporting one name a companion file imports,
 one a companion re-exports, one only its own file reads, and one nobody
 reads, beside an unexported name its own file reads; and a test file
 holding the three forms the scan cannot follow.
 */
const CAT_FILES: readonly SourceText[] = [
  fixture({
    path: 'napping-cats.test.ts',
    lines: [
      'function sleepsAllDay(): string {',
      '  return \'zzz\';',
      '}',
      'const unusedKibble = \'salmon\';',
      '/**',
      ' @example',
      ' ```ts',
      ' const greeting = purrsOnCommand();',
      ' ```',
      ' */',
      'function purrsOnCommand(): string {',
      '  return \'purr\';',
      '}',
      'const keyOnly = 1;',
      'const memberOnly = 2;',
      'const shadowed = 3;',
      'function feedCat(): string {',
      '  return \'kibble\';',
      '}',
      'function napTimer(): number {',
      '  return 60;',
      '}',
      'const computedKey = \'tail\';',
      'export const bowl = { feedCat, keyOnly: 1, [computedKey]: 2, };',
      'export type NapTimerKind = typeof napTimer;',
      'export const tail = bowl.memberOnly;',
      'export function groom(shadowed: number): void {}',
    ],
  },),
  fixture({
    path: 'whisker-fixture.test-fixture.ts',
    lines: [
      'export function sharedTreat(): string {',
      '  return \'treat\';',
      '}',
      'export function passedOnTreat(): string {',
      '  return \'cream\';',
      '}',
      'export const TREAT_COUNT = 3;',
      'const BOWL_SIZE = 2;',
      'export function countedTreats(): number {',
      '  return TREAT_COUNT * BOWL_SIZE;',
      '}',
      'export function secretStash(): string {',
      '  return \'tuna\';',
      '}',
    ],
  },),
  fixture({
    path: 'corpus-run/whisker-fixture-user.test.ts',
    lines: [
      'import { countedTreats, sharedTreat, } from \'../whisker-fixture.test-fixture.ts\';',
      'export { passedOnTreat, } from \'../whisker-fixture.test-fixture.ts\';',
      'export const treats = [sharedTreat(), countedTreats(),];',
    ],
  },),
  fixture({
    path: 'stray-forms.test.ts',
    lines: [
      'import * as whiskers from \'./whisker-fixture.test-fixture.ts\';',
      'export const later = await import(\'./whisker-fixture.test-fixture.ts\');',
      'export default whiskers;',
    ],
  },),
  fixture({
    path: 'built-entry.test.ts',
    lines: [
      'import * as built from \'../dist/final/node/index.mjs\';',
      'export const loaded = await import(\'../dist/final/node/index.mjs\');',
      'export const sourced = await import(\'../src/nap.ts\');',
      'export const namespaced = built;',
    ],
  },),
  fixture({
    path: 'stray-family.test.ts',
    lines: [
      'export const viaFamily = await import(\'@monochromatic-dev/translation-repair-test-fixture/ts/nap.test-fixture.ts\');',
    ],
  },),
  fixture({
    path: 'stray-test.test.ts',
    lines: [
      'import * as cats from \'./napping-cats.test.ts\';',
      'export const namespaced = cats;',
    ],
  },),
];

await describe({
  name: 'top-level declarations a test file or test fixture nothing reads (ledger B118)',
  children: [
    it({
      name: 'FINDS unexported names read nowhere, read only in a TSDoc example, only as a property key, only as a '
        + 'member name or only through a shadowing parameter, fixture exports no other file imports (read in '
        + 'their own file or not), and each form the scan cannot follow; KEEPS names read through a shorthand '
        + 'property, `typeof` or a computed key, and fixture exports another file imports or re-exports',
      fn: async () => {
        expect(deadTopLevelDeclarations({ files: CAT_FILES, },),).toEqual([
          'napping-cats.test.ts#keyOnly',
          'napping-cats.test.ts#memberOnly',
          'napping-cats.test.ts#purrsOnCommand',
          'napping-cats.test.ts#shadowed',
          'napping-cats.test.ts#sleepsAllDay',
          'napping-cats.test.ts#unusedKibble',
          'stray-family.test.ts#dynamic import',
          'stray-forms.test.ts#default export',
          'stray-forms.test.ts#dynamic import',
          'stray-forms.test.ts#namespace import',
          'stray-test.test.ts#namespace import',
          'whisker-fixture.test-fixture.ts#TREAT_COUNT',
          'whisker-fixture.test-fixture.ts#secretStash',
        ],);
      },
    },),
    it({
      name: 'FINDS NO TOP-LEVEL DECLARATION NOTHING READS, and no form the scan cannot follow, across the '
        + 'package\'s tests and fixtures',
      fn: async () => {
        expectNoFindings({ findings: deadTopLevelDeclarations({ files: await readPackageSource(), },), },);
      },
    },),
  ],
},);
