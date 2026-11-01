# Slint IDE scroll pipeline

The user reported choppy scrolling in the source view (2026-10-10),
suspected Slint's new Vello renderer, and ruled that choppiness is not excusable even under extreme host load.

## What was measured

All numbers come from the nested compositor with the installed single file,
on a host with a one-minute load of 35 to 87,
with only runs in which the frame recorder dropped nothing counted.

- Vello starts and draws in the nested compositor (it needs no compositor change;
  the compositor already has a dmabuf GPU path),
  but FemtoVG, software, Vello, and the default (FemtoVG) showed the same frame cadence, with a longest pause of 50 to
  67 ms in every cell.
  (The "about half of the possible frames" figure first reported here was a harness artifact:
  the harness counts the 1.5 s idle tail after the last wheel step as possible frames,
  so even perfect painting of the 1.9 s input phase scores about 115 of 204.
  Restricted to the input phase, a rerun painted 103 of 115 frame slots.)
  The measurements gave no sign that the renderer is the bottleneck, so Vello is not adopted.
- In the app, `render()` ran 1.7 times per wheel step, and every call was a full repaint:
  it reshaped all 30 viewport rows (no row cache), rasterized the whole tile on the CPU, and built and set a new
  `Image`.
  Per call: median 1.4 ms, p95 5.6 ms, maximum 76 ms; the slow calls were raster and image upload.
  The file is `src/native/render.rs`; the tile is the viewport plus three rows
  (`src/native/viewport.rs`, `place`), so almost every scroll step changes the frame stamp.

## Design

Scrolling inside an already rasterized tile must do no work in the app:
the `Flickable` moves the existing image on its own.

1. Overscan tile: the tile covers about three view heights, one above and one below the view,
   and keeps its origin while the view stays inside its middle region.
   The frame stamp then stays unchanged and `render()` has nothing to do.
2. Skipping the call: the viewport-changed handler checks, in constant time and without borrowing the shaped view,
   whether the view still lies inside the tile; if it does, it returns before any shaping, painting, or model update.
   Line numbers, selection, find, and annotation rectangles are in content coordinates and need no update either.
3. Re-centering: when the view reaches the edge region of the tile, a new tile is built.
   First on the UI thread (stage 1, built), then on a worker thread (stage 2, built in `src/native/tile_build.rs`),
   which shapes and rasterizes off the UI thread and hands back the image, the shaped rows, and the rectangles;
   the UI thread only swaps them, image and offset in the same step, so the picture does not jump.
   Fast scrolling past the tile shows the stale tile until the new one arrives, never a blank.
4. Row cache: shaped rows are kept by line, text revision, scale, and wrap width,
   so a re-centered tile reshapes only lines it has not seen.
5. Acceptance is structural and independent of load:
   a test counts shaping, rasterizing, and image sets while the view moves inside the tile (zero),
   and a guard removes the skip and must make the test fail.
   A timing measurement under the same recorded load is reported beside it, never instead of it.

## Order

1. Stage 1: the overscan tile with hysteresis, the early return, the row cache, and the structural test and guard.
2. Stage 2: the worker thread for tile building (built: scroll re-centers only, with the structural test
   `leaving_the_overscan_tile_re_centers_on_the_worker_thread` and the guard `recenter-on-window-thread`).
3. Soft wrap (`doc/planning/slint-ide-soft-wrap.md`) lands on this pipeline, not before it:
   wrapping needs the row cache, per-line heights, and the tile build that stage 2 moves off the UI thread.
   Built on 2026-10-11 on `feat/ide-soft-wrap`: the row cache is keyed by wrap width,
   a new wrap width was first a worker re-center, and the same worker measures the whole text between tile jobs.
   Since 2026-10-13 a new wrap width is taken by the window thread within a counted allowance
   (384 lines, 24,000 units of work, 32 MiB of raster, `src/native/rewrap.rs`) so that no frame shows the old
   wrapping, and only a change over the allowance is a worker re-center; scroll re-centers stay on the worker.
