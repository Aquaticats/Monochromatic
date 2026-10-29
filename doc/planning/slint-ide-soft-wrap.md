# Slint IDE soft wrap

The user asked on 2026-10-10 that the source view always wrap long lines to the pane width,
with a wrap indicator in the gutter for each wrapped line,
and no horizontal scrolling at all.
For every open detail the user said to do what JetBrains does ("IDEA is OSS").
This plan fixes the behavior from IntelliJ IDEA Community's source,
maps the code that has to change,
and orders the work.

## Where the user's words and JetBrains differ

IntelliJ draws no glyph in the gutter.
Its wrap marks are painted in the text area:
one at the end of the upper row and one just left of the continuation text,
and only on the caret's line by default.
The gutter only leaves the line number off continuation rows.
The user's explicit words win for the indicator's place:
the mark goes in the gutter, on each continuation row.
Everything the user did not say follows IntelliJ.

## Behavior to build

Each item names the IntelliJ source it comes from
(`JetBrains/intellij-community`, master at `d0936c36705bdaf87deca0f157a40c651345600c`).

- Gutter:
  a line number on the first visual row of a logical line only
  (`EditorGutterComponentImpl`, the `startsWithSoftWrap()` test);
  continuation rows show no number and carry the wrap indicator instead.
  The indicator follows IntelliJ's own mark for a row that starts after a soft wrap
  (`CompositeSoftWrapPainter`, `ArrowSoftWrapPainter`, `EditorColors.SOFT_WRAP_SIGN_COLOR`):
  the first of U+2925, U+21B3, U+2937, and U+21AA that the font can display, drawn in the editor font at the row's
  baseline, and a drawn arrow when none can be displayed.
  Its color is `c0c0c0` in the Default scheme; the Darcula scheme has `parent_scheme="Default"` and no override,
  so dark uses the same value.
  Shape and color are two channels.
  The user's instruction to follow IntelliJ settles the look; only the place (the gutter) comes from the user.
- Wrap width: the text area's width (view minus gutter, scrollbar, and caret room),
  no minimum; with zero width, wrapping waits (`SoftWrapApplianceManager`).
- Where a row breaks (`SoftWrapEngine`, `GenericLineWrapPositionStrategy`):
  at a space or tab, or after `,`, `;`, `)`, or before `(` and `.`,
  choosing the rightmost candidate in the overflowing row, spaces staying on the upper row;
  every BMP code point from U+2F00 up (CJK and others) is a break opportunity;
  a token with no candidate breaks at the code point where the width is exceeded.
- Continuation indent: the first non-blank column of the logical line, plus one column for the mark
  (`USE_CUSTOM_SOFT_WRAP_INDENT = true`, `CUSTOM_SOFT_WRAP_INDENT = 0`).
- Home and End (`EditorActionUtil.moveCaretToLineStart`, `moveCaretToLineEnd`, smart Home on):
  Home goes to the start of the visual row, and from there to the first non-blank column
   of the logical line's first row;
  End goes to the end of the visual row (trailing blanks skipped), and from there to the logical line end.
  Shift variants extend the selection.
  The built rules are IntelliJ's in full (`src/line_boundary.rs`):
  on the first row Home alternates between the line start and the first non-blank character,
  and End first goes before the trailing blanks, then to the end of the row, then to the end of the line,
  and on the last row from the line end back before its trailing blanks.
  Two steps are left out because the caret cannot stand in the indent of a continuation row or in virtual space:
  the move into the indent, and the smart-indent column of a line of blanks only.
- Up and Down keep a remembered pixel x across vertical moves and move by visual rows
  (`CaretImpl.myDesiredX`); horizontal moves clear it.
  PageUp and PageDown move by visual rows.
- Ctrl+Left and Ctrl+Right move by document offset.
- A caret at the exact wrap offset is drawn at the start of the lower row;
  a move that keeps its visual position, such as End, leaves it at the end of the upper row.
  The reading position therefore needs an affinity bit.
- A click picks the visual row under the pointer; a triple click selects the whole logical line,
  all its visual rows.
- Selection, find matches, and diagnostic underlines are painted per visual fragment,
  so each wrapped row gets its own segment.
- No horizontal scrolling: the scroll range, the horizontal tile, shift-wheel motion,
  `scroll-x`, and the caret-room widening all go.

## What changes in the code

The survey of 2026-10-10 found that wrapping is not a drop-in extension of the rows model.
Parley already lays out several visual lines per source line and the painter already loops over them,
but the vertical model, the geometry readback, the gutter model,
and the caret, aim, and hit code assume one row per source line.
Places that must change, silent failures first:

1. `src/shaped_row.rs`: `hit` probes at half the layout height, so every click on a wrapped line lands
 in the middle row;
   `range` hard-codes `y: self.top` and a height of one code row, so selection, find, underlines,
    and reveal all inherit it.
2. `src/text_raster.rs` and `src/annotation_paint.rs`: exact float matches of `rectangle.y` against a row's top
   lose selected ink and underline splitting on continuation rows.
3. `src/shaped_text/view.rs`, `src/document.rs`, `src/vertical_motion.rs`, `src/caret_motion.rs`:
   caret y has no visual offset, the reading position has no affinity,
   Up and Down aim with an unwrapped row, Home and End use the logical line.
4. `src/native/rows.rs` (`wanted_key`), `src/native/viewport.rs`, `src/native/render.rs`, `src/annotation/blocks.rs`:
   the row key and frame stamp must include the wrap width,
   and the cached hint packing is keyed by scale only.
5. `src/native/input.rs`, `src/native/language/pointer.rs`, `src/pointer_selection.rs`:
   pointer and hover resolve a logical line, so a click on a continuation row maps wrongly.
6. `src/annotation_layout.rs`: hint and message x and underline y come from a logical row;
   hint packing sets block height from x, and messages can run past the pane width.
7. `src/native/rows.rs` (`measure`, `present`) and `ui/app.slint`: the gutter has one number
 and one mark per logical line;
   it needs a per visual row model with the number or the wrap indicator.
8. `src/tab_layout.rs`: tab stops are measured on an unbroken layout and then used on a wrapped one.
9. `src/row_map.rs`: every height, `locate`, `page`, `extent`, and `block_top` assumes one code row per line.

Open engineering question, settled by measurement of 2026-10-11 (a throwaway release test in the IDE's container,
two CPUs, host load 36 to 120; the test source is kept at `~/temp/agent/ide-wrap-measure/tests-wrap_measure.rs`):
the vertical map needs every line's wrapped height before it can answer `locate` or the scroll extent,
but the code shapes only materialized lines.
Option A, shaping every line at the wrap width, is affordable on a worker thread:

- Shaping 50,000 lines of Rust at widths 600, 900, and 1400 and scales 1 and 2 took 1.0 to 2.2 s of worker time
  (median of five passes), 14 to 18 microseconds per line at the median and under 100 at the 99th percentile;
  5,000 lines took 80 to 150 ms; 20,000 lines with five 3,000 character lines took 0.3 to 0.9 s.
- Re-breaking all kept layouts at a new width, which a window resize needs, took 40 to 80 ms for 50,000 lines,
  about one microsecond per line.
- The kept layouts held about 158 MB for 50,000 lines (3.2 KB per line).
- Parley's `OverflowWrap::Anywhere` and `set_line_break_override` behaved as assumed,
 no row start fell inside a grapheme
  in the cases tried, and the layout width scales with the builder scale (use `width * scale`).
  The syntax brush runs of production were not pushed, so shaping cost is a lower bound.
  966 of 57,938 rows reported an advance wider than the width at width 600. The advance includes the hanging space;
  the visible advance fits every row (test `every_row_of_real_source_fits_its_width`, 98,020 rows of the crate's own
  source at widths 600, 900, 1400 and scales 1, 2). Ten rows break inside a token that has no rule opportunity
  and is wider than a row, which is the intended fallback; the test excuses exactly those.

The decision follows from the numbers and from the smoothness requirement:
shaping and re-breaking run on the worker in chunks, starting at the viewport and spreading outward
(slice 5 below adds a bounded step on the window thread for the lines near the view after a new width);
lines not yet shaped count as one row (or their last known height) in the vertical map, and each finished chunk
corrects the map with the existing keep-view-still machinery, so the first full pass of a very large file never
blocks a frame and the scroll extent settles as the pass completes.
Option B's estimate is therefore only the starting value of that map, not a separate design.

## Parley 0.11.1 findings

Read from the crate source that matches `Cargo.lock` (checksum verified), without running code.

- Style properties exist (defined in `parlance` 0.1.0, re-exported by Parley):
  `TextWrapMode` (`Wrap` default, `NoWrap`), `OverflowWrap` (`Normal` default, `Anywhere`, `BreakWord`),
  `WordBreak` (`Normal`, `BreakAll`, `KeepAll`).
  They are set with `push_default` or `push` on the ranged builder.
  A token longer than the width breaks only with `OverflowWrap::Anywhere` or `BreakWord`, per cluster;
  with `Normal` it overflows the width.
  `break_all_lines(Some(width))` takes the width in layout units (the scale passed to the builder).
- Where a row breaks: Parley uses ICU line breaking (UAX 14) and keeps the rightmost fitting candidate,
  an overflowing space hangs on the upper row.
  ICU differs from IntelliJ: it does not break after `,`, `;`, `)` before a letter, nor before `(` or `.`.
  CJK ideographs do break between each other.
  The hook `set_line_break_override` on the builder (a function of the code point pair) can force
   or suppress a boundary,
  so the IntelliJ rules can be added as an override; whether an override that splits a grapheme is safe is unverified.
  The `complex-scripts` feature is off, so Thai, Lao, Khmer, and Myanmar get no dictionary breaking.
- Reading rows: `layout.lines()` gives each visual row with its byte range and metrics (`block_min_coord`,
  `block_max_coord`, `line_height`); the row index is only available by enumerating.
  `Cursor::from_byte_index(layout, byte, affinity).geometry(...)` gives row and x;
  at a soft break `Upstream` draws at the end of the upper row and `Downstream` at the start of the lower row,
  which confirms the affinity bit.
  `Cursor::from_point` gives index and affinity.
  `Selection::geometry` returns one rectangle per visual row.
  The existing note `doc/troubleshooting/parley-cursor-from-point-inside-grapheme.md` still applies.
- Continuation indent: `Layout::set_text_indent(amount, IndentOptions { each_line: false, hanging: true })` indents
 every
  row but the first and takes one amount per layout, so a layout per logical line works;
  the indent becomes the row offset in `align()`, so `align` must run after breaking (unverified at runtime).
  The breaker also exposes per row `set_line_x` and `set_line_max_advance`; its glyph shift is unverified.
  The plan's earlier note that a per row width is impossible was wrong (a narrower width is possible).
- Trailing whitespace: an overflowing space hangs on the upper row and is part of its range and advance;
  `Layout::width()` excludes it and `full_width()` includes it.
  A tab hangs like a space: measured on 2026-10-12 at a width of 26 columns, a token of 26 glyphs followed by a tab
  or by a space ends its row with that blank, and a second blank after it starts the next row
  (so a continuation row can start with a tab).
  When the last blank of a text overflows the width, the engine also adds one line with no characters after the last
  row; the source view does not count that line (`ShapedRow::has_empty_tail`).
- Tabs: Parley 0.11.1 has no tab stops; `src/tab_layout.rs` measures them on an unbroken layout and corrects with
  letter spacing, which must be redone per row (see `doc/troubleshooting/parley-tab-stops.md`).
- Still unverified: runtime behavior of the indent and the breaker setters, the UAX 14 class of a tab, and whether
  Flickable blocks shift-wheel motion when its content width equals its width.

## Order of work

Status as of 2026-10-12 (the branch `feat/ide-soft-wrap`):

1. Layout model: **built.**
   The wrap width is part of the row cache (every kept row carries the width it is broken at),
   the tile's `Viewport` and `FrameStamp`, and the tile job;
   Parley breaks with the IntelliJ rules and indent
   (`src/wrap.rs`,
   `src/shaped_text.rs`,
   and since 2026-10-15 `src/shaped_text/layout.rs`);
   `RowMap` holds the visual rows of every line (`src/line_rows.rs`, `src/row_map.rs`);
   the tile worker measures the whole text at the wrap width in chunks from the view outward
   (`src/native/tile_pass.rs`), and breaks its kept rows again on a resize instead of shaping them.
2. Geometry readback: **built.**
   Caret, selection, find rectangles, underlines, and the terminator mark are placed on their visual row;
   hits resolve the visual row (`hit_at`); a click keeps the clicked side of a wrap offset in
   `State::caret_affinity` until slice 3, which moved it into `ReadingPosition::affinity`.
3. Gutter and scrolling: **built.**
   One gutter entry per visual row, the number on first rows only, the wrap indicator (IntelliJ's symbol chain checked
   against the fonts, `c0c0c0`, or a drawn arrow) on continuation rows;
   horizontal scrolling is gone.
4. Keys and pointer: **built.**
   Click placement, hover, and Ctrl+click resolve the visual row; the caret scrolls its visual row into view.
   Home, End, Up, Down, PageUp, and PageDown move by visual rows with a remembered pixel x,
   the side of a wrap offset is part of the reading position,
   drags run across visual rows, a double click takes a word that spans rows, a triple click takes the whole line,
   and two quick presses on different rows of one line are two single clicks.
   Slice 3 also fixed two layout defects its tests found: the continuation indent was doubled at scale 2,
   and a blank hanging on the last row left a counted row with no characters.
5. Annotations, find, and selection on visual rows: **built.**
   Underlines, find rectangles, and selection ink are per visual row.
   Hints and message rows are fitted to the text area's width: hints are packed against the line as it is broken,
   a hint that would pass the right edge ends at it, a label wider than the area breaks into pieces,
   and a message line is broken again at the width as a whole, with its 12-row cap kept and a message that would
   pass the edge moved left.
   The block of a line stands over its first visual row, because the user decided on 2026-10-05 that virtual rows stand
   over the code line; IntelliJ puts a block inlay over the visual row that holds its offset
   (`VisualLinesIterator.setInlays`), which is a known difference.
   A new wrap width packs the hinted lines near the view at once and the others when the view comes near them.
6. Tests, guards, and the README: **built.**
   The guards `caret-room`, `find-match-visual-rows`, and `underline-visual-row` replace `caret-room`,
   `horizontal-tile`, and `scroll-extent`; `horizontal-resets-column` guards the Left and Right keys,
   not scrolling, and stays.
   Slice 4 added guards for the annotation rows, the clip of a tall line, the character budget of the tab passes,
   and the caret stops of a very long line.
7. Slice 4 (2026-10-12), the last: **built.**
   Besides item 5 above:
   the caret stops of a line were found in time that grew with the square of its length
   (100,000 characters took over 90 s to open in a debug build) and now grow with its length;
   a tile of lines wrapped to more rows than the 64 MiB raster limit holds is painted in a clip of the rows near the
   view instead of being refused;
   tab correction was measured to settle after 4 to 76 passes with no bound known beforehand,
   so it keeps a bound, now a budget of characters shaped (the passes a line gets fall with its length);
   a test that raced the 200 ms scroll quiet time reads a clock the test holds;
   and scrolling inside a built tile of a wrapped file is counted to do no source work.
   The README (`package/ide/app/README.md`, "Soft wrap") holds the numbers and the list of known differences
   from IntelliJ.
8. Slice 5 (2026-10-13), a new width on the window thread: **built.**
   A new wrap width no longer waits for the worker's tile.
   The window thread breaks the kept layouts of the lines near the view again (shaping the lines not yet shaped),
   updates their heights in the row map with the view kept still, and paints the stage 1 tile for the new width in
   the draw that follows the resize, within an allowance of 384 lines, 24,000 units of work, and a 32 MiB raster
   (`src/native/rewrap.rs`).
   A change over the allowance goes to the worker as before, and the whole-file re-break stays on the worker.
   The earlier guard `resize-stays-on-worker` and its tests are replaced by the opposite structural assertion,
   with guards for each bound.
   The measurement of the drag in the nested compositor is in the README ("A new width on the window thread"):
   the step took a few milliseconds, and the count of frames that do not fit did not change in the 16 ms drag,
   because the compositor shows the previous step's content at each new size, while a 100 ms drag showed far
   fewer such frames than the worker-only tile.
   The remaining one-step lag was measured on 2026-10-13: it is the recorder's capture model and winit's redraw gate,
   not the application (`doc/troubleshooting/nested-wayland-recording-previous-resize-step.md`).

## Not decided here

- Whether a "soft wrap" toggle exists. The user said always, so none is planned.

## Smoothness requirement that applies to all of it

The user ruled that choppy scrolling is not excusable even under extreme host load.
Wrapping adds layout work, so the same rule binds it:
a scroll step must not wait on shaping, rasterizing, or layout on the UI thread.
The scroll measurements of 2026-10-10 (app renders had a median of 2 ms but a p95 near 30 ms and spikes of 60 to
77 ms; the first reading that only half the possible frames painted was a harness denominator artifact, see
`doc/planning/slint-ide-scroll-pipeline.md`) are the baseline to beat.
