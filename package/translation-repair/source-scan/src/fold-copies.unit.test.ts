/**
 Guards against a fold that copies what it has built at every step: a
 `reduce` whose callback returns its accumulator spread into a new list, or
 cut and rejoined, and a loop that reassigns a list or text as a copy of
 itself. Each step then costs as much as everything before it, so the fold
 costs the square of its input where one pass would do (rule ITR in
 `AGENTS.md`). `scanNearest` in `pair-sections-steps.ts` did this until the
 sixteenth T8 batch rewrote it as a loop, and nothing read the shape
 (ledger B70).

 WHAT THE SCAN READS. Inside a `reduce` or `reduceRight` callback, the names
 its first parameter binds, destructured or not, and any path rooted at one:
 a spread of one inside a list, a copying method called on one (`concat`,
 `slice`, `toSpliced`, `toSorted`, `toReversed`, `with`, `flat`), a `Map` or
 `Set` built from one, and, where the fold starts from text, `+` with one.
 Inside a loop, an assignment whose value copies its own target the same
 ways, and a `set` on a map whose value copies what the map's `get` read,
 directly or through a name bound to that read (`readCapLog` queued each
 label's streams so until B73's batch). A path through `??` or `||` is read
 from its left side. A spread inside a record is not read: a record of fixed
 fields costs the same at every step. Named exemptions state why a fold's
 copies are bounded or are its meaning.

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
 Methods that return a copy of the list or text they are called on.
 */
const COPYING_METHODS: ReadonlySet<string> = new Set([
  'concat',
  'flat',
  'slice',
  'toReversed',
  'toSorted',
  'toSpliced',
  'with',
],);

/**
 Constructors that copy the collection they are handed.
 */
const COPYING_CONSTRUCTORS: ReadonlySet<string> = new Set(['Map', 'Set',],);

/**
 Folds whose copies are bounded or are their meaning, each as
 `path#site: names`, with why.
 */
const BOUNDED_OR_SEQUENTIAL: Readonly<Record<string, string>> = {
  'assembly/assembly-repetition-span.ts#grownSpans: covered':
    'each run is checked against the regions the runs before it left, merged, so the regions are rebuilt between checks',
  'page-check/corpus-run/archive-block-repair.ts#repairArchiveBlocks: revisedText':
    'each removal span reads the separators the later blocks\' revisions left, so the text is rewritten block by block',
  'page-check/corpus-run/archive-stub.ts#isStubMarkerParagraph/unwrap: text':
    'folds over the fixed MARKER_WRAPS table, not over the input',
  'page-check/corpus-run/name-gloss-restore.ts#restore/place: current':
    'each gloss is placed by a search of the text the glosses before it left',
  'markup/invisible-variants.ts#foldInvisibleVariants/fold: folded':
    'folds over the fixed FOLDS table, not over the input',
  'lane-contest/seeded-error.ts#applySeededErrors/applyOne: state':
    'each needle is found in the text the seeds before it left, and their regions shift with it',
  'page-pass/splice-slices.ts#placeSlices/spliceOne: current':
    'each edit composes its separators against the tail the later edits wrote',
  'stage/stage-quorum.ts#collectRounds: pending':
    'the next round\'s queue, rebuilt once per round, and the rounds are bounded by maxRetryRounds',
};

/**
 Names a binding pattern binds, destructured or not.

 @param pattern - parameter or declaration target

 @returns Every name it binds

 @example
 ```ts
 const names = boundNames({ pattern: callback.params[0], },);
 ```
 */
function boundNames({ pattern, }: { readonly pattern: unknown; },): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const names = new Set<string>();
  /**
   Pattern nodes still to read.
   */
  const pending: unknown[] = [pattern,];
  while (pending.length > 0) {
    /**
     Pattern node read now.
     */
    const node = pending.pop();
    if (!isTreeNode(node,))
      continue;
    if (node.type === 'Identifier')
      names.add(identifierName({ node, },),);
    else if (node.type === 'ObjectPattern') {
      for (const property of node.properties as readonly TreeNode[])
        pending.push((property.type === 'RestElement') ? property.argument : property.value,);
    }
    else if (node.type === 'ArrayPattern')
      pending.push(...(node.elements as readonly unknown[]),);
    else if (node.type === 'AssignmentPattern')
      pending.push(node.left,);
    else if (node.type === 'RestElement')
      pending.push(node.argument,);
  }
  return names;
}

/**
 Operators whose left side is what a path names when it is there.
 */
const FALLBACK_OPERATORS: ReadonlySet<string> = new Set([
  '??',
  '||',
],);

/**
 Node kinds that wrap one expression without changing what it names.
 */
const WRAPPER_KINDS: ReadonlySet<string> = new Set([
  'ChainExpression',
  'ParenthesizedExpression',
  'TSAsExpression',
  'TSNonNullExpression',
  'TSSatisfiesExpression',
],);

/**
 The expression one step nearer a path's root.

 @param node - member access or wrapper

 @returns What it reads from or wraps, nothing for any other node

 @example
 ```ts
 const inner = innerOf({ node: member, },); // state for state.kept
 ```
 */
function innerOf({ node, }: { readonly node: TreeNode; },): unknown {
  if (node.type === 'MemberExpression')
    return node.object;
  if (node.type === 'LogicalExpression')
    return FALLBACK_OPERATORS.has(node.operator as string,) ? node.left : undefined;
  return WRAPPER_KINDS.has(node.type,) ? node.expression : undefined;
}

/**
 The map a call reads an entry from.

 @param node - call expression

 @returns Path `get` is called on, nothing for any other call

 @example
 ```ts
 const map = entryMapOf({ node: call, },); // waiting for waiting.get(label)
 ```
 */
function entryMapOf({ node, }: { readonly node: TreeNode; },): unknown {
  if ((!isTreeNode(node.callee,)) || (node.callee.type !== 'MemberExpression') || (node.callee.computed === true)
    || (identifierName({ node: node.callee.property, },) !== 'get'))
    return undefined;
  return node.callee.object;
}

/**
 The name a member path, a chain or a cast is rooted at.

 @param node - expression read

 @returns Root identifier's name, empty when the path starts elsewhere

 @example
 ```ts
 const root = rootName({ node: spread.argument, },); // 'state' for state.kept
 ```
 */
function rootName({ node, }: { readonly node: unknown; },): string {
  for (let current = node; isTreeNode(current,); current = innerOf({ node: current, },)) {
    if (current.type === 'Identifier')
      return identifierName({ node: current, },);
    if (current.type === 'CallExpression') {
      /**
       The map whose entry the call reads, when it reads one, named by its
       own root.
       */
      const map = entryMapOf({ node: current, },);
      /**
       The name that map's path is rooted at.
       */
      const mapName = isTreeNode(map,) ? rootName({ node: map, },) : '';
      return (mapName === '') ? '' : `${mapName}.get`;
    }
  }
  return '';
}

/**
 Whether an expression starts as text: a string or a template literal.

 @param node - expression read

 @returns Whether it is text

 @example
 ```ts
 const textFold = startsAsText({ node: reduceCall.arguments[1], },);
 ```
 */
function startsAsText({ node, }: { readonly node: unknown; },): boolean {
  return isTreeNode(node,)
    && (((node.type === 'Literal') && ((typeof node.value) === 'string')) || (node.type === 'TemplateLiteral'));
}

/**
 Whether a subtree copies any of the given names.

 @param root - subtree read

 @param names - names whose copies count

 @param textFold - whether `+` with one of the names counts, as in a fold that starts from text

 @returns Whether a copy appears

 @example
 ```ts
 const copies = copiesAny({ root: callback.body, names, textFold: false, },);
 ```
 */
function copiesAny(
  {
    root,
    names,
    textFold,
  }: {
    readonly root: TreeNode;
    readonly names: ReadonlySet<string>;
    readonly textFold: boolean;
  },
): boolean {
  /**
   Nodes still to visit, each with the list it sits in when it is a spread.
   */
  const pending: { readonly node: TreeNode; readonly inList: boolean; }[] = [{
    node: root,
    inList: false,
  },];
  while (pending.length > 0) {
    /**
     Node visited now.
     */
    const { node, inList, } = pending.pop() as (typeof pending)[number];
    pending.push(...childNodes({ node, },).map(function withParent(child,) {
      return {
        node: child,
        inList: node.type === 'ArrayExpression',
      };
    },),);
    if ((node.type === 'SpreadElement') && inList && names.has(rootName({ node: node.argument, },),))
      return true;
    if ((node.type === 'CallExpression') && isTreeNode(node.callee,) && (node.callee.type === 'MemberExpression')
      && (node.callee.computed !== true)
      && COPYING_METHODS.has(identifierName({ node: node.callee.property, },),)
      && names.has(rootName({ node: node.callee.object, },),))
      return true;
    if ((node.type === 'NewExpression') && COPYING_CONSTRUCTORS.has(identifierName({ node: node.callee, },),)
      && (node.arguments as readonly TreeNode[]).some(argument => names.has(rootName({ node: argument, },),)))
      return true;
    if (textFold && (node.type === 'BinaryExpression') && (node.operator === '+')
      && (names.has(rootName({ node: node.left, },),) || names.has(rootName({ node: node.right, },),)))
      return true;
  }
  return false;
}

/**
 The callback of a `reduce` or `reduceRight` call, and whether the fold
 starts from text.

 @param node - call expression

 @returns Callback and its start, or nothing for any other call

 @example
 ```ts
 const fold = foldOf({ node, },);
 ```
 */
function foldOf({ node, }: { readonly node: TreeNode; },): readonly { readonly callback: TreeNode; readonly textFold: boolean; }[] {
  if ((node.type !== 'CallExpression') || (!isTreeNode(node.callee,)) || (node.callee.type !== 'MemberExpression'))
    return [];
  if (!['reduce', 'reduceRight',].includes(identifierName({ node: node.callee.property, },),))
    return [];
  /**
   The call's callback and its start.
   */
  const [callback, start,] = node.arguments as readonly unknown[];
  if ((!isTreeNode(callback,)) || (!FUNCTION_KINDS.has(callback.type,)))
    return [];
  return [{
    callback,
    textFold: startsAsText({ node: start, },),
  },];
}

/**
 Names a program binds to an entry read from a map, each with that map's
 name.

 @param program - parsed program

 @returns Map each such name was read from

 @example
 ```ts
 const entryNames = entryBindings({ program, },); // queue → waiting for const queue = waiting.get(model) ?? [];
 ```
 */
function entryBindings({ program, }: { readonly program: TreeNode; },): ReadonlyMap<string, string> {
  /**
   Bindings found so far.
   */
  const bound = new Map<string, string>();
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
    if (node.type !== 'VariableDeclarator')
      continue;
    /**
     What the binding reads, as `map.get` when it reads a map's entry.
     */
    const read = rootName({ node: node.init, },);
    /**
     The name bound.
     */
    const name = identifierName({ node: node.id, },);
    if (read.endsWith('.get',) && (name !== ''))
      bound.set(name, read.slice(0, -'.get'.length,),);
  }
  return bound;
}

/**
 The map a `set` call writes a copy of its own entry into.

 @param node - call expression

 @param entryNames - names bound to an entry read from a map, each with that map's name

 @returns Map's name, empty when the call sets no such copy

 @example
 ```ts
 const map = entryCopied({ node, entryNames, },); // 'waiting' for waiting.set(label, [...(waiting.get(label) ?? []), line])
 ```
 */
function entryCopied(
  {
    node,
    entryNames,
  }: {
    readonly node: TreeNode;
    readonly entryNames: ReadonlyMap<string, string>;
  },
): string {
  if ((node.type !== 'CallExpression') || (!isTreeNode(node.callee,)) || (node.callee.type !== 'MemberExpression')
    || (node.callee.computed === true) || (identifierName({ node: node.callee.property, },) !== 'set'))
    return '';
  /**
   The map written, by the name its path is rooted at.
   */
  const map = rootName({ node: node.callee.object, },);
  /**
   The value written.
   */
  const [, value,] = node.arguments as readonly unknown[];
  if ((map === '') || (!isTreeNode(value,)))
    return '';
  /**
   Names standing for the entry read from that map: its `get`, and every name bound to one.
   */
  const names = new Set([
    `${map}.get`,
    ...[...entryNames,]
      .filter(function readsMap([, from,],): boolean {
        return from === map;
      },)
      .map(function nameOf([name,],): string {
        return name;
      },),
  ],);
  return copiesAny({ root: value, names, textFold: false, },) ? map : '';
}

/**
 Folds and loops in the package's source that copy what they build, as
 `path#site: names`.

 @param files - files read; tests and fixtures are skipped

 @returns Findings, sorted and without repeats

 @example
 ```ts
 const copied = foldCopies({ files, },);
 ```
 */
function foldCopies({ files, }: { readonly files: readonly SourceText[]; },): readonly string[] {
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
     Names the file binds to an entry read from a map.
     */
    const entryNames = entryBindings({ program, },);
    /**
     Nodes still to visit, each with its enclosing named function and loop depth.
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
       The function's own name, empty when it has none.
       */
      const ownName = opensFunction ? identifierName({ node: node.id, },) : '';
      /**
       Enclosing named function for the node's children.
       */
      const here = (ownName === '') ? site : ownName;
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
      for (const { callback, textFold, } of foldOf({ node, },)) {
        /**
         Names the callback's accumulator binds.
         */
        const names = boundNames({ pattern: (callback.params as readonly unknown[])[0], },);
        if (isTreeNode(callback.body,) && copiesAny({ root: callback.body, names, textFold, },)) {
          /**
           The callback's own name, or a stand-in for an anonymous one.
           */
          const callbackName = identifierName({ node: callback.id, },) || '<anonymous>';
          found.add(`${file.path}#${site}/${callbackName}: ${[...names,].join(', ',)}`,);
        }
      }
      /**
       The map whose entry the node sets to a copy of itself inside a loop,
       empty for any other node.
       */
      const copiedMap = (loops === 0) ? '' : entryCopied({ node, entryNames, },);
      if (copiedMap !== '')
        found.add(`${file.path}#${site}: ${copiedMap} entry`,);
      if ((node.type !== 'AssignmentExpression') || (node.operator !== '=') || (loops === 0))
        continue;
      /**
       The assignment's target.
       */
      const name = identifierName({ node: node.left, },);
      if ((name !== '') && isTreeNode(node.right,)
        && copiesAny({ root: node.right, names: new Set([name,],), textFold: false, },))
        found.add(`${file.path}#${site}: ${name}`,);
    }
  }
  return [...found,].toSorted();
}

await describe({
  name: 'fold copies (ledger B70)',
  children: [
    it({
      name: 'FINDS a fold copying its list by spread, through a field, through a destructured name, by concat, '
        + 'by slice, by toSpliced and into a Set, text grown by + from a text start, a loop reassigning a '
        + 'copy of itself, and a loop setting a map entry to a copy of what it read there, directly or through '
        + 'a name, and leaves a sum, an appending fold, a record fold, an entry appended in place, a copy '
        + 'outside any loop, and tests',
      fn: async () => {
        expect(foldCopies({
          files: [
            {
              path: 'cat.ts',
              text: [
                'const cats = [\'tabby\', \'calico\',];',
                'export function line() { return cats.reduce(function queue(acc: string[], cat) { return [...acc, cat]; }, []); }',
                'export function nearest() { return cats.reduce(function scan(state: { found: string[] }, cat) { return { found: [...state.found, cat] }; }, { found: [] }); }',
                'export function seen() { return cats.reduce(function note({ names }: { names: string[] }, cat) { return { names: [...names, cat] }; }, { names: [] }); }',
                'export function joined() { return cats.reduce(function join(acc: string[], cat) { return acc.concat([cat]); }, []); }',
                'export function spliced(text: string) { return cats.reduce(function cut(current, cat) { return current.slice(0, 1) + cat + current.slice(1); }, text); }',
                'export function dropped() { return cats.reduce(function drop(state: { left: string[] }, cat) { return { left: state.left.toSpliced(0, 1) }; }, { left: cats }); }',
                'export function distinct() { return cats.reduce(function keep(acc: Set<string>, cat) { const next = new Set(acc); next.add(cat); return next; }, new Set<string>()); }',
                'export function named() { return cats.reduce(function purr(acc, cat) { return acc + cat; }, \'\'); }',
                'export function rebuilt() { let pile: string[] = []; for (const cat of cats) pile = [...pile, cat]; return pile; }',
                'export function trimmed(url: string) { let run = url; while (run.endsWith(\'.\')) run = run.slice(0, -1); return run; }',
                'export function sum() { return cats.reduce(function add(acc, cat) { return acc + cat.length; }, 0); }',
                'export function pushed() { return cats.reduce(function push(acc: string[], cat) { acc.push(cat); return acc; }, []); }',
                'export function tally() { return cats.reduce(function count(acc: { n: number }, cat) { return { ...acc, n: acc.n + cat.length }; }, { n: 0 }); }',
                'export function once() { let pile: string[] = []; pile = [...pile, \'tabby\']; return pile; }',
                'export function grouped() { const byCat = new Map<string, string[]>(); for (const cat of cats) byCat.set(cat[0], [...(byCat.get(cat[0]) ?? []), cat]); return byCat; }',
                'export function shed() { const left = new Map<string, string[]>(); for (const cat of cats) { const pile = left.get(cat) ?? []; left.set(cat, pile.toSpliced(0, 1)); } return left; }',
                'export function appended() { const byCat = new Map<string, string[]>(); for (const cat of cats) { const bed = byCat.get(cat[0]); if (bed === undefined) byCat.set(cat[0], [cat]); else bed.push(cat); } return byCat; }',
                'export function seeded() { const box = new Map<string, string[]>(); box.set(\'a\', [...(box.get(\'a\') ?? []), \'tabby\']); return box; }',
              ].join('\n',),
              isTest: false,
            },
            {
              path: 'cat.unit.test.ts',
              text: 'const copied = [\'nap\'].reduce(function copy(acc: string[], c) { return [...acc, c]; }, []);',
              isTest: true,
            },
          ],
        },),).toEqual([
          'cat.ts#distinct/keep: acc',
          'cat.ts#dropped/drop: state',
          'cat.ts#grouped: byCat entry',
          'cat.ts#joined/join: acc',
          'cat.ts#line/queue: acc',
          'cat.ts#named/purr: acc',
          'cat.ts#nearest/scan: state',
          'cat.ts#rebuilt: pile',
          'cat.ts#seen/note: names',
          'cat.ts#shed: left entry',
          'cat.ts#spliced/cut: current',
          'cat.ts#trimmed: run',
        ],);
      },
    },),
    it({
      name: 'FINDS NO FOLD COPYING WHAT IT BUILDS across the package but the folds named as bounded or sequential',
      fn: async () => {
        expectFindingsAsListed({
          findings: foldCopies({ files: await readPackageSource(), },),
          listed: Object.keys(BOUNDED_OR_SEQUENTIAL,).toSorted(),
        },);
      },
    },),
  ],
},);
