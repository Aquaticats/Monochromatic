/**
 * Unit tests for `resolveTextPosition`'s clamping behaviour.
 *
 * Covers:
 *
 * - a character offset inside a text node
 * - an offset past the end of a plain-text line
 * - an offset past the end of a syntax-highlighted line, whose last child is a
 *   `<span>` element rather than a text node
 * - a line that holds no text node at all
 *
 * The resolver reads `document.createTreeWalker` and `NodeFilter` from the
 * global scope, so each test installs a minimal DOM for its duration.
 *
 * The test imports the package source, like the package's other unit tests:
 * editord ships only a browser bundle for the client editor, so there is no
 * Node artifact that exports this helper.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { resolveTextPosition, } from './text-resolve.ts';

/**
 * Node type value `Node.TEXT_NODE` reports.
 */
const TEXT_NODE = 3;

/**
 * Node type value `Node.ELEMENT_NODE` reports.
 */
const ELEMENT_NODE = 1;

/**
 * Mask `NodeFilter.SHOW_TEXT` reports.
 */
const SHOW_TEXT = 4;

/**
 * Node shape the resolver's DOM calls read: child nodes, a last child, and text
 * content.
 */
type FakeNode = {
  readonly nodeType: number;
  readonly textContent: string;
  readonly children: readonly FakeNode[];
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `Node.lastChild`, which the DOM types as `Node | null`
  readonly lastChild: FakeNode | null;
};

/**
 * Text-only walker over the fake tree, narrowed to the one method the resolver
 * calls.
 */
type FakeWalker = {
  /**
   * Advances to the next text node.
   *
   * @returns next text node, mirroring `TreeWalker.nextNode`'s `Node | null`
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors `TreeWalker.nextNode`, which the DOM types as `Node | null`
  readonly nextNode: () => FakeNode | null;
};

/**
 * Builds a text node.
 *
 * @param text - text the node holds
 *
 * @returns text node
 *
 * @example
 * ```ts
 * textNode('hello');
 * ```
 */
function textNode(text: string,): FakeNode {
  return {
    nodeType: TEXT_NODE,
    textContent: text,
    children: [],
    lastChild: null,
  };
}

/**
 * Builds an element node from its children.
 *
 * @param children - child nodes in document order
 *
 * @returns element node whose `textContent` concatenates its descendants' text
 *
 * @example
 * ```ts
 * elementNode([textNode('a')]);
 * ```
 */
function elementNode(children: readonly FakeNode[],): FakeNode {
  return {
    nodeType: ELEMENT_NODE,
    textContent: children
      .map(function textOf(child,) {
        return child.textContent;
      },)
      .join('',),
    children,
    lastChild: children.at(-1,)
      ?? null,
  };
}

/**
 * Collects every text node under a root in document order.
 *
 * @param root - node to walk
 *
 * @returns text nodes below the root
 *
 * @example
 * ```ts
 * collectText(pane);
 * ```
 */
function collectText(root: FakeNode,): FakeNode[] {
  const found: FakeNode[] = [];
  /**
   * Subtrees still to visit, in document order; each step unshifts the next
   * node's children so the walk stays depth-first.
   */
  const pending: FakeNode[] = [root,];
  while (pending.length > 0) {
    const next = pending.shift();
    if (next === undefined)
      break;
    if (next.nodeType === TEXT_NODE) {
      found.push(next,);
      continue;
    }
    pending.unshift(...next.children,);
  }
  return found;
}

/**
 * Creates a text-only tree walker over the fake tree.
 *
 * @param textNodes - text nodes of the walked subtree, in document order
 *
 * @returns walker exposing the method the resolver calls
 *
 * @example
 * ```ts
 * const walker = createWalker(collectText(line));
 * ```
 */
function createWalker(textNodes: readonly FakeNode[],): FakeWalker {
  /**
   * Iterator over the subtree's text nodes; exhausted once the walk ends.
   */
  const nodes = textNodes[Symbol.iterator]();
  return {
    nextNode: function nextNode() {
      /**
       * Next step of the iterator; `done` is true once the walk is exhausted.
       */
      const step = nodes.next();
      if (step.done === true)
        return null;
      return step.value;
    },
  };
}

/**
 * Installs a fake `document` and `NodeFilter`, restoring both when the returned
 * guard leaves `using` scope.
 *
 * @returns disposable that restores the globals
 *
 * @example
 * ```ts
 * using _dom = installFakeDom();
 * ```
 */
function installFakeDom(): Disposable {
  const hadDocument = 'document' in globalThis;
  const hadNodeFilter = 'NodeFilter' in globalThis;
  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: {
      createTreeWalker(root: FakeNode,) {
        return createWalker(collectText(root,),);
      },
    },
  },);
  Object.defineProperty(globalThis, 'NodeFilter', {
    configurable: true,
    value: {
      SHOW_TEXT,
    },
  },);
  return {
    [Symbol.dispose](): void {
      if (!hadDocument)
        Reflect.deleteProperty(globalThis, 'document',);
      if (!hadNodeFilter)
        Reflect.deleteProperty(globalThis, 'NodeFilter',);
    },
  };
}

/**
 * Wraps one fake line in an editor element.
 *
 * @param line - line contents
 *
 * @returns editor element whose only child is the line
 *
 * @example
 * ```ts
 * createEditor(elementNode([textNode('abc')]));
 * ```
 */
function createEditor(line: FakeNode,): HTMLDivElement {
  return elementNode([line,]) as unknown as HTMLDivElement;
}

await describe({
  name: resolveTextPosition.name,
  concurrency: 1,
  children: [
    it({
      name: 'resolves an offset inside a text node',
      async fn() {
        using _dom = installFakeDom();
        await Promise.resolve();
        const line = elementNode([textNode('abcdef',),],);
        const resolved = resolveTextPosition({
          editor: createEditor(line,),
          lineIndex: 0,
          character: 3,
        },);
        expect(resolved?.offset,).toBe(3,);
        expect(resolved?.node.nodeType,).toBe(TEXT_NODE,);
      },
    },),
    it({
      name: 'clamps an offset past the end of a plain line to the last text node',
      async fn() {
        using _dom = installFakeDom();
        await Promise.resolve();
        const line = elementNode([textNode('abc',),],);
        const resolved = resolveTextPosition({
          editor: createEditor(line,),
          lineIndex: 0,
          character: 10,
        },);
        expect(resolved?.node.nodeType,).toBe(TEXT_NODE,);
        expect(resolved?.node.textContent,).toBe('abc',);
        expect(resolved?.offset,).toBe(3,);
      },
    },),
    it({
      name: 'clamps an offset past the end of a highlighted line to its last text node',
      async fn() {
        using _dom = installFakeDom();
        await Promise.resolve();
        /**
         * Highlight span wrapping the tail of the line; it is the line's last
         * child, so returning `lastChild` hands the selection an element node.
         */
        const span = elementNode([textNode('de',),],);
        const line = elementNode([textNode('abc',), span,],);
        const resolved = resolveTextPosition({
          editor: createEditor(line,),
          lineIndex: 0,
          character: 10,
        },);
        expect(resolved?.node.nodeType,).toBe(TEXT_NODE,);
        expect(resolved?.node.textContent,).toBe('de',);
        expect(resolved?.offset,).toBe(2,);
      },
    },),
    it({
      name: 'falls back to the line element when it holds no text node',
      async fn() {
        using _dom = installFakeDom();
        await Promise.resolve();
        const line = elementNode([elementNode([],),],);
        const resolved = resolveTextPosition({
          editor: createEditor(line,),
          lineIndex: 0,
          character: 4,
        },);
        expect(resolved?.node,).toBe(line,);
        expect(resolved?.offset,).toBe(0,);
      },
    },),
  ],
},);
