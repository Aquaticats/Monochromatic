# file-manager-gtk-sticky

Sticky-band variant of the GTK file manager (`package/file-manager/app`).
Same product model,
 reused verbatim from the original crate's public `model`/`fs`/`types`
modules,
 but the original's lane engine (per-lane scroll offsets,
 parent-chain accumulation,
rail clamping,
 and forward/backward collision-relaxation passes,
 about 400 lines across
`layout/lane.rs` and `layout/lane/geometry.rs`) is replaced by one stateless rule per pane in
`src/band.rs`:

```txt
y = band_top + clamp(scroll - band_top, 0, band_height - PANE_HEIGHT)
```

That is the rule CSS `position: sticky` applies to an element inside its containing block.
A pane's band spans from its own grid row to its deepest direct child's bottom edge (the
original's green `Y6L` rail).
Non-overlap needs no solver because the tidy tree layout makes bands within a column disjoint;
the unit tests sweep scroll offsets to hold that invariant and the boundary test asserts it live.

The derivation,
 and the behavioral differences against the original,
 are recorded in
`doc/audit/file-manager-sticky-flow.md`.

## What is deliberately out of scope

Thumbnails,
 drag-and-drop,
 and the Windows/macOS shims are the original's concern;
 this package
exists to compare layout engines,
 so its preview panes show a typed icon plus filename only.

## Interaction model

- Up/Down move the list selection inside the focused pane;
   Enter (or a single click) opens the
  selected entry:
   directories descend,
   files open a preview pane.
   Ctrl forces a duplicate pane.
- Left/Right move focus to the first pane of the adjacent column.
- Backspace (or the header close button) closes the focused pane;
   children of a closed pane
  become roots,
   per the shared model.

## Tasks

[`mise.toml`](mise.toml) defines this package's tasks.
Run one with `mise run //package/file-manager/variant-gtk-sticky:<task>`.
The repository's Rust rules run from the root `mise run lint`.

## Environment variables

- `FM_STICKY_START_DIR`:
   directory the first root pane lists (defaults to `$HOME`).
- `FM_STICKY_STATE_PATH`:
   when set,
   the app mirrors a shallow JSON state snapshot into this file
  after every mutation and scroll change,
   using the same key set as the Electron prototype
  (`activePath`,
   `columnCount`,
   `overlapCount`,
   `paneCount`,
   `ready`,
   `rootPinned`,
  `scrolledDown`,
   `scrollTopPx`),
   so both boundary tests share assertions.
- `FM_STICKY_DEBUG_TINT`:
   when set,
   each pane's sticky band renders as a green rail for
  screenshots,
   matching the original's `Y6L` overlays.
- `FM_STICKY_QUIT_MS`:
   self-quit after this many milliseconds,
   for unattended runs.
