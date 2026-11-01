/**
 Guards against text grown a piece at a time inside a loop (ledger B29): a
 `let` initialised with a string and extended with `+=`, or reassigned as
 itself plus more, inside a loop over the input, which rule RG2 in
 `AGENTS.md` rules out for text. `whitespaceTokensOf` and `handleReading`
 did this until they were rewritten to slice by index and join pieces once.

 WHAT THE SCAN READS. A binding counts as text where its `let` initialiser is
 a string or template literal; one declared without an initialiser, or with
 anything else, is out of reach. A name bound twice in one file is read by
 each binding's initialiser, so a string binding anywhere in the file marks
 the name. Named exemptions state why the loop does not read text.

 THE FIXTURE CASE COMES FIRST, so the package-wide case is read against a
 scan shown able to find each kind (ledger M21). Fixtures are cat-themed;
 the package case reads this package's own source.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import {
  childNodes,
  identifierName,
  isTreeNode,
  parseSource,
  type SourceText,
  type TreeNode,
} from './source-scan.test-fixture.ts';
import { expectFindingsAsListed, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 Node kinds that loop.
 */
const LOOP_KINDS: ReadonlySet<string> = new Set([
  'DoWhileStatement',
  'ForInStatement',
  'ForOfStatement',
  'ForStatement',
  'WhileStatement',
],);

/**
 Node kinds that open a function, which starts a fresh loop depth.
 */
const FUNCTION_KINDS: ReadonlySet<string> = new Set(['ArrowFunctionExpression', 'FunctionDeclaration', 'FunctionExpression',],);

/**
 Accumulators that do not read text, each as `path#site: name`, with why.
 */
const NOT_TEXT: Readonly<Record<string, string>> = {
  'page-pass/corpus-run/ordinal-style.ts#roman: out':
    'builds a roman numeral from a small number over a fixed table; the loop reads the table, not text',
};

/**
 Names a program binds with `let` to a string or template literal.

 @param program - parsed program

 @returns Names bound to text

 @example
 ```ts
 const textNames = textLets({ program, },);
 ```
 */
function textLets({ program, }: { readonly program: TreeNode; },): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const names = new Set<string>();
  /**
   Nodes still to visit.
   */
  const pending: TreeNode[] = [program,];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const node = pending.pop() as TreeNode;
    pending.push(...childNodes({ node, },),);
    if ((node.type !== 'VariableDeclaration') || (node.kind !== 'let'))
      continue;
    for (const declarator of node.declarations as readonly TreeNode[]) {
      /**
       What the binding starts as.
       */
      const { init, } = declarator;
      /**
       Whether that is text.
       */
      const startsAsText = isTreeNode(init,)
        && (((init.type === 'Literal') && ((typeof init.value) === 'string')) || (init.type === 'TemplateLiteral'));
      if (startsAsText && isTreeNode(declarator.id,))
        names.add(identifierName({ node: declarator.id, },),);
    }
  }
  return names;
}

/**
 Whether an assignment grows its target from itself: `x += y`, or `x = x + y`,
 or `` x = `${x}...` ``.

 @param node - assignment expression

 @param name - its target's name

 @returns Whether it extends the target

 @example
 ```ts
 const grows = growsItself({ node, name: 'held', },);
 ```
 */
function growsItself(
  {
    node,
    name,
  }: {
    readonly node: TreeNode;
    readonly name: string;
  },
): boolean {
  if (node.operator === '+=')
    return true;
  if ((node.operator !== '=') || (!isTreeNode(node.right,)))
    return false;
  /**
   What the target is reassigned to.
   */
  const { right, } = node;
  if ((right.type === 'BinaryExpression') && (right.operator === '+') && isTreeNode(right.left,))
    return identifierName({ node: right.left, },) === name;
  if (right.type === 'TemplateLiteral') {
    /**
     The template's first interpolation, when it has one.
     */
    const [first,] = (right.expressions as readonly TreeNode[] | readonly never[]);
    return (first !== undefined) && (identifierName({ node: first, },) === name);
  }
  return false;
}

/**
 Text grown inside a loop in the package's source, as `path#site: name`.

 @param files - files read; tests and fixtures are skipped

 @returns Findings, sorted and without repeats

 @example
 ```ts
 const grown = textGrownInLoops({ files, },);
 ```
 */
function textGrownInLoops({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
  /**
   Findings so far.
   */
  const found = new Set<string>();
  for (const file of files) {
    if (file.isTest)
      continue;
    /**
     The parsed file.
     */
    const { program, } = parseSource({ file, },);
    /**
     Names the file binds to text.
     */
    const textNames = textLets({ program, },);
    /**
     Nodes still to visit, each with its enclosing function and loop depth.
     */
    const pending: { readonly node: TreeNode; readonly site: string; readonly loops: number; }[] = [{
      node: program,
      site: '<module>',
      loops: 0,
    },];
    while (pending.length > 0) {
      /**
       Node visited now.
       */
      const { node, site, loops, } = pending.pop() as (typeof pending)[number];
      /**
       Whether the node opens a function.
       */
      const opensFunction = FUNCTION_KINDS.has(node.type,);
      /**
       Enclosing function's name for the node's children.
       */
      const here = (opensFunction && isTreeNode(node.id,)) ? identifierName({ node: node.id, },) : site;
      /**
       Loop depth for the node's children.
       */
      const depth = opensFunction ? 0 : (loops + (LOOP_KINDS.has(node.type,) ? 1 : 0));
      pending.push(...childNodes({ node, },).map(function withContext(child,) {
        return {
          node: child,
          site: here,
          loops: depth,
        };
      },),);
      if ((node.type !== 'AssignmentExpression') || (loops === 0) || (!isTreeNode(node.left,)))
        continue;
      /**
       The assignment's target.
       */
      const name = identifierName({ node: node.left, },);
      if (textNames.has(name,) && growsItself({ node, name, },))
        found.add(`${file.path}#${site}: ${name}`,);
    }
  }
  return [...found,].toSorted();
}

await describe({
  name: 'text accumulators (ledger B29)',
  children: [
    it({
      name: 'FINDS text grown by +=, by x = x + y and by a template opening on itself inside a loop, and leaves a '
        + 'counter, text built outside a loop, text joined from pieces, and tests',
      fn: async () => {
        expect(textGrownInLoops({
          files: [
            {
              path: 'cat.ts',
              text: [
                'export function purr(text: string): string { let held = \'\'; for (const c of text) held += c; return held; }',
                'export function knead(text: string): string { let out = \'\'; for (const c of text) { out = out + c; } return out; }',
                `export function groom(text: string): string { let fur = \`\`; let at = 0; while (at < text.length) { fur = \`\${fur}\${text[at]}\`; at += 1; } return fur; }`,
                'export function count(text: string): number { let seen = 0; for (const c of text) seen += c.length; return seen; }',
                'export function nap(): string { let rest = \'z\'; rest += \'z\'; return rest; }',
                'export function stretch(text: string): string { const pieces: string[] = []; for (const c of text) pieces.push(c); return pieces.join(\'\'); }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'let sheet = \'\'; for (const c of \'meow\') sheet += c;',
              isTest: true,
            },
          ],
        },),).toEqual([
          'cat.ts#groom: fur',
          'cat.ts#knead: out',
          'cat.ts#purr: held',
        ],);
      },
    },),
    it({
      name: 'FINDS NO TEXT GROWN IN A LOOP across the package but the accumulators named as reading no text',
      fn: async () => {
        expectFindingsAsListed({
          findings: textGrownInLoops({ files: await readPackageSource(), },),
          listed: Object.keys(NOT_TEXT,).toSorted(),
        },);
      },
    },),
  ],
},);
