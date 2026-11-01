/**
 * `<tree-file-entry>` web component.
 *
 * A clickable file entry in the file tree with recency markers
 * and context menu support. Styled by the parent `<file-tree>`'s
 * shadow stylesheet (no shadow DOM of its own).
 *
 * Dispatches `file-select` (composed, crosses shadow boundary)
 * on click and `show-context` on right-click.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import { nameToOrder, } from './order.ts';

/**
 * Prevents the default browser context menu.
 *
 * @param event - context menu event to suppress
 */
function suppressContextMenu(event: Event,): void {
  event.preventDefault();
}

/**
 * `<tree-file-entry>`: clickable file label in the tree sidebar.
 *
 * @example
 * ```ts
 * const entry = createTreeFileEntry({ path: '/src/main.ts', name: 'main.ts', recencyIndex: 0 });
 * container.appendChild(entry);
 * ```
 */
export class TreeFileEntry extends HTMLElement {
  /**
   * Absolute file path.
   */
  entryPath = '';
  /**
   * Display name.
   */
  entryName = '';
  /**
   * Position in recent files list (-1 = not recent).
   */
  recencyIndex = -1;
  /**
   * Guards against re-rendering when the element is re-inserted into the DOM.
   */
  #initialized = false;

  /**
   * Renders the file label and attaches event handlers.
   */
  connectedCallback(): void {
    if (this.#initialized)
      return;
    this.#initialized = true;

    this.dataset
      .path = this.entryPath;
    this.setAttribute(
      'tabindex',
      '0',
    );
    this.style
      .order = String(nameToOrder({ name: this.entryName, },),);

    /**
     * Recency badge slot rendered to the left of the entry name.
     */
    const toggle = h({
      tag: 'span',
      class: 'toggle',
    },);
    if (this.recencyIndex
      !== (-1)) {
      toggle.textContent = String(this.recencyIndex,);
      this.dataset
        .recency = String(this.recencyIndex,);
    }

    this.replaceChildren(
      toggle,
      h({
        tag: 'span',
        class: 'name',
        text: this.entryName,
      },),
    );

    /**
     * Captured for the listener closures because event callbacks rebind `this`.
     */
    const entry = this;
    this.addEventListener(
      'click',
      function handleFileClick() {
        entry.focus();
        entry.dispatchEvent(new CustomEvent(
          'file-select',
          {
            bubbles: true,
            composed: true,
            detail: { path: entry.entryPath, },
          },
        ),);
      },
    );
    this.addEventListener(
      'mouseup',
      function handleFileContext(event: MouseEvent,) {
        if (event.button
          !== 2)
          return;
        event.preventDefault();
        entry.dispatchEvent(new CustomEvent(
          'show-context',
          {
            bubbles: true,
            detail: {
              x: event.clientX,
              y: event.clientY,
              path: entry.entryPath,
              kind: 'file' as const,
            },
          },
        ),);
      },
    );
    this.addEventListener(
      'contextmenu',
      suppressContextMenu,
    );
  }
}

/**
 * Creates a `<tree-file-entry>` element.
 *
 * @param path - absolute file path
 *
 * @param name - display name
 *
 * @param recencyIndex - position in recent files list (-1 if not recent)
 *
 * @returns configured element (renders on DOM insertion)
 *
 * @example
 * ```ts
 * const result = createTreeFileEntry({ path: '/home/user/project/src/main.ts', name: 'main.ts', recencyIndex: 0, });
 * ```
 */
export function createTreeFileEntry({
  path,
  name,
  recencyIndex,
}: {
  readonly path: string;
  readonly name: string;
  readonly recencyIndex: number;
},): TreeFileEntry {
  /* oxlint-disable typescript-eslint/no-unsafe-type-assertion -- guaranteed by customElements.define */
  /**
   * Custom element instance returned to the caller.
   */
  const entry = document.createElement('tree-file-entry',) as TreeFileEntry;
  /* oxlint-enable typescript-eslint/no-unsafe-type-assertion */
  entry.entryPath = path;
  entry.entryName = name;
  entry.recencyIndex = recencyIndex;
  return entry;
}

customElements.define(
  'tree-file-entry',
  TreeFileEntry,
);
