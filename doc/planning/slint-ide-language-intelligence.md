# Slint IDE language intelligence design

## Status and purpose

This is the verified design for the Language module of `package/ide/app`,
the third gate of [the implementation plan](slint-ide-implementation.md).
Nothing under `package/` was changed.
Every claim is tied to the pinned Helix source
(revision `ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`, cited as `path:line` inside that tree)
and to the output of a compiled disposable spike that drove `helix-lsp` outside Helix's editor
against real servers on disposable projects.
The spike's location, commands, and key source are in the section "Spike".

Decisions that are not mine to make are collected in "Decisions for the coordinator".

## Most consequential findings

- `helix-lsp` is usable outside the editor.
  One dedicated thread with a current-thread tokio runtime owned `Registry`,
  drove rust-analyzer and the TypeScript 7 native server through all five feature paths,
  and exchanged bounded messages with a polling "interface" thread.
- The Helix default TypeScript server cannot start on this host.
  `typescript-language-server` 6.0.0 bundles no TypeScript and the repository pins TypeScript 7.0.2,
  which ships no `tsserver.js`.
  The server answers `initialize` with an error, keeps running, and `helix-lsp` reports nothing to the embedder.
- TypeScript 7.0.2 has its own server (`tsc --lsp --stdio`).
  Through `helix-lsp` it passed definition, references, hover, pull diagnostics, and reload synchronization.
  Inlay hints need VS Code style settings, not the keys in Helix's `languages.toml`.
- `Registry` derives the LSP root from the process working directory.
  A foreign working directory starts the server in the wrong directory with no root.
  A project inside a larger version-controlled tree can get the enclosing tree as its root.
- A client used before `initialize` completes panics for every request method,
  and a `didOpen` sent before then is silently dropped.
- rust-analyzer's `cargo check` diagnostics are pushed once at start and never again
  unless the client sends `didSave`.
  rust-analyzer also created `Cargo.lock` and `target/` in the disposable project.
- The integration adds no package to `Cargo.lock`.
  It needs two new direct declarations (`arc-swap`, `futures-util`) that already resolve there.

## Host and versions measured

- Helix crates: `helix-lsp`, `helix-core`, `helix-loader` at the pinned revision,
  compiled in `localhost/monochromatic/ide` (cargo 1.98.0, rustc 1.98.0) with the bounds from `[vars].cargo`.
- rust-analyzer `1.100.0-nightly (1303417 2026-09-21)` through the rustup proxy at `~/.cargo/bin/rust-analyzer`,
  selected by `RUSTUP_TOOLCHAIN=nightly-2026-09-22`, with `rust-src` installed.
- `typescript-language-server` 6.0.0 at
  `~/.local/share/mise/installs/npm-typescript-language-server/6.0.0/bin/`.
  Its `package.json` lists `typescript` only under `devDependencies`,
  and the pnpm install beside it contains only `vscode-jsonrpc` and `vscode-languageserver-protocol`.
  Its lookup order is user `tsserver.path`, workspace `node_modules/typescript/lib/tsserver.js`,
  `tsserver.fallbackPath`, then a bundled `typescript` (`lib/cli.mjs` `findTypescriptVersion`).
  No `tsserver.js` exists under the mise installs, the pnpm store links, or the repository's `node_modules/.pnpm`.
- TypeScript 7.0.2 from the repository's store (`lib/tsc.js`, `lib/getExePath.js`, a native `tsc` binary).
  `tsc --lsp --stdio` identifies as `typescript-go` 7.0.2.
- Node 26.10.0.

## Constructing `Registry` and `Client` outside Helix

### What they need

- A language registry: `helix_core::syntax::Loader`, built exactly as `src/syntax.rs` already does,
  from `helix_loader::config::default_lang_config()` (`helix-loader/src/config.rs:6`)
  decoded into `Configuration` (`helix-core/src/syntax/config.rs:17`).
  `Registry::new` takes it as `Arc<ArcSwap<Loader>>` (`helix-lsp/src/lib.rs:590`),
  so the application must name `arc_swap::ArcSwap`, which no Helix crate re-exports.
- A language configuration per file:
  `Loader::language_for_filename` then `Loader::language(..).config()`
  (`helix-core/src/syntax.rs:377`, `:335`, `:61`).
  It supplies `language_servers`, root markers (`roots`), and the LSP `language-id`.
- Server definitions: `Loader::language_server_configs()` (`helix-core/src/syntax.rs:427`),
  read by `Registry::start_client` (`helix-lsp/src/lib.rs:620`).
- A document path, `root_dirs`, and a snippet flag for
  `Registry::get(&LanguageConfiguration, Option<&Path>, &[PathBuf], bool)` (`helix-lsp/src/lib.rs:705`).
  Pass `false` for snippets.
- A tokio runtime context for every one of these calls (see "Runtime requirements").
- `Registry` is mandatory.
  `Client::start` is public (`helix-lsp/src/client.rs:211`),
  but `Client::initialize` and the `capabilities` cell are `pub(crate)` (`client.rs:572`, `:62`),
  and only the free function `start_client` performs the handshake (`lib.rs:946` to `:966`).

### What `helix-view` and `helix-term` supply that the application reimplements

- Starting servers for a document and diffing which servers hold it:
  `Editor::launch_language_servers` (`helix-view/src/editor.rs:1816`).
- Per-document LSP identity: URL, `language_id`, an `i32` version that starts at zero and
  increments per change (`helix-view/src/document.rs:1895`, `:1925`, `:1481`, `:2064`, `:2143`).
  The application's `Document::revision` is `u64` and survives navigation;
  LSP needs a separate `i32` counter per `didOpen`.
- Filtering to initialized servers and to servers that support a feature:
  `Document::language_servers` and `language_servers_with_feature` (`document.rs:1942`, `:1959`),
  using `LanguageServerFeatures::has_feature` (`helix-core/src/syntax/config.rs:381`)
  and `Client::supports_feature` (`client.rs:317`).
- The whole dispatch of server-to-client traffic:
  `Application::handle_language_server_message` (`helix-term/src/application.rs:777` to `:1189`).
- Lifecycle hooks: `didOpen` when a server reports initialized, `didChange`, `didClose`
  (`helix-view/src/handlers/lsp.rs:392`, `:412`, `:428`).
- Diagnostics storage and the version check (`helix-view/src/handlers/lsp.rs:284`, `:296`).
- Pull diagnostics scheduling (`helix-term/src/handlers/diagnostics.rs:162`, `:289`).
- The five commands: goto and references (`helix-term/src/commands/lsp.rs:946`, `:1044`),
  hover (`:1095`), inlay hints (`:1348`), and location conversion (`:74`).
- Shutdown: `Editor::close_language_servers` (`helix-view/src/editor.rs:2440`).
- Workspace trust before starting servers (`helix-view/src/editor.rs:1831`).
  The application loads only the built-in configuration, never a project `.helix/languages.toml`,
  so no project-supplied command runs, unless Option A under "TypeScript server" is adopted,
  whose launcher is a file in the project's `node_modules`.
  Starting a server on a project still runs project code (see "Open risks").

### Runtime requirements

- `Registry::new` panics outside a runtime.
  Spike case `no-runtime`:
  `there is no reactor running, must be called from the context of a Tokio 1.x runtime`,
  raised at `helix-lsp/src/file_event.rs:56` where the file-event task is spawned.
- `Registry::get` spawns the child with `tokio::process::Command` (`client.rs:230`),
  three transport tasks (`helix-lsp/src/transport.rs:85` to `:98`), and the initialize task (`lib.rs:946`).
- Nothing in `helix-lsp/src` calls `spawn_blocking`, `block_in_place`, or `Handle::current`.
  Its only blocking export is the re-exported `futures_executor::block_on` (`lib.rs:9`), which the design never calls.
- A current-thread runtime with `enable_all()` on one dedicated thread is sufficient.
  All real-server cases ran that way; case `fake-multi-thread` repeated the scripted session
  on the multi-thread runtime with identical results.
- The `tokio` features already declared in `package/ide/app/Cargo.toml` suffice;
  `helix-lsp` itself enables the rest (`helix-lsp/Cargo.toml`).

### Workspace root resolution

`start_client` calls `helix_loader::find_workspace()` (`lib.rs:900`),
which walks up from the process working directory to the first `.git`, `.svn`, `.jj`, or `.helix`
and otherwise returns the working directory (`helix-loader/src/lib.rs:257` to `:275`).
The working directory is cached on first use (`helix-stdx/src/env.rs:12` to `:39`).
`find_lsp_workspace` then returns nothing when the file is not under that workspace (`lib.rs:996`),
returns the topmost ancestor holding a root marker,
and stops at the first `root_dirs` entry (`lib.rs:1015`) or at the workspace (`lib.rs:1023`).
The server's process directory is the resulting root,
or the workspace when there is none (`lib.rs:914`, `client.rs:236`).

Measured `initialize` parameters (`results/roots-*.wire.log`, `results/fake-utf16-incremental.wire.log`):

- Working directory is the project root, no root marker, `root_dirs` empty:
  `rootUri` null, no workspace folders, server directory is the project root.
- Same with `root_dirs = ["."]`: `rootUri` is the project root.
- Working directory elsewhere: `rootUri` null and the server process runs in that other directory.
- Working directory elsewhere, then `std::env::set_current_dir(root)` before the first Helix call:
  identical to starting in the project root.
- Project is a subdirectory of a tree with `.git`, no root marker: `rootUri` is the enclosing tree,
  with or without `root_dirs = [project root]`.
- Same, root marker only in the project: `rootUri` is the project.
- Same, root marker in the project and at the top of the tree (a monorepo):
  `rootUri` is the top of the tree; with `root_dirs = [project root]` it is the project.
- Rust (`Cargo.toml`) and TypeScript (`package.json`, `tsconfig.json`) fixtures: `rootUri` is the project.

Design consequences:

- Make the project root the process working directory at startup, before any Helix call.
  `Workspace` is built from the explicit project argument (`src/native.rs:158`)
  and ripgrep children get an explicit directory (`src/search_process.rs:26`);
  the coordinator should confirm nothing else reads the ambient directory when integrating.
- Always pass `root_dirs = [canonical project root]`.
- After `Registry::get`, read `Client::workspace_folders()` (`client.rs:433`) once the server is initialized.
  If a folder is not inside the project root, stop that server (`Registry::stop`, `lib.rs:690`)
  and report a distinct state; do not silently analyze the enclosing tree.
  This case is reachable for any language with no root marker in a project nested in a repository.

## Lifecycle

### On-demand start and initialization

- Nothing starts until `Registry::get` is called for a displayed file's language (`lib.rs:705`).
  It reuses a running client whose root matches (`Client::try_add_doc`, `client.rs:76`), otherwise starts one.
- `get` returns one `(name, Result<Arc<Client>>)` per configured server.
  `Ok` means the process was spawned and `initialize` was sent from a spawned task; it does not mean ready.
- Readiness is `Client::is_initialized()` (`client.rs:300`) and the synthetic `initialized` notification
  that the transport injects into `Registry::incoming` (`transport.rs:421` to `:438`).
- Client capabilities are hard-coded (`client.rs:586` to `:765`) and cannot be changed by the embedder.
  They advertise `workspace.applyEdit`, workspace-edit resource operations, file operations,
  `didChangeWatchedFiles` dynamic registration, `publishDiagnostics.versionSupport`,
  pull diagnostics, inlay hints with no resolve support, and position encodings in the order
  utf-8, utf-32, utf-16 (`client.rs:593`, `:607`, `:618`, `:716`, `:712`, `:726`, `:757`).
  The client identifies as `helix` (`client.rs:767`).
  Servers therefore believe edits are possible; refusal happens per request (see "Server-to-client traffic").
- After `initialized`, send `workspace/didChangeConfiguration` with the server's settings,
  as `helix-term` does (`application.rs:817`).
- Measured time from spawn to `initialized`: rust-analyzer 14 to 509 ms, TypeScript native 5 to 29 ms.
  Initialized is not ready: after its restart rust-analyzer answered the same hover first with `null`,
  then with error `-32801 content modified` (its own workspace load, no client edit),
  and with content on the third attempt about 3 s after start.

### `didOpen`

- `Client::text_document_did_open(url, version, &Rope, language_id)` (`client.rs:954`), unconditional.
- While a client is uninitialized the transport drops every notification except `initialized`
  and queues requests (`transport.rs:456` to `:463`).
  Spike: a `didOpen` with marker version `-1` sent before initialization never appears in any wire log,
  while the later `didOpen` with version 1 does.
- Rule: send `didOpen` only after observing `initialized` for that server,
  and once per (server, open document).
  Helix has the same rule as a hook (`helix-view/src/handlers/lsp.rs:392`)
  plus a noted race in the launch path (`editor.rs:1888`); the application avoids the race by having one code path.

### `didChange` and negotiated synchronization

- `Client::text_document_did_change(VersionedTextDocumentIdentifier, &old, &new, &ChangeSet) -> Option<()>`
  (`client.rs:1081`).
  It sends the whole text for `Full`, per-change ranges for `Incremental`
  (`changeset_to_changes`, `client.rs:971`, in the negotiated column unit),
  and returns `None` without sending for `None` or absent sync capability (`client.rs:1091` to `:1117`).
- The change set is `Reload::changes.changes()`:
  the transaction `Document::prepare_reload` already computes with `compare_ropes`
  (`package/ide/app/src/document.rs:156`).
  The Language module needs the old rope, the new rope, and that change set for each accepted reload,
  so `Reload` must expose its transaction (it currently exposes only `text()`).
- It panics on an uninitialized client (`client.rs:1088`).
  A server still starting at reload time gets the current text through its eventual `didOpen` instead.
- Measured with the scripted server, which publishes its own copy of the text after every open and change
  (`results/fake-*.events.jsonl`, event `ui-sync-check`):
  incremental with utf-8, utf-16 (default), and utf-32 columns, and full sync, all end with identical text
  after an edit that inserts a line and rewrites text following an accented letter and an emoji.
- With sync kind `None` or no sync capability nothing is sent and the server's copy stays old:
  a hover after reload returned an empty result computed on the old text.
  Closing and reopening (`didClose` then `didOpen` with the new text and version) resynchronized it
  (case `fake-none-reopen`).
  Proposal: when `text_document_did_change` returns `None`, resynchronize by close and reopen
  if the server accepts open and close, and otherwise mark the server unsynchronized for the file
  and stop routing position-based requests to it.
- Real servers: rust-analyzer and TypeScript native both negotiated incremental sync;
  after the reload, hover and definition at the shifted position returned the same symbol
  and pull diagnostics no longer contained the fixed type error.

### `didSave`

- `Client::text_document_did_save` exists (`client.rs:1134`) and honors the server's save options.
- A read-only application never saves, but after an external reload the text on disk equals the new text,
  so `didSave` is truthful then.
- Measured (cases `rust`, `rust-save`): without `didSave`, rust-analyzer ran `cargo check` only at start;
  its pushed `rustc` diagnostics carried version 1 and were never replaced after the reload.
  With `didSave` after `didChange`, `cargo check` reran and a versioned (version 2) empty set arrived.
- Whether to send it is in "Decisions for the coordinator".

### `didClose`

- `Client::text_document_did_close(TextDocumentIdentifier)` (`client.rs:1126`),
  only to servers that received `didOpen` and are initialized.
- Servers stay running for the next file of the same language.
- rust-analyzer answered with an unversioned empty `publishDiagnostics` for the closed file.

### Shutdown and exit

- `Client::shutdown_and_exit()` waits for the `shutdown` response, then sends `exit` (`client.rs:788`).
  On an uninitialized client the transport's send loop ends when it sees `shutdown` (`transport.rs:453`),
  so the response never comes and the call waits for the request timeout.
- `Client::force_shutdown()` sends both without waiting (`client.rs:804`);
  `wait_shutdown_flushed()` resolves when `exit` reached the server's stdin (`client.rs:823`).
  Helix quits with exactly this pair under a one second cap (`helix-view/src/editor.rs:2440`).
- After `shutdown` is sent the transport itself answers further server requests with `null` (`transport.rs:226`).
- Measured: graceful shutdown of TypeScript native and the scripted server, forced shutdown of rust-analyzer;
  every case ended with the synthetic `exit` notification and zero processes left in the disposable projects
  (`results/*.leftover.txt` are empty, checked by working directory under the fixture root).
- Proposal: on window close use `force_shutdown` plus `wait_shutdown_flushed` under a cap for every client,
  then drop the registry. Children are spawned with `kill_on_drop(true)` (`client.rs:238`).
- An exited server stays a zombie until the last `Arc<Client>` is dropped
  (spike `ui-children` events show state `Z` while the spike deliberately retained a handle).

### Crash

- When the server's stdout closes, the transport fails every pending request with `Error::StreamClosed`
  and injects a synthetic `exit` notification (`transport.rs:316` to `:349`).
- `helix-lsp` does not remove the client: the embedder must call `Registry::remove_by_id`
  (`lib.rs:604`, as `application.rs:977` does) and drop its own `Arc<Client>` clones.
- Measured with `SIGKILL` (cases `rust`, `fake-utf8-incremental`, `ts-native-relative`):
  a request pending at the kill failed with `stream-closed` after 5 ms (scripted server with a hover delay);
  `exit` arrived 62 ms after the kill of rust-analyzer;
  a request on a retained handle after the exit failed only by timeout
  (20 s default, 3 s with the scripted server's `timeout = 3`);
  displaying the file again started a fresh server, and hover worked after its warm-up.
- Proposal: on `exit` for an initialized server mark it "exited", clear its diagnostics and hints,
  and restart only on the next explicit open or request for that language, never in a loop.

## The five requests

### Position encoding

- Negotiated encoding: `Client::offset_encoding()` (`client.rs:413`),
  from `capabilities.position_encoding`, defaulting to utf-16 (`lib.rs:69`).
- Conversion helpers, all taking a `Rope` and character offsets:
  `helix_lsp::util::pos_to_lsp_pos`, `lsp_pos_to_pos`, `range_to_lsp_range`, `lsp_range_to_range`
  (`lib.rs:223`, `:146`, `:254`, `:265`).
- Measured: rust-analyzer and TypeScript native chose utf-8; the scripted server exercised utf-16 and utf-32.
  In the scripted file, character offset 14 on a line with `é` and an emoji before it was sent as
  column 18 in utf-8, 15 in utf-16, and 14 in utf-32, and the server reported the expected character each time.
  With the real servers every returned range quoted the expected identifier when converted back,
  including line 352 of the standard library's `string.rs`.
- `lsp_pos_to_pos` maps a line past the end to end of file (`lib.rs:152`) instead of failing.
  The scripted server's hint on line 99 came back as the last character offset.
  The application must check `position.line < len_lines` itself for anything it draws.
- Conversions of a reply must use the rope of the revision the request was sent for,
  and for another file the text read from that file (see "Stale-response fencing").

### Go-to-definition

- `Client::goto_definition(TextDocumentIdentifier, lsp::Position, None)`
  returns `Option<impl Future<Output = Result<Option<lsp::GotoDefinitionResponse>>>>` (`client.rs:1475`).
- The response is `Scalar(Location)`, `Array(Vec<Location>)`, or `Link(Vec<LocationLink>)`.
- Measured: rust-analyzer and TypeScript native return an array of `Location`;
  on whitespace both return an empty array.

### References

- `Client::goto_reference(identifier, position, include_declaration, None)`
  returns `Option<impl Future<Output = Result<Option<Vec<lsp::Location>>>>>` (`client.rs:1569`).
- Measured: three locations for the fixture function in both servers (import or `use`, call, definition).
  On whitespace rust-analyzer returns `null` and TypeScript native an empty array;
  both are "empty successful result".

### Hover

- `Client::text_document_hover(identifier, position, None)`
  returns `Option<impl Future<Output = Result<Option<lsp::Hover>>>>` (`client.rs:1313`).
- Helix asks for Markdown (`client.rs:660`); both servers returned `MarkupContent` of kind `markdown`
  with fenced code and a `range`.
  On a blank line both returned `null`.

### Inlay hints

- `Client::text_document_range_inlay_hints(identifier, lsp::Range, None)`
  returns `Option<impl Future<Output = Result<Option<Vec<lsp::InlayHint>>>>>` (`client.rs:1240`).
- Details are in "Inlay hints".

### Diagnostics for the displayed file

- Push arrives as `Notification::PublishDiagnostics` on `Registry::incoming` (`lib.rs:541`).
- Pull is `Client::text_document_diagnostic(identifier, previous_result_id)`
  returning `Option<impl Future<Output = Result<lsp::DocumentDiagnosticReportResult>>>` (`client.rs:1396`).
- Details are in "Diagnostics".

### Target validation

Convert every returned URI with `helix_core::Uri::try_from(lsp::Url)` (`helix-core/src/uri.rs:86`, `:96`),
which accepts only the `file` scheme, percent-decodes, and normalizes the path.
Then canonicalize and classify. Spike verdicts (`classify_target` in the spike):

- `inside-root`: project files, and TypeScript's `lib.es5.d.ts` under the fixture's `node_modules`.
  TypeScript native returned that URI as `.../node_modules/%40typescript/...`;
  decoding produced the right file, and the converted range quoted `max`.
- `outside-root`: rust-analyzer's definition of `String` at
  `~/.rustup/toolchains/<toolchain>/lib/rustlib/src/rust/library/alloc/src/string.rs`,
  and the scripted `file:///etc/hostname`.
- `rejected-scheme`: `untitled:Untitled-1` and `jdt://contents/...`
  (`unsupported scheme 'untitled' in URL untitled:Untitled-1`).
- `rejected-missing`: a `file` URI whose path does not exist.
- `rejected-not-a-file`: reserved for directories and special files.

Rules:

- Compare documents by converted path, never by URI string:
  servers encode the same path differently (`%40` against `@`).
- Refused targets are never read or opened and are reported as unavailable targets, not as "no result".
- The policy for `outside-root` targets is in "Decisions for the coordinator".

## Diagnostics

Measured behavior with Helix's client capabilities:

- rust-analyzer advertises `diagnosticProvider`
  (`identifier: "rust-analyzer"`, `interFileDependencies: true`, `workspaceDiagnostics: false`).
  - Pull returned its own analysis: `E0308 expected u32, found &'static str`, source `rust-analyzer`.
  - Push carried `cargo check` results with source `rustc`,
    with `version: 1` for the open file and `version: null` for a file that was not open.
  - It sent `workspace/diagnostic/refresh` after each `cargo check`.
- TypeScript native advertises `diagnosticProvider` (`identifier: "typescript"`).
  - Pull returned `2322 Type 'string' is not assignable to type 'number'`, source `ts`, as a `full` report
    with no `resultId`.
  - Its only push was an unversioned empty set for `tsconfig.json`.
- Neither real server pushed unversioned diagnostics for the displayed file.
  The scripted server did, and with `SPIKE_FAKE_DIAG_VERSION=1` it pushed versioned ones.

Design:

- Key each stored set by server, channel (push or pull identifier), and file path,
  and stamp it with the file generation and document revision it was accepted for.
  Aggregate for display by `source`.
- Pull: request after `didOpen`, after each `didChange`, and on `workspace/diagnostic/refresh`;
  fence the reply like any other request. Keep `resultId` per server when present.
  A failure whose error data says `retriggerRequest` is retried, as Helix does
  (`helix-term/src/handlers/diagnostics.rs:226` to `:238`).
- Versioned push: accept only when `version` equals the current LSP version of the displayed file,
  as Helix does (`helix-view/src/handlers/lsp.rs:296`).
- Every accepted reload invalidates all push sets of the previous revision at once.
  They are removed from display, not carried forward; a later set replaces them.

Proposed conservative freshness rule for unversioned push after an external reload:

1.  At the reload, drop that server's displayed push diagnostics for the file and open a hold.
2.  The hold ends when the server answers the first request the application sent it after the `didChange`
    (the pull-diagnostic or inlay-hint request that follows every reload).
    For a server that takes none of those requests, the hold ends after a fixed delay from the `didChange`.
3.  An unversioned set that arrives during the hold is discarded.
4.  A set that arrives after the hold is accepted only if every range starts on an existing line of the current text;
    otherwise the whole set is discarded.
5.  Any later set from the same server replaces it. A file switch, close, or server exit clears it.
6.  The stored set is tagged "unversioned" so tests and logs never present it as version-checked.

This rests on servers handling messages in order, which the protocol does not promise;
it reduces the window for stale display and does not claim to close it.

## Inlay hints

- Capability: `inlay_hint_provider` of `true` or an options object (`client.rs:1248`).
  rust-analyzer: `{"resolveProvider": false}`. TypeScript native: `true`.
- Range: any `lsp::Range`.
  Helix requests the visible lines plus one view height before and two after (`helix-term/src/commands/lsp.rs:1362`).
  Both a whole-file range and a partial line returned only hints inside the range.
- Resolve: Helix advertises no resolve support (`client.rs:726`) and `Client` has no resolve method.
  The scripted server advertising `resolveProvider: true` with `data` on a hint changes nothing:
  the application uses labels as delivered.
- rust-analyzer returned string labels with `kind` 1 (type) and 2 (parameter):
  `: &str`, `: u32`, `width:`, `height:`, `: String`, with explicit `paddingLeft` and `paddingRight` booleans.
  Type hints carried `textEdits`.
- TypeScript native returned nothing until the server's settings table held VS Code style keys
  under `typescript.inlayHints`
  (`parameterNames.enabled = "all"`, `variableTypes.enabled = true`, and so on).
  The same table is sent as `initializationOptions`, as `workspace/didChangeConfiguration`,
  and as the `workspace/configuration` reply; the spike did not isolate which of the three the server reads.
  The keys from Helix's `typescript-language-server` table (`includeInlayParameterNameHints`, ...) produced no hints.
  With the right keys it returned label parts: `: string`, `name:`, `: number`, `...values:`, `...data:`.
  Parameter parts carried a `location`.
- The application joins part values, keeps position, kind, and padding,
  and ignores `textEdits`, part `location`, part `command`, `tooltip`, and `data`.

## Server-to-client traffic

### Handled inside `helix-lsp`

- Framing, request ids, response matching, and per-request timeout
  (`ls_config.timeout`, default 20 s, `client.rs:460` to `:502`).
- Queueing requests and dropping notifications until initialized (`transport.rs:456`).
- The synthetic `initialized` and `exit` notifications (`transport.rs:426`, `:333`).
- Answering server requests with `null` once `shutdown` was sent (`transport.rs:226`).
- Forwarding server stderr lines and every message body to the `log` facade
  (`transport.rs:168`, `:147`, `:200`).
- Glob matching and sending of `workspace/didChangeWatchedFiles`,
  but only for registrations and file events the embedder feeds it (`helix-lsp/src/file_event.rs`).

### Handed to the embedder

Everything else arrives as `(LanguageServerId, Call)` on `Registry::incoming` (`lib.rs:585`).
`MethodCall::parse` and `Notification::parse` only decode (`lib.rs:494`, `:548`).
An unanswered request stalls the server, so the stream must be drained continuously.
Proposed policy, with the exact replies the spike sent (`results/fake-utf16-incremental.wire.log`):

- `workspace/applyEdit`: refuse with a normal result.

  ```json
  {"jsonrpc":"2.0","id":1006,
   "result":{"applied":false,"failureReason":"read-only client: workspace edits are not applied"}}
  ```

  (Line break and key order adjusted for width; the wire form is one line with `id` last.)
  The file on disk was unchanged and the scripted server continued normally.
  A JSON-RPC error is avoided because servers built on `vscode-languageserver-node` can abort on error replies
  (`transport.rs:231`, `application.rs:1095`).
- `workspace/configuration`: one entry per item, the dotted section of the server's configured settings or `null`
  (`application.rs:1051`). Sent: `{"result":[{"flag":true,"nested":{"value":42}},42,null]}`.
  rust-analyzer asked for section `rust-analyzer` and got `[null]`, the same answer Helix gives,
  because Helix's settings table is not nested under that name.
  TypeScript native asked for `js/ts`, `typescript`, `javascript`, `editor`.
- `client/registerCapability`: always `{"result":null}`.
  `workspace/didChangeWatchedFiles` registrations are passed to `registry.file_event_handler.register`;
  all others are ignored.
  TypeScript native registered `workspace/didChangeConfiguration` and two watchers with absolute glob patterns
  (`<root>/**/*` and `<root>/node_modules/**/*`). rust-analyzer registered nothing
  (Helix sets `files.watcher = "server"`, `languages.toml:252`).
- `client/unregisterCapability`: `{"result":null}`, unregistering watchers.
- `window/workDoneProgress/create`: `{"result":null}`.
  rust-analyzer sent fourteen during one start and hundreds of `$/progress` notifications;
  progress must be coalesced inside the worker and never forwarded one by one.
- `window/showMessageRequest`: `{"result":null}` (no action selected); the message may be logged.
- `workspace/workspaceFolders`: the client's folders (`client.rs:433`).
- `window/showDocument`: `{"result":{"success":false}}`.
- `workspace/diagnostic/refresh`: `{"result":null}`, then pull again for the displayed file.
- Any other method: `{"error":{"code":-32601,"message":"Method not found: <method>"}}`.
- Malformed parameters: error `-32602`.
- Notifications: `publishDiagnostics` as designed; `window/showMessage` and `window/logMessage` to the log;
  unknown notifications ignored (`Error::Unhandled`).

File watching:

- The embedder is the only source of file events (`file_event_handler.file_changed(path)`).
  Helix itself calls it only for its own saves and moves (`helix-view/src/editor.rs:2249`).
- The spike called it for the reloaded file and `helix-lsp` sent
  `workspace/didChangeWatchedFiles` with type 2 (changed) to the server that had registered a matching glob.
- The application observes only the displayed file and expanded directories.
  A server that depends on client watching (TypeScript native) will not learn about other external changes.
  This is a known limit to record, not solved here.

## Capability states

Each state occurred in the spike and is observable as follows.

- No language: `Loader::language_for_filename` returns `None`.
- No server configured: `language_servers` is empty and `Registry::get` yields nothing
  (case `no-server`, `query.sql`).
- Missing executable: `Registry::get` yields `Err(Error::ExecutableNotFound)` from `which` (`client.rs:228`).
  Case `missing-executable`:
  `command 'definitely-not-installed-language-server' not found: cannot find binary path`.
  If a launch wrapper replaces `command`, this check sees the wrapper;
  the application must resolve the real server name itself to keep this state.
- Starting: `get` returned `Ok(client)` and `client.is_initialized()` is false.
  Every request method unwraps the capability cell (`client.rs:304`, `:1088`, `:1319`, `:1481`, `:1576`)
  and panicked when called in this state (event `probe`, `panicked: true`).
  The application must gate every call on `is_initialized()`.
- Failed start, three shapes:
  - The process exits before initializing: synthetic `exit` while `is_initialized()` is false
    (case `fake-exit`).
  - `initialize` is answered with an error and the process stays alive:
    `typescript-language-server` with TypeScript 7 (case `ts-default`).
    `helix-lsp` only logs `failed to initialize language server: ...` (`lib.rs:957`);
    no notification reaches the embedder and the client stays uninitialized.
    The server's reason is in the log record:
    `The TypeScript of the workspace (TypeScript 7.0.2 at ".../node_modules/typescript/lib") provides no tsserver.js.`
  - `initialize` is never answered (case `fake-hang`): same silence.
  - For the error-reply and no-reply shapes the only exact signal is time:
    the initialize request cannot succeed after the server's request timeout.
    Proposal: a watchdog per starting server at `timeout` plus a margin;
    on expiry call `Registry::stop`, drop the handle, and report "failed to start".
    Optionally capture `helix_lsp` error records through the existing `tracing` bridge to show the reason sooner.
- Unsupported capability: the request method returns `None` on an initialized client
  (case `fake-minimal`: definition, references, inlay; every scripted case: pull diagnostics).
- Failed request: the future resolves to `Err`.
  `Error::Rpc` with the server's code (`-32603` from the scripted server),
  `Error::Timeout`, or `Error::StreamClosed`.
  rust-analyzer answered a hover overtaken by `didChange` with `-32801 content modified`,
  and also one hover during its own workspace load with no client edit.
  Codes `-32801` and `-32800` mean superseded and must not be shown as failures:
  drop the reply when the revision moved on, otherwise retry a bounded number of times.
- Empty successful result: `Ok(None)` or `Ok(Some(empty))`, shapes listed per request.
- Exited: synthetic `exit` after initialization; requests report "no live server" until a restart.
- Root outside the project: see "Workspace root resolution".

## Stale-response fencing

- A response carries only its JSON-RPC id. Document identity and revision do not travel on the wire
  for the four position requests; `publishDiagnostics` may carry `version`.
- The Language worker captures a snapshot per request:
  file generation, document revision, LSP version, the rope of that revision, the encoding, and the server id.
  The reply is converted against that rope inside the request's own task and sent with both identities.
- The interface thread compares generation and revision with what it displays when it reads the reply.
  Spike: a hover sent just before a reload came back tagged revision 1 while revision 2 was displayed
  and was dropped (`ui_verdict: dropped-stale-revision`); the next request was accepted.
  With rust-analyzer the overtaken request failed with `-32801`; it was dropped by the same rule
  before its error could be shown.
- Navigation to another file changes the generation; replies and diagnostics for the old generation are dropped.
- Server restarts change the server id; stored results name the id so an old server's late output cannot be mixed in.

## Threading

- One Language worker thread, built like `SearchWorker::new` (`package/ide/app/src/search_worker.rs:107`):
  a current-thread runtime moved into a named thread that runs one loop.
- The loop is a `tokio::select!` over the command receiver and `registry.incoming.next()`.
  With no server running the stream yields `None` and that branch is disabled until the next command.
- Commands from the interface: a bounded `tokio::sync::mpsc` queue filled with `try_send`, which never blocks.
- Replies to the interface: bounded channels polled without blocking from a Slint timer,
  as `src/native/reload.rs` polls `ReloadWorker::try_take` every 20 ms.
  The spike's stand-in polled a `std::sync::mpsc::sync_channel` on a 20 ms sleep.
- Use latest-value slots (`tokio::sync::watch`, as `search_worker.rs` does) for state that replaces itself:
  per-server status, diagnostics for the displayed file, hints for the displayed range.
  Use a small bounded queue for one-shot replies (definition, references, hover).
  Never queue `$/progress`.
- Slint 1.18.1 also offers `slint::invoke_from_event_loop` and `Weak::upgrade_in_event_loop`
  (`i-slint-core-1.18.1/api.rs:1286`, `:1227`).
  Polling is proposed because it matches the existing workers and keeps all interface state on one thread.
- `Loader` moved into the worker thread in the spike.
  The reload worker's `SyntaxEngine` owns a separate `Loader`.
  In the spike's debug build the first event appeared 105 to 252 ms after process start,
  which includes building it (0.5 to 0.9 s on the first runs after a rebuild),
  so construct it on the worker thread, not on the interface thread.
- Shutdown order: interface drops the command sender or sends quit, the worker runs the forced shutdown,
  the loop ends, the registry drops, the thread is joined.

## Launch hook for write confinement

- The hook is the configuration value before `Loader::new` freezes it:
  `Configuration::language_server` is a public map of `LanguageServerConfiguration`
  with public `command`, `args`, `environment`, `config`, and `timeout` (`helix-core/src/syntax/config.rs:20`, `:438`).
- Either mutate those fields or merge a TOML override with `helix_loader::merge_toml_values(defaults, extra, 3)`
  (`helix-loader/src/lib.rs:199`), the same merge Helix applies to a user `languages.toml`.
  The spike used the merge for every non-default case.
- `Client::start` applies them as `Command::new(which(command)).envs(environment).args(args).current_dir(root)`
  (`client.rs:228` to `:239`).
- Verified: command and arguments replaced (TypeScript native, scripted server);
  environment delivered (the scripted server is driven only by `SPIKE_FAKE_*` variables set there);
  a command containing a path separator resolves against the process working directory
  (`node_modules/typescript/bin/tsc`, case `ts-native-relative`).
- A wrapper therefore goes in `command`, with the original command and arguments appended to `args`.
- Facts for the confinement owner: rust-analyzer left `Cargo.lock` and `target/` in the disposable Rust project
  (`results/root-listings.json`); TypeScript native left nothing at the top level of its project.

## Dependency delta

- `helix-lsp` is declared in `package/ide/app/Cargo.toml` and locked in `Cargo.lock`
  with `arc-swap`, `futures-executor`, `futures-util`, `globset`, `helix-lsp-types`, `helix-stdx`, `log`,
  `parking_lot`, `slotmap`, `sonic-rs`, `thiserror`, `tokio`, `tokio-stream`.
- It is compiled but not linked: `target/debug/deps/libhelix_lsp-*.rlib` exists,
  and the built `target/debug/monochromatic-ide` contains zero occurrences of `helix_lsp`
  (control: 4741 of `helix_core`). No source file refers to it.
- The spike started from a copy of the application's `Cargo.lock`.
  After building, all 178 of its packages are in the application's 646; only the spike's own crate is new.
  The integration therefore adds no package and no version change.
- New direct declarations needed:
  - `arc-swap = "1"` for the `Registry::new` argument type.
  - `futures-util` (or `tokio-stream`) for `StreamExt::next` on `Registry::incoming`.
- Not needed: `log`. `tracing-subscriber`'s default `tracing-log` bridge is already resolved,
  and `tracing_subscriber::fmt().init()` in `src/native.rs:155` installs it.
  `helix_lsp` records appear once the filter names that target.
  They include full message bodies at info level, so keep the target at `warn`.
- Not needed: `helix-stdx`, if the working directory is set with `std::env::set_current_dir` before any Helix call.

## Decisions for the coordinator

### TypeScript server

- Option A: override the TypeScript family to the project's own TypeScript 7 server
  (`node_modules/typescript/bin/tsc --lsp --stdio`), with VS Code style inlay settings.
  - Pros: works today with the compiler version the repository pins; all five paths measured; no new dependency.
  - Cons: departs from Helix's default server entry; the executable comes from the project's `node_modules`;
    needs its own settings table; depends on client file watching for files other than the displayed one;
    projects without TypeScript 7 need a fallback state.
- Option B: keep `typescript-language-server` 6.0.0 and provide a TypeScript 6 or older through `tsserver.path`.
  - Pros: Helix's default entry and settings unchanged.
  - Cons: a new dependency to adopt; analysis by a different compiler than the project builds with;
    not runnable on this host today.
- Option C: change nothing.
  - Pros: no work.
  - Cons: TypeScript shows only a start failure, so the TypeScript half of the gate cannot pass.
- Ranking: A > B > C.
  A over B because it uses the repository's actual compiler and adds nothing.
  B over C because C cannot satisfy the required TypeScript slice.

### Targets outside the project root

- Option A: open `file` targets outside the root read-only, without changing the root or the tree,
  as the workspace plan already states for local dependency definitions.
  - Pros: definition of `String` or a `node_modules` type works; matches the plan.
  - Cons: the reader shows files outside the project; needs a visible "outside project" marker and no tree reveal.
- Option B: refuse them with an "outside project" notice.
  - Pros: simplest containment.
  - Cons: go-to-definition fails for every standard-library and dependency symbol.
- Ranking: A > B, because the scope names local dependency definitions as navigable.

### `didSave` after an external reload

- Option A: send it when the server's sync options ask for save notifications.
  - Pros: `cargo check` diagnostics refresh (measured).
  - Cons: each external change runs `cargo check`, which writes `target/`; depends on the confinement design.
- Option B: disable rust-analyzer's check-on-save in its settings and rely on its pull diagnostics.
  - Pros: no `cargo check` runs at all; consistent results.
  - Cons: fewer diagnostics than `rustc` gives.
- Option C: send nothing and leave settings alone.
  - Pros: no work.
  - Cons: `rustc` diagnostics appear for the first revision and vanish after the first reload.
- Ranking: A > B > C.
  A over B because it keeps the complete diagnostics the server offers.
  B over C because C shows a result that silently disappears.

### Adopted on 2026-10-05

The coordinating session adopted option A for all three,
because the accepted scope already determines each answer.
The user confirmed the TypeScript 7 server choice on 2026-10-05.

- TypeScript server: option A.
  The scope requires actual TypeScript feature paths,
  option B needs a new dependency,
  and option C cannot pass the gate.
  A project without its own TypeScript 7 server shows the missing-executable state.
- Targets outside the project root: option A.
  [The implementation plan](slint-ide-implementation.md) already says a local dependency definition
  opens without creating another project root.
- `didSave` after an external reload: option A.
  Disk is authoritative,
  so the notification is truthful.
  The `cargo check` it triggers must write only to private state,
  which is the subject of [the write confinement plan](slint-ide-write-confinement.md).
  Until confinement is wired in,
  real servers run only against disposable projects.

## Proposed module layout

All under `package/ide/app/src/`, each within the existing line budgets,
with tests in sibling `_tests.rs` files.

- `language.rs`: public surface for the native layer: `LanguageWorker` handle
  (`open`, `reload`, `close`, `request`, `try_take_*`), the request and reply types,
  and the `ServerState` enum (the states in "Capability states").
- `language/config.rs`: build the `Loader` from the pinned defaults plus the application override;
  the launch hook; the per-server settings tables.
- `language/worker.rs`: thread, runtime, the `select!` loop, command dispatch, shutdown.
- `language/session.rs`: per-server and per-document state: `Arc<Client>` list in configured order,
  which servers hold the document, LSP version counter, start watchdog, root check.
- `language/lifecycle.rs`: `didOpen`, `didChange` (including the unsynchronized fallback),
  optional `didSave`, `didClose`.
- `language/incoming.rs`: the server-to-client policy.
- `language/request.rs`: the four position requests, feature routing, error classification.
- `language/position.rs`: the one place that converts between Helix character offsets and LSP positions,
  with the line-bound check.
- `language/target.rs`: URI to path conversion and the containment verdicts.
- `language/diagnostics.rs`: push and pull stores, the version check, the freshness hold, aggregation by source.
- `language/hints.rs`: inlay request ranges and label shaping.
- `native/language.rs`: timer polling, fencing against `file_generation` and `Document::revision`, and rendering hooks.
- Changes outside the module: `document.rs` exposes the reload's change set and old text;
  `native.rs` sets the working directory and widens the log filter.

## Test plan

Unit tests, no server:

- Position conversion for utf-8, utf-16, utf-32 with tabs, CRLF, combining marks, and multi-code-point graphemes;
  a line past the end is rejected.
- Target verdicts: inside, outside, symlink leaving the root, non-file schemes, missing path, directory,
  percent-encoded path equal to the displayed file.
- Server-request policy: each method's exact reply, including the `applyEdit` refusal,
  unknown method, and malformed parameters.
- Diagnostics: versioned accept and drop, invalidation on reload, the unversioned hold, aggregation by source.
- Fencing: replies for an old revision, an old generation, and an old server id are dropped; a current one is accepted.
- Each guard is shown failing once with the guard removed, in a disposable copy.

Integration tests with a scripted server (a small test binary speaking LSP over stdio, configured by environment
like the spike's `fake-server.mjs`; adopting `node` for tests is not required):

- Start states: missing executable, no server, exit during initialize, error reply to initialize, no reply (watchdog).
- No request or `didOpen` before `initialized`; exactly one `didOpen` after.
- External-reload synchronization for full, incremental in each encoding, none, and absent sync:
  the server's text equals the document text after a reload built from both required correspondence examples
  in [the scope](../decision/slint-ide-0x-scope.md) and after an edit around non-ASCII text.
- Refusal of server-initiated edits: the server sends `workspace/applyEdit`, receives `applied: false`,
  the file bytes are unchanged, and the session continues.
- Every other server request receives its policy reply; an unanswered request never occurs.
- Capability states: unsupported, failed (`-32603`), superseded (`-32801`), empty, timeout.
- Crash: pending request fails, the server is removed, the next open restarts it, no zombie remains after drop.
- Shutdown: forced shutdown reaches `exit`; no child process remains.
- Root: working directory elsewhere, nested project with and without markers; an outside root is refused.

Real-server checks on disposable projects (TypeScript and Rust), run through a `mise` inspection task:

- All five paths, the reload, and the stale-reply case, as in the spike.
- Write restrictions, with the confinement owner's mechanism in place.

Native checks: hover popup, hint layout, diagnostics marks, and navigation in light and dark,
through the nested compositor.

## Open risks

- Starting a language server runs project code:
  rust-analyzer runs `cargo metadata`, build scripts, and procedural macros;
  the TypeScript launcher is the project's own `node_modules` file. Confinement is another owner's subject;
  the application has no trust prompt today.
- rust-analyzer wrote `Cargo.lock` and `target/` into the project. The scope forbids project-file writes.
- The LSP root can be the enclosing repository for a nested project without a root marker.
- `helix-lsp` advertises edit capabilities the application refuses.
  A server that insists on an applied edit may degrade; none of the three servers measured did.
- A failed `initialize` is invisible to the embedder except through logs; the watchdog is a workaround, not an API.
- Request methods panic on uninitialized clients; one missed gate is a crash of the worker thread.
  The worker should treat a panic as a reported failure of the whole Language module, never of the window.
- TypeScript native relies on client file watching; the application forwards only what it observes.
- Unversioned push diagnostics cannot be proven fresh; the rule is conservative, not exact.
- rust-analyzer's readiness has no single protocol signal: requests return `null` while it loads.
  "Empty result" during the first seconds is indistinguishable from a true empty result
  unless progress tokens are tracked; the spike only retried.
- The spike used one file and one server per language. Multiple servers per language
  (Helix configures several for some languages) and the remaining measured language inventory are untested.
- The spike's Rust follows the repository's comment style but was not run through `lint:rust`.

## Spike

### Location and files

`${HOME}/temp/agent/ide-lsp-spike-VR1Mtl` (scratch; it can vanish).

- `Cargo.toml`, `Cargo.lock` (seeded from the application's lock), `src/*.rs`: the binary `ide-lsp-spike`.
- `fake-server.mjs`: scripted server driven by `SPIKE_FAKE_*` variables.
- `fixtures.mjs`: creates the disposable projects under the system temporary directory.
  It copies TypeScript 7.0.2 from the repository's `node_modules/.pnpm` into the TypeScript fixture;
  no server is ever pointed at the repository.
- `run.mjs`: runs every case on the host; `facts.mjs` and `show.mjs` digest results.
- `results/<case>.events.jsonl`, `.wire.log`, `.stderr.txt`, `.leftover.txt`,
  `results/run-all-3.log`, `results/root-listings.json`, `results/overrides/*.toml`.

### Rebuild and rerun

```sh
# rebuild: bounded container, private cargo volume, never the shared ide-cargo volume
podman run --rm --memory=2g --cpus=2 --pids-limit=512 --ulimit nofile=4096:4096 --security-opt label=disable \
  --volume "${HOME}/temp/agent/ide-lsp-spike-VR1Mtl:/work" --volume ide-lsp-spike-cargo:/cargo \
  --workdir /work --env CARGO_BUILD_JOBS=2 localhost/monochromatic/ide cargo build
```

```sh
# rerun every case on the host; pass case names after the repository root to run a subset
node "${HOME}/temp/agent/ide-lsp-spike-VR1Mtl/run.mjs" /var/home/user/Monochromatic
node "${HOME}/temp/agent/ide-lsp-spike-VR1Mtl/facts.mjs" '^rust$'
node "${HOME}/temp/agent/ide-lsp-spike-VR1Mtl/show.mjs" ts-native '^reply'
```

The host needs `rust-analyzer`, `node`, and `typescript-language-server` on `PATH`.

### Cases

- `no-runtime`: `Registry::new` without a runtime.
- `rust`: rust-analyzer, all steps, crash and restart, forced shutdown. `rust-save`: the same with `didSave`.
- `ts-default`: Helix's default TypeScript entry. `ts-native`, `ts-native-hints-helix`, `ts-native-hints-vscode`,
  `ts-native-relative`: the TypeScript 7 server with different settings and launcher paths.
- `fake-utf8-incremental` (with crash), `fake-utf16-incremental`, `fake-utf32-incremental`, `fake-full`,
  `fake-none`, `fake-none-reopen`, `fake-omit-sync`, `fake-versioned-diagnostics`, `fake-minimal`,
  `fake-multi-thread`, `fake-hang`, `fake-exit`: scripted server.
- `missing-executable`, `no-server`.
- `roots-root-dir-dot`, `roots-elsewhere`, `roots-elsewhere-chdir`, `roots-marked`, `roots-marked-root-dir`,
  `roots-single-marker`, `roots-two-markers`, `roots-two-markers-root-dir`.

Every case exited zero and left no process.

### Source excerpts

The excerpts keep the code and drop most of the per-line explanatory comments the files on disk carry.

```toml
# Cargo.toml
[dependencies]
anyhow = "1"
helix-core = { git = "https://github.com/helix-editor/helix", rev = "ba40e547426b0f9896c8bdc699a4ab11f2b37dbc" }
helix-lsp = { git = "https://github.com/helix-editor/helix", rev = "ba40e547426b0f9896c8bdc699a4ab11f2b37dbc" }
helix-loader = { git = "https://github.com/helix-editor/helix", rev = "ba40e547426b0f9896c8bdc699a4ab11f2b37dbc" }
serde = "1"
serde_json = "1"
toml = "1.1"
tokio = { version = "1.50", features = ["rt-multi-thread", "sync", "time", "process", "io-util", "macros"] }
arc-swap = "1"
futures-util = { version = "0.3", default-features = false, features = ["std"] }
log = { version = "0.4", features = ["std"] }
```

```rust
// src/language_config.rs: the registry and the launch hook
pub fn build_loader(override_toml: Option<&str>) -> anyhow::Result<Loader> {
    let defaults: toml::Value = helix_loader::config::default_lang_config();
    let merged = match override_toml {
        Some(text) => {
            let extra: toml::Value = toml::from_str(text).context("Cannot parse the language override")?;
            helix_loader::merge_toml_values(defaults, extra, 3)
        }
        None => defaults,
    };
    let config: Configuration = merged.try_into().context("Cannot decode the language configuration")?;
    let loader = Loader::new(config).context("Cannot build file-type matchers")?;
    return Ok(loader);
}
```

```rust
// src/worker.rs: thread, runtime, and loop
async fn run(mut worker: Worker, mut commands: Receiver<Command>) {
    loop {
        tokio::select! {
            command = commands.recv() => {
                let Some(command) = command else { break; };
                if !worker.handle(command).await { break; }
            }
            // With no server running the stream yields `None`; the pattern disables this branch.
            Some((server_id, call)) = worker.registry.incoming.next() => {
                worker.on_call(server_id, call).await;
            }
        }
    }
}

pub fn spawn(loader: Loader, root: PathBuf, root_dirs: Vec<PathBuf>, commands: Receiver<Command>,
             events: SyncSender<Value>, multi_thread: bool) -> anyhow::Result<std::thread::JoinHandle<()>> {
    let mut builder = if multi_thread {
        tokio::runtime::Builder::new_multi_thread()
    } else {
        tokio::runtime::Builder::new_current_thread()
    };
    let runtime = builder.enable_all().build()?;
    let thread = std::thread::Builder::new().name("ide-language".to_string()).spawn(move || {
        runtime.block_on(async move {
            // Registry::new spawns a task, so it must run inside the runtime.
            let shared = Arc::new(ArcSwap::from_pointee(loader));
            let registry = Registry::new(shared.clone());
            let worker = Worker { loader: shared, registry, root, root_dirs, events, clients: Vec::new(),
                                  stale: Vec::new(), document: None, last_change_ms: 0 };
            run(worker, commands).await;
        });
    })?;
    return Ok(thread);
}
```

```rust
// src/worker.rs: on-demand start and its states (inside Worker::open)
let config = {
    let loader = self.loader.load();
    let Some(language) = loader.language_for_filename(&path) else { /* state: no-language */ return; };
    loader.language(language).config().clone()
};
let language_id = config.language_server_language_id.clone()
    .unwrap_or_else(|| return config.language_id.clone());
if config.language_servers.is_empty() { /* state: no-server-configured */ return; }
let started: Vec<(String, helix_lsp::Result<Arc<Client>>)> = self.registry
    .get(&config, Some(path.as_path()), &self.root_dirs, false)
    .collect();
for (name, outcome) in started {
    match outcome {
        Err(helix_lsp::Error::ExecutableNotFound(error)) => { /* state: missing-executable */ }
        Err(error) => { /* state: start-failed */ }
        Ok(client) => {
            // "ready" when a running server was reused, otherwise "starting".
            self.clients.push(client.clone());
            self.send_did_open(&client); // does nothing until the client is initialized
        }
    }
}
```

```rust
// src/worker.rs: didOpen only after initialization, once per server
fn send_did_open(&mut self, client: &Arc<Client>) {
    if !client.is_initialized() { return; }
    let Some(document) = self.document.as_mut() else { return; };
    if document.opened.contains(&client.id()) { return; }
    client.text_document_did_open(document.url.clone(), document.version, &document.text,
                                  document.language_id.clone());
    document.opened.push(client.id());
}

// inside Worker::on_notification
Notification::Initialized => {
    if let Some(settings) = client.config() {
        client.did_change_configuration(settings.clone());
    }
    self.send_did_open(client);
}
Notification::Exit => {
    self.registry.remove_by_id(server_id);
    self.clients.retain(|other| return other.id() != server_id);
}
```

```rust
// src/worker.rs: external reload (inside Worker::reload)
let new_text = Rope::from_str(&text);
let transaction = compare_ropes(&document.text, &new_text);
let old_text = document.text.clone();
document.text = new_text;
document.revision += 1;
document.version += 1;
for client in &self.clients {
    if !client.is_initialized() || !document.opened.contains(&client.id()) { continue; }
    let identifier = lsp::VersionedTextDocumentIdentifier::new(document.url.clone(), document.version);
    let sent = client.text_document_did_change(identifier, &old_text, &document.text, transaction.changes());
    if sent.is_none() && reopen_when_unsynced {
        client.text_document_did_close(lsp::TextDocumentIdentifier::new(document.url.clone()));
        client.text_document_did_open(document.url.clone(), document.version, &document.text,
                                      document.language_id.clone());
    }
}
if save_after_reload { /* client.text_document_did_save(identifier, &document.text) per client */ }
self.registry.file_event_handler.file_changed(document.path.clone());
```

```rust
// src/incoming.rs: policy for server-to-client requests
pub async fn answer(registry: &Registry, client: &Arc<Client>, method: &str, params: jsonrpc::Params) -> Answer {
    let parsed = MethodCall::parse(method, params);
    return match parsed {
        Err(helix_lsp::Error::Unhandled) => (Err(jsonrpc::Error {
            code: jsonrpc::ErrorCode::MethodNotFound,
            message: format!("Method not found: {method}"),
            data: None,
        }), "method-not-found"),
        Err(error) => (Err(jsonrpc::Error::invalid_params(format!("Malformed {method}: {error}"))), "malformed"),
        Ok(MethodCall::WorkDoneProgressCreate(_)) => (Ok(Value::Null), "acknowledged"),
        Ok(MethodCall::ApplyWorkspaceEdit(_)) => (Ok(json!(lsp::ApplyWorkspaceEditResponse {
            applied: false,
            failure_reason: Some("read-only client: workspace edits are not applied".to_string()),
            failed_change: None,
        })), "refused"),
        Ok(MethodCall::WorkspaceFolders) => {
            let folders = client.workspace_folders().await;
            (Ok(json!(&*folders)), "answered")
        }
        Ok(MethodCall::WorkspaceConfiguration(request)) => {
            let mut values: Vec<Value> = Vec::new();
            for item in &request.items {
                values.push(configuration_section(client.config(), item.section.as_ref()));
            }
            (Ok(Value::Array(values)), "answered")
        }
        Ok(MethodCall::RegisterCapability(request)) => {
            for registration in request.registrations {
                if registration.method != "workspace/didChangeWatchedFiles" { continue; }
                let Some(options) = registration.register_options else { continue; };
                let decoded: Result<lsp::DidChangeWatchedFilesRegistrationOptions, _> =
                    serde_json::from_value(options);
                if let Ok(watchers) = decoded {
                    registry.file_event_handler.register(
                        client.id(), Arc::downgrade(client), registration.id, watchers);
                }
            }
            (Ok(Value::Null), "acknowledged")
        }
        // UnregisterCapability: unregister watchers the same way, then acknowledge.
        Ok(MethodCall::UnregisterCapability(_)) => (Ok(Value::Null), "acknowledged"),
        Ok(MethodCall::ShowDocument(_)) => (Ok(json!(lsp::ShowDocumentResult { success: false })), "refused"),
        Ok(MethodCall::WorkspaceDiagnosticRefresh) => (Ok(Value::Null), "acknowledged"),
        Ok(MethodCall::ShowMessageRequest(_)) => (Ok(Value::Null), "answered-no-selection"),
    };
}
```

```rust
// src/requests.rs: the five calls (inside start); every method returns None for an unsupported capability
if !client.is_initialized() { /* state: starting */ return; }
let encoding = client.offset_encoding();
let identifier = lsp::TextDocumentIdentifier::new(snapshot.url.clone());
let lsp_position = pos_to_lsp_pos(&snapshot.text, position, encoding);
let pending: Option<JsonFuture> = if kind == "definition" {
    match client.goto_definition(identifier, lsp_position, None) {
        Some(request) => Some(erase(async move { return to_json(request.await) })),
        None => None,
    }
} else if kind == "references" {
    match client.goto_reference(identifier, lsp_position, true, None) { /* same shape */ }
} else if kind == "hover" {
    match client.text_document_hover(identifier, lsp_position, None) { /* same shape */ }
} else if kind == "inlay" {
    let range = range_to_lsp_range(&snapshot.text, helix_core::Range::new(position, end), encoding);
    match client.text_document_range_inlay_hints(identifier, range, None) { /* same shape */ }
} else {
    match client.text_document_diagnostic(identifier, None) { /* same shape */ }
};
let Some(future) = pending else { /* state: unsupported-capability */ return; };
tokio::spawn(async move {
    let outcome = future.await;
    // Err: failed-request (rpc code, timeout, stream-closed). Ok(null) or Ok([]): empty. Otherwise ok.
    // The reply carries snapshot.generation and snapshot.revision and is converted against snapshot.text.
    let _ = events.try_send(reply);
});
```

```rust
// src/requests.rs: target validation
pub fn classify_target(uri_text: &str, root: &Path) -> Value {
    let url = match lsp::Url::parse(uri_text) {
        Ok(url) => url,
        Err(_) => return json!({ "verdict": "rejected-unparseable", "uri": uri_text }),
    };
    let uri = match helix_core::Uri::try_from(url) {
        Ok(uri) => uri,
        Err(error) => return json!({ "verdict": "rejected-scheme", "uri": uri_text, "reason": error.to_string() }),
    };
    let Some(path) = uri.as_path() else { return json!({ "verdict": "rejected-scheme", "uri": uri_text }); };
    let real = match std::fs::canonicalize(path) {
        Ok(real) => real,
        Err(error) => return json!({ "verdict": "rejected-missing", "path": path, "reason": error.to_string() }),
    };
    if !real.is_file() { return json!({ "verdict": "rejected-not-a-file", "path": real }); }
    if real.starts_with(root) { return json!({ "verdict": "inside-root", "path": real }); }
    return json!({ "verdict": "outside-root", "path": real });
}
```

```rust
// src/driver.rs: the interface-side fence, applied when a reply is read (inside Ui::poll)
if event["event"] == "reply" {
    let verdict = if event["generation"] != json!(self.generation) {
        "dropped-other-file"
    } else if event["revision"] != json!(self.revision) {
        "dropped-stale-revision"
    } else {
        "accepted"
    };
    event["ui_verdict"] = json!(verdict);
}
```

```toml
# results/overrides/ts-native-relative.toml: the TypeScript 7 server with hints enabled
[language-server.typescript-native]
command = "node_modules/typescript/bin/tsc"
args = ["--lsp", "--stdio"]

[[language]]
name = "typescript"
language-servers = ["typescript-native"]

[language-server.typescript-native.config.typescript.inlayHints]
parameterNames = { enabled = "all", suppressWhenArgumentMatchesName = false }
parameterTypes = { enabled = true }
variableTypes = { enabled = true }
propertyDeclarationTypes = { enabled = true }
functionLikeReturnTypes = { enabled = true }
enumMemberValues = { enabled = true }
```
