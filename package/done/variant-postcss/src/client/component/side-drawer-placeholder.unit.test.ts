/**
 The drawer offers no placeholder identity and no destination that goes nowhere.

 The Figma wireframe the drawer was built from shows a profile row reading
 "Firstname" and a Contact entry, and the implementation copied both literally:
 `Firstname` is one person's placeholder rather than app data, and Contact's
 `href` is `#`, which changes the fragment and scrolls instead of opening
 anything. The drawer must not present either until a real data contract and a
 real destination exist.

 The builders create DOM through `hDom`, so this file installs a document
 stand-in that keeps text, attributes, and children, and reads the rendered tree
 back from it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Marker property telling this file's elements apart from anything else.
 */
const FAKE_ELEMENT_MARKER = Symbol('fake element created by this test file',);

//region Document stand-in

/**
 Element stand-in carrying the members the drawer builders write.
 */
type FakeElement = {
  readonly [FAKE_ELEMENT_MARKER]: true;
  /**
   Space-separated class names, as the DOM member holds them.
   */
  className: string;
  /**
   Text content the builders set.
   */
  textContent: string;
  /**
   Markup the builders set, which only the close button's SVG uses.
   */
  innerHTML: string;
  /**
   Inline styles the builders set.
   */
  readonly style: Record<string, string>;
  /**
   Attributes the builders set, which carry each link's destination.
   */
  readonly attributes: Record<string, string>;
  /**
   Children in append order.
   */
  readonly children: FakeElement[];
  /**
   Records one attribute.
   */
  readonly setAttribute: (name: string, value: string,) => void;
  /**
   Records nothing: no test dispatches a DOM event.
   */
  readonly addEventListener: () => void;
  /**
   Appends children in call order, reporting the new child count like the DOM.
   */
  readonly append: (...added: readonly FakeElement[]) => number;
  /**
   Class-list stand-in whose single mutation adds a token.
   */
  readonly classList: { readonly add: (name: string,) => void; };
};

/**
 Builds one element stand-in.

 @param tagName - tag the builder asked for

 @returns element with empty text, attributes, and children

 @example
 ```ts
 const element = fakeElement('nav');
 ```
 */
function fakeElement(tagName: string,): FakeElement {
  /**
   Direct children in append order, which the appender below writes into.
   */
  const children: FakeElement[] = [];
  /**
   The array's own push, bound to the child list: `Element.append` takes however
   many nodes the caller passes, and rest parameters are banned in this workspace.
   */
  const append = Array.prototype.push.bind(children,);
  const element: FakeElement = {
    [FAKE_ELEMENT_MARKER]: true,
    className: '',
    textContent: '',
    innerHTML: '',
    style: {},
    attributes: {},
    children,
    setAttribute(name: string, value: string,): void {
      element.attributes[name] = value;
    },
    addEventListener(): void {
      // no test dispatches a DOM event
    },
    append,
    classList: {
      add(name: string,): void {
        element.className = element.className.length === 0 ? name : `${element.className} ${name}`;
      },
    },
  };
  Object.defineProperty(element, 'tagName', { value: tagName, },);
  return element;
}

/**
 Installs a document stand-in so the builders' `createElement` calls reach
 {@link fakeElement}, and returns everything created in call order.

 @returns elements created from now on, which the assertions inspect

 @example
 ```ts
 const created = installDocumentStandIn();
 ```
 */
function installDocumentStandIn(): FakeElement[] {
  /**
   Elements created after this call, in creation order.
   */
  const created: FakeElement[] = [];
  Object.defineProperty(globalThis, 'document', {
    configurable: true,
    value: {
      createElement(tagName: string,): FakeElement {
        /**
         Element handed back to the builder.
         */
        const element = fakeElement(tagName,);
        created.push(element,);
        return element;
      },
    },
  },);
  return created;
}

//endregion Document stand-in

//region Reading the rendered tree

/**
 Collects the text of an element and of every descendant.

 @param root - element to read

 @returns every non-empty text content in the subtree

 @example
 ```ts
 const texts = textsOf(nav);
 ```
 */
function textsOf(root: FakeElement,): string[] {
  /**
   Text collected so far, the element's own first.
   */
  const texts: string[] = [];
  if (root.textContent.length > 0)
    texts.push(root.textContent,);
  for (const child of root.children)
    texts.push(...textsOf(child,),);
  return texts;
}

/**
 Collects the `href` of every anchor in a subtree, in document order.

 @param root - element to search

 @returns destination of each anchor that carries one

 @example
 ```ts
 const hrefs = hrefsUnder(nav);
 ```
 */
function hrefsUnder(root: FakeElement,): string[] {
  /**
   Destinations collected so far.
   */
  const hrefs: string[] = [];
  if (root.attributes.href !== undefined)
    hrefs.push(root.attributes.href,);
  for (const child of root.children)
    hrefs.push(...hrefsUnder(child,),);
  return hrefs;
}

//endregion Reading the rendered tree

/**
 Elements the builders created, which lets one case read the tree it asked for.
 */
const createdElements = installDocumentStandIn();

const { buildHeader, buildNav, } = await import('../../../dist/final/node/index.mjs');

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildNav.name,
      children: [
        it({
          name: 'offers only destinations this app serves',
          fn: async () => {
            createdElements.length = 0;
            /**
             Navigation element under test.
             */
            const nav = buildNav() as unknown as FakeElement;
            expect(hrefsUnder(nav,),).toEqual(['/', '/in-progress', '/settings',],);
          },
        },),
        it({
          name: 'names no entry after a contact page nothing implements',
          fn: async () => {
            createdElements.length = 0;
            /**
             Navigation element under test.
             */
            const nav = buildNav() as unknown as FakeElement;
            expect(textsOf(nav,),).toEqual(['Inbox', 'In Progress', 'Settings',],);
          },
        },),
      ],
    },),
    describe({
      name: buildHeader.name,
      children: [
        it({
          name: 'carries no placeholder identity',
          fn: async () => {
            createdElements.length = 0;
            /**
             Header element under test, rendered as the inline sidebar renders it.
             */
            const header = buildHeader() as unknown as FakeElement;
            expect(textsOf(header,),).toEqual([],);
          },
        },),
        it({
          name: 'holds the close button it is given and nothing else',
          fn: async () => {
            createdElements.length = 0;
            /**
             Close button stand-in the header must keep as its only child.
             */
            const closeButton = fakeElement('button',);
            /**
             Header element under test, rendered as the phone layout renders it.
             */
            const header = buildHeader(closeButton as unknown as HTMLElement,) as unknown as FakeElement;
            expect(header.children,).toEqual([closeButton,],);
            expect(textsOf(header,),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
