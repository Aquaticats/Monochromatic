# file-manager-electron

Sticky-flow prototype of the Niri-like file manager.
The GTK original (`package/file-manager/app`) positions panes on fixed canvases and
re-implements scroll-following ("lanes"),
 clamping,
 and collision avoidance in a
400-plus-line layout engine.
This prototype renders the same pane model as normal-flow HTML and lets two CSS declarations
(`overflow: auto` on the strip,
 `position: sticky` on each pane) do all of that work.
No script runs during scrolling.

The full derivation and the behavioral differences against the original are recorded in
`doc/audit/file-manager-sticky-flow.md`.

## How the layout works

- One scroller (`.strip`,
   a flex row with `overflow: auto`) owns both scroll axes.
- Each column is a plain block stack of `.rail` wrapper divs.
  Wrapper heights and margins are numbers computed once per model change (`src/bands.ts`):
  a pane's rail spans from its own grid row down to its deepest direct child's bottom edge,
  the same rectangle as the GTK original's green `Y6L` lane.
- Each pane is `position: sticky; top: 0` inside its rail.
  The browser pins it to the scroller top while its rail passes,
   and releases it when the
  rail's end pushes it off.
- Non-overlap is structural:
   rails are normal-flow siblings,
   and the tidy tree layout
  (`src/strip.ts`,
   a TypeScript port of the original's `model.rs`) guarantees the flow
  margins between them are never negative.

## Interaction model

- Up/Down move the selection inside the focused pane;
   Enter opens it
  (directory entries descend,
   file entries open a preview pane).
- Click descends as well;
   Ctrl+click forces a duplicate pane instead of dedup-and-focus.
- Left/Right move focus to the top pane of the adjacent column.
- Backspace (or the × control) closes the focused pane;
   children of a closed pane become
  roots,
   matching the original.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/file-manager/variant-electron:<task>`.

The unit tests run against the built artifacts:
the pane model,
band-geometry and sort order from `dist/app`,
the strip keyboard dispatch,
the DOM reconciliation and the stylesheet minima,
and the main-process listing confinement from `dist/main`.

## Pure Wayland verification

The boundary test builds a throwaway fixture directory tree,
 launches the app through
`/usr/bin/env --unset=DISPLAY` inside the nested compositor,
 drives it purely with
compositor keyboard input,
 and polls the state file the main process mirrors renderer
state into (`MONOCHROMATIC_FILE_MANAGER_ELECTRON_STATE_PATH`),
which the main process writes through the shared `writeJsonFileAtomically` of
`@monochromatic-dev/electron-infra-core` that its bundle inlines.
The decisive assertions are `rootPinned: true` while `scrolledDown: true` (sticky is
actually sticking) and `overlapCount: 0` (flow actually prevents overlap).
A screenshot with the rail debug tint (`MONOCHROMATIC_FILE_MANAGER_ELECTRON_DEBUG_TINT`)
is captured for visual comparison against the GTK original's `Y6L` lane overlays.

## Environment variables

- `MONOCHROMATIC_FILE_MANAGER_ELECTRON_ROOT`:
   directory the first root pane lists
  (defaults to the home directory).
   Listing requests are confined under it.
- `MONOCHROMATIC_FILE_MANAGER_ELECTRON_STATE_PATH`:
   when set,
   the main process mirrors
  renderer state into this JSON file for boundary tests.
- `MONOCHROMATIC_FILE_MANAGER_ELECTRON_DEBUG_TINT`:
   when set,
   rails render with the green
  debug outline for screenshots.
