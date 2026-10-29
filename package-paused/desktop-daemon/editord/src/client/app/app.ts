/**
 * editord client entry point.
 *
 * Creates components, connects WebSocket, wires events, boots.
 */

// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../binary-viewer/binary-viewer.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../editor/editor-pane.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../file-tree/file-tree.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../search/search-overlay.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../hover/hover-popup.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../completion/completion-popup.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../references/references-popup.ts';
// oxlint-disable-next-line import/no-unassigned-import -- side-effect: register custom elements
import '../rename/rename-input.ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core';
import { createDebounced, } from '../debounce.ts';
import { AUTO_SAVE_DEBOUNCE_MS, } from '../timing.ts';

import type {
  DirEntry,
  FileKind,
  SearchResult,
} from '../../../protocol.ts';
import type { BinaryViewer, } from '../binary-viewer/binary-viewer.ts';
import type { CompletionPopup, } from '../completion/completion-popup.ts';
import type { EditorPane, } from '../editor/editor-pane.ts';
// Editing commands are called as standalone functions rather than
// methods on `editorPane`. Previously EditorPane had seven wrapper
// methods (deleteCurrentLine, indentLines, etc.) that each did
// nothing but `performXxx({ pane: this })`. Calling the perform
// functions directly here removes that indirection and keeps the
// class focused on state it actually owns (content, diagnostics,
// cursor, lifecycle).
import {
  performDeleteLine,
  performDuplicateLine,
  performIndent,
  performSelectAndCopy,
  performSwapDown,
  performSwapUp,
  performUnindent,
} from '../editor/editor-pane-commands.ts';
import type {
  ContextAction,
  FileTree,
} from '../file-tree/file-tree.ts';
import type { HoverPopup, } from '../hover/hover-popup.ts';
import {
  l,
  tagged,
} from '../log.ts';
import { createRecentFiles, } from '../recent-files.ts';
import type { ReferencesPopup, } from '../references/references-popup.ts';
import type { RenameInput, } from '../rename/rename-input.ts';
import type { SearchOverlay, } from '../search/search-overlay.ts';
import { createEditorWsClient, } from '../ws/client.ts';
import { bootSession, } from './boot.ts';
import { dispatchContextAction, } from './context-actions.ts';
import {
  type AppState,
  type CurrentFileStateAccess,
  wireFileWatching,
  wireSelectEvents,
} from './events.ts';
import { loadFile, } from './file-loader.ts';
import { wireFullscreen, } from './fullscreen.ts';
import { wireKeybindings, } from './keybindings.ts';
import { wireLsp, } from './lsp.ts';

/**
 * Tagged logger for the app.
 */
const appLog = tagged({
  tag: 'app',
  l,
},);
/**
 * URL query parameters from the page URL.
 */
const params = new URLSearchParams(globalThis.location
  .search,);
/**
 * Auth token passed by editord on startup.
 */
const token = nonNullishOrThrow(params.get('token',),);
/**
 * File path to open, passed as `?file=...` query parameter.
 */
const filePath = params.get('file',);
/**
 * Port derived from the current page origin.
 */
const { port, } = globalThis.location;
/**
 * WebSocket client instance.
 */
const ws = createEditorWsClient({
  port,
  token,
},);
/**
 * App container element.
 */
const appElement = nonNullishOrThrow(document.querySelector<HTMLElement>('#app',),);

// oxlint-disable typescript-eslint/no-unsafe-type-assertion -- custom elements registered via define
/**
 * File tree sidebar component.
 */
const fileTree = document.createElement('file-tree',) as FileTree;
/**
 * Contenteditable text editor component.
 */
const editorPane = document.createElement('editor-pane',) as EditorPane;
/**
 * Modal search dialog component.
 */
const searchOverlay = document.createElement('search-overlay',) as SearchOverlay;
/**
 * Hover tooltip component.
 */
const hoverPopup = document.createElement('hover-popup',) as HoverPopup;
/**
 * Autocomplete popup component.
 */
const completionPopup = document.createElement('completion-popup',) as CompletionPopup;
/**
 * References popup component.
 */
const referencesPopup = document.createElement('references-popup',) as ReferencesPopup;
/**
 * Rename input component.
 */
const renameInput = document.createElement('rename-input',) as RenameInput;
/**
 * Binary/media file viewer component.
 */
const binaryViewer = document.createElement('binary-viewer',) as BinaryViewer;
// oxlint-enable typescript-eslint/no-unsafe-type-assertion

fileTree.fetchDir = async function fetchDir(path: string,): Promise<readonly DirEntry[]> {
  /**
   * Directory listing returned by the server; surface goes back as the fetchDir result.
   */
  const { entries, } = await ws.request({
    type: 'listDir',
    path,
  },);
  return entries;
};
fileTree.onContextAction = function handleContextAction(action: ContextAction,): void {
  void dispatchContextAction({
    action,
    ws,
  },);
};
/**
 * Returns the search scope: selected directory if any, otherwise the root.
 *
 * @returns absolute directory path to scope the search within
 */
function resolveSearchScope(): string {
  return fileTree.selectedDir
    !== '' ? fileTree.selectedDir : ws.rootDir;
}
searchOverlay.getRootDir = function getScope(): string {
  return resolveSearchScope();
};
searchOverlay.onSearch = async function handleSearch(
  query: string,
): Promise<readonly SearchResult[]> {
  /**
   * Directory the search is scoped to; either the file-tree selection or the project root.
   */
  const scope = resolveSearchScope();
  /**
   * Search hits returned by the server for the given query and scope.
   */
  const { results, } = await ws.request({
    type: 'search',
    query,
    scope,
  },);
  return results;
};

appElement.append(
  fileTree,
  editorPane,
  binaryViewer,
  searchOverlay,
  hoverPopup,
  completionPopup,
  referencesPopup,
  renameInput,
);
wireFullscreen({ appElement, },);

/**
 * Mutable app state shared with event handlers.
 */
const state: AppState = {
  currentFilePath: filePath,
  currentFileKind: 'text',
};

/**
 * Reads currently open file path for modules that should not mutate app state.
 *
 * @returns current file path, or null when no file is open
 *
 * @example
 * ```ts
 * const path = getCurrentFilePath();
 * ```
 */
function getCurrentFilePath(): string | null {
  return state.currentFilePath;
}

/**
 * Updates currently open file path while keeping state ownership in this module.
 *
 * @param path - next current file path, or null when no file is open
 *
 * @example
 * ```ts
 * setCurrentFilePath('/tmp/app.ts');
 * ```
 */
function setCurrentFilePath(path: string | null,): void {
  state.currentFilePath = path;
}

/**
 * Reads the kind of the currently open file after load operations update it.
 *
 * @returns current file kind used to gate text-only features
 *
 * @example
 * ```ts
 * const kind = getCurrentFileKind();
 * ```
 */
function getCurrentFileKind(): FileKind {
  return state.currentFileKind;
}

/**
 * Current-file capability passed to modules that should not own app state.
 */
const currentFileState: CurrentFileStateAccess = {
  getCurrentFilePath,
  setCurrentFilePath,
  getCurrentFileKind,
};

/**
 * Tracks recently opened files for recency markers in the file tree.
 */
const recentFiles = createRecentFiles();

/**
 * Records a file open in the recency tracker and reveals the file in the tree.
 *
 * @param path - absolute file path that was opened
 */
function recordFileOpen(path: string,): void {
  recentFiles.push(path,);
  fileTree.updateRecency({ paths: recentFiles.paths, },);
  void fileTree.revealFiles({ paths: [path,], },);
}

/**
 * Loads a file and updates the current file state.
 */
async function loadFileSafe(
  {
    path,
    line,
    character,
  }: {
    readonly path: string;
    readonly line?: number | undefined;
    readonly character?: number | undefined;
  },
): Promise<void> {
  /**
   * Loaded file's category (`text` vs binary variants); null when the load was rejected.
   */
  const kind = await loadFile({
    ws,
    editorPane,
    binaryViewer,
    token,
    path,
    line,
    character,
  },);
  if (kind !== null)
    state.currentFileKind = kind;
}

/**
 * LSP feature callbacks returned from wiring.
 */
const {
  formatDocument,
  requestCompletions,
  refreshInlayHints,
  gotoDefinitionAtCursor,
  expandSelection,
  shrinkSelection,
  renameAtCursor,
} = wireLsp({
  ws,
  editorPane,
  hoverPopup,
  completionPopup,
  referencesPopup,
  renameInput,
  getCurrentFilePath,
  loadFileSafe,
},);

wireSelectEvents({
  fileTree,
  searchOverlay,
  referencesPopup,
  currentFileState,
  recordFileOpen,
  loadFileSafe,
  refreshInlayHints,
},);

/**
 * Saves the current editor content to the server. Skips non-text files.
 */
async function saveCurrentFile(): Promise<void> {
  if ((state.currentFilePath
    === null) || (state.currentFileKind
      !== 'text'))
    return;
  try {
    await ws.request({
      type: 'save',
      path: state.currentFilePath,
      content: editorPane
        .getText(),
    },);
  }
  catch (error) {
    appLog.error(`save failed: ${String(error,)}`,);
  }
}

editorPane.addEventListener(
  'contentchange',
  createDebounced({
    fn: function autoSave() {
      void saveCurrentFile();
    },
    delayMs: AUTO_SAVE_DEBOUNCE_MS,
  },)
    .debounced,
);

/**
 * Reveals a file in the tree, scrolls to it, loads it, and refreshes hints.
 *
 * @param path - absolute file path to reveal and load
 */
async function revealAndLoadFile({ path, }: { readonly path: string; },): Promise<void> {
  await fileTree.revealFiles({ paths: [path,], },);
  fileTree.scrollToFile({ path, },);
  await loadFileSafe({ path, },);
  if (state.currentFileKind
    === 'text')
    refreshInlayHints();
}

wireKeybindings({
  saveCurrentFile: function save() {
    void saveCurrentFile();
  },
  formatDocument: function format() {
    void formatDocument();
  },
  gotoDefinition: gotoDefinitionAtCursor,
  renameAtCursor,
  deleteCurrentLine: function deleteLine() {
    performDeleteLine({ pane: editorPane, },);
  },
  selectAndCopyCurrentLine: function copyLine() {
    return performSelectAndCopy({ pane: editorPane, },);
  },
  indentLines: function indent() {
    performIndent({ pane: editorPane, },);
  },
  unindentLines: function unindent() {
    performUnindent({ pane: editorPane, },);
  },
  duplicateLineDown: function duplicate() {
    performDuplicateLine({ pane: editorPane, },);
  },
  swapLineDown: function swapDown() {
    performSwapDown({ pane: editorPane, },);
  },
  swapLineUp: function swapUp() {
    performSwapUp({ pane: editorPane, },);
  },
  openTerminalAtCurrentFile: function openTerminal() {
    /**
     * Directory to spawn the terminal in: the open file's parent, or the project root when nothing is open.
     */
    const dir = state.currentFilePath
      !== null
      ? state.currentFilePath
        .slice(
        0,
        state.currentFilePath
          .lastIndexOf('/',),
      )
      : ws.rootDir;
    void ws.request({
      type: 'openInTerminal',
      path: dir,
    },);
  },
  requestCompletions,
  expandSelection,
  shrinkSelection,
  navigateToRecentFile: function navigateToRecent(index: number,) {
    /**
     * Path at the given recency slot; undefined when the user requested a slot the history has not filled yet.
     */
    const path = recentFiles.paths[index];
    if (path === undefined)
      return;
    setCurrentFilePath(path,);
    recentFiles.push(path,);
    fileTree.updateRecency({ paths: recentFiles.paths, },);
    void revealAndLoadFile({ path, },);
  },
  completionPopup,
  referencesPopup,
  hoverPopup,
},);

wireFileWatching({
  ws,
  fileTree,
  getCurrentFilePath,
  loadFileSafe,
},);

await bootSession({
  ws,
  editorPane,
  fileTree,
  searchOverlay,
  currentFileState,
  recentFiles,
  loadFileSafe,
  refreshInlayHints,
  queryFilePath: filePath,
},);
