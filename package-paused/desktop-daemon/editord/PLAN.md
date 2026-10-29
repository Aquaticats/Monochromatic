# editord implementation plan

## Phase 1; Edit and save a file (done)

- [x] h3 server on port 4400 with token auth
- [x] Static file serving for index.html and dist/client/
- [x] WebSocket via crossws with `open` and `save` operations
- [x] `<editor-pane>` web component: contenteditable, per-line `<div>`s, paste-as-plain-text
- [x] Client app: connect WS, open file from `?file=` param, Ctrl+S saves
- [x] Client bundle via tsdown (fully self-contained, ~15KB)
- [x] Dark theme (#ccc on #000) in index.html

### Known rough edges from phase 1

- ~~`--editor-padding` CSS variable not defined~~ (fixed: defined as `0.5rem`)
- ~~Errors from WS operations surface as uncaught promise rejections~~ (fixed: try/catch with tagged logging)
- ~~`setText`/`getText` on `<editor-pane>` require runtime cast~~
  (fixed: `EditorPane` class exported, typed cast in app.ts)
- ~~File paths resolve relative to server cwd, which may differ from user expectation~~
  (fixed: path containment via `assertWithinRoot` against `rootDir`)
- No visual feedback on save (logged via tagged logger, no UI indicator)
- No favicon, no PWA manifest yet

## Phase 2; File tree and navigation (done)

- [x] `resolveRoot` server operation: walk up to highest writable ancestor directory
- [x] `listDir` server operation: `fs.readdir` with `{ withFileTypes: true }`, sorted entries
- [x] `<file-tree>` web component: native `<details><summary>` expand/collapse, lazy-loading, click to open
- [x] Layout: file tree as a left sidebar, editor pane fills remaining space
- [x] Root directory sent to client on WebSocket connection
- [x] One-level-ahead preloading of subdirectory contents

## Phase 2.5; Hardening (done)

- [x] Path containment: `assertWithinRoot` validates all file operations against `rootDir`
- [x] Shared protocol types: `src/protocol.ts` eliminates type duplication between server and client
- [x] Tagged loggers: all server and client modules use `tagged()` from `@monochromatic-dev/logger-core`
- [x] Named parameters: all functions with 2+ params use destructured object parameters
- [x] WebSocket close handling: pending requests rejected on disconnect (prevents promise leaks)
- [x] Error propagation: fire-and-forget async calls wrapped with try/catch and tagged logging
- [x] Cross-runtime: `Bun.file()` replaced with `readFile` from `node:fs/promises`
- [x] Static serving: silent catch narrowed to ENOENT only; unexpected errors rethrow

## Phase 3; Search (done)

- [x] `search` server operation: spawn `rg` subprocess, return `SearchResult[]` (file-path + content matches)
- [x] Double-shift detection: track Shift keyup timestamps, trigger on <400ms gap with no intervening keys
- [x] `<search-overlay>` web component: `<dialog>` modal with input, scrollable results, keyboard navigation
- [x] File-path results listed before content results; `%` prefix for content-only mode
- [x] `EditorPane.scrollToLine`: content search results open the file at the matching line

## Phase 4; File watching and themes (done)

- [x] `DirWatcher` class: per-directory non-recursive watcher, initially `fs.watch`, superseded by chokidar in Phase 13
- [x] Event classification from watcher event kind, initially stat-after-debounce, superseded by chokidar
- [x] Save suppression: `suppressPath` ignores self-triggered events for 500ms
- [x] Ignore patterns: `.git`, `node_modules`, `.DS_Store`, swap files, temp files
- [x] `watchDir` client notification registers directories for watching
- [x] `fileChanged` server push with `path`, `changeType`, `isDirectory`
- [x] Light theme (#444 on #fff) via `data-theme="light"` attribute on `:root`
- [x] `--editor-padding` CSS variable defined as `0.5rem`

## Phase 5; Syntax highlighting (done)

- [x] Lezer parser integration: `@lezer/javascript` with TypeScript/JSX dialect configuration
- [x] `getParserForPath` extension-to-parser mapping: JS/TS, JSON/JSONL, CSS, HTML, Markdown, YAML, TOML
- [x] CSS Custom Highlight API: token ranges applied via `CSS.highlights` without DOM mutation
- [x] `::highlight()` CSS rules in shadow DOM stylesheet, colors via CSS custom properties
- [x] Dark theme colors (One Dark inspired) and light theme colors in `index.html`
- [x] `requestAnimationFrame` batching: coalesces rapid edits into one parse-and-highlight pass
- [x] 100KB file size limit: files over 100KB skip highlighting entirely
- [x] Tag groups: keyword, string, comment, number, type, function, property, heading, link, emphasis

## Phase 6; LSP integration (done)

- [x] JSON-RPC framing over stdio for LSP communication
- [x] `LspClient` class: spawn, initialize, request/response, notification routing
- [x] `LspManager` coordinator: oxlint (linting), tsgo (types/hover/completions), dprint (formatting)
- [x] Document sync: `didOpen`, `didChange` (debounced full-text), `didSave`, `didClose`
- [x] `DiagnosticStore`: multi-source aggregation (URI -> source -> diagnostics[])
- [x] `document-sync.ts`: routing notifications to relevant servers by language ID
- [x] `find-project-root.ts`: walk-up config file search for LSP `rootUri`, cached
- [x] Diagnostic aggregation: merge diagnostics from oxlint + tsgo, push to client
- [x] Diagnostic underlines via CSS Custom Highlight API (`::highlight(diag-error)`, etc.)
- [x] Hover info from tsgo: debounced mouse tracking, `<hover-popup>` tooltip
- [x] Completions from tsgo: Ctrl+Space / dot trigger, `<completion-popup>` dropdown
- [x] Formatting from dprint: Ctrl+Shift+F / Ctrl+Alt+L (JetBrains parity)
- [x] Go-to-definition from tsgo: Ctrl+Click and Ctrl+B navigate to definition location
- [x] Find references from tsgo: Ctrl+B falls back to references when already at definition;
      single result navigates directly, multiple results show `<references-popup>`
- [x] Graceful degradation: servers that fail to start are skipped; features degrade
- [x] Late-init recovery: documents opened before LSP ready are re-opened after init

## Phase 7; Inlay hints (done)

- [x] Wire protocol: `inlayHint` request with range, `inlayHintResult` response with hints
- [x] Server: `requestInlayHints` via `textDocument/inlayHint` to tsgo
- [x] Server: `initializationOptions.userPreferences.inlayHints` enables all hint categories in tsgo
- [x] Client: `inlay-layer.ts` groups hints + diagnostics by line, sets `data-inlay` attributes
- [x] Client: `::before` pseudo-element renders hints above code lines with pill background
- [x] Client: line numbers moved from `::before` to `::after` to free `::before` for hints
- [x] Client: `--line-num-offset` measured via `getComputedStyle(div, '::before').height`
      in a follow-up rAF so line numbers align with code even when hints wrap
- [x] Client: `--inlay-indent` CSS custom property for column-aligned hint positioning
- [x] Client: parameter hints strip trailing colon for cleaner display
- [x] Client: severity-colored backgrounds for diagnostic messages in annotations
- [x] Client: hints soft-wrap via `white-space: pre-wrap`
- [x] Debounced refresh on content changes (750ms), immediate on file open

## Phase 8; Binary file viewer (done)

- [x] `FileKind` type: `'text' | 'image' | 'audio' | 'video' | 'binary'`
- [x] `file-kind.ts`: extension-based detection for images, audio, video; SVG excluded (editable text)
- [x] `hex-dump.ts`: xxd-style format, 16 bytes/line, grouped as two 8-byte halves,
      ASCII on right, truncated at 16 384 bytes with footer
- [x] `<binary-viewer>` web component: four modes; `<img>`, `<audio controls>`, `<video controls>`, `<pre>` hex dump
- [x] `fileContent` response includes `kind` field for viewer routing
- [x] Editor pane hidden when binary viewer is active, and vice versa

## Phase 9; File tree context menu and filesystem operations (done)

- [x] `<context-menu>` class: Popover API with CSS anchor positioning and `position-try-fallbacks`
- [x] Button items and inline input items (for rename/copy/move with pre-filled names)
- [x] Light dismiss via browser-native popover behavior
- [x] Server operations: `deleteEntry`, `copyEntry`, `moveEntry`, `newEntry`
- [x] Server operations: `openInTerminal` (xdg-terminal-exec fallback chain), `openInDefaultApp` (xdg-open)
- [x] `fsActionDone` response for all filesystem mutations
- [x] `app-context-actions.ts` bridges context menu actions to WebSocket requests

## Phase 10; Selection ranges (done)

- [x] Wire protocol: `selectionRange` request with positions, `selectionRangeResult` with nested chains
- [x] Server: `textDocument/selectionRange` forwarded to tsgo
- [x] Client: `expandSelection` walks the chain to find the next larger enclosing range
- [x] Client: `shrinkSelection` walks the chain to find the largest strictly smaller range
- [x] Ctrl+W expands, Ctrl+Shift+W shrinks; stateless (fresh chain per invocation)

## Phase 11; Editor line operations (done)

- [x] Ctrl+C copies entire current line when no text is selected
- [x] Ctrl+D duplicates current line down (clones DOM node, repositions cursor)
- [x] Ctrl+Shift+Up / Ctrl+Shift+Down swaps current line with adjacent line
- [x] Tab / Shift+Tab indents / unindents current line or selection (2-space indent unit)
- [x] Fullscreen keyboard lock: FAB enters fullscreen, locks Ctrl+W from browser

## Phase 12; Infrastructure (done)

- [x] `resolve-fs-id.ts`: stable filesystem volume ID (Linux `stat -f`, macOS `stat -f %v`, Windows `vol`)
- [x] `connected` message includes `fsId` for localStorage key stability across mounts
- [x] `file-tree-order.ts`: CSS `order`-based alphabetical sorting (first 4 chars -> base-128 int)
- [x] `file-tree-entry.styles.ts`: extracted entry styles with custom disclosure arrows
- [x] `completion-popup-render.ts`: extracted DOM rendering helpers for completion items

## Phase 13; Save durability and watcher migration (done)

- [x] `write-file-atomic.ts`: temp + fsync + rename helper,
  `O_NOFOLLOW`/`O_EXCL` on the temp open,
  `lstat`-and-refuse on symlinked targets (`ELOOP`),
  `fchmod` to preserve mode
- [x] `save.ts`, `apply-workspace-edit.ts`, `new-entry.ts` migrated to `writeFileAtomic`
- [x] `DirWatcher` (`watch-filesystem.ts`) reimplemented on chokidar 5
  with `depth: 0`, `atomic: true`, `awaitWriteFinish`;
  public API (`watchDir`, `suppressPath`, `close`, `FsChangeEvent`) preserved
- [x] Orphan `.*.editord.*~` temp sweep on first `watchDir` of each directory
- [x] `dirWatcher.suppressPath` added for the previously-missing fs-action paths
  (apply-workspace-edit, new-entry, move-entry source+dest, delete-entry, copy-entry)

Verification procedure for end-to-end checks: `HANDOVER.chokidar-atomic-migration.md`.

## Future (post-MVP)

- Completion item kind icons
- Signature help on function calls
- Code actions (quick fixes from oxlint/tsgo)
- Workspace diagnostics (diagnostics for files not currently open)
- PWA manifest and service worker for installability
- Theme toggle keybinding
