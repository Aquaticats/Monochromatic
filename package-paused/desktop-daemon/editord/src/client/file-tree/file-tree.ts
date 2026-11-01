/**
 * `<file-tree>` web component.
 *
 * A directory tree sidebar composed of `<tree-dir-entry>` and
 * `<tree-file-entry>` child components. Clicking a file dispatches
 * a `file-select` CustomEvent with the absolute path.
 *
 * The tree coordinates data fetching and context menus via
 * event delegation: child components dispatch bubbling events
 * that the tree handles centrally.
 */

import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { DirEntry, } from '../../../protocol.ts';
import {
  createContextMenu,
  type ContextMenu,
} from '../context-menu/context-menu.ts';
import {
  showDirContextMenu,
  showFileContextMenu,
} from './context.ts';
import type {
  DirOpenDetail,
  ShowContextDetail,
} from './dir-entry.ts';
import { preloadChildren, } from './entries.ts';
import { STYLES, } from './file-tree.styles.ts';
import {
  createEntryElements,
  loadDirChildren,
} from './load.ts';
import {
  collectExpandedDirs,
  resolveSelectedDir,
  restoreExpansion as doRestoreExpansion,
  updateRecencyMarkers,
} from './ops.ts';
import { performRefreshDir, } from './refresh.ts';
import {
  revealFiles as doRevealFiles,
  scrollToFile as doScrollToFile,
} from './reveal.ts';
import type { FileTreeState, } from './state.ts';
import type { ContextAction, } from './types.ts';

export type { ContextAction, };
export type { DirEntry, };

/**
 * `<file-tree>`: directory tree sidebar with `<tree-dir-entry>` toggles.
 */
export class FileTree extends HTMLElement {
  /**
   * Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;
  /**
   * Container div for tree entries.
   */
  #tree: HTMLDivElement | null = null;
  /**
   * Last focused element for resolving `selectedDir`.
   */
  #lastFocused: HTMLElement | null = null;
  /**
   * Absolute root directory path.
   */
  #rootPath = '';
  /**
   * Shared mutable state for data fetching and caching.
   */
  readonly #state: FileTreeState;
  /**
   * Context menu component for right-click actions.
   */
  #contextMenu: ContextMenu | null = null;
  /**
   * Callback invoked when a context menu action is selected.
   */
  #onContextAction: ((action: ContextAction,) => void) | null = null;

  /**
   * Callback to fetch directory contents.
   *
   * @returns current fetchDir callback, or null
   */
  get fetchDir(): ((path: string,) => Promise<readonly DirEntry[]>) | null {
    return this.#state
      .fetchDir;
  }
  /**
   * Installs the fetchDir callback.
   *
   * @param fn - fetchDir callback to install
   */
  set fetchDir(fn: ((path: string,) => Promise<readonly DirEntry[]>) | null,) {
    this.#state
      .fetchDir = fn;
  }
  /**
   * Callback invoked when a directory is expanded for the first time.
   *
   * @returns current onDirExpanded callback, or null
   */
  get onDirExpanded(): ((path: string,) => void) | null {
    return this.#state
      .onDirExpanded;
  }
  /**
   * Installs the onDirExpanded callback.
   *
   * @param fn - onDirExpanded callback to install
   */
  set onDirExpanded(fn: ((path: string,) => void) | null,) {
    this.#state
      .onDirExpanded = fn;
  }
  /**
   * Callback invoked when a context menu action is selected.
   *
   * @returns current onContextAction callback, or null
   */
  get onContextAction(): ((action: ContextAction,) => void) | null {
    return this.#onContextAction;
  }
  /**
   * Installs the onContextAction callback.
   *
   * @param fn - onContextAction callback to install
   */
  set onContextAction(fn: ((action: ContextAction,) => void) | null,) {
    this.#onContextAction = fn;
  }

  /**
   * Initializes the shadow root and internal state.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
    this.#state = {
      fetchDir: null,
      onDirExpanded: null,
      prefetchCache: new Map(),
      loadedDirs: new Set(),
      loadPromises: new Map(),
      recentPaths: [],
    };
  }

  /**
   * Resolves the directory path of the last-focused element.
   *
   * @returns directory path, or empty string when nothing has been focused
   */
  get selectedDir(): string {
    return resolveSelectedDir({ lastFocused: this.#lastFocused, },);
  }

  /**
   * Renders the tree container and attaches event delegation.
   */
  connectedCallback(): void {
    /**
     * Stable reference to `this` so inner event handlers retain the component instance.
     */
    const tree = this;
    this.#tree = h({
      tag: 'div',
      class: 'tree',
    },);
    this.#contextMenu = createContextMenu();
    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: STYLES,
      },),
      this.#tree,
    );

    this.#shadow
      .addEventListener(
      'focusin',
      function handleFocusIn(event,) {
        if (event.target
          instanceof HTMLElement)
          tree.#lastFocused = event.target;
      },
    );
    this.#tree
      .addEventListener(
      'dir-open',
      function handleDirOpen(event,) {
        loadDirChildren({
          // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- CustomEvent from TreeDirEntry
          detail: (event as CustomEvent<DirOpenDetail>).detail,
          state: tree.#state,
        },);
      },
    );
    this.#tree
      .addEventListener(
      'show-context',
      function handleShowContext(event,) {
        /**
         * Detail payload from the bubbling `show-context` CustomEvent: coordinates and target.
         */
        const {
          x,
          y,
          path,
          kind,
          // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- CustomEvent from tree entries
        } = (event as CustomEvent<ShowContextDetail>).detail;
        if (tree.#contextMenu
          === null)
          return;
        /**
         * Forwards context menu action to the external callback.
         *
         * @param action - selected context menu action to forward
         */
        function fireAction(action: ContextAction,): void {
          tree.#onContextAction?.(action,);
        }
        if (kind === 'file') {
          showFileContextMenu({
            contextMenu: tree.#contextMenu,
            x,
            y,
            path,
            onAction: fireAction,
          },);
        }
        else {
          showDirContextMenu({
            contextMenu: tree.#contextMenu,
            x,
            y,
            path,
            onAction: fireAction,
          },);
        }
      },
    );
  }

  /**
   * Loads and renders the root directory's entries.
   *
   * @param rootPath - absolute path to the root directory
   */
  async expandRoot(rootPath: string,): Promise<void> {
    /**
     * Captured fetch callback; absent when the tree was never wired up with a backing fetcher.
     */
    const { fetchDir, } = this.#state;
    if ((this.#tree
      === null) || (fetchDir === null))
      return;
    this.#rootPath = rootPath;
    /**
     * Root-level directory listing used to populate the top of the tree.
     */
    const entries = await fetchDir(rootPath,);
    /**
     * DOM nodes representing the root listing; replaces any prior children.
     */
    const children = createEntryElements({
      parentPath: rootPath,
      entries,
      recentPaths: this.#state
        .recentPaths,
    },);
    this.#tree
      .replaceChildren(...children,);
    void preloadChildren({
      parentPath: rootPath,
      entries,
      fetchDir,
      prefetchCache: this.#state
        .prefetchCache,
    },);
    this.#state
      .onDirExpanded?.(rootPath,);
  }

  /**
   * Collects the absolute paths of all currently expanded directories.
   *
   * @returns array of absolute directory paths that are expanded
   */
  get expandedDirs(): string[] {
    if (this.#tree
      === null)
      return [];
    return collectExpandedDirs({ tree: this.#tree, },);
  }

  /**
   * Restores previously expanded directories by programmatically opening each.
   *
   * @param dirs - absolute paths of directories to expand
   */
  async restoreExpansion({ dirs, }: { readonly dirs: readonly string[]; },): Promise<void> {
    if ((this.#tree
      === null) || (this.#state
        .fetchDir
        === null)
      || (dirs.length
        === 0))
      return;
    /**
     * Trailing-slash prefix used to drop any restored path that escaped the configured root.
     */
    const rootPrefix = `${this.#rootPath}/`;
    /**
     * Restored paths that still sit under the current root; everything else is silently dropped.
     */
    const validDirs = dirs.filter(function withinRoot(dir,) {
      return dir.startsWith(rootPrefix,);
    },);
    if (validDirs.length
      === 0)
      return;
    await doRestoreExpansion({
      tree: this.#tree,
      dirs: validDirs,
      loadPromises: this.#state
        .loadPromises,
    },);
  }

  /**
   * Updates recency markers on all file entries in the tree.
   *
   * @param paths - ordered recent file paths (index 0 = most recent)
   */
  updateRecency({ paths, }: { readonly paths: readonly string[]; },): void {
    this.#state
      .recentPaths = paths;
    if (this.#tree
      !== null) {
      updateRecencyMarkers({
        tree: this.#tree,
        paths,
      },);
    }
  }

  /**
   * Ensures the given file paths are visible by expanding ancestor directories.
   *
   * @param paths - absolute file paths to reveal
   */
  async revealFiles({ paths, }: { readonly paths: readonly string[]; },): Promise<void> {
    if ((this.#tree
      === null) || (this.#rootPath
        === ''))
      return;
    /**
     * Stable reference to `this` so the inline `restore` callback retains the component instance.
     */
    const tree = this;
    await doRevealFiles({
      tree: this.#tree,
      hostElement: this,
      rootPath: this.#rootPath,
      paths,
      restoreExpansion: function restore(opts,) {
        return tree.restoreExpansion(opts,);
      },
    },);
  }

  /**
   * Scrolls the tree so the given file entry is visible.
   *
   * @param path - absolute file path to scroll into view
   */
  scrollToFile({ path, }: { readonly path: string; },): void {
    if (this.#tree
      === null)
      return;
    doScrollToFile({
      tree: this.#tree,
      path,
    },);
  }

  /**
   * Re-fetches and re-renders the contents of a single directory.
   *
   * @param path - absolute path of the directory to refresh
   */
  async refreshDir({ path, }: { readonly path: string; },): Promise<void> {
    if (this.#tree
      === null)
      return;
    await performRefreshDir({
      tree: this.#tree,
      path,
      rootPath: this.#rootPath,
      state: this.#state,
    },);
  }
}

customElements.define(
  'file-tree',
  FileTree,
);
