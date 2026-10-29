# Handover: file-manager sticky prototypes, audit doc, and troubleshooting write-ups

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/file-manager-gtk-sticky` is now `package/file-manager/variant-gtk-sticky`;
> `package/desktop-app/file-manager-electron` is now `package/file-manager/variant-electron`.

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/module/jsonc-edit` is now `package/jsonc-edit/core-ts`.

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Working session state for the sticky-layout prototype effort.
Update this file whenever a work unit lands so the context survives auto-compaction.

## What exists and is committed

- `package/desktop-app/file-manager-electron`:
   sticky-flow Electron prototype.
  Model is a TS port of the original's `model.rs` (tests ported case for case);
  layout is `overflow: auto` plus `position: sticky` with flow numbers from `src/bands.ts`.
  Complete:
   README,
   zero oxlint findings,
   unit tests pass,
   nested-Wayland boundary test passes
  (`mise run //package/desktop-app/file-manager-electron:test:wayland`).
- `package/desktop-app/file-manager-gtk-sticky`:
   GTK variant.
  Reuses the original crate's public `model`/`fs`/`types` via path dependency;
  the lane engine plus solver is replaced by `src/band.rs`
  (`y = band_top + clamp(scroll - band_top, 0, band_height - PANE_HEIGHT)`).
  One absolute `GtkFixed` canvas (per-column canvases were stretched by hexpand propagation).
  Complete:
   README,
   clippy plus repo rust-linter clean,
   8 unit tests pass,
  boundary test passes in under a second
  (`mise run //package/desktop-app/file-manager-gtk-sticky:test:wayland`).
- `doc/audit/file-manager-sticky-flow.md`:
   first version of the audit (committed),
  currently references other docs;
   being expanded to fully self-contained with diagrams.

## Verified facts to preserve

- Both boundary tests share one key sequence (enter,
   left,
   enter,
   left,
   down,
   enter,
   backspace)
  and one shallow JSON state schema
  (`activePath`,
   `columnCount`,
   `overlapCount`,
   `paneCount`,
   `ready`,
   `rootPinned`,
  `scrolledDown`,
   `scrollTopPx`).
- Decisive assertions:
   `rootPinned: true` while `scrolledDown: true`,
   `overlapCount: 0`.
- Line counts (code lines,
   comments/blanks excluded):
  original lane engine 549 (`layout.rs` 170,
   `layout/lane.rs` 275,
   `lane/geometry.rs` 40,
  `layout/scroll.rs` 64);
   GTK sticky `band.rs` 75 plus `layout.rs` 227;
   Electron `bands.ts` 119.
- Behavioral deltas vs approved baseline (documented in the audit):
  release point at rail end vs lane-bottom-at-viewport-bottom;
  every parent sticks (not only roots);
   solver's gap compression gone;
   scope cuts
  (no thumbs/DnD/bulk closes in prototypes).

## Footguns encountered this session (all to get troubleshooting docs)

1. GTK4 hexpand propagation stretched per-column `GtkFixed` canvases ~74px;
   explicit `set_hexpand(false)` did NOT stop it empirically (docs say it should);
   probe built at the path in `/tmp/agent/probe-path.txt` to settle it;
    fix used:
    one absolute
   canvas (`file-manager-gtk-sticky` commit "one absolute canvas so panes align with bands").
2. GtkListView keyboard cursor unset until first arrow/`scroll_to`;
    bare Enter activates
   nothing;
    fix `list.scroll_to(0, FOCUS|SELECT, None)` (needs gtk4 feature v4_12).
3. Initial window focus lands on first focusable widget (header close button);
   Enter closed the pane;
    fix `close.set_focusable(false)`.
4. gtk4 feature v4_12 deprecates `CssProvider::load_from_data` -> use `load_from_string`
   (build-breaking under clippy -D warnings).
5. `serde_json::json!` macro expands to `Result::unwrap` -> trips repo clippy
   disallowed-methods;
    fix:
    build `serde_json::Map` explicitly.
6. nested-wayland-session drops keystrokes sent before the client maps its surface;
   fix:
    app mirrors a `ready` fact gated on GTK `connect_map`,
    tests wait for it.
7. tsdown:
    two configs sharing one `outDir`;
    the second run's default clean deletes the
   first's output (preload build deleted `main.mjs`).
    Self-inflicted (user confirmation):
   the repo's one-`dist/`-subdir-per-config convention exists to prevent exactly this.
   Fix now shipped:
    preload builds to its own `dist/preload` with default clean and
   `build:stage` copies `preload.cjs` into `dist/app`;
    the earlier `clean: false` patch is
   replaced and demoted to a rejected workaround in the troubleshooting doc.
8. tsc `isolatedDeclarations`:
    exported computed const needs explicit type annotation (TS9010).
9. Electron sandboxed preload must be CommonJS (bundled as `.cjs` via its own tsdown config).
10. Electron under the nested compositor prints
    `Fatal Wayland communication error: Broken pipe` and a `Failed to shutdown` FATAL at quit;
    benign in passing runs;
     one early run flaked nonzero (unreproduced across 5 later runs).
11. Operational:
     AF_UNIX socket paths must stay under SUN_LEN (~108 bytes);
     deep scratchpad
    paths fail (`path must be shorter than SUN_LEN`).
     Use `~/temp/agent/...` with a short child name,
    and verify the complete socket path remains within the platform limit.
    Also `pkill -f <pattern>` matches the invoking shell's own command line and kills it.

## Landed since the last update

- Nine troubleshooting docs committed (doc/troubleshooting/:
   gtk4-label-ellipsize-natural-width,
  gtk4-listview-keyboard-activation,
   gtk4-cssprovider-load-from-data-deprecation,
  serde-json-macro-clippy-disallowed-unwrap,
   nested-wayland-gui-test-footguns,
  tsdown-shared-outdir-clean,
   typescript-isolated-declarations-computed-const,
  electron-sandboxed-preload-cjs,
   electron-nested-wayland-shutdown-broken-pipe),
   all with
  source-cited root causes;
   GTK citations pinned to mirror commit 904b21fb235a.
- IMPORTANT correction:
   the canvas-stretch root cause is GtkLabel's documented
  ellipsize-natural-width behavior,
   NOT hexpand propagation.
  Proven via a minimal probe (set_hexpand(false) works),
   a clean-worktree reproduction
  (stretch persists with the flag),
   and a max-width-chars isolation (stretch gone).
  layout.rs's comment is corrected;
   the commit message of
  "fix(file-manager-gtk-sticky):
   one absolute canvas so panes align with bands" still
  misattributes the mechanism and needs a commit comment on GitHub after push (repo rule:
   no
  amending).
- `doc/audit/file-manager-sticky-flow.md` rewritten fully self-contained with ASCII diagrams
  and footgun references;
   lint-clean.
- `doc/audit/file-manager-sticky-flow.html` companion:
   self-contained live sticky demo
  (same architecture at miniature scale,
   tidy-tree rows computed in-page),
   SVG diagrams,
  rails toggle,
   spawn/reset buttons,
   live rootPinned/overlapCount readout.
  Verified with agent-browser:
   console clean;
   scroll to 200 pins root (offset 0);
   scroll to
  450 releases root (offset −66 = 384−450) while gamma pins (offset 0);
   spawn adds pane 7;
  reset restores 9 panes and scroll 0;
   toggle flips the rails class both ways.

## Remaining

- Nothing pending in this effort;
   keep this handover updated if new work starts.
- Done:
   the correcting GitHub commit comment on `6238b2b25` (hexpand misattribution) is
  posted (commitcomment-191869973);
   auto-push made the supporting commits reachable first.

## Investigation assets

- GTK expand probe:
   cargo project at the path recorded in `/tmp/agent/probe-path.txt`
  (`gtk-expand-probe`),
   prints widths and `compute_expand` for a plain vs
  `set_hexpand(false)` `GtkFixed` with an hexpanding child,
   under the nested compositor.
- Manual compositor driving:
   place the socket under `~/temp/agent/` with a short child name
  that keeps the complete path within `SUN_LEN`;
  drive with python `socket.AF_UNIX` one-liners (`nc` is absent on this host);
  commands:
   `ping`,
   `key <name>`,
   `screenshot <abs path>`,
   `resize`,
   `quit`.
- GTK source for citations:
   clone the GitHub mirror `GNOME/gtk` shallowly under `~/temp/agent/`.

## Repo-rule notes that bit during this session

The native readonly-rule allowlist and scoped-disable advice in this historical handover is superseded.
The repository now uses the project-owned semantic rule,
which requires honest types or accurate effect contracts and prohibits inline suppression.

- `no-nullish-union` (repo oxlint):
   no `T | null` / `T | undefined` annotations;
   use optional
  properties,
   guards,
   or exported `unique symbol` sentinels (pattern now in
  `file-manager-electron/src/strip-types.ts`).
- `typescript/prefer-readonly-parameter-types` cannot model branded primitive intersections
  (precedent:
   `package/module/jsonc-edit/src/edit-state.ts`);
   allow-list is only for types we
  do NOT control (user instruction);
   mutable-by-design carriers use scoped disable regions
  (precedent:
   `package/rolldown-plugin/import-attributes/src/scan-importer.ts`).
- Boundary-test state files:
   shallow scalar objects only;
   assert equality on booleans/counts,
  never raw floats;
   make every step's expectation distinguishable from the previous state
  (e.g. `activePath` flips) so polling cannot false-pass.
