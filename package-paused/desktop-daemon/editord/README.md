# editord

Local editor daemon that serves a contenteditable PWA frontend over WebSocket.

## Browser baseline

Chromium desktop latest.
editord runs as a PWA in real Chrome, not Electron,
so it targets only the latest stable Chromium release.

## Motivation

Smooth scrolling in existing editors is significantly worse than Neovide:

- **WebStorm**: JVM/Swing rendering with GC-induced frame drops and poor GPU utilization
- **VSCode**: Monaco calls `preventDefault()` on wheel events, opting out of Chromium's compositor-driven scrolling.
  Reimplements scrolling in JS due to line virtualization.
  Electron does not expose Chrome's smooth scrolling flag, so the compositor optimization is unavailable even in theory.
- **Neovide**: Purpose-built Rust render loop using Skia, physics-based scroll animation, runs at vsync with no GC

editord sidesteps these issues by running the editor UI as a PWA in real Chromium (not Electron),
using raw `contenteditable` with per-line `<div>` elements and native CSS overflow scrolling.
The browser's compositor thread owns scroll entirely: no `preventDefault`, no JS scroll reimplementation.

## Architecture

````text
editord (Bun + h3)              Chromium PWA
+-----------------------+       +---------------------------+
| HTTP: serve index.html|       | <editor-pane>             |
|       serve dist/     |       |   contenteditable         |
|       serve media/*   |       |   one <div> per line      |
|                       |       |   diagnostic underlines   |
| WebSocket:            | <---> |   inlay hints (::before)  |
|   open/save/listDir   |       |   browser-native:         |
|   search(query)       |       |     undo/redo, selection  |
|   hover/completion    |       |                           |
|   format/gotoDefn     |       | <binary-viewer>           |
|   findReferences      |       |   image/audio/video/hex   |
|   inlayHint(range)    |       |                           |
|   selectionRange      |       | <file-tree>               |
|   didChange (sync)    |       |   context menu (CRUD)     |
|   diagnostics (push)  |       |   CSS order sorting       |
|   deleteEntry/copy/   |       |   directory watching      |
|     move/new          |       |                           |
|   openInTerminal      |       | <search-overlay>          |
|   openInDefaultApp    |       | <hover-popup>             |
|   watchDir            |       | <completion-popup>        |
|   fileChanged (push)  |       | <references-popup>        |
|                       |       |                           |
| LSP servers:          |       |                           |
|   oxlint --lsp        |       |                           |
|   tsgo --lsp --stdio  |       |                           |
|   dprint lsp          |       |                           |
+-----------------------+       +---------------------------+
```text

**Frontend owns the canonical buffer.**
The DOM contenteditable element is the text state.
editord is a file I/O and search service: no OT, no CRDT, no sync protocol.
Full text round-trips on open/save.

## Key design decisions

- **Real Chrome, not Electron**: access to `chrome://flags/#smooth-scrolling` and all compositor optimizations;
  Chrome updates independently
- **Raw contenteditable**: browser handles keystroke-to-render natively;
  editord is notified asynchronously for persistence
- **No virtualization**: entire file rendered in DOM.
  Loading a huge file is a user error.
  This means normal scrolls are pure GPU layer translations on the compositor thread.
- **Per-line `<div>` elements**: natural for contenteditable (Enter creates new divs, Backspace merges them);
  enables `children[n]` indexing for go-to-line
- **Native `<details><summary>`**: file tree directories use browser-native expand/collapse;
  JS only handles lazy-loading on first expand and one-level-ahead preloading
- **CSS `order` sorting**: file tree entries are sorted alphabetically via CSS `order` integers
  derived from the first four characters of the filename, avoiding DOM re-insertion
- **Web components with shadow DOM**: encapsulated styling via h-css, DOM construction via h-dom
- **Shared protocol types**: `src/protocol.ts` defines all wire types once;
  both server and client import from the same module
- **Path containment**: all filesystem operations validate paths against the root directory
  via `assertWithinRoot`, preventing traversal even with a valid auth token
- **Tagged loggers**: all server and client modules use structured tagged logging
  from `@monochromatic-dev/logger-core`
- **WebSocket with token auth**: token generated per-session via `crypto.randomUUID()`,
  passed as URL query param
- **Recent files with recency markers**: file tree shows numbers 0 (current) through 9 (oldest)
  in the toggle column; Ctrl+0..9 navigates to the Nth recent file, auto-expanding ancestor
  directories with scroll anchoring to keep the user's view stable; list persists across sessions
- **JetBrains keymap**: double-shift for Search Everywhere (replaces command palette)
- **Inlay hints via `::before`**: type annotations and parameter names rendered as `::before`
  pseudo-elements on line divs, with line numbers moved to `::after` to free up `::before`;
  `--line-num-offset` measured via `getComputedStyle` in a follow-up rAF so line numbers
  stay aligned with code even when hints wrap across multiple visual lines
- **Two themes**: dark (#ccc on #000), light (#444 on #fff); toggled via `data-theme` attribute
- **Binary file viewer**: non-text files detected by extension (`FileKind`);
  images, audio, and video render in native `<img>`, `<audio>`, `<video>` elements;
  unknown binaries display a hex dump (16 bytes/line, truncated at 16 384 bytes)
- **Context menu via Popover API**: right-click on file tree entries opens a context menu
  using `popover="auto"` with CSS anchor positioning and `position-try-fallbacks`
  for viewport edge detection; supports inline input items for rename/copy operations
- **Directory watching**: chokidar per expanded directory (`depth: 0`, `atomic: true`,
  `awaitWriteFinish` 150 ms), pushes `fileChanged` events to the client; save operations
  suppress self-triggered events for 500 ms; ignores `.git`, `node_modules`, swap files,
  and temp files
- **Filesystem volume ID**: `connected` message includes `fsId` (filesystem volume identifier
  from `stat -f`) so localStorage keys are stable per volume, not per mount path
- **Diagnostic store**: multi-source diagnostic aggregation; each LSP server (oxlint, tsgo)
  publishes independently without overwriting the other's diagnostics for the same file
- **Selection ranges**: LSP `textDocument/selectionRange` for syntactic expand/shrink selection;
  Ctrl+W expands to the next larger enclosing scope, Ctrl+Shift+W shrinks back
- **Find references with fallback**: Ctrl+B tries go-to-definition first;
  if already at the definition, falls through to find references;
  single reference navigates directly, multiple references show a popup
- **Fullscreen keyboard lock**: floating action button enters fullscreen mode and calls
  `navigator.keyboard.lock(['KeyW'])` to capture Ctrl+W from the browser

## Running

```sh
mise run //packages/desktop-daemon/editord:start:server
````

Open the printed URL in Chrome.
Add `&file=path/to/file` to open a specific file.

For development with watch mode:

```sh
mise run //packages/desktop-daemon/editord:dev
```

## Project structure

```text
src/
  protocol.ts                  -- shared wire types (ClientMessage, ServerMessage, DirEntry, FileKind, etc.)
  server/
    index.ts                   -- entry point: h3 app, static serving, WebSocket, token auth
    log.ts                     -- root tagged logger for server subsystems
    ws.ts                      -- WebSocket handler: auth, message dispatch
    operations/
      assert-within-root.ts    -- path containment guard against traversal
      copy-entry.ts            -- copy file or directory within root
      delete-entry.ts          -- delete file or directory within root
      file-kind.ts             -- extension-to-FileKind detection (image, audio, video, binary, text)
      hex-dump.ts              -- xxd-style hex dump for binary file preview (16 bytes/line, 16KB cap)
      list-dir.ts              -- list directory entries, sorted dirs-first
      move-entry.ts            -- rename or move file/directory within root
      new-entry.ts             -- create new empty file or directory within root
      open-external.ts         -- cross-platform xdg-open/terminal launch (detached process)
      open.ts                  -- read file from disk, determine FileKind
      resolve-fs-id.ts         -- filesystem volume ID via stat for localStorage keying
      resolve-root.ts          -- find highest writable ancestor directory
      save.ts                  -- write file to disk
      search.ts                -- dual rg search: file paths + content matches, concurrent
      stream-rg.ts             -- streaming rg subprocess helper with early kill on limit
      watch-filesystem.ts      -- per-directory chokidar watcher with atomic-save handling and suppression
      watch-filesystem-filter.ts -- ignore patterns and timing constants for directory watcher
    lsp/
      types.ts                 -- LSP type definitions (Position, Diagnostic, Hover, InlayHint, etc.)
      json-rpc.ts              -- Content-Length framing for LSP stdio communication
      lsp-client.ts            -- single LSP server process manager (spawn, init, request/notify)
      lsp-pool.ts              -- lazy pool of LSP clients keyed by (type, projectRoot)
      lsp-features.ts          -- request handlers (hover, completion, format, definition, references, inlayHint, selectionRange)
      language-id.ts           -- file extension to LSP language ID mapping
      lsp-manager.ts           -- multi-server coordinator (oxlint, tsgo, dprint)
      diagnostic-store.ts      -- multi-source diagnostic aggregation (URI -> source -> diagnostics)
      document-sync.ts         -- LSP document lifecycle (didOpen, didChange, didSave, didClose) routing
      find-project-root.ts     -- walk-up config file search for LSP rootUri, cached
  client/
    index.html                 -- PWA shell with dark/light theme custom properties
    app.ts                     -- entry: connect WS, mount components, recent files, session restore
    log.ts                     -- root tagged logger for client subsystems
    editor-pane.ts             -- <editor-pane> web component: contenteditable, paste, highlight, line operations
    editor-pane.styles.ts      -- shadow DOM core layout styles (host, editor, line divs, line numbers)
    highlight-styles.ts        -- ::highlight() CSS rules for syntax tokens and diagnostic underlines
    inlay-styles.ts            -- ::before CSS rules for inlay hint pills and severity variants
    file-tree.ts               -- <file-tree> web component: <details> expand, lazy-load, preload, recency, context menu
    file-tree.styles.ts        -- shadow DOM styles for file tree (incl. recency number opacity)
    file-tree-entry.styles.ts  -- shadow DOM styles for file entries (summary layout, disclosure arrows)
    file-tree-order.ts         -- CSS order-based alphabetical sorting (first 4 chars -> base-128 int)
    recent-files.ts            -- ordered tracker for 10 most recently opened file paths
    session-state.ts           -- session persistence to localStorage (file, cursor, scroll, recent files)
    app-session.ts             -- wires save triggers and restores session state on boot
    app-keybindings.ts         -- global keyboard shortcut handler (Ctrl+S, Ctrl+B, Ctrl+D, Tab, etc.)
    app-context-actions.ts     -- bridges context menu actions to WebSocket filesystem requests
    app-fullscreen.ts          -- fullscreen FAB with keyboard lock (Ctrl+W capture)
    app-lsp.ts                 -- LSP feature coordinator: wires all LSP subsystems together
    app-lsp-actions.ts         -- format document and go-to-definition with references fallback
    app-lsp-completions.ts     -- Ctrl+Space / dot-triggered completion requests
    app-lsp-hover.ts           -- debounced mousemove hover with popup positioning
    app-lsp-inlay.ts           -- debounced inlay hint refresh on content changes
    app-lsp-selection.ts       -- LSP-backed expand/shrink selection (Ctrl+W / Ctrl+Shift+W)
    highlighter.ts             -- syntax highlighting: Lezer parse, offset-to-Range mapping, CSS.highlights
    highlight-tags.ts          -- Lezer tag-to-highlight-group mapping (keyword, string, comment, etc.)
    languages.ts               -- file extension to Lezer parser mapping (JS/TS dialects)
    ws-client.ts               -- typed WebSocket client with request/response correlation
    position.ts                -- DOM selection to text position conversion utilities
    diagnostics-layer.ts       -- CSS Custom Highlight API for diagnostic underlines
    inlay-layer.ts             -- applies data-inlay attributes to line divs from hints + diagnostics
    inlay-line.ts              -- per-line annotation assembly (grouping, formatting, severity)
    inlay-format.ts            -- hint/diagnostic label formatting and severity ranking
    inlay-fetch.ts             -- fetches inlay hints from server via textDocument/inlayHint
    inlay-measure.ts           -- measures ::before height for line number offset alignment
    binary-viewer.ts           -- <binary-viewer> web component: image/audio/video/hex display
    binary-viewer.styles.ts    -- shadow DOM styles for binary viewer
    hover-popup.ts             -- <hover-popup> web component: type info tooltip
    hover-popup.styles.ts      -- shadow DOM styles for hover popup
    completion-popup.ts        -- <completion-popup> web component: autocomplete dropdown
    completion-popup.styles.ts -- shadow DOM styles for completion popup
    completion-popup-render.ts -- DOM rendering helpers for completion item list
    references-popup.ts        -- <references-popup> web component: find-references list
    references-popup.styles.ts -- shadow DOM styles for references popup
    context-menu.ts            -- context menu using Popover API with CSS anchor positioning
    search-overlay.ts          -- <search-overlay> web component: project-wide ripgrep search
    search-overlay.styles.ts   -- shadow DOM styles for search overlay
    toast.ts                   -- transient status message near the cursor
    char-from-point.ts         -- DOM caretPositionFromPoint/caretRangeFromPoint cross-browser wrapper
    position-from-point.ts     -- maps viewport coordinates to line/character position
    middle-out.ts              -- middle-out range expansion for visible-first rendering
```

## WebSocket protocol

All messages are JSON with a `type` discriminant.
Client requests include a client-generated `id` for response correlation.
Notifications have no `id` and expect no response.

**Client to server (requests):**

- `{ type: "open", id, path }`: read file, responds with `fileContent`
- `{ type: "save", id, path, content }`: write file, responds with `saved`
- `{ type: "listDir", id, path }`: list directory, responds with `dirListing`
- `{ type: "search", id, query, scope }`: ripgrep search scoped to a directory
- `{ type: "hover", id, path, line, character }`: request hover info
- `{ type: "completion", id, path, line, character }`: request completions
- `{ type: "format", id, path }`: request document formatting
- `{ type: "gotoDefinition", id, path, line, character }`: request go-to-definition
- `{ type: "findReferences", id, path, line, character }`: request find-references
- `{ type: "inlayHint", id, path, range }`: request inlay hints for a range
- `{ type: "selectionRange", id, path, positions }`: request expand/shrink selection chain
- `{ type: "deleteEntry", id, path }`: delete file or directory
- `{ type: "copyEntry", id, path, destPath }`: copy file or directory
- `{ type: "moveEntry", id, path, destPath }`: rename or move file/directory
- `{ type: "newEntry", id, parentPath, name, isDirectory }`: create new file or directory
- `{ type: "openInTerminal", id, path }`: launch terminal at directory
- `{ type: "openInDefaultApp", id, path }`: open file in default application

**Client to server (notifications):**

- `{ type: "didChange", path, content }`: notify LSP servers of content change (no response)
- `{ type: "didClose", path }`: notify LSP servers file was closed (no response)
- `{ type: "watchDir", path }`: register a directory for filesystem change notifications

**Server to client:**

- `{ type: "connected", rootDir, fsId }`: handshake with root directory and filesystem volume ID
- `{ type: "fileContent", id, path, content, kind }`: file read result with `FileKind` for viewer routing
- `{ type: "saved", id, path }`: write confirmation
- `{ type: "dirListing", id, path, entries }`: directory listing with `{ name, isDirectory }` entries
- `{ type: "searchResults", id, results }`: search results (file-path and content matches)
- `{ type: "fileChanged", path, changeType, isDirectory }`: push notification for filesystem changes
- `{ type: "diagnostics", path, diagnostics }`: push diagnostics from LSP servers
- `{ type: "hoverResult", id, contents, range? }`: hover info response
- `{ type: "completionResult", id, items }`: completion items response
- `{ type: "formatResult", id, edits }`: formatting text edits response
- `{ type: "definitionResult", id, path, line, character }`: go-to-definition response
- `{ type: "referencesResult", id, locations }`: find-references response with path + position
- `{ type: "inlayHintResult", id, hints }`: inlay hints with position, label, kind
- `{ type: "selectionRangeResult", id, ranges }`: nested selection range chains
- `{ type: "fsActionDone", id }`: confirmation for filesystem mutations (delete/copy/move/new/open)
- `{ type: "error", id?, message }`: error response

## JetBrains parity

Any behavioral difference between editord and JetBrains IDEs is considered a bug.
JetBrains is the reference implementation for keybindings, search UX,
file tree interactions, and general editor behavior.

## LSP integration

editord proxies three language servers over JSON-RPC/stdio:

- **oxlint** (`oxlint --lsp`): linting diagnostics for JS/TS
- **tsgo** (`tsgo --lsp --stdio`):
  type diagnostics, hover, completions, go-to-definition,
  find references, inlay hints, selection ranges for JS/TS
- **dprint** (`dprint lsp`): formatting for JS/TS, JSON, CSS, HTML, Markdown, YAML, TOML

The server aggregates diagnostics from oxlint and tsgo via a `DiagnosticStore`
that maintains per-source diagnostic sets per file, merging them into a single push.
Feature requests (hover, completion, format, definition, references, selectionRange)
are routed to the appropriate server.
Servers that fail to start are skipped; the editor degrades gracefully.

**Keybindings:**

- **Ctrl+S**: save current file
- **Ctrl+Z** / **Ctrl+Shift+Z**: undo / redo
- **Ctrl+Y**: delete current line
- **Ctrl+C**: copy current line when no text is selected
- **Ctrl+D**: duplicate current line down
- **Ctrl+Shift+Up** / **Ctrl+Shift+Down**: swap line up / down
- **Tab** / **Shift+Tab**: indent / unindent current line or selection
- **Ctrl+B**: go to definition (falls back to find references if already at definition)
- **Ctrl+Click**: go to definition at click position
- **Ctrl+Space**: trigger completions
- **Ctrl+Shift+F** / **Ctrl+Alt+L**: format document (JetBrains parity)
- **Ctrl+W**: expand selection to next larger syntactic scope
- **Ctrl+Shift+W**: shrink selection to previous smaller scope
- **Ctrl+0..9**: navigate to recent file by recency index (0 = current, 9 = oldest)
- **Double Shift**: search everywhere (file paths and content)
- **Mouse hover**: show type information (350ms debounce)
- **Escape**: dismiss hover popup, completion popup, or references popup

## Not in MVP

- Minimap, split panes
- Terminal, git, extensions, settings UI
- Multi-window, multi-project, remote development
- Large file handling, virtualization, over-rendering
- Auto-launching Chrome
- PWA manifest and service worker for installability
