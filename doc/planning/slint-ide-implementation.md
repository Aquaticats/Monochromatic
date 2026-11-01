# Slint IDE implementation proposal

## Status and purpose

The [0.x scope][scope] is accepted.
The user reviewed the approach and explicitly requested implementation.
Work has started in `package/ide/app`.
The confirmed deliverable remains incomplete until every gate is verified.

## Implementation queue

Current process,
verification,
and resumption details are in [the handover][handover].

- [x] Nested compositor clap help and isolated clipboard support.
  Full helper tests,
  release build,
  Clippy,
  text/binary clipboard transfer,
  clearing,
  and abrupt owner-disconnect checks pass.
- [x] Bundle JetBrains Mono and Inter independently of installed copies.
  Font files,
  notices,
  checksums,
  explicit embedding,
  and native font isolation are verified.
- [x] Finish the source-view gate.
  On 2026-10-05 conventional caret,
  selection,
  paging,
  word,
  and pointer bindings,
  two-column pixel tab stops,
  focus traversal,
  and readable dark selection ink landed on `main`
  (234 library and integration tests,
  30 native tests,
  lint);
  the package README lists the bindings and the differences from editord.
  The earlier notes in this item are historical.
  Shared Parley rows now own CJK/Latin baseline,
  caret,
  selection,
  and hit geometry.
  Bounded Swash glyph reuse avoids repeated outline rasterization.
  Native notched-wheel samples include fractional offsets through tile transitions.
  Clipboard readback verifies source,
  CJK,
  and combining sequences.
  The 29-test IDE suite and native build pass.
  Both required external-change correspondence cases now pass in the running GUI,
  including stable fractional viewport placement and missing-file recovery.
  Syntax highlighting,
  annotations,
  remaining keyboard navigation,
  and complete lint/geometry coverage are still pending.
- [x] Native workspace tree,
  asynchronous file switching,
  and Ctrl+0 through Ctrl+9 promotion/reveal/badges.
  Headless native callbacks and actual nested Wayland input pass,
  including failed-open retention and long-distance reveal.
- [x] Combined path/content search.
  Dark/light native input, scoped directory search, content-line opening, and independent failures pass.
  Seven guard-removal controls fail as intended and pass after restoration;
  whole-model pointer cancellation is tested separately as a Slint lifecycle behavior.
- [x] In-file find.
  The user settled the matcher on 2026-10-05:
  plain literal case-insensitive substring matching with the existing `regex` dependency,
  no vetting run.
  Chrome's ICU collation folding (13 of 29 captured cases) is a recorded deliberate difference.
  The native find bar,
  worker,
  and match painting pass headless tests,
  guard-removal controls,
  and dark and light nested sessions;
  see [the find matching plan](slint-ide-find-matching.md).
- [ ] Complete tree parity:
  resizable sidebar and event-driven directory invalidation beyond the current bounded polling implementation.
  The resizable sidebar landed on 2026-10-05.
  On 2026-10-05 the user chose OS file-change notifications and approved the `notify` crate without vetting;
  event-driven invalidation is in progress on branch `feat/ide-tree-watch`.
- [ ] Required language-intelligence feature paths and synchronization.
  The compiled `helix-lsp` spike and verified design are in
  [the language intelligence design](slint-ide-language-intelligence.md);
  the headless module core landed on `main` on 2026-10-05
  (scripted-server tests and 18 of 18 real-server checks each for TypeScript 7 and rust-analyzer).
  Bubblewrap confinement and native navigation
  (definition,
  references,
  hover,
  and server-state notes)
  landed the same day.
  Source-view rendering of hints and diagnostics,
  quiet server shutdown,
  and the confined nested-session check of all five paths remain.
- [x] Measured Helix-supported runtime languages.
  Twenty-seven grammars cover 22 of the 23 measured languages;
  see [the runtime language record](slint-ide-runtime-languages.md).
- [x] Project-write confinement for language-server subprocesses.
  Bubblewrap,
  no network,
  per-server process-id namespace policy,
  and a read-only project bind-back are the default on `main` since 2026-10-05;
  see [the write confinement plan](slint-ide-write-confinement.md).
- [x] Light mode and live system-theme changes through the private appearance portal.
  The nested compositor switches the private preference at runtime
  (`color-scheme dark|light` on its control socket),
  and the IDE repaints within one frame with no IDE change;
  headless theme tests with guard controls landed on 2026-10-05.
  Selected rows in the tree and combined search still need the source view's readable ink.
- [ ] Complete native integration tests,
  scoped lint,
  packaging,
  and behavior-difference documentation.

Keep visible chrome minimal.
The read-only badge,
Copy button,
and debug footer were removed at the user's request.
Copy remains on Ctrl+C and read-only semantics remain accessible.
Only actionable errors reserve additional UI when necessary.

Cargo resolves Slint to 1.18.1.
Use those installed interfaces rather than assuming the initial 1.17 inspection is identical.
The source reader submits one bounded background job at 250 ms intervals;
UI polling applies completed work to the latest reading state.

## Architecture

Propose a standalone Rust plus Slint application under `package/ide/app`.
Follow the existing native application task and packaging conventions,
without modifying the paused editor.

Do not revive the Bun daemon or introduce an HTTP/WebSocket protocol for a local-only native application.
Use Helix internals behind application-owned modules,
not Helix's terminal UI or modal commands.

### Document module

Own the current immutable text snapshot,
revision,
caret,
selection,
and viewport anchor.
Disk is authoritative;
there is no dirty-buffer or save lifecycle.

On an external change:

1. Read the new file off the UI thread.
2. Compare old and new text and compute correspondence.
3. Map caret,
   selected range,
   and the visible source anchor through the changes.
4. Install the new revision only if its file identity and generation still match.
5. Notify language servers in their negotiated synchronization mode before requesting fresh intelligence.
6. Refresh syntax and annotations without holding for selection release.

Start with Helix's rope,
diff,
and change-set primitives.
Do not assume its default selection affinities satisfy the user's replacement example.
Test both supplied examples first,
and keep application-specific range-mapping policy separate if needed.
At installation,
map the latest caret/selection/scroll state for the displayed base revision,
not a stale interaction snapshot captured when background work started.

Own conversions between UTF-8 offsets,
Helix character positions,
negotiated LSP position encoding,
and display positions in one place.
Include tabs,
CRLF,
combining characters,
and multi-code-point graphemes in tests.

### Workspace module

Own local file reads,
lazy directory listings,
filesystem change observation,
and cancellable file-path/content searches.
Reuse editord's established search behavior as the starting reference,
including ripgrep rather than building a persistent search index.

Watch changes as invalidations,
then reconcile with disk.
Handle atomic replacement,
delete/recreate,
read failure,
and directory refresh without silently trusting an event stream forever.
Fence directory responses and search results by their own generations,
not only the open document's generation.

No filesystem mutation interface belongs in this module.
Navigating to a local dependency definition does not create another project root.
Validate returned paths and URI schemes before opening a target.

### Language module

Reuse `helix-lsp` client/registry behavior and Helix's language configuration where it fits the contract.
Start servers on demand,
maintain document lifecycle,
route the required feature requests,
and aggregate displayed-file diagnostics by source.

Distinguish unsupported capability,
missing executable,
starting server,
failed request,
and empty successful result.
Do not relabel a broken supported capability as unsupported.

Requests carry document identity and revision context.
Reject stale responses after navigation or reload.
Versioned diagnostics can be checked directly;
unversioned push diagnostics need an explicitly conservative freshness policy,
not a fictitious guarantee from request IDs.

Reject server-initiated workspace edits.
Do not expose formatting,
rename,
code-action mutation,
or execute-command workflows.
Run language-server processes with read-only project access and writable private tool state.
Cache environment redirection alone is not enforcement;
verify the actual subprocess write restrictions with disposable fixtures.

### Source-view module

Slint owns the native window,
file tree,
search surface,
popups,
scroll container,
and presentation.
The application owns source positions and reading interaction.

Propose a dedicated read-only code view rather than assuming stock `TextEdit` is a syntax-aware editor widget.
Use a shared layout model for highlighted text,
annotations,
selection geometry,
caret placement,
mouse hit testing,
and source-position lookup.
Do not layer independently laid-out invisible input and colored text without proving they remain aligned.

Investigate Helix's existing `DocumentFormatter` and `TextAnnotations` before writing layout logic.
They expose source positions,
grapheme traversal,
virtual text,
and visual coordinates.
Those coordinates use terminal-style columns;
matching them to Slint glyph geometry and font fallback is still a native integration check.

Materialize visible content and overscan rather than a widget per character across the entire file.
Keep the document model independent of that rendering decision.
Full-file reading and reconciliation are permitted;
UI work and background scheduling still need explicit bounds.

Inlay placement is decided by the first rendering probe.
Accessibility,
keyboard navigation,
copy fidelity,
and light/dark rendering are part of that probe,
not deferred decorative work.

## Reuse proposal

- `helix-core`:
  rope,
  change sets,
  diff,
  source-position primitives,
  syntax loading,
  highlighting,
  and reusable document/annotation formatting where native geometry verifies.
- `helix-lsp` and its required supporting crates:
  protocol transport,
  clients,
  capabilities,
  and synchronization.
- Helix language registry,
  queries,
  and selected runtime grammars:
  keep matching code/runtime versions and respect each asset's license.
- Existing repo Slint applications:
  native startup,
  task conventions,
  headless UI tests,
  and live host verification patterns.

Prefer Git dependencies on one inspected Helix revision over copying or forking the whole editor.
Keep the matching runtime assets versioned with those dependencies.
Document the revision pin as an integration-consistency requirement.
Retain MPL-2.0 notices and applicable source-distribution obligations.

The current Slint package version and Helix revision are source-inspection baselines,
not evidence that their combined application already compiles or runs.

## Build sequence and gates

1. Prove the source view and document correspondence in a disposable implementation spike.
   Require selectable highlighted text,
   click-to-caret,
   drag and keyboard selection,
   exact clipboard contents,
   annotations,
   scrolling,
   and both supplied external-change examples.
   Expose caret/selection and revision state to tests.
2. Build the live reader around that proven view:
   local root,
   lazy tree,
   search,
   file switching,
   and disk refresh.
   Verify races and stale responses at each interface.
3. Complete an end-to-end language-intelligence slice with actual TypeScript and Rust projects.
   Exercise all five feature paths,
   external reload synchronization,
   capability reporting,
   shutdown,
   and write restrictions.
4. Extend the verified path across the measured Helix-supported language inventory.
   Package the selected grammars and tool configuration;
   do not start every language server on launch or make the app an automatic dependency installer.
5. Verify the complete native application on the current host,
   run scoped lint/tests,
   and update the behavior-difference record.

These are implementation gates within one deliverable,
not permission to postpone required features to an unspecified later 0.x release.

## Evidence inspected

### Repository incumbents

- `package/terminal/app/Cargo.toml`:
  Rust/Slint 1.17 integration and renderer configuration.
- `package/terminal/app/ui/app.slint`:
  positioned text under a Slint `Flickable`.
  This is a rendering precedent,
  not proof of a complete selectable source view.
- `package/terminal/app/mise.toml` and `package/music-player/desktop-app/mise.toml`:
  scoped build/lint/test and Slint testing tasks.
- `package-paused/desktop-daemon/editord/src/server/operations/search.ts`:
  concurrent file-path and content searches.

### Helix

Source clone:
`~/temp/agent/slint-ide-languages.mEr8K9/helix`.
Revision:
`ba40e547426b0f9896c8bdc699a4ab11f2b37dbc`.

- `helix-core/src/diff.rs:141`:
  `compare_ropes` returns a transaction;
  line diff and bounded-hunk character diff are implemented in this file.
- `helix-core/src/transaction.rs:518`:
  `ChangeSet::map_pos` exposes position association policy.
- `helix-core/src/selection.rs:180`:
  default range mapping uses sticky endpoint associations.
- `helix-core/src/syntax.rs:514`:
  syntax creation/update and range highlighting.
- `helix-core/src/doc_formatter.rs:1` and `helix-core/src/text_annotations.rs:15`:
  grapheme/source-coordinate formatting and inline annotations.
- `helix-lsp/src/client.rs:211`:
  process launch and transport construction.
- `helix-lsp/src/client.rs:1081`:
  `text_document_did_change` honors negotiated full,
  incremental,
  or absent synchronization.
  Never force full-sync messages unconditionally.
- `helix-lsp/src/lib.rs:581`:
  registry with incoming calls and file-event handling.
- `helix-core/Cargo.toml` and `helix-lsp/Cargo.toml`:
  supporting crate dependencies do not require the terminal frontend.

### Installed Slint 1.17

Inspected the installed compiler and core sources under the Cargo registry.

- `i-slint-compiler-1.17.0/builtins.slint:1641`:
  `TextInput` owns a plain string;
  read-only selection and selection-offset setting are exposed,
  while cursor/anchor readback fields are marked internal/test-only.
- `i-slint-compiler-1.17.0/widgets/common/textedit-base.slint:6`:
  `TextEditBase` wraps `TextInput` and a scroll view.
- `i-slint-compiler-1.17.0/builtins.slint:731`:
  styled text is a separate element.

This identifies a rendering integration risk,
not a claim that Slint cannot implement the required viewer.
The proposed first gate must establish the usable implementation before the rest of the app depends on it.

## Font bundle correction and proposed verification guidance

The user's requirement is now explicit:
prefer the official variable font assets and include real italics for both families.
The earlier static Regular/SemiBold subset was too narrow.
The font asset README records the replacement files,
axis ranges,
checksums,
and the current toolkit optical-size boundary.

Proposed `AGENTS.md` edit,
declined by the user on 2026-10-05 and kept here as a rejected idea:
tighten `VUB` to make font fidelity part of consumer-boundary verification.
Suggested replacement body:

> Verify built,
> deployed,
> or installed artifacts through their real consumers.
> For fonts,
> prefer variable assets and real italics;
> exercise axes,
> features,
> and reading geometry.

This keeps the existing consumer-verification obligation
and prevents treating a regular static font screenshot as complete typography support.

## Next action

Combined path/content search is complete and verified in dark and light native sessions.
In-file find is being implemented with plain literal case-insensitive substring matching;
the matcher vetting run was dropped at the user's direction on 2026-10-05.
In parallel,
the full measured Helix runtime inventory,
language-server write confinement,
and a compiled `helix-lsp` integration spike are in progress,
each recorded in [the handover][handover].
The tree and file-switching paths now work through actual native input.
The initial syntax slice is implemented;
annotations and language intelligence follow the remaining reader navigation features.
The native source/clipboard/reload checks do not establish completion of the whole application.

[handover]: ../handover/slint-ide-0x.md
[scope]: ../decision/slint-ide-0x-scope.md
