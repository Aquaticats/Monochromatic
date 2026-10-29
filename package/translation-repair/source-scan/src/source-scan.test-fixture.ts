// Rolldown's parser (oxc), as the repository's import-attributes plugin reads
// source; TypeScript 7 exposes no stable compiler API to parse with.
import { parseSync, } from 'rolldown/utils';

//region Source scan
// What the package-wide guards over the translation-repair family's source share (audit
// area six, ledger B19): the family's TypeScript files as text, each parsed
// once, a walk over the syntax tree, and the reading of names and wrapped
// expressions the scans share. Every scan that parses the source parses it
// here (scans that read the text alone, such as `log-root-scan.unit.test.ts`,
// list the files themselves); the walk and name helpers came here with ledger
// B77, and the scans written before it still carry copies of their own.
//
// The files themselves come from `family-source.test-fixture.ts`, which reads every package of the
// translation-repair family and names each file by its package directory and its path under `src`.

/**
 One TypeScript file under a family package's `src`.
 */
export type SourceText = {
  /**
   Package directory and path relative to that package's `src`, joined by a slash, which names locations.
   */
  readonly path: string;

  /**
   File text.
   */
  readonly text: string;

  /**
   Whether it is a test or a test fixture rather than package source.
   */
  readonly isTest: boolean;
};

/**
 An ESTree node as the scans read it: its kind, its offsets and its fields.
 */
export type TreeNode = Readonly<Record<string, unknown>> & {
  readonly type: string;
  readonly start: number;
  readonly end: number;
};

/**
 A parsed file: its program and its comments, in order.
 */
type ParsedSource = {
  /**
   Program node.
   */
  readonly program: TreeNode;

  /**
   Comments with their offsets.
   */
  readonly comments: readonly {
    readonly start: number;
    readonly end: number
  }[];
};

/**
 Whether a value is an ESTree node with offsets.

 @param value - field of a node

 @returns Whether it is a node

 @example
 ```ts
 const node = isTreeNode(program.body[0],);
 ```
 */
export function isTreeNode(value: unknown,): value is TreeNode {
  return ((typeof value) === 'object') && (value !== null)
    && ('type' in value)
    && ((typeof value.type) === 'string')
    && ('start' in value)
    && ((typeof value.start) === 'number')
    && ('end' in value)
    && ((typeof value.end) === 'number');
}

/**
 Every node held in a node's fields, lists flattened.

 @param node - node read

 @returns Its child nodes

 @example
 ```ts
 const children = childNodes({ node: program, },);
 ```
 */
export function childNodes({ node, }: { readonly node: TreeNode; },): readonly TreeNode[] {
  return Object.values(node,)
    .flatMap(function children(value,): readonly unknown[] {
      return Array.isArray(value,) ? value : [value,];
    },)
    .filter(isTreeNode,);
}

/**
 Every node under a root, the root included, walked with a stack.

 @param root - node to walk from

 @returns Nodes in no particular order

 @example
 ```ts
 const nodes = nodesUnder({ root: program, },);
 ```
 */
export function nodesUnder({ root, }: { readonly root: TreeNode; },): readonly TreeNode[] {
  /**
   Nodes visited.
   */
  const visited: TreeNode[] = [];

  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [root,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    visited.push(node,);
    pending.push(...childNodes({ node, },),);
  }
  return visited;
}

/**
 Every node of a parsed file with the node that holds it.

 @param program - file's program

 @returns Each node's parent, the program having none

 @example
 ```ts
 const parents = parentsOf({ program, },);
 ```
 */
export function parentsOf({ program, }: { readonly program: TreeNode; },): ReadonlyMap<TreeNode, TreeNode> {
  /**
   Parents found so far.
   */
  const parents = new Map<TreeNode, TreeNode>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  for (let node = pending.pop(); node !== undefined; node = pending.pop()) {
    for (const child of childNodes({ node, },)) {
      parents.set(
        child,
        node,
      );
      pending.push(child,);
    }
  }
  return parents;
}

/**
 The nodes holding a node, nearest first.

 @param node - node read

 @param parents - each node's parent

 @returns Its ancestors up to the program

 @example
 ```ts
 const chain = ancestorsOf({ node, parents, },);
 ```
 */
export function ancestorsOf(
  {
    node,
    parents,
  }: {
    readonly node: TreeNode;
    readonly parents: ReadonlyMap<TreeNode, TreeNode>;
  },
): readonly TreeNode[] {
  /**
   Ancestors found so far.
   */
  const chain: TreeNode[] = [];
  for (let at = parents.get(node,); at !== undefined; at = parents.get(at,))
    chain.push(at,);
  return chain;
}

/**
 Name an identifier node carries.

 @param node - node read, of any kind or none

 @returns Its name, empty for any other node

 @example
 ```ts
 const name = identifierName({ node: call.callee, },);
 ```
 */
export function identifierName({ node, }: { readonly node: unknown; },): string {
  if ((!isTreeNode(node,)) || (node.type !== 'Identifier'))
    return '';
  /**
   The node's name field.
   */
  const { name, } = node;
  return ((typeof name) === 'string') ? name : '';
}

/**
 Name a member expression reads, written plainly or computed from a string.

 @param node - member expression read

 @returns Member's name, empty where it is computed from anything but a
 string literal

 @example
 ```ts
 const member = memberName({ node: call.callee, },); // 'now' for Date['now']
 ```
 */
export function memberName({ node, }: { readonly node: TreeNode; },): string {
  if (node.computed !== true)
    return identifierName({ node: node.property, },);
  /**
   The computed key.
   */
  const { property, } = node;
  if ((!isTreeNode(property,)) || (property.type !== 'Literal'))
    return '';
  /**
   The literal's value.
   */
  const { value, } = property;
  return ((typeof value) === 'string') ? value as string : '';
}

/**
 Text a string literal node carries.

 @param node - node read

 @returns Its value, empty for any other node

 @example
 ```ts
 const from = literalText({ node: declaration.source, },);
 ```
 */
export function literalText({ node, }: { readonly node: unknown; },): string {
  if ((!isTreeNode(node,)) || (node.type !== 'Literal'))
    return '';
  /**
   The literal's value, of whatever kind.
   */
  const { value, } = node;
  return ((typeof value) === 'string') ? value : '';
}

/**
 Node kinds that wrap one expression without changing the value it holds.
 */
const WRAPPER_KINDS: ReadonlySet<string> = new Set([
  'ChainExpression',
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
  'TSTypeAssertion',
],);

/**
 The wrappers around an expression, outermost first, and what they hold.

 @param node - expression read

 @returns Each wrapper in turn, and the innermost expression, the node itself
 when it is no wrapper

 @example
 ```ts
 const { wrappers, inner, } = unwrapped({ node: declarator.init, },); // inner is the object of ({ tabby: 1 }) as const
 ```
 */
export function unwrapped({ node, }: { readonly node: unknown; },): {
  readonly wrappers: readonly TreeNode[];
  readonly inner: unknown;
} {
  /**
   Wrappers passed so far.
   */
  const wrappers: TreeNode[] = [];
  for (let inner: unknown = node; ;) {
    if ((!isTreeNode(inner,)) || (!WRAPPER_KINDS.has(inner.type,))) {
      return {
        wrappers,
        inner,
      };
    }
    wrappers.push(inner,);
    inner = inner.expression;
  }
}

/**
 Parses one file.

 @param file - file read

 @returns Its program and comments

 @throws {@link Error} when the file does not parse, naming the file and the `message` of each error the parser reports,
 since a file a scan cannot read hides what it holds

 @example
 ```ts
 const { program, comments, } = parseSource({ file, },);
 ```
 */
export function parseSource({ file, }: { readonly file: SourceText; },): ParsedSource {
  /**
   Parser result.
   */
  const parsed = parseSync(
    file.path,
    file.text,
  );
  if (parsed.errors
    .length
    > 0)
    throw new Error(`${file.path} does not parse: ${parsed.errors
      .map(function messageOf({ message, }: { readonly message: string; },): string {
        return message;
      },)
      .join('; ',)}`,);
  /**
   Program node, checked to carry the offsets the scans read.
   */
  const program: unknown = parsed.program;
  if (!isTreeNode(program,))
    throw new Error(`${file.path} parsed to a program without offsets`,);
  return {
    program,
    comments: parsed.comments,
  };
}

//endregion Source scan
