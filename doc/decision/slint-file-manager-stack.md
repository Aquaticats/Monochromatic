# Slint plus Rust file-manager stack

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

## Status

Accepted direction,
 2026-07-05.
Implementation is gated by the spike criteria in `doc/planning/file-manager.md`.
Product decisions (pane model,
scope,
naming) were resolved in a grilling session on 2026-07-05 and recorded there.

## Context

The project is a new infinite horizontal-scrolling desktop file manager.
The user wants a Niri-like interaction model where each pane is a file list.
The first real version must target Windows,
macOS,
and Linux.
Electron is out of scope because its security posture and runtime surface
are not acceptable for a privileged file manager.

The existing assessment in `doc/handover/slint-file-manager-assessment.md` corrected an earlier overstatement:
third-party file managers can register themselves for some default-folder-viewer paths,
but the behavior is OS-specific,
partial,
and needs explicit revert and recovery behavior.
Slint remains viable for the window and widgets,
not for native shell integration by itself.

Evidence checked before this decision:

- Slint desktop docs for Windows,
  macOS,
  and Linux support.
- Slint Node docs,
  which mark the Node integration as Beta.
- Slint `ListView` docs,
  which document visible-item instantiation for large vertical lists.
- Slint source at `/tmp/agent/slint-file-manager-assessment-20260705/internal/core/model/repeater.rs`,
  which shows vertical `viewport-y` based list virtualization.
- Slint source at `/tmp/agent/slint-file-manager-assessment-20260705/internal/core/data_transfer.rs`,
  which currently models plain text,
  images,
  and application-local `user_data`,
  with a TODO for custom MIME data providers.
- Slint issue `#1967`,
  which keeps broader drag and drop work open.
- Slint issue `#12354`,
  which tracks right-click context-menu failure on `StandardTableView` and `ListView` rows.
- Qt,
  Flutter,
  Tauri,
  Avalonia,
  Iced,
  and GPUI docs or repository metadata for cross-desktop suitability.

## Decision

Use **Slint plus Rust** for the first implementation.

The chosen shape is:

- Slint for UI declaration,
  window layout,
  panes,
  row delegates,
  dialogs,
  and settings screens.
- Rust for application state,
  filesystem operations,
  watchers,
  search,
  previews,
  file operations,
  native shell integration,
  and packaging hooks.
- Per-OS Rust adapters for Windows,
  macOS,
  and Linux default-folder-viewer behavior.

The architecture must define responsibilities directly.
Do not introduce an undefined "core" boundary.
The planning document uses responsibility boundaries:
Slint UI layer,
Rust application state layer,
Rust filesystem runtime layer,
Rust platform integration layer,
and packaging and recovery layer.

## Accepted risks

Slint plus Rust is accepted only as a spike-gated direction.
The following risks can reopen the decision:

- horizontal pane virtualization cannot be kept bounded,
- native file-list drag and drop cannot be implemented through Slint,
  a native adapter,
  or an upstream patch,
- row context menus cannot be made reliable,
- accessibility requirements require platform hooks Slint cannot expose,
- default-manager integration dominates the project enough that Qt's mature platform layer is worth the heavier stack.

The spike gates and failure actions are documented in `doc/planning/file-manager.md`.

## Rejected alternatives

### Slint plus TypeScript

Pros:
fast UI-logic iteration,
JavaScript ecosystem familiarity,
and direct Slint Node examples.

Cons:
Slint Node is marked Beta,
adds a Node runtime and package-supply-chain surface,
and does not solve filesystem authority,
file watching,
native shell registration,
file drag and drop,
or packaging.
The Slint Node event-loop code also has a polling fallback on Windows and some runtimes.

Rejection reason:
Slint plus TypeScript adds another trust and runtime boundary without removing the hard native work.

### Qt

Pros:
mature cross-desktop toolkit,
strong native platform integration,
well-understood file drag and drop,
menus,
accessibility,
and packaging patterns.

Cons:
heavier deployment,
Qt object model and QML or C++ complexity,
LGPLv3 or commercial-license obligations,
and a larger framework surface.

Rejection reason:
Qt is the first fallback,
but Slint plus Rust is smaller and better aligned
with a Rust-first privileged desktop application if the Slint spikes pass.

### GPUI

Pros:
Rust-native,
GPU accelerated,
well suited to custom high-performance pane rendering,
and used by Zed for a demanding desktop UI.

Cons:
pre-1.0,
active breaking changes,
weaker general-purpose documentation,
and less mature reusable file-manager widgets.

Rejection reason:
GPUI is the second fallback if custom horizontal rendering becomes the dominant problem.
It is not the first choice for a file manager that still needs conventional widgets,
menus,
and native shell behavior.

### Flutter

Pros:
strong custom UI,
all-desktop support,
native-compiled rendering,
and access to platform APIs through channels.

Cons:
Dart becomes the application language,
native file-manager behavior moves through plugins or platform channels,
and the stack is less aligned with Rust filesystem and native integration code.

Rejection reason:
Flutter could ship a custom UI,
but it makes the privileged filesystem and shell-integration layers less direct than Slint plus Rust.

### Avalonia

Pros:
mature desktop UI framework,
strong Windows and macOS support,
XAML tooling,
and a complete dotnet app model.

Cons:
requires a dotnet and C# stack,
Linux desktop support targets X11 directly while Wayland is in private preview,
and Rust integration would become foreign-function work.

Rejection reason:
Avalonia is credible for a C# product,
but this project benefits more from Rust-native filesystem and platform layers.

### Iced

Pros:
Rust-native,
active,
cross-platform,
and simple Elm-inspired architecture.

Cons:
the project README describes Iced as experimental software,
and file-manager-grade platform behavior would need more custom infrastructure.

Rejection reason:
Iced is attractive for Rust UI experiments,
but Slint has a more direct declarative UI workflow and documented virtualized lists.

### Tauri

Pros:
smaller and better isolated than Electron,
Rust application process,
capability controls,
and OS WebView reuse instead of bundling Chromium.

Cons:
still uses WebView processes,
IPC,
frontend web dependencies,
and browser-syntax security boundaries.
A privileged file manager would expose a large authority boundary to UI code.

Rejection reason:
Tauri improves on Electron,
but the project goal explicitly avoids a browser or WebView UI layer for the primary privileged file manager.

## Ranking

1. Slint plus Rust,
   chosen if spike gates pass.
2. Qt,
   first fallback for mature native file-manager behavior.
3. GPUI,
   fallback if custom pane rendering dominates widget maturity.
4. Flutter,
   viable but pulls the product into Dart and platform channels.
5. Avalonia,
   viable for dotnet but not Rust-aligned.
6. Iced,
   Rust-native but experimental.
7. Tauri,
   safer than Electron but still WebView and IPC based.
8. Slint plus TypeScript,
   weakest Slint option for this trust boundary.

Slint plus Rust beats Qt because the desired product is Rust-first and can stay smaller if the Slint spikes pass.
Qt beats GPUI because file-manager widgets and native behavior are more mature.
GPUI beats Flutter because it stays Rust-native for the custom pane problem.
Flutter beats Avalonia because it has stronger custom-rendered cross-desktop positioning for this UI shape.
Avalonia beats Iced because it is more mature for shipped desktop apps.
Iced beats Tauri because it avoids WebView IPC.
Tauri beats Slint plus TypeScript only because Tauri has a more explicit security model
than an ad hoc Node-hosted Slint app,
though neither is preferred.

## Consequences

- The first implementation path is `doc/planning/file-manager.md`.
- Implementation should start with Slint plus Rust prototypes,
  not with a full file-manager build.
- Qt remains the explicit fallback if Slint fails horizontal virtualization,
  native file drag and drop,
  context menus,
  or accessibility gates.
- Slint plus TypeScript should not be proposed again unless Slint's Node binding leaves Beta
  and the project has a reason to move application logic into TypeScript.
