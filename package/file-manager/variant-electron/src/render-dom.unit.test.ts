/**
 Unit tests for the renderer's DOM reconciliation.
 
 Each test installs a document stand-in that keeps children, class names,
 attributes, and datasets, so the strip can be rendered without a browser and
 the assertions read what a later style could key off. Tests import from built
 `dist/app` so they verify the artifact the Electron renderer consumes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { BridgeFileEntry, } from '../dist/app/bridge-types.js';
import {
  createRendererStores,
  renderStrip,
  type PaneEventHandlers,
  type RendererStores,
} from '../dist/app/render-dom.js';
import {
  createStrip,
  directoryLocation,
  openRoot,
  previewLocation,
  spawnChild,
  type PaneId,
} from '../dist/app/strip.js';

/**
 Listing the entry assertions read: one directory, one file, one symbolic
 link, in the order the main process already sorted them.
 */
const FIXTURE_ENTRIES: readonly BridgeFileEntry[] = [
  {
    kind: 'directory',
    name: 'docs',
    path: '/home/docs',
  },
  {
    kind: 'file',
    name: 'note.txt',
    path: '/home/note.txt',
  },
  {
    kind: 'symlink',
    name: 'link',
    path: '/home/link',
  },
];

/**
 Handlers the reconciliation needs but no test drives through the DOM.
 */
const NOOP_HANDLERS: PaneEventHandlers = {
  onEntryOpen(): void {
    // no pointer event is dispatched by these tests
  },
  onPaneClose(): void {
    // no pointer event is dispatched by these tests
  },
  onPaneFocus(): void {
    // no pointer event is dispatched by these tests
  },
};

/**
 Marker property telling this file's elements apart from anything else.
 */
const FAKE_ELEMENT_MARKER = Symbol('fake element created by this test file',);

/**
 Value a fake `querySelector` returns when no descendant matches, so the
 caller's `instanceof` check fails the way a missing element does.
 */
const NO_MATCHING_ELEMENT = Symbol('selector matched zero fake elements',);

/**
 Element stand-in carrying the members the reconciliation reads and writes.
 
 Selector support covers what the renderer queries: one class selector,
 matched against each descendant's own class names, in document order.
 */
type FakeElement = {
  readonly [FAKE_ELEMENT_MARKER]: true;

  /**
   Appends one child, the only call shape the renderer uses.
   */
  readonly append: (added: FakeElement,) => void;

  /**
   Records nothing: no test dispatches a DOM event.
   */
  readonly addEventListener: () => void;

  /**
   Direct children in append order.
   */
  readonly children: FakeElement[];

  /**
   Space-separated class names, written as one string like the DOM member.
   */
  className: string;

  /**
   Class-list stand-in whose single mutation is a forced or inverted toggle.
   */
  readonly classList: { readonly toggle: (name: string, force?: boolean,) => void; };

  /**
   Data attributes, kept in step with the attribute map like the real member.
   */
  readonly dataset: Record<string, string>;

  /**
   Whether one attribute is present.
   */
  readonly hasAttribute: (name: string,) => boolean;

  /**
   First descendant matching one class selector, or the no-match sentinel.
   */
  readonly querySelector: (selector: string,) => FakeElement | typeof NO_MATCHING_ELEMENT;

  /**
   Descendants matching one class selector.
   */
  readonly querySelectorAll: <ElementType extends FakeElement,>(
    selector: string,
  ) => ElementType[];

  /**
   Records nothing: the fake keeps no parent link.
   */
  readonly remove: () => void;

  /**
   Sets one attribute and reflects it into `dataset`.
   */
  readonly setAttribute: (name: string, value: string,) => void;

  /**
   Inline styles, written by the band layout.
   */
  readonly style: Record<string, string>;

  /**
   Tag the element stands for.
   */
  readonly tagName: string;

  /**
   Text content the renderer writes.
   */
  textContent: string;

  /**
   Adds, sets, or removes an attribute by presence.
   */
  readonly toggleAttribute: (name: string, force?: boolean,) => boolean;
};

/**
 Class names currently set on an element.
 
 @param element - Element to read.
 
 @returns Class names without empty tokens.
 */
function classTokensOf(element: FakeElement,): string[] {
  return element.className
    .split(' ',)
    .filter(function nonEmpty(token,): boolean {
      return token.length > 0;
    },);
}

/**
 Whether one class name is set.
 
 @param element - Element to read.
 
 @param name - Class name to look for.
 
 @returns Whether the class name is present.
 */
function hasClassOf(
  {
    element,
    name,
  }: {
    readonly element: FakeElement;
    readonly name: string;
  },
): boolean {
  return classTokensOf(element,)
    .includes(name,);
}

/**
 Adds or removes one class name.
 
 @param element - Element to mutate.
 
 @param enabled - Whether the class name should be present.
 
 @param name - Class name to add or remove.
 */
function setClassOf(
  {
    element,
    enabled,
    name,
  }: {
    readonly element: FakeElement;
    readonly enabled: boolean;
    readonly name: string;
  },
): void {
  /**
   Class names other than the one being changed.
   */
  const remaining = classTokensOf(element,)
    .filter(function isOther(token,): boolean {
      return token !== name;
    },);

  element.className = (enabled ? [...remaining, name,] : remaining).join(' ',);
}

/**
 Dataset key a `data-` attribute name reflects to.
 
 @param name - Attribute name beginning with `data-`.
 
 @returns Camel-cased dataset key.
 */
function datasetNameOf(name: string,): string {
  const [, ...segments] = name.split('-',);

  return segments
    .map(function toCamel(segment, index,): string {
      return (index === 0)
        ? segment
        : `${segment.charAt(0,)
          .toUpperCase()}${segment.slice(1,)}`;
    },)
    .join('',);
}

/**
 Whether an attribute name reflects into `dataset`.
 
 @param name - Attribute name to check.
 
 @returns Whether the name begins with `data-`.
 */
function isDatasetName(name: string,): boolean {
  return name.startsWith('data-',);
}

/**
 Every descendant of an element in document order.
 
 @param element - Element to walk.
 
 @returns Descendants, parents before children.
 */
function descendantsOf(element: FakeElement,): FakeElement[] {
  return element.children
    .flatMap(function withOwnSubtree(child,): FakeElement[] {
      return [child, ...descendantsOf(child,),];
    },);
}

/**
 Whether a value is an element this test file created.
 
 @param value - Candidate to check.
 
 @returns Whether the value is a fake element.
 */
function isFakeElement(value: unknown,): value is FakeElement {
  return ((typeof value) === 'object')
    && (value !== null)
    && (Reflect.get(
      value,
      FAKE_ELEMENT_MARKER,
    )
      === true);
}

/**
 Creates one detached element stand-in.
 
 @param tagName - Tag the element stands for.
 
 @returns Fresh element.
 */
function createFakeElement({ tagName, }: { readonly tagName: string; },): FakeElement {
  /**
   Attributes by name, the source of truth `dataset` mirrors.
   */
  const attributes = new Map<string, string>();

  /**
   Children in append order.
   */
  const children: FakeElement[] = [];

  const element: FakeElement = {
    [FAKE_ELEMENT_MARKER]: true,
    addEventListener(): void {
      // no test dispatches a DOM event
    },
    append(added: FakeElement,): void {
      children.push(added,);
    },
    children,
    className: '',
    classList: {
      toggle(name: string, force?: boolean,): void {
        setClassOf({
          element,
          enabled: force ?? (!hasClassOf({
            element,
            name,
          },)),
          name,
        },);
      },
    },
    dataset: {},
    hasAttribute(name: string,): boolean {
      return attributes.has(name,);
    },
    querySelector(selector: string,): FakeElement | typeof NO_MATCHING_ELEMENT {
      return element.querySelectorAll<FakeElement>(selector,)[0]
        ?? NO_MATCHING_ELEMENT;
    },
    querySelectorAll<ElementType extends FakeElement,>(selector: string,): ElementType[] {
      /**
       Class name the selector asks for; every selector this renderer uses is one class.
       */
      const wanted = selector.replace('.', '',);

      return descendantsOf(element,)
        .filter(function hasWantedClass(candidate,): boolean {
          return hasClassOf({
            element: candidate,
            name: wanted,
          },);
        },) as ElementType[];
    },
    remove(): void {
      // the fake keeps no parent link, so removal has nothing to undo
    },
    setAttribute(name: string, value: string,): void {
      attributes.set(
        name,
        value,
      );
      if (isDatasetName(name,))
        element.dataset[datasetNameOf(name,)] = value;
    },
    style: {},
    tagName,
    textContent: '',
    toggleAttribute(name: string, force?: boolean,): boolean {
      /**
       Whether the attribute is present after the toggle.
       */
      const present = force ?? (!attributes.has(name,));

      if (present) {
        attributes.set(
          name,
          '',
        );
        if (isDatasetName(name,))
          element.dataset[datasetNameOf(name,)] = '';
      }
      else {
        attributes.delete(name,);
        if (isDatasetName(name,))
          Reflect.deleteProperty(
            element.dataset,
            datasetNameOf(name,),
          );
      }

      return present;
    },
  };

  return element;
}

/**
 `instanceof` stand-in for the browser list constructor.
 
 The reconciliation type-checks its list element with
 `instanceof HTMLUListElement` before filling it, so the stand-in answers true
 for the `ul` element the fake document creates and false for the no-match
 sentinel.
 */
const listElementConstructor = {
  [Symbol.hasInstance](candidate: unknown,): boolean {
    return isFakeElement(candidate,)
      && (candidate.tagName === 'ul');
  },
};

/**
 Document stand-in installed for the duration of one test.
 */
type FakeDom = Disposable & {
  /**
   Creates a detached element to stand in for the strip scroller.
   
   @returns Fresh element.
   */
  readonly createStripElement: () => FakeElement;
};

/**
 Installs the document stand-in the renderer creates elements through.
 
 @returns Disposal restoring both globals, plus the strip-element factory.
 */
function installFakeDom(): FakeDom {
  /**
   Document value the test process started with, if any.
   */
  const originalDocument = globalThis.document;

  /**
   List-constructor value the test process started with, if any.
   */
  const originalListElement = Reflect.get(
    globalThis,
    'HTMLUListElement',
  );

  /**
   Document stand-in whose only member is element creation.
   */
  const fakeDocument = {
    createElement: function createElement(tagName: string,): FakeElement {
      return createFakeElement({ tagName, },);
    },
  };

  globalThis.document = fakeDocument as unknown as Document;
  Reflect.set(
    globalThis,
    'HTMLUListElement',
    listElementConstructor,
  );

  return {
    createStripElement(): FakeElement {
      return createFakeElement({ tagName: 'div', },);
    },
    [Symbol.dispose](): void {
      globalThis.document = originalDocument;
      Reflect.set(
        globalThis,
        'HTMLUListElement',
        originalListElement,
      );
    },
  };
}

/**
 Reads back the element the renderer stored for one pane.
 
 @param id - Pane whose element is wanted.
 
 @param stores - Stores the last render wrote.
 
 @returns The stored element as the fake this test reads.
 
 @throws Error when the renderer stored no element for the pane.
 */
function paneElementOf(
  {
    id,
    stores,
  }: {
    readonly id: PaneId;
    readonly stores: RendererStores;
  },
): FakeElement {
  /**
   Element the renderer stored for the pane, if it stored one.
   */
  const element = stores.paneElements
    .get(id,);

  if (!isFakeElement(element,))
    throw new Error(`Renderer stored no element for pane ${id}.`,);

  return element;
}

/**
 Opens a root directory pane over `/home` in a fresh strip.
 
 @returns Root pane id and the strip holding it.
 */
function openFixtureRoot(): {
  readonly id: PaneId;
  readonly strip: ReturnType<typeof createStrip>;
} {
  return openRoot({
    location: directoryLocation({ path: '/home', },),
    strip: createStrip(),
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: renderStrip.name,
      children: [
        it({
          name: 'marks a pane active and names its kind with data attributes',
          fn: async (): Promise<void> => {
            using dom = installFakeDom();
            const root = openFixtureRoot();
            const stores = createRendererStores();
            const stripElement = dom.createStripElement();

            renderStrip({
              handlers: NOOP_HANDLERS,
              stores,
              strip: root.strip,
              stripElement: stripElement as unknown as HTMLElement,
            },);

            const pane = paneElementOf({
              id: root.id,
              stores,
            },);
            expect(pane.className,).toBe('pane',);
            expect(pane.dataset
              .kind,).toBe('directory',);
            expect(Object.hasOwn(pane.dataset, 'active',),).toBe(true,);
          },
        },),
        it({
          name: 'names a preview pane kind and leaves the previous pane unfocused',
          fn: async (): Promise<void> => {
            using dom = installFakeDom();
            const root = openFixtureRoot();
            const preview = spawnChild({
              forceDuplicate: false,
              location: previewLocation({ path: '/home/note.txt', },),
              parent: root.id,
              strip: root.strip,
            },);
            const stores = createRendererStores();
            const stripElement = dom.createStripElement();

            renderStrip({
              handlers: NOOP_HANDLERS,
              stores,
              strip: preview.strip,
              stripElement: stripElement as unknown as HTMLElement,
            },);

            const previewPane = paneElementOf({
              id: preview.id,
              stores,
            },);
            const rootPane = paneElementOf({
              id: root.id,
              stores,
            },);
            expect(previewPane.className,).toBe('pane',);
            expect(previewPane.dataset
              .kind,).toBe('preview',);
            expect(Object.hasOwn(previewPane.dataset, 'active',),).toBe(true,);
            expect(Object.hasOwn(rootPane.dataset, 'active',),).toBe(false,);
          },
        },),
        it({
          name: 'names an entry kind and marks exactly the selected entry',
          fn: async (): Promise<void> => {
            using dom = installFakeDom();
            const root = openFixtureRoot();
            const stores = createRendererStores();
            stores.listings
              .set(
              root.id,
              FIXTURE_ENTRIES,
              );
            const stripElement = dom.createStripElement();

            /**
             Renders once and returns the pane's entry elements.
             
             @returns Entry elements in listing order.
             */
            function renderEntries(): FakeElement[] {
              renderStrip({
                handlers: NOOP_HANDLERS,
                stores,
                strip: root.strip,
                stripElement: stripElement as unknown as HTMLElement,
              },);

              return paneElementOf({
                id: root.id,
                stores,
              },)
                .querySelectorAll<FakeElement>('.entry',);
            }

            const firstRender = renderEntries();
            expect(firstRender,).toHaveLength(FIXTURE_ENTRIES.length,);
            expect(firstRender.map(function kindOf(entry,): string {
              return entry.dataset
                .kind
                ?? '';
            },),).toEqual(['directory', 'file', 'symlink',],);
            expect(firstRender.map(function classesOf(entry,): string {
              return entry.className;
            },),).toEqual(['entry', 'entry', 'entry',],);
            expect(firstRender.map(function selectedOf(entry,): boolean {
              return Object.hasOwn(entry.dataset, 'selected',);
            },),).toEqual([true, false, false,],);

            stores.selections
              .set(
              root.id,
              2,
              );
            const movedSelection = renderEntries();
            expect(movedSelection.map(function selectedOf(entry,): boolean {
              return Object.hasOwn(entry.dataset, 'selected',);
            },),).toEqual([false, false, true,],);
          },
        },),
      ],
    },),
  ],
},);
