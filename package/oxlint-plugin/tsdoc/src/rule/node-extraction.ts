/**
 AST node name and kind extraction utilities for TSDoc diagnostics.
 
 Extracted from `require-tsdoc.ts` to keep files under 100 countable lines.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  Span,
} from '@oxlint/plugins';

import {
  isRecord,
  isRecordArray,
  type ReadonlyRecord,
} from '../ast-access.ts';
import {
  findTsdocComment,
  NO_TSDOC,
} from '../tsdoc-utils.ts';

/**
 Human-readable labels for AST node types that require TSDoc.
 */
export const NODE_KIND_LABELS: Readonly<Record<string, string>> = {
  FunctionDeclaration: 'function',
  ClassDeclaration: 'class',
  MethodDefinition: 'method',
  TSInterfaceDeclaration: 'interface',
  TSTypeAliasDeclaration: 'type alias',
  TSEnumDeclaration: 'enum',
  TSEnumMember: 'enum member',
  VariableDeclaration: 'variable',
  PropertyDefinition: 'property',
  Property: 'accessor',
};

/**
 Absence marker for {@link readNamedChild} meaning "child is not an
 identifier-shaped record carrying a string name"; never a real name.
 
 Local to this module because `readNamedChild` is internal and its absence
 never crosses a file boundary.
 
 @example
 ```ts
 const name = readNamedChild({ node, key: 'id', });
 if (name === NO_NAMED_CHILD)
   return 'anonymous';
 ```
 */
const NO_NAMED_CHILD: unique symbol = Symbol('tsdoc/no-named-child',);

/**
 Parameters for {@link readNamedChild}.
 */
type ReadNamedChildParams = {
  /**
   Parent AST node carrying the child under `key`.
   */
  readonly node: ReadonlyRecord;
  /**
   Property name of the child identifier (`id`, `key`, ...).
   */
  readonly key: string;
};

/**
 Reads the `.name` string of an identifier-shaped child node.
 
 @returns child's `name` string, or {@link NO_NAMED_CHILD} when the child is not
 a record with a string `name`
 
 @example
 ```ts
 const name = readNamedChild({ node, key: 'id' });
 ```
 */
function readNamedChild({
  node,
  key,
}: ReadNamedChildParams,): string | typeof NO_NAMED_CHILD {
  /**
   Child node under `key`; only an identifier-shaped record yields a name.
   */
  const child = node[key];
  if (!isRecord(child,))
    return NO_NAMED_CHILD;
  /**
   Identifier text of the child; present only on identifier nodes.
   */
  const { name, } = child;
  return (typeof name)
    === 'string' ? name : NO_NAMED_CHILD;
}

/**
 Extracts a human-readable name from an AST node for diagnostic messages,
 reading nested identifiers via {@link readNamedChild}.
 
 @param node - AST node to extract name from
 
 @returns declaration name, or "anonymous" when no name is available
 
 @example
 ```ts
 const name = extractNodeName(functionNode); // e.g. 'myFunction'
 ```
 */
export function extractNodeName(node: ForeignBorrowed<Span>,): string {
  /**
   Narrowed view that exposes the untyped properties added by the host AST.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped
  const typed = node as Span & Record<string, unknown>;

  // VariableDeclaration: dig into declarators[0].id.name
  if (typed.type
    === 'VariableDeclaration') {
    /**
     Declarator list of the variable statement; first declarator carries the canonical name.
     */
    const { declarations, } = typed;
    if (!isRecordArray(declarations,))
      return 'anonymous';
    /**
     First declarator; its `id` identifier holds the variable name.
     */
    const [first,] = declarations;
    if (!isRecord(first,))
      return 'anonymous';
    /**
     Variable name read from the first declarator's `id`.
     */
    const name = readNamedChild({
      node: first,
      key: 'id',
    },);
    return name === NO_NAMED_CHILD ? 'anonymous' : name;
  }

  // MethodDefinition / PropertyDefinition / Property: key.name
  if ((typed.type
    === 'MethodDefinition')
    || (typed.type
      === 'PropertyDefinition')
    || (typed.type
      === 'Property'))
    {
    /**
     Member name read from the node's `key` (`foo` in `class C { foo() {} }`).
     */
    const name = readNamedChild({
      node: typed,
      key: 'key',
    },);
    return name === NO_NAMED_CHILD ? 'anonymous' : name;
  }

  // Most declarations: .id.name
  /**
   Declaration name read from `id`; present on functions, classes, type aliases, etc.
   */
  const idName = readNamedChild({
    node: typed,
    key: 'id',
  },);
  if (idName !== NO_NAMED_CHILD)
    return idName;

  // FunctionDeclaration without name, TSEnumMember with direct name
  /**
   Direct `.name` on the node, used for enum members and unnamed functions.
   */
  const { name, } = typed;
  if ((typeof name)
    === 'string')
    return name;

  return 'anonymous';
}

/**
 Resolves a human-readable kind label for an AST node type, looked up in
 {@link NODE_KIND_LABELS}.
 
 @param node - AST node to get kind for
 
 @returns kind label like "function", "class", "variable"
 
 @example
 ```ts
 extractNodeKind(functionNode) // → 'function'
 ```
 */
export function extractNodeKind(node: ForeignBorrowed<Span>,): string {
  /* oxlint-disable typescript/no-unsafe-type-assertion -- oxlint plugin API is untyped. */
  /**
   AST type tag (e.g. `FunctionDeclaration`); the key into {@link NODE_KIND_LABELS}.
   */
  const nodeType = (node as Span & Record<string, unknown>).type;
  /* oxlint-enable typescript/no-unsafe-type-assertion */
  if ((typeof nodeType)
    !== 'string')
    return 'declaration';
  return NODE_KIND_LABELS[nodeType]
    ?? 'declaration';
}

/**
 Parameters for {@link reportMissing}.
 */
export type ReportMissingParams = {
  /**
   AST node that should have TSDoc.
   */
  readonly node: Span;
  /**
   Oxlint rule context.
   */
  readonly context: Context;
};

/**
 Reports a diagnostic when {@link findTsdocComment} returns {@link NO_TSDOC}
 for node, filling the message via {@link extractNodeKind} and
 {@link extractNodeName}.
 
 @example
 ```ts
 reportMissing({ node, context });
 ```
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
export function reportMissing({
  node,
  context,
}: ForeignBorrowed<ReportMissingParams>,): void {
  if (findTsdocComment({
    node,
    context,
  },)
    === NO_TSDOC) {
    context.report({
      node,
      messageId: 'missing',
      data: {
        kind: extractNodeKind(node,),
        name: extractNodeName(node,),
      },
    },);
  }
}
