/**
 Guards against plain-object tables keyed by text (ledger B77). An object
 literal looked up by a key the package did not write answers `constructor`,
 `toString` and the prototype's other names with the prototype's own values,
 so a provider's delta type, a corpus asset's extension or a stored status
 spelled as one of them reads as a function where a table entry belongs; and
 one filled by such a key drops a `__proto__` entry without a word, since
 assigning to that name sets the prototype instead. A table keyed by text is
 a `ReadonlyMap`, and a record filled by text is a `Map` until it is handed on
 whole. A table whose keys are a closed set of names keeps them in its type,
 so the compiler refuses a lookup by unchecked text.

 WHAT THE SCAN READS, in the package's source; tests and test fixtures are not
 read. A declaration typed with string keys, written as `Record<string, …>`,
 that inside `Readonly`, a type literal with a `string` index signature, or a
 type alias the package's source declares as any of these, whether the type is
 the declaration's annotation or an `as` around its value. Such a declaration
 is a table when it sits at the top of its module with an object literal for
 its value, and an accumulator wherever its value is `{}`; and any name so
 typed, declared or taken as a parameter (annotated in place, or destructured
 from an object a type literal types), is filled by text wherever an
 assignment or an update writes it through a key that is not a written string
 or number. A `satisfies` is not read as typing it: it checks the object
 without widening it, so the keys stay the literal's own. Out of the scan's
 reach: a table built by `Object.fromEntries` or handed back from a function
 and then read by text, a class field, a parameter destructured from an object
 an alias types, a key typed by a template literal, and a literal-keyed table
 reached through a cast of text to its keys; ledger B77 records how the
 package's own cases of these were read.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a scan
 shown able to find each form (ledger M21). Fixtures are cat-themed; the
 package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  identifierName,
  isTreeNode,
  nodesUnder,
  parseSource,
  type SourceText,
  type TreeNode,
  unwrapped,
} from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 First type argument of a type reference, absent when it is given none.

 @param type - type reference read

 @returns Its first argument

 @example
 ```ts
 const keyType = firstTypeArgument({ type, },); // string for Record<string, number>
 ```
 */
function firstTypeArgument({ type, }: { readonly type: TreeNode; },): unknown {
  /**
   The reference's argument list.
   */
  const { typeArguments, } = type;
  if (!isTreeNode(typeArguments,))
    return undefined;
  return (typeArguments.params as readonly unknown[])[0];
}

/**
 Whether a type literal declares a `string` index signature.

 @param type - type literal read

 @returns True when one of its members is keyed by `string`

 @example
 ```ts
 const keyed = hasStringIndex({ type, },); // true for { [cat: string]: number }
 ```
 */
function hasStringIndex({ type, }: { readonly type: TreeNode; },): boolean {
  return (type.members as readonly TreeNode[]).some(function isStringIndex(member,): boolean {
    if (member.type !== 'TSIndexSignature')
      return false;
    /**
     The signature's key parameter.
     */
    const [key,] = member.parameters as readonly TreeNode[];
    /**
     The key's annotation.
     */
    const annotation = key?.typeAnnotation;
    return isTreeNode(annotation,)
      && isTreeNode(annotation.typeAnnotation,)
      && (annotation.typeAnnotation.type === 'TSStringKeyword');
  },);
}

/**
 Whether a type is keyed by any string.

 @param type - type node read, of any kind or none

 @param aliases - names of the package's type aliases keyed by any string

 @returns True for `Record<string, …>`, the same inside `Readonly`, a type
 literal with a `string` index signature, and a reference to one of the
 aliases

 @example
 ```ts
 const keyed = isStringKeyed({ type, aliases, },);
 ```
 */
function isStringKeyed(
  {
    type,
    aliases,
  }: {
    readonly type: unknown;
    readonly aliases: ReadonlySet<string>;
  },
): boolean {
  if (!isTreeNode(type,))
    return false;
  if (type.type === 'TSTypeLiteral')
    return hasStringIndex({ type, },);
  if (type.type !== 'TSTypeReference')
    return false;
  /**
   Name the reference names.
   */
  const name = identifierName({ node: type.typeName, },);
  if (name === 'Record') {
    /**
     The record's key type.
     */
    const keyType = firstTypeArgument({ type, },);
    return isTreeNode(keyType,) && (keyType.type === 'TSStringKeyword');
  }
  if (name === 'Readonly') {
    return isStringKeyed({
      type: firstTypeArgument({ type, },),
      aliases,
    },);
  }
  return aliases.has(name,);
}

/**
 Names of the type aliases the files declare as keyed by any string,
 following aliases of aliases.

 @param nodes - nodes of every file read

 @returns Alias names

 @example
 ```ts
 const aliases = stringKeyedAliases({ nodes, },); // Naps for type Naps = Record<string, number>
 ```
 */
function stringKeyedAliases({ nodes, }: { readonly nodes: readonly TreeNode[]; },): ReadonlySet<string> {
  /**
   Every alias declaration, by name.
   */
  const declarations = nodes.filter(function isAlias(node,): boolean {
    return node.type === 'TSTypeAliasDeclaration';
  },);

  /**
   Aliases found keyed by text so far.
   */
  const found = new Set<string>();
  for (let grown = true; grown;) {
    /**
     Aliases this pass adds.
     */
    const added = declarations.filter(function isNewlyKeyed(declaration,): boolean {
      return (!found.has(identifierName({ node: declaration.id, },),))
        && isStringKeyed({
          type: declaration.typeAnnotation,
          aliases: found,
        },);
    },);
    for (const declaration of added)
      found.add(identifierName({ node: declaration.id, },),);
    grown = added.length > 0;
  }
  return found;
}

/**
 Casts, the wrappers that give a value the type they name; a `satisfies`
 checks a value against a type and leaves it its own.
 */
const CAST_KINDS: ReadonlySet<string> = new Set(['TSAsExpression', 'TSTypeAssertion',],);

/**
 Whether a declarator is typed as keyed by any string, by its annotation or
 by a cast around its value.

 @param declarator - variable declarator read

 @param aliases - names of the package's type aliases keyed by any string

 @returns True when any of those types is keyed by any string

 @example
 ```ts
 const typed = isTypedStringKeyed({ declarator, aliases, },);
 ```
 */
function isTypedStringKeyed(
  {
    declarator,
    aliases,
  }: {
    readonly declarator: TreeNode;
    readonly aliases: ReadonlySet<string>;
  },
): boolean {
  /**
   The declared name's annotation.
   */
  const annotation = isTreeNode(declarator.id,) ? declarator.id.typeAnnotation : undefined;
  return [
    isTreeNode(annotation,) ? annotation.typeAnnotation : undefined,
    ...unwrapped({ node: declarator.init, },).wrappers
      .filter(function isCast(wrapper,): boolean {
        return CAST_KINDS.has(wrapper.type,);
      },)
      .map(function typeOf(wrapper,): unknown {
        return wrapper.typeAnnotation;
      },),
  ].some(function keyed(type,): boolean {
    return isStringKeyed({
      type,
      aliases,
    },);
  },);
}

/**
 Variable declarators a module's top level declares, exported or not.

 @param program - module read

 @returns Its top-level declarators

 @example
 ```ts
 const declarators = topLevelDeclarators({ program, },);
 ```
 */
function topLevelDeclarators({ program, }: { readonly program: TreeNode; },): readonly TreeNode[] {
  return (program.body as readonly TreeNode[]).flatMap(function declaratorsOf(statement,): readonly TreeNode[] {
    /**
     The declaration itself, out of an export.
     */
    const declaration = (statement.type === 'ExportNamedDeclaration') ? statement.declaration : statement;
    if ((!isTreeNode(declaration,)) || (declaration.type !== 'VariableDeclaration'))
      return [];
    return declaration.declarations as readonly TreeNode[];
  },);
}

/**
 Node kinds that take parameters.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set([
  'ArrowFunctionExpression',
  'FunctionDeclaration',
  'FunctionExpression',
],);

/**
 Type a parameter or pattern is annotated with in place.

 @param node - parameter or pattern read

 @returns Annotated type, absent when it carries none

 @example
 ```ts
 const type = annotatedType({ node: parameter, },); // Record<string, number> for (seen: Record<string, number>)
 ```
 */
function annotatedType({ node, }: { readonly node: TreeNode; },): unknown {
  /**
   The annotation wrapper.
   */
  const { typeAnnotation, } = node;
  return isTreeNode(typeAnnotation,) ? typeAnnotation.typeAnnotation : undefined;
}

/**
 Names one parameter binds that it types as keyed by any string: a name
 annotated in place, or a name destructured from an object whose annotation
 is a type literal giving that property such a type, defaults allowed.

 @param parameter - parameter read

 @param aliases - names of the package's type aliases keyed by any string

 @returns Local names

 @example
 ```ts
 const names = keyedNamesOf({ parameter, aliases, },); // ['beds'] for ({ beds, }: { readonly beds: Record<string, string>; })
 ```
 */
function keyedNamesOf(
  {
    parameter,
    aliases,
  }: {
    readonly parameter: TreeNode;
    readonly aliases: ReadonlySet<string>;
  },
): readonly string[] {
  /**
   The binding itself, out of a default value.
   */
  const binding = ((parameter.type === 'AssignmentPattern') && isTreeNode(parameter.left,))
    ? parameter.left
    : parameter;
  /**
   Its annotation.
   */
  const type = annotatedType({ node: binding, },);
  if (binding.type === 'Identifier') {
    return isStringKeyed({
      type,
      aliases,
    },)
      ? [identifierName({ node: binding, },),]
      : [];
  }
  if ((binding.type !== 'ObjectPattern') || (!isTreeNode(type,)) || (type.type !== 'TSTypeLiteral'))
    return [];
  /**
   Each property's type, by the name the type literal gives it.
   */
  const memberTypes = new Map((type.members as readonly TreeNode[])
    .filter(function isProperty(member,): boolean {
      return member.type === 'TSPropertySignature';
    },)
    .map(function named(member,): [string, unknown,] {
      return [
        identifierName({ node: member.key, },),
        annotatedType({ node: member, },),
      ];
    },),);
  return (binding.properties as readonly TreeNode[]).flatMap(function keyedName(property,): readonly string[] {
    if (property.type !== 'Property')
      return [];
    /**
     The local name the property binds, out of a default value.
     */
    const local = (isTreeNode(property.value,) && (property.value.type === 'AssignmentPattern'))
      ? property.value.left
      : property.value;
    return isStringKeyed({
      type: memberTypes.get(identifierName({ node: property.key, },),),
      aliases,
    },)
      ? [identifierName({ node: local, },),]
      : [];
  },);
}

/**
 Names of the parameters a file's functions type as keyed by any string.

 @param nodes - file's nodes

 @param aliases - names of the package's type aliases keyed by any string

 @returns Parameter names

 @example
 ```ts
 const names = keyedParameterNames({ nodes, aliases, },); // seen for (seen: Record<string, number>) => …
 ```
 */
function keyedParameterNames(
  {
    nodes,
    aliases,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly aliases: ReadonlySet<string>;
  },
): readonly string[] {
  return nodes
    .filter(function takesParameters(node,): boolean {
      return FUNCTION_KINDS.has(node.type,);
    },)
    .flatMap(function parametersOf(node,): readonly TreeNode[] {
      return node.params as readonly TreeNode[];
    },)
    .flatMap(function keyedNames(parameter,): readonly string[] {
      return keyedNamesOf({
        parameter,
        aliases,
      },);
    },);
}

/**
 Names written through a key built at run time, once per write: an
 assignment or an update whose target indexes one of the names by anything
 but a written string or number.

 @param nodes - file's nodes

 @param names - names to watch

 @returns Each name written, in no particular order

 @example
 ```ts
 const written = computedWrites({ nodes, names, },); // ['counts'] for counts[cat] = 1
 ```
 */
function computedWrites(
  {
    nodes,
    names,
  }: {
    readonly nodes: readonly TreeNode[];
    readonly names: ReadonlySet<string>;
  },
): readonly string[] {
  return nodes.flatMap(function writtenBy(node,): readonly string[] {
    /**
     What the node writes to, absent when it writes nothing.
     */
    const target = (node.type === 'AssignmentExpression')
      ? node.left
      : ((node.type === 'UpdateExpression') ? node.argument : undefined);
    if ((!isTreeNode(target,)) || (target.type !== 'MemberExpression') || (target.computed !== true))
      return [];
    if (isTreeNode(target.property,) && (target.property.type === 'Literal'))
      return [];
    /**
     The name indexed.
     */
    const name = identifierName({ node: unwrapped({ node: target.object, },).inner, },);
    return names.has(name,) ? [name,] : [];
  },);
}

/**
 The package's tables, accumulators and records keyed by text.

 @param files - files read; tests and fixtures are skipped

 @returns Each as `path: name is a text-keyed table`,
 `path: name starts a text-keyed record as {}` or
 `path: name is filled by a computed key`, sorted with repeats kept

 @example
 ```ts
 const found = textKeyedTables({ files, },);
 ```
 */
function textKeyedTables({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Each source file with its nodes and program.
   */
  const parsed = files
    .filter(function isSource({ isTest, },): boolean {
      return !isTest;
    },)
    .map(function parseOne(file,): {
      readonly path: string;
      readonly program: TreeNode;
      readonly nodes: readonly TreeNode[];
    } {
      /**
       The file's program.
       */
      const { program, } = parseSource({ file, },);
      return {
        path: file.path,
        program,
        nodes: nodesUnder({ root: program, },),
      };
    },);

  /**
   Aliases keyed by text, across every file, since a table may be typed by an
   alias another file exports.
   */
  const aliases = stringKeyedAliases({
    nodes: parsed.flatMap(function nodesOf({ nodes, },): readonly TreeNode[] {
      return nodes;
    },),
  },);

  return parsed
    .flatMap(function foundIn({
      path,
      program,
      nodes,
    },): readonly string[] {
      /**
       Start offsets of the module's top-level declarators.
       */
      const topLevel = new Set(topLevelDeclarators({ program, },)
        .map(function startOf(declarator,): number {
          return declarator.start;
        },),);

      /**
       Declarators the file types as keyed by any string.
       */
      const keyedDeclarators = nodes.filter(function isKeyedDeclarator(node,): boolean {
        return (node.type === 'VariableDeclarator') && isTypedStringKeyed({
          declarator: node,
          aliases,
        },);
      },);

      /**
       Every name the file types as keyed by any string, declared or taken as
       a parameter.
       */
      const keyedNames = new Set([
        ...keyedDeclarators.map(function nameOf(declarator,): string {
          return identifierName({ node: declarator.id, },);
        },),
        ...keyedParameterNames({
          nodes,
          aliases,
        },),
      ],);
      return [
        ...keyedDeclarators.flatMap(function reported(declarator,): readonly string[] {
          /**
           The value the wrappers hold.
           */
          const { inner, } = unwrapped({ node: declarator.init, },);
          if ((!isTreeNode(inner,)) || (inner.type !== 'ObjectExpression'))
            return [];
          /**
           The declared name.
           */
          const name = identifierName({ node: declarator.id, },);
          if (topLevel.has(declarator.start,))
            return [`${path}: ${name} is a text-keyed table`,];
          if ((inner.properties as readonly unknown[]).length === 0)
            return [`${path}: ${name} starts a text-keyed record as {}`,];
          return [];
        },),
        ...computedWrites({
          nodes,
          names: keyedNames,
        },)
          .map(function reported(name,): string {
            return `${path}: ${name} is filled by a computed key`;
          },),
      ];
    },)
    .toSorted();
}

await describe({
  name: 'text-keyed tables (ledger B77)',
  children: [
    it({
      name: 'FINDS tables typed with string keys by Record, Readonly, an index signature, an alias of either '
        + 'from any file or a cast, and accumulators started as {} anywhere, and leaves tables keyed by a '
        + 'union or only checked by satisfies, Maps, records written with fields inside a function, and tests',
      fn: async () => {
        expect(textKeyedTables({
          files: [
            {
              path: 'litter.ts',
              text: [
                'export type Naps = Record<string, number>;',
                'type Purrs = Readonly<Naps>;',
                'const MEOWS: Readonly<Record<string, string>> = { tabby: \'mrrp\', };',
                'export const NAPS: Naps = { tabby: 1, } as const;',
                'const PURRS: Purrs = {};',
                'const HISSES: { readonly [cat: string]: number } = { tom: 2, };',
                'const YOWLS = ({ tom: \'yowl\', }) as Record<string, string>;',
                'const PADS = { tom: \'mat\', } as const satisfies Record<string, string>;',
                'const COATS: Record<\'tabby\' | \'tuxedo\', string> = { tabby: \'striped\', tuxedo: \'black\', };',
                'const BEDS: ReadonlyMap<string, string> = new Map([[\'tabby\', \'box\',],],);',
                'export function tally(seen: Record<string, number>, cat: string, coats: Record<\'tabby\', number>,) {',
                '  seen[cat] += 1;',
                '  seen[\'tabby\'] = 0;',
                '  coats[cat as \'tabby\'] = 1;',
                '}',
                'export function settle(',
                '  { beds, cat, pads = {}, }: {',
                '    readonly beds: Record<string, string>; readonly cat: string; readonly pads?: Naps;',
                '  },',
                '  { bowls, }: { readonly bowls: Record<\'tabby\', string>; },',
                '  rugs: Naps = {},',
                '): void {',
                '  beds[cat] = \'box\';',
                '  pads[cat] = 1;',
                '  bowls[cat as \'tabby\'] = \'tuna\';',
                '  rugs[cat] = 2;',
                '}',
                'export function count(cats: readonly string[],): Readonly<Record<string, number>> {',
                '  const counts: Record<string, number> = {};',
                '  for (const cat of cats) counts[cat] = (counts[cat] ?? 0) + 1;',
                '  counts[String(cats.length,)]++;',
                '  const body: Record<string, unknown> = { cats, };',
                '  return { ...counts, ...body, ...COATS, ...PADS, ...Object.fromEntries(BEDS,), };',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'basket.ts',
              text: [
                'import type { Naps, } from \'./litter.ts\';',
                'export function nap(): Naps {',
                '  const naps: Naps = {};',
                '  return naps;',
                '}',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'export const NAPS: Record<string, number> = { tabby: 1, };',
              isTest: true,
            },
          ],
        },),).toEqual([
          'basket.ts: naps starts a text-keyed record as {}',
          'litter.ts: HISSES is a text-keyed table',
          'litter.ts: MEOWS is a text-keyed table',
          'litter.ts: NAPS is a text-keyed table',
          'litter.ts: PURRS is a text-keyed table',
          'litter.ts: YOWLS is a text-keyed table',
          'litter.ts: beds is filled by a computed key',
          'litter.ts: counts is filled by a computed key',
          'litter.ts: counts is filled by a computed key',
          'litter.ts: counts starts a text-keyed record as {}',
          'litter.ts: pads is filled by a computed key',
          'litter.ts: rugs is filled by a computed key',
          'litter.ts: seen is filled by a computed key',
        ],);
      },
    },),
    it({
      name: 'HOLDS NO TABLE OR ACCUMULATOR keyed by text in the package\'s source',
      fn: async () => {
        /**
         Every package file, tests among them to be skipped.
         */
        const files = await readPackageSource();
        expect(files.some(function isSource(file,): boolean {
          return !file.isTest;
        },),).toBe(true,);
        expectNoFindings({ findings: textKeyedTables({ files, },), },);
      },
    },),
  ],
},);
