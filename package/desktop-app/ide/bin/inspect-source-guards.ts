#!/usr/bin/env node
// Observe committed source-view reading regressions failing after guard removal in a disposable package copy.
import { spawnSync } from 'node:child_process';
import {
  cpSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  realpathSync,
  statSync,
  writeFileSync
} from 'node:fs';
import { homedir } from 'node:os';
import {
  join,
  relative,
  sep
} from 'node:path';
import { containerBounds } from './container-bounds.ts';
import { gpuArguments } from './gpu-arguments.ts';

// The bound of every container this script starts: the shared computed share of the host, or the caller's IDE_CONTAINER_* values.
const containerLimits = containerBounds();

const privateRoot = join(
  homedir(),
  'temp',
  'agent'
);
if ((statSync(privateRoot)
  .mode
  & 0o077) !== 0) throw new Error(`Guard scratch root must exclude group and other permissions: ${  privateRoot}`);
if (!process.env
  .usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = realpathSync(process.env
  .usage_cache);
if ((!cache.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(cache)
  .isDirectory())) throw new Error(`Target cache must be a disposable directory below ${  privateRoot}`);
// A private Cargo home copy keeps disposable builds off the shared volume's package-cache lock.
let cargoHome = 'ide-cargo';
if (process.env
  .usage_cargo) {
  cargoHome = realpathSync(process.env
    .usage_cargo);
  if ((!cargoHome.startsWith(realpathSync(privateRoot) + sep)) || (!statSync(cargoHome)
    .isDirectory())) throw new Error(`Cargo home copy must be a disposable directory below ${  privateRoot}`);
}
const artifact = mkdtempSync(join(
  privateRoot,
  'ide-source-guard-'
));
const source = join(
  artifact,
  'package'
);
const origin = process.cwd();
cpSync(
  origin,
  source,
  {
    recursive: true,
    filter: path => ![
      'target',
      '.git',
      'node_modules'
    ].includes(relative(
      origin,
      path
    )
      .split(sep)[0] ?? '')
  }
);
mkdirSync(
  join(
    source,
    'target'
  ),
  { recursive: true }
);
console.log(`SOURCE_GUARD_ARTIFACT=${  artifact}`);

/**
 One text replacement that a guard makes in a file of the disposable copy.
 */
interface Edit {
  file: string;
  before: string;
  after: string;
  occurrence?: number | undefined;
}

/**
 A guard: the edits that remove it and the native, library, or integration test that must then fail.
 */
interface Guard extends Edit {
  name: string;
  also?: Edit[];
  native?: boolean;
  lib?: boolean;
  integration?: string;
  test: string;
  failure: string;
}

/**
 A container run recorded in `results.json` and echoed as one JSON line.
 */
interface RunResult {
  name: string;
  phase: string;
  test: string;
  status: number | null;
  accepted: boolean;
}

/**
 `text` with one occurrence of `before` replaced by `after`.

 @param text - source of the file to mutate

 @param before - anchor that the guard exists to protect; absence means the guard moved with its code

 @param after - replacement that removes the guard

 @param occurrence - zero-based match to replace, for anchors that repeat

 @returns the edited text

 @throws {Error} when `before` occurs fewer than `occurrence` + 1 times
 */
function replaceOne({
  text,
  before,
  after,
  occurrence = 0
}: {
  text: string;
  before: string;
  after: string;
  occurrence?: number | undefined;
}): string {
  let offset = 0;
  let start = -1;
  for (let index = 0; index <= occurrence; index++) {
    start = text.indexOf(
      before,
      offset
    );
    if (start < 0) throw new Error(`Mutation source anchor is absent: ${  before}`);
    offset = start + before.length;
  }
  return text.slice(
    0,
    start
  ) + after
    + text.slice(start + before.length);
}
const equality = 'assertion `left == right` failed';
const cases: Guard[] = [
  // A line past the raster limit is painted in a clip that follows the view, in a raster within the limit.
  {
    name: 'clip-chosen',
    file: 'src/shaped_text/limit.rs',
    before: 'if bottom - first_top <= cap {',
    after: 'if true {',
    native: true,
    test: 'a_line_past_the_raster_limit_is_painted_in_a_clip_that_follows_the_view',
    failure: 'tile texture pixels'
  },
  {
    name: 'clip-edges',
    file: 'src/native/viewport.rs',
    before: 'Some(clip) => (first_top + clip.top, first_top + clip.bottom),',
    after: 'Some(_) => (first_top, current.row_map.block_top(end)),',
    native: true,
    test: 'a_line_past_the_raster_limit_is_painted_in_a_clip_that_follows_the_view',
    failure: 'does not hold the view'
  },
  {
    name: 'clip-gutter',
    file: 'src/native/rows.rs',
    before: 'if view.viewport.clip.is_some() && (top + CODE_ROW <= band.0 || top >= band.1) {',
    after: 'if false {',
    native: true,
    test: 'a_line_past_the_raster_limit_is_painted_in_a_clip_that_follows_the_view',
    failure: 'the gutter lists'
  },
  // The stacked code tile: each texture is within the bin count Vello accepts and the device's side, abuts the next, is
  // placed by its own top, and is drawn into textures that a tile of the same shape keeps.
  {
    name: 'stack-bin-rule',
    file: 'src/shaped_text/limit.rs',
    before: 'let by_bins = (MAX_BINS / bins_across).max(1) * BIN_SIDE;',
    after: 'let by_bins = (MAX_BINS * 2 / bins_across).max(1) * BIN_SIDE;',
    integration: 'tile_vello_limit',
    test: 'no_accepted_texture_covers_more_bins_than_vello_counts_before_it_warns',
    failure: 'covers'
  },
  {
    name: 'clip-texture-side',
    file: 'src/shaped_text/limit.rs',
    before: 'return side.min(by_bins).min(by_bytes);',
    after: 'return by_bins.min(by_bytes);',
    integration: 'raster_limit',
    test: 'the_texture_size_of_the_device_bounds_each_texture_of_the_stack',
    failure: equality
  },
  {
    name: 'stack-own-top',
    file: 'src/tile_scene.rs',
    before: '    let shift = rows.start as f32;\n    let size =',
    after: '    let shift = 0.0_f32;\n    let size =',
    native: true,
    test: 'a_tile_drawn_across_textures_equals_the_tile_in_one_texture',
    failure: 'differs from the one texture'
  },
  {
    name: 'stack-band-cut',
    file: 'src/tile_scene/encode.rs',
    before: 'let band_top = top.clamp(0.0, height);',
    after: 'let band_top = *top;',
    native: true,
    test: 'a_tile_drawn_across_textures_equals_the_tile_in_one_texture',
    failure: 'min > max'
  },
  {
    name: 'stack-slices-abut',
    file: 'src/native/gpu_tile/source.rs',
    before: '                top += held.target.height;',
    after: '                top += 0;',
    native: true,
    test: 'the_window_shows_a_stacked_tile_as_the_one_texture_shows_it',
    failure: 'differs from the one texture'
  },
  {
    name: 'stack-textures-reused',
    file: 'src/native/gpu_tile/source.rs',
    before: '        if self.tile.len() != stack.textures.len() {\n            return false;',
    after: '        if true {\n            return false;',
    native: true,
    test: 'a_tile_of_the_same_shape_reuses_its_textures_and_the_list_of_images',
    failure: 'made new textures'
  },
  {
    name: 'window-texture-limit-told',
    file: 'src/native/gpu_tile/types.rs',
    before: '            window.set_texture_limit(i32::try_from(limit).unwrap_or(i32::MAX));',
    after: '            let _ = limit;',
    native: true,
    test: 'the_window_is_never_larger_than_the_devices_texture_side',
    failure: equality
  },
  {
    name: 'window-max-width',
    file: 'ui/app.slint',
    before: 'max-width: root.texture-limit * 1phx;',
    after: 'max-width: 100000phx;',
    native: true,
    test: 'the_window_is_never_larger_than_the_devices_texture_side',
    failure: 'max-width'
  },
  {
    name: 'window-max-height',
    file: 'ui/app.slint',
    before: 'max-height: root.texture-limit * 1phx;',
    after: 'max-height: 100000phx;',
    native: true,
    test: 'the_window_is_never_larger_than_the_devices_texture_side',
    failure: 'max-height'
  },
  {
    name: 'clip-origin',
    file: 'src/shaped_text.rs',
    before: 'Some(clip) => (first_top + clip.top, clip.bottom - clip.top),',
    after: 'Some(clip) => (first_top, clip.bottom - clip.top),',
    integration: 'wrap_rows',
    test: 'a_clipped_tile_holds_the_pixels_of_its_band',
    failure: equality
  },
  {
    name: 'clip-rebase',
    file: 'src/shaped_text/view.rs',
    before: 'skip = clip.top;',
    after: 'skip = 0.0;',
    integration: 'wrap_rows',
    test: 'a_clipped_tile_holds_the_pixels_of_its_band',
    failure: equality
  },
  {
    name: 'clip-rounding',
    file: 'src/shaped_text/limit.rs',
    before: '            cut = top + allowed;\n',
    after: '',
    native: true,
    test: 'a_clip_stays_within_the_raster_limit_for_every_window_size',
    failure: 'passes the limit'
  },
  // Tall window: the shaper cuts a frame to the raster limit, the tile names its clip, a refused tile keeps the picture, and
  // the clip is centered on the view and keeps the last row of the text.
  {
    name: 'frame-fit',
    file: 'src/shaped_text/limit.rs',
    before: '    if fitted == viewport.clip {',
    after: '    if true {',
    integration: 'raster_limit',
    test: 'a_frame_taller_than_its_map_held_is_cut_to_the_limit_and_painted',
    failure: 'past the limit'
  },
  {
    name: 'tile-stamp-names-its-clip',
    file: 'src/native/tile_build/job.rs',
    before: '        job.stamp.with_clip(bounds.clip)',
    after: '        job.stamp',
    native: true,
    test: 'a_tile_cut_by_the_shaper_is_stamped_with_the_clip_it_was_cut_to',
    failure: 'does not name the clip it was cut to'
  },
  // The tile is a texture: blanking it after a refusal hands the GPU an empty scene of the shown tile's size.
  {
    name: 'refused-tile-keeps-image',
    file: 'src/native/render.rs',
    before: '                current.frame_stamp = None;\n                drop(current);\n',
    after: '                current.frame_stamp = None;\n                if let Some(view) = current.shaped.as_ref() {\n                    current.gpu.submit_tile(ide_app::tile_scene::TileStack {\n                        textures: vec![ide_app::tile_scene::TileScene {\n                            scene: vello::Scene::new(),\n                            width: view.width,\n                            height: view.height,\n                        }],\n                        width: view.width,\n                        height: view.height,\n                    });\n                }\n                drop(current);\n',
    native: true,
    test: 'a_refused_tile_keeps_the_text_on_screen_and_names_the_failure',
    failure: 'a refused tile blanked the source'
  },
  {
    name: 'failed-job-freed',
    file: 'src/native/tile.rs',
    before: '                current.tile_job = None;\n                #[cfg(debug_assertions)]\n                current.worker_failures.push(message.clone());',
    after: '                #[cfg(debug_assertions)]\n                current.worker_failures.push(message.clone());',
    native: true,
    test: 'a_failed_worker_reply_frees_its_job_and_names_the_failure',
    failure: 'the failed job is still held'
  },
  {
    name: 'clip-centered-on-view',
    file: 'src/shaped_text/limit.rs',
    before: 'let middle = (band.0 + band.1) / 2.0;',
    after: 'let middle = (from + to) / 2.0;',
    native: true,
    test: 'no_window_size_or_scale_makes_a_raster_past_the_limit',
    failure: 'the raster does not hold the view'
  },
  {
    name: 'clip-keeps-last-row',
    file: 'src/shaped_text/limit.rs',
    before: '        if to >= bottom - 0.5 {',
    after: '        if false {',
    native: true,
    test: 'no_window_size_or_scale_makes_a_raster_past_the_limit',
    failure: 'the raster does not hold the view'
  },
  {
    name: 'tall-window-edge',
    file: 'src/native/viewport.rs',
    before: '    return (RECENTER_EDGE_VIEWS * height).min(room);',
    after: '    return RECENTER_EDGE_VIEWS * height + 0.0 * room;',
    native: true,
    test: 'a_tall_window_over_wrapped_text_finishes_the_measurement_pass',
    failure: 'the measurement pass did not finish'
  },
  {
    name: 'wrapped-tile-holds',
    file: 'src/native/viewport.rs',
    before: 'if shown.width == width && holds(current, shown, view, height) {',
    after: 'if false && holds(current, shown, view, height) {',
    native: true,
    test: 'scrolling_inside_a_built_tile_of_a_wrapped_file_does_no_source_work',
    failure: 'the source tile worker did not finish the re-center'
  },
  // Tab correction: the passes a line gets fall with its length, so the characters shaped stay near a budget.
  {
    name: 'tab-budget',
    file: 'src/tab_layout.rs',
    before: 'for _ in 0..passes_for(projection.text.len()) {',
    after: 'for _ in 0..TAB_PASSES {',
    lib: true,
    test: 'tab_layout::tests::a_long_tabbed_line_shapes_no_more_passes_than_its_length_allows',
    failure: 'more than the'
  },
  {
    name: 'tab-budget-formula',
    file: 'src/tab_layout.rs',
    before: 'return (TAB_BUDGET / length.max(1)).clamp(TAB_MIN_PASSES, TAB_PASSES);',
    after: 'return TAB_PASSES;',
    lib: true,
    test: 'tab_layout::tests::passes_fall_with_the_length_of_the_line',
    failure: equality
  },
  // A very long line: its caret stops come from a rope, so the lookups of the walk scan chunks, not the whole line.
  {
    name: 'long-line-stops',
    file: 'src/text_projection.rs',
    before: 'let slice = if source.len() <= FLAT_LINE_BYTES {',
    after: 'let slice = if true {',
    lib: true,
    test: 'text_projection::tests::the_work_of_finding_stops_grows_with_the_line_not_its_square',
    failure: 'the lookups scanned'
  },
  // Tab stops: the half-space rule, the unusable-advance guard, and the scale conversion of corrections.
  {
    name: 'tab-half-space',
    file: 'src/tab_stop.rs',
    before: 'if distance < space / 2.0 {',
    after: 'if false {',
    integration: 'tab_stop',
    test: 'tab_narrower_than_half_a_space_uses_the_following_stop',
    failure: 'a sliver tab was kept instead of the next stop'
  },
  {
    name: 'tab-unusable-space',
    file: 'src/tab_stop.rs',
    before: 'if !usable {',
    after: 'if false {',
    integration: 'tab_stop',
    test: 'unusable_space_advance_keeps_one_space_width',
    failure: equality
  },
  {
    name: 'tab-scale',
    file: 'src/tab_layout.rs',
    before: 'spacings.push((*byte, extra / scale));',
    after: 'spacings.push((*byte, extra));',
    integration: 'tab_geometry',
    test: 'tab_geometry_is_the_same_at_every_scale',
    failure: 'at scale 1'
  },
  {
    name: 'tab-leading-shortcut',
    file: 'src/tab_layout.rs',
    before: 'if *byte != ordinal {',
    after: 'if false {',
    integration: 'tab_geometry',
    test: 'tabs_end_on_multiples_of_two_measured_space_advances',
    failure: 'the tab ends at'
  },
  // Reading geometry: hits never split a grapheme, and a selected terminator is marked.
  {
    name: 'grapheme-snap',
    file: 'src/shaped_row.rs',
    before: 'return self.source_start + self.snapped(proposed, x, row);',
    after: 'return self.source_start + proposed;',
    integration: 'reading_boundaries',
    test: 'every_pixel_hits_the_nearest_grapheme_boundary',
    failure: 'hit inside a grapheme'
  },
  {
    name: 'terminator-mark',
    file: 'src/shaped_text/view.rs',
    before: 'if start <= row_end && end > row_end {',
    after: 'if false {',
    integration: 'reading_boundaries',
    test: 'selected_terminators_are_copied_and_marked',
    failure: 'terminator mark'
  },
  // Vertical movement: the remembered column belongs to one caret position, and the text ends stop the caret.
  {
    name: 'stale-column',
    file: 'src/vertical_motion.rs',
    before: '        && column.head == head\n',
    after: '        && true\n',
    integration: 'vertical_motion',
    test: 'stale_preferred_column_is_ignored',
    failure: 'a column remembered for another caret position moved the caret'
  },
  {
    name: 'first-line-stop',
    file: 'src/vertical_motion.rs',
    before: 'if rows < 0 && first_row {',
    after: 'if false {',
    integration: 'vertical_motion',
    test: 'first_and_last_line_stop_at_the_ends_of_text',
    failure: equality
  },
  {
    name: 'last-line-stop',
    file: 'src/vertical_motion.rs',
    before: 'if rows > 0 && last_row {',
    after: 'if false {',
    integration: 'vertical_motion',
    test: 'first_and_last_line_stop_at_the_ends_of_text',
    failure: 'a last line without terminator ends at the end of text'
  },
  // Word units stay on their line.
  {
    name: 'word-stays-on-line',
    file: 'src/caret_motion.rs',
    before: 'while start > first && class_at',
    after: 'while start > 0 && class_at',
    integration: 'caret_motion',
    test: 'word_range_prefers_a_word_and_stays_on_its_line',
    failure: 'leading blanks stay on their own line'
  },
  // Click counting: interval, row, distance, and the restart after a triple click.
  {
    name: 'click-interval',
    file: 'src/pointer_selection.rs',
    before: '&& at.saturating_duration_since(previous.at) <= CLICK_INTERVAL',
    after: '&& true',
    integration: 'pointer_selection',
    test: 'slow_or_distant_presses_are_single_clicks',
    failure: 'a pause longer than the interval must start over'
  },
  {
    name: 'click-row',
    file: 'src/pointer_selection.rs',
    before: '&& previous.row == row',
    after: '&& true',
    integration: 'pointer_selection',
    test: 'slow_or_distant_presses_are_single_clicks',
    failure: 'a press on another row must start over'
  },
  {
    name: 'click-slop',
    file: 'src/pointer_selection.rs',
    before: '&& (previous.x - x).abs() <= CLICK_SLOP',
    after: '&& true',
    integration: 'pointer_selection',
    test: 'slow_or_distant_presses_are_single_clicks',
    failure: 'a press farther away than the slop must start over'
  },
  {
    name: 'click-restart',
    file: 'src/pointer_selection.rs',
    before: '&& previous.count < 3',
    after: '&& true',
    integration: 'pointer_selection',
    test: 'quick_presses_cycle_through_character_word_and_line',
    failure: equality
  },
  {
    name: 'drag-keeps-unit',
    file: 'src/pointer_selection.rs',
    before: 'return (origin.1, start);',
    after: 'return (origin.0, start);',
    integration: 'pointer_selection',
    test: 'drag_extends_by_whole_units_and_keeps_the_pressed_unit',
    failure: 'dragging left must keep the pressed word and take the whole earlier word'
  },
  // Selection ink: the white-first rule and the translucent-fill fallback.
  {
    name: 'ink-white-first',
    file: 'src/selection_ink.rs',
    before: 'if contrast(WHITE, background) >= WHITE_MINIMUM {',
    after: 'if contrast(WHITE, background) >= contrast(BLACK, background) {',
    integration: 'selection_ink',
    test: 'dark_scheme_selection_ink_is_light_on_the_fluent_selection_background',
    failure: 'dark-scheme selected text stayed dark on the selection background'
  },
  {
    name: 'ink-translucent',
    file: 'src/selection_ink.rs',
    before: 'if background[3] != 255 {',
    after: 'if false {',
    integration: 'selection_ink',
    test: 'translucent_selection_background_keeps_the_palette_ink',
    failure: equality
  },
  // Native bindings: key filters, the column reset, caret room, drag selection, and focus stops.
  {
    name: 'alt-is-not-a-caret-key',
    file: 'src/native/input.rs',
    before: '        if alt {\n            return;\n        }',
    after: '',
    native: true,
    test: 'horizontal_keys_move_by_grapheme_word_line_and_document_and_extend_with_shift',
    failure: 'Alt combinations are not caret keys'
  },
  {
    name: 'control-vertical-unbound',
    file: 'src/native/input.rs',
    before: '            && !control\n',
    after: '',
    native: true,
    test: 'horizontal_keys_move_by_grapheme_word_line_and_document_and_extend_with_shift',
    failure: 'Ctrl+Down and Ctrl+PageDown are unbound'
  },
  {
    name: 'horizontal-resets-column',
    file: 'src/native/input.rs',
    before: '            column = None;\n',
    after: '',
    native: true,
    test: 'vertical_keys_keep_a_preferred_column_and_extend_with_shift',
    failure: 'Down after Home must stay in column zero'
  },
  {
    name: 'selection-collapse-left',
    file: 'src/native/input.rs',
    before: 'if !shift && start != end && motion == Motion::Left {',
    after: 'if false {',
    native: true,
    test: 'horizontal_keys_move_by_grapheme_word_line_and_document_and_extend_with_shift',
    failure: 'Left collapses a selection to its start'
  },
  {
    name: 'selection-collapse-right',
    file: 'src/native/input.rs',
    before: '} else if !shift && start != end && motion == Motion::Right {',
    after: '} else if false {',
    native: true,
    test: 'horizontal_keys_move_by_grapheme_word_line_and_document_and_extend_with_shift',
    failure: 'Right collapses a selection to its end'
  },
  {
    name: 'page-keys-move-view',
    file: 'src/native/input.rs',
    before: '                scrolled = down_pixels;\n',
    after: '',
    native: true,
    test: 'page_keys_move_by_the_visible_height_and_keep_the_caret_in_view',
    failure: 'PageDown must move the view by the same whole lines'
  },
  {
    name: 'shift-click-ends-count',
    file: 'src/native/input.rs',
    before: '            pressed.clicks.reset();\n',
    after: '',
    native: true,
    test: 'shift_click_extends_the_selection_from_its_anchor',
    failure: 'a plain click after Shift+click must be a single click'
  },
  {
    name: 'caret-follow',
    file: 'src/native/input.rs',
    before: '        follow(&window, &state, scrolled);',
    after: '        render(&window, &state);',
    native: true,
    test: 'view_follows_the_caret_with_the_smallest_scroll',
    failure: 'one line past the bottom edge must scroll by the missing part of that line only'
  },
  // Soft wrap: the caret room at the right edge of the wrapped text, and visual rows in hits and selected ink.
  {
    name: 'caret-room',
    file: 'src/native/viewport.rs',
    before: 'pub(super) const CARET_ROOM: f32 = 2.0;',
    after: 'pub(super) const CARET_ROOM: f32 = 0.0;',
    native: true,
    test: 'the_caret_after_a_full_row_stays_inside_the_view',
    failure: 'the caret after a full row is cut by the view edge'
  },
  {
    name: 'hit-visual-row',
    file: 'src/shaped_row.rs',
    before: 'return self.hit_physical(x * scale, y * scale);',
    after: 'return self.hit_physical(x * scale, 0.0);',
    integration: 'wrap',
    test: 'hit_probes_the_visual_row_under_the_point',
    failure: 'lower row hit'
  },
  {
    name: 'click-visual-row',
    file: 'src/native/input.rs',
    before: 'let hit = candidate.hit_at(x, within, view.viewport.scale);',
    after: 'let hit = candidate.hit_at(x, 0.0, view.viewport.scale);',
    native: true,
    test: 'a_click_on_a_continuation_row_places_the_caret_in_that_row',
    failure: 'placed the caret at'
  },
  {
    name: 'selected-ink-visual-row',
    file: 'src/tile_scene.rs',
    before: 'if let Some(index) = row.visual_row_holding(rectangle.y, factor) {',
    after: 'if let Some(index) = row.visual_row_holding(rectangle.y, factor).filter(|first| return *first == 0) {',
    integration: 'wrap_rows',
    test: 'selected_ink_reaches_continuation_rows',
    failure: 'has selected ink'
  },
  // Soft wrap: the gutter, the measured rows, and the worker that measures them.
  {
    name: 'gutter-number-on-first-row-only',
    file: 'src/native/rows.rs',
    before: 'numbers.push(if visual == 0 {',
    after: 'numbers.push(if true {',
    native: true,
    test: 'the_gutter_numbers_first_rows_and_marks_continuation_rows',
    failure: 'continuation rows exist'
  },
  {
    name: 'pass-take-limit',
    file: 'src/native/tile.rs',
    before: 'worker.take_pass(TAKE_LIMIT)',
    after: 'worker.take_pass(usize::MAX)',
    native: true,
    test: 'opening_fifty_thousand_lines_never_blocks_the_window_thread',
    failure: 'past the bound of'
  },
  {
    name: 'view-kept-still-across-wrapped-rows',
    file: 'src/native/rows.rs',
    before: 'return (next.code_top(line) + within).max(0.0);',
    after: 'return offset;',
    native: true,
    test: 'rows_above_the_view_changing_height_move_no_visible_pixel',
    failure: 'the line at the top of the view moved on screen'
  },
  {
    name: 'rebreak-in-place',
    file: 'src/shaped_text/pass.rs',
    before: '&& self.rebreak(&mut stale, request.scale, width)',
    after: '&& false',
    integration: 'wrap_rows',
    test: 'a_new_width_breaks_kept_rows_again_without_shaping',
    failure: 'no line may be shaped again'
  },
  {
    name: 'pass-again-after-new-width',
    file: 'src/native/tile.rs',
    before: '    if current.shown.width != bounds.width {\n        current.pass_posted = None;\n    }',
    after: '    if false {\n        current.pass_posted = None;\n    }',
    native: true,
    test: 'a_measurement_applied_before_the_tile_of_a_new_width_leaves_no_stale_heights',
    failure: 'keep a stale height'
  },
  {
    name: 'tile-heights-merged',
    file: 'src/native/tile.rs',
    before: 'rows::absorb(&mut current, built.measured.iter().copied())',
    after: 'false',
    native: true,
    test: 'a_recenter_while_the_pass_runs_is_built_on_the_worker',
    failure: 'the worker\'s tile was discarded'
  },
  {
    name: 'mouse-drag-selects',
    file: 'ui/app.slint',
    before: '                    mouse-drag-pan-enabled: false;\n',
    after: '',
    native: true,
    test: 'drag_selects_text_by_characters_and_by_words_without_panning',
    failure: equality
  },
  {
    name: 'left-button-only',
    file: 'ui/app.slint',
    before: '                            if !self.pressed { return; }\n',
    after: '',
    native: true,
    test: 'drag_selects_text_by_characters_and_by_words_without_panning',
    failure: 'a drag with the middle button changed the selection'
  },
  {
    name: 'observer-scope-not-a-stop',
    file: 'ui/app.slint',
    before: '        focus-on-tab-navigation: false;\n',
    after: '',
    native: true,
    test: 'tab_cycles_tree_divider_and_source_while_find_is_closed',
    failure: 'Tab must cycle tree, divider, and source only'
  },
  {
    name: 'find-scope-not-a-stop',
    file: 'ui/find.slint',
    before: '        focus-on-tab-navigation: false;\n',
    after: '',
    native: true,
    test: 'tab_visits_tree_source_and_open_find_bar_in_reading_order',
    failure: 'Tab order with an open find bar'
  },
  {
    name: 'back-tab-leaves-source',
    file: 'ui/app.slint',
    before: ' || event.text == Key.Backtab',
    after: '',
    native: true,
    test: 'tab_cycles_tree_divider_and_source_while_find_is_closed',
    failure: 'the dedicated back-tab key did not leave the source view'
  },
  // Overscan tile: a view still inside the tile must not reach render, so it spends no tile, row, or image.
  {
    name: 'overscan-early-return',
    file: 'src/native/viewport.rs',
    before: '        if changed {\n            render(&window, &state);\n        } else {',
    after: '        if true {\n            render(&window, &state);\n        } else {',
    native: true,
    test: 'scrolling_inside_the_overscan_tile_does_no_source_work',
    failure: 'scrolling inside the overscan tile must not enter render'
  },
  // Scroll re-center: a reader's re-center that the worker should build must not reach the window thread's shaping and painting.
  {
    name: 'recenter-on-window-thread',
    file: 'src/native/viewport.rs',
    before: '        if current.shown.width == current.width {\n            current.scroll_wanted',
    after: '        if false {\n            current.scroll_wanted',
    native: true,
    test: 'leaving_the_overscan_tile_re_centers_on_the_worker_thread',
    failure: 'the window thread paints no tile for a scroll re-center'
  },
  // A new wrap width: the window thread paints the tile for it within a counted allowance; what does not fit goes to the worker.
  {
    name: 'resize-on-window-thread',
    file: 'src/native/viewport.rs',
    before: '            current.rewrap = true;',
    after: '            current.scroll_wanted = Some(tile::requested(current));',
    native: true,
    test: 'the_first_frame_after_a_new_width_is_wrapped_to_it',
    failure: 'the displayed tile is not for the new width'
  },
  {
    name: 'drawn-frame-after-resize',
    file: 'src/native/viewport.rs',
    before: '            current.rewrap = true;',
    after: '            current.scroll_wanted = Some(tile::requested(current));',
    native: true,
    test: 'a_frame_drawn_right_after_a_resize_event_is_wrapped_to_the_new_width',
    failure: 'rows of the drawn frame run to its right edge'
  },
  {
    name: 'rewrap-view-past-cap',
    file: 'src/native/rewrap.rs',
    before: 'if requested.count > REWRAP_LINES {',
    after: 'if false {',
    also: [
      {
        file: 'src/native/rewrap.rs',
        before: '                lines: REWRAP_LINES,',
        after: '                lines: usize::MAX,'
      },
      {
        file: 'src/native/rewrap.rs',
        before: 'return columns * rows * 4 <= REWRAP_RASTER_BYTES;',
        after: 'return true;'
      }
    ],
    native: true,
    test: 'a_view_taller_than_the_clip_cap_is_left_to_the_worker',
    failure: 'brought in lines for a view past the cap'
  },
  {
    name: 'rewrap-lines-bound',
    file: 'src/native/rewrap.rs',
    before: 'if requested.count > REWRAP_LINES {',
    after: 'if false {',
    also: [{
      file: 'src/native/rewrap.rs',
      before: '                lines: REWRAP_LINES,',
      after: '                lines: usize::MAX,'
    }],
    native: true,
    test: 'a_view_with_more_lines_than_the_allowance_is_left_to_the_worker',
    failure: 'was not left to the worker'
  },
  {
    name: 'rewrap-work-budget',
    file: 'src/shaped_text/pass.rs',
    before: '&& !limit.spend(line_work(document, row))',
    after: '&& !limit.spend(0)',
    native: true,
    test: 'lines_over_the_work_allowance_are_left_to_the_worker',
    failure: 'was not left to the worker'
  },
  {
    name: 'rewrap-tab-passes',
    file: 'src/shaped_text/pass.rs',
    before: '1 + passes_for(length)',
    after: '1',
    native: true,
    test: 'tabbed_lines_are_charged_for_their_corrective_passes_and_left_to_the_worker',
    failure: 'was not left to the worker'
  },
  {
    name: 'rewrap-raster-bound',
    file: 'src/native/rewrap.rs',
    before: 'return columns * rows * 4 <= REWRAP_RASTER_BYTES;',
    after: 'return true;',
    native: true,
    test: 'a_tile_with_a_huge_raster_is_left_to_the_worker',
    failure: 'was not left to the worker'
  },
  // Soft wrap keys: Home and End by visual row, Up and Down and paging by visual row, and the side of a wrap offset.
  {
    name: 'home-row-start',
    file: 'src/line_boundary.rs',
    before: 'return downstream(rows[row].start);',
    after: 'return downstream(line_start);',
    integration: 'wrap_keys',
    test: 'home_goes_to_the_row_start_then_the_first_non_blank',
    failure: 'inside the last row'
  },
  {
    name: 'home-continuation-start',
    file: 'src/line_boundary.rs',
    before: 'if head > rows[row].start {',
    after: 'if true {',
    integration: 'wrap_keys',
    test: 'home_goes_to_the_row_start_then_the_first_non_blank',
    failure: 'at a continuation row start: the first non-blank'
  },
  {
    name: 'home-first-row-toggle',
    file: 'src/line_boundary.rs',
    before: 'Some(indent) if indent < head => downstream(indent),',
    after: 'Some(indent) if false => downstream(indent),',
    integration: 'wrap_keys',
    test: 'home_goes_to_the_row_start_then_the_first_non_blank',
    failure: 'an end of the first row, past the first non-blank'
  },
  {
    name: 'end-trailing-blanks',
    file: 'src/line_boundary.rs',
    before: String.raw`if text.char(index) != ' ' && text.char(index) != '\t' {`,
    after: 'if true {',
    integration: 'wrap_keys',
    test: 'end_goes_to_the_row_end_then_the_line_end',
    failure: 'before the two trailing blanks of the first row'
  },
  {
    name: 'end-upper-row-side',
    file: 'src/line_boundary.rs',
    before: 'affinity: if row < last {',
    after: 'affinity: if false {',
    integration: 'wrap_keys',
    test: 'end_goes_to_the_row_end_then_the_line_end',
    failure: 'the end of the upper row is drawn on it'
  },
  {
    name: 'end-from-row-end',
    file: 'src/line_boundary.rs',
    before: 'if head == rows[row].end && row < last {',
    after: 'if false {',
    integration: 'wrap_keys',
    test: 'end_goes_to_the_row_end_then_the_line_end',
    failure: 'from the end of an upper row to the line end'
  },
  {
    name: 'end-blank-row-whole',
    file: 'src/line_boundary.rs',
    before: 'if starts_row {',
    after: 'if false {',
    integration: 'wrap_keys',
    test: 'end_goes_to_the_row_end_then_the_line_end',
    failure: 'a row of blanks only keeps its whole end'
  },
  {
    name: 'vertical-wrapped-rows',
    file: 'src/vertical_motion.rs',
    before: 'let current = shaper.wrapped_row(document, line, scale, width);',
    after: 'let current = shaper.row(document, line, scale);',
    integration: 'wrap_keys',
    test: 'down_and_up_visit_every_visual_row',
    failure: 'Down 1'
  },
  {
    name: 'vertical-remembered-side',
    file: 'src/vertical_motion.rs',
    before: '        && column.affinity == affinity\n',
    after: '        && true\n',
    integration: 'wrap_keys',
    test: 'a_remembered_column_belongs_to_one_side_of_a_wrap_offset',
    failure: 'moved a caret at the end of the upper row'
  },
  {
    name: 'page-counts-rows',
    file: 'src/row_map.rs',
    before: 'if row >= self.rows_of(line) {',
    after: 'if true {',
    integration: 'wrap_rows',
    test: 'a_page_step_counts_visual_rows_with_the_blocks_above_their_first_rows',
    failure: 'the first row of line 3 carries its 26 px block'
  },
  {
    name: 'select-keeps-side',
    file: 'src/document.rs',
    before: '            affinity: position.affinity,\n        };',
    after: '            affinity: ReadingAffinity::Downstream,\n        };',
    integration: 'document',
    test: 'the_side_of_a_wrap_offset_is_kept_by_selection_and_by_a_reload_that_leaves_the_head',
    failure: 'a selection keeps the side it names'
  },
  {
    name: 'reload-resets-side',
    file: 'src/document.rs',
    before: 'if mapped.head != position.head {',
    after: 'if false {',
    integration: 'document',
    test: 'the_side_of_a_wrap_offset_is_kept_by_selection_and_by_a_reload_that_leaves_the_head',
    failure: 'a head that moved takes the default side'
  },
  {
    name: 'empty-tail-not-a-row',
    file: 'src/shaped_row.rs',
    before: 'return self.layout.len() - usize::from(self.has_empty_tail());',
    after: 'return self.layout.len();',
    integration: 'wrap_keys',
    test: 'a_hanging_blank_at_the_end_of_the_text_leaves_no_empty_row',
    failure: 'no counted row is empty'
  },
  {
    name: 'indent-scale',
    file: 'src/shaped_text/layout.rs',
    before: 'let space = self.space_advance(request.scale) / request.scale;',
    after: 'let space = self.space_advance(request.scale);',
    integration: 'wrap',
    test: 'continuation_row_starts_at_the_indent',
    failure: 'scale 2: the continuation row starts at'
  },
  {
    name: 'rebreak-indent-scale',
    file: 'src/shaped_text/pass.rs',
    before: 'let space = self.space_advance(scale) / scale;',
    after: 'let space = self.space_advance(scale);',
    integration: 'wrap_rows',
    test: 'a_new_width_breaks_kept_rows_like_a_fresh_shaping_at_scale_two',
    failure: 'broken again like a fresh shaping'
  },
  // Soft wrap keys and pointer in the window.
  {
    name: 'key-end-follows-rows',
    file: 'src/native/input.rs',
    before: '        return Some(Edge::End);\n',
    after: '        return None;\n',
    native: true,
    test: 'home_and_end_follow_the_visual_rows',
    failure: 'End goes before the blanks that end the first row'
  },
  {
    name: 'page-by-visual-rows',
    file: 'src/native/input.rs',
    before: '(up as isize, down as isize)',
    after: '(up as isize, 1)',
    native: true,
    test: 'page_keys_move_by_visual_rows_and_keep_the_caret_in_view',
    failure: 'moves the caret by'
  },
  {
    name: 'word-keys-by-offset',
    file: 'src/native/input.rs',
    before: 'modified = Some(Motion::WordRight);',
    after: 'modified = Some(Motion::Right);',
    native: true,
    test: 'word_motion_crosses_a_wrap_offset_by_document_offset',
    failure: 'Ctrl+Right from'
  },
  {
    name: 'click-keeps-side',
    file: 'src/native/input.rs',
    before: '        position.affinity = if position.head == hit {\n            side\n        } else {',
    after: '        position.affinity = if false {\n            side\n        } else {',
    native: true,
    test: 'a_click_at_a_wrap_offset_keeps_the_side_that_was_clicked',
    failure: "right of the upper row's last glyph the caret is on the upper row"
  },
  {
    name: 'click-counts-visual-rows',
    file: 'src/native/input.rs',
    before: 'let row = line + current.row_map.line_rows().extra_before(line) + visual;',
    after: 'let row = line;',
    native: true,
    test: 'quick_presses_on_two_rows_of_one_line_are_not_a_double_click',
    failure: 'a second press on the next row of the line is a single click'
  },
  {
    name: 'drag-visual-row',
    file: 'src/native/input.rs',
    before: 'let hit = candidate.hit_at(x, within, view.viewport.scale);',
    after: 'let hit = candidate.hit_at(x, 0.0, view.viewport.scale);',
    native: true,
    test: 'a_drag_selects_across_visual_rows',
    failure: 'the drag reaches the third row'
  },
  {
    name: 'triple-click-whole-line',
    file: 'src/pointer_selection.rs',
    before: 'return line_range(text, position);',
    after: 'return word_range(text, position);',
    native: true,
    test: 'double_and_triple_clicks_work_on_wrapped_lines',
    failure: 'the triple click takes the whole line'
  },
  {
    name: 'view-follows-visual-row',
    file: 'src/native/viewport.rs',
    before: 'return row.caret_at(position, affinity, view.viewport.scale).row;',
    after: 'return 0;',
    native: true,
    test: 'the_view_follows_the_caret_through_every_visual_row',
    failure: 'left the caret row out of view'
  },
  {
    name: 'reveal-measures-its-line',
    file: 'src/native/viewport.rs',
    before: '    if down > 0 && has_wrap_width(current.width) {',
    after: '    if false {',
    native: true,
    test: 'revealing_a_late_row_of_an_unmeasured_last_line_puts_it_in_view',
    failure: 'the revealed range is outside the view'
  },
  // Minimap: the viewport box follows the scroll, a press centers its row, a drag scrolls with the box, and the wheel
  // over it reaches the view.
  {
    name: 'minimap-box-share',
    file: 'ui/app.slint',
    before: 'max(0px, root.minimap-track - root.minimap-box-height) * clamp(-scroll.content-y / root.minimap-range, 0, 1)',
    after: 'max(0px, root.minimap-track - root.minimap-box-height) * 0',
    native: true,
    test: 'the_box_matches_the_geometry_at_every_offset',
    failure: 'box top'
  },
  {
    name: 'minimap-slide-share',
    file: 'ui/app.slint',
    before: 'root.minimap-track) * clamp(-scroll.content-y / root.minimap-range, 0, 1)',
    after: 'root.minimap-track) * 0',
    native: true,
    test: 'the_box_matches_the_geometry_at_every_offset',
    failure: 'the picture\'s slide'
  },
  {
    name: 'minimap-drag-ratio',
    file: 'src/minimap/geometry.rs',
    before: 'return (start_offset + (y - start_y) * range / room).clamp(0.0, range);',
    after: 'return (start_offset + (y - start_y)).clamp(0.0, range);',
    integration: 'minimap',
    test: 'dragging_the_box_moves_it_with_the_pointer',
    failure: 'put the box at'
  },
  {
    name: 'minimap-position-clamp',
    file: 'src/minimap/geometry.rs',
    before: 'return ((y.clamp(0.0, track) + self.slide(offset)) / self.scale()).min(self.content);',
    after: 'return ((y + self.slide(offset)) / self.scale()).min(self.content);',
    integration: 'minimap',
    test: 'positions_outside_the_range_are_clamped_at_both_ends',
    failure: equality
  },
  // Minimap picture: a Vello scene of the window's own shaped lines drawn by the one renderer in chunks of at most 512 rows,
  // with the marks of the markup, a row cache that shapes only the lines that changed, and a slice that fits the device.
  {
    name: 'minimap-renders-glyphs',
    file: 'src/minimap/scene.rs',
    before: 'Ok(drawn) => items.extend(drawn),',
    after: 'Ok(_) => {}',
    native: true,
    test: 'the_minimap_of_a_tall_window_is_a_picture_of_glyphs_not_blocks',
    failure: 'inked rows'
  },
  {
    name: 'minimap-glyph-inks',
    file: 'src/minimap/scene.rs',
    before: 'Ok(drawn) => items.extend(drawn),',
    after: 'Ok(_) => {}',
    integration: 'minimap_scene',
    test: 'a_slice_draws_the_glyphs_of_its_lines_in_the_palette_inks',
    failure: 'a run for every line at least'
  },
  {
    name: 'minimap-fills-selection',
    file: 'src/minimap/fills.rs',
    before: '        fill(&mut items, cell, 0, fills.selection);',
    after: '        let _ = (cell, fills.selection);',
    native: true,
    test: 'the_minimap_of_a_selection_is_the_average_of_the_windows_pixels',
    failure: 'a selection at scale 1'
  },
  {
    name: 'minimap-fills-joined',
    file: 'src/minimap/fills.rs',
    before: '    for cell in joined(selected) {',
    after: '    for cell in selected {',
    integration: 'minimap_scene',
    test: 'the_selection_and_the_matches_are_marks_under_the_glyphs',
    failure: 'one mark for each of the two lines'
  },
  {
    name: 'minimap-fills-match',
    file: 'src/minimap/fills.rs',
    before: '        fill(&mut items, cell, 0, faded(fills.foreground, MATCH_FILL));',
    after: '        let _ = MATCH_FILL;',
    native: true,
    test: 'the_minimap_of_find_matches_is_the_average_of_the_windows_pixels',
    failure: 'find matches at scale 1'
  },
  {
    name: 'minimap-fills-boundary',
    file: 'src/minimap/fills.rs',
    before: '        fill(\n            &mut items,\n            cell,\n            width.max(1),\n            faded(fills.foreground, MATCH_BOUNDARY),\n        );',
    after: '        let _ = (width, MATCH_BOUNDARY);',
    native: true,
    test: 'the_minimap_of_find_matches_is_the_average_of_the_windows_pixels',
    failure: 'find matches at scale 1'
  },
  {
    name: 'minimap-frame-winding',
    file: 'src/tile_scene/encode.rs',
    before: '                    frame.move_to((inner.x0, inner.y0));\n                    frame.line_to((inner.x0, inner.y1));\n                    frame.line_to((inner.x1, inner.y1));\n                    frame.line_to((inner.x1, inner.y0));',
    after: '                    frame.move_to((inner.x0, inner.y0));\n                    frame.line_to((inner.x1, inner.y0));\n                    frame.line_to((inner.x1, inner.y1));\n                    frame.line_to((inner.x0, inner.y1));',
    also: [{
      file: 'src/tile_scene/encode.rs',
      before: '                    scene.fill(Fill::NonZero, Affine::IDENTITY, brush(*ink), None, &frame);',
      after: '                    scene.fill(Fill::EvenOdd, Affine::IDENTITY, brush(*ink), None, &frame);'
    }],
    native: true,
    test: 'a_frame_scaled_down_keeps_the_coverage_of_its_edges',
    failure: 'edge alpha'
  },
  {
    name: 'minimap-line-key-text',
    file: 'src/minimap/line_cache.rs',
    before: '        digest.bytes(chunk.as_bytes());',
    after: '        let _ = chunk;',
    integration: 'minimap_scene',
    test: 'an_edit_shapes_only_the_lines_it_changes',
    failure: equality
  },
  {
    name: 'minimap-line-key-text-reload',
    file: 'src/minimap/line_cache.rs',
    before: '        digest.bytes(chunk.as_bytes());',
    after: '        let _ = chunk;',
    native: true,
    test: 'an_external_edit_of_fifty_thousand_lines_shapes_only_the_line_it_touches',
    failure: 'the first pictures shaped'
  },
  {
    name: 'minimap-line-key-styles',
    file: 'src/minimap/line_cache.rs',
    before: '        styles: styles_digest(styles, start, start + slice.len_chars()),',
    after: '        styles: 0,',
    integration: 'minimap_scene',
    test: 'new_syntax_classes_shape_only_the_lines_they_reach',
    failure: 'one class changed on one line'
  },
  {
    name: 'minimap-line-key-wrap',
    file: 'src/minimap/line_cache.rs',
    before: '        wrap: wrap.map_or(0, f32::to_bits),',
    after: '        wrap: 0,',
    integration: 'minimap_scene',
    test: 'an_edit_shapes_only_the_lines_it_changes',
    failure: 'another wrap width shapes every line again'
  },
  {
    name: 'minimap-line-key-scale',
    file: 'src/minimap/line_cache.rs',
    before: '        scale: scale.to_bits(),',
    after: '        scale: 0,',
    integration: 'minimap_scene',
    test: 'an_edit_shapes_only_the_lines_it_changes',
    failure: 'another scale shapes every line again'
  },
  {
    name: 'minimap-work-budget',
    file: 'src/minimap/shape.rs',
    before: '                if cost > left {',
    after: '                if false {',
    integration: 'minimap_scene',
    test: 'a_line_the_budget_cannot_pay_for_is_drawn_blank',
    failure: 'attempt to subtract with overflow'
  },
  {
    name: 'minimap-cancel',
    file: 'src/minimap/shape.rs',
    before: '        if line % 32 == 0 && cancelled.is_cancelled() {',
    after: '        if false {',
    integration: 'minimap_scene',
    test: 'a_cancelled_build_answers_nothing',
    failure: 'build(&mut shaper'
  },
  {
    name: 'minimap-bands-uncapped',
    file: 'src/native/minimap/request.rs',
    before: '        BAND,\n        usize::MAX,\n        current.minimap.heading,',
    after: '        BAND,\n        1,\n        current.minimap.heading,',
    native: true,
    test: 'a_picture_taller_than_the_device_limit_covers_the_whole_track',
    failure: 'the picture is'
  },
  {
    name: 'minimap-texture-limit-key',
    file: 'src/native/minimap/request.rs',
    before: '        texture_limit: ide_app::shaped_text::texture_dimension(),\n',
    after: '        texture_limit: 0,\n',
    native: true,
    test: 'textures_that_stack_show_the_picture_one_texture_shows',
    failure: 'the picture cut for the lowered texture limit did not arrive'
  },
  {
    name: 'minimap-texture-runs',
    file: 'src/minimap/plan.rs',
    before: '        if index > start && rows + u64::from(*height) > most {',
    after: '        if false {',
    integration: 'minimap_scene',
    test: 'chunks_are_grouped_into_textures_that_fit_the_limit',
    failure: equality
  },
  {
    name: 'minimap-texture-top',
    file: 'src/native/gpu_tile/slice.rs',
    before: '                top: (top + start) as f32,',
    after: '                top: top as f32,',
    native: true,
    test: 'textures_that_stack_show_the_picture_one_texture_shows',
    failure: 'the composited column'
  },
  {
    name: 'minimap-slices-in-place',
    file: 'src/native/gpu_tile/types.rs',
    before: '    let Some(list) = shown.as_any().downcast_ref::<VecModel<Slice>>() else {',
    after: '    let Some(list) = shown.as_any().downcast_ref::<VecModel<Slice>>().filter(|_| false) else {',
    native: true,
    test: 'a_picture_of_the_same_shape_changes_the_rows_of_the_list_and_never_replaces_it',
    failure: 'the window was given a new list of textures'
  },
  {
    name: 'minimap-picture-waits-for-frame',
    file: 'src/native/minimap/picture.rs',
    before: '    state\n        .borrow()\n        .minimap\n        .redraw\n',
    after: '    ask_for_a_frame(window);\n    state\n        .borrow()\n        .minimap\n        .redraw\n',
    native: true,
    test: 'a_picture_that_a_frame_draws_asks_for_no_frame_of_its_own',
    failure: 'a frame was asked for at the hand-over'
  },
  {
    name: 'minimap-picture-asks-when-idle',
    file: 'src/native/minimap/picture.rs',
    before: '            if !gpu.minimap_waiting() {\n                return;\n            }\n',
    after: '            if true {\n                return;\n            }\n',
    native: true,
    test: 'a_picture_that_no_frame_draws_asks_for_a_frame',
    failure: 'an idle window did not ask for a frame'
  },
  {
    name: 'tile-left-for-next-step',
    file: 'src/native/tile/wait.rs',
    before: '        Some(at) => STEP_WAIT.checked_sub(at.elapsed()),',
    after: '        Some(_) => None,',
    native: true,
    test: 'a_tile_that_finishes_between_two_steps_is_left_for_the_next_step',
    failure: 'the poll took a tile although the next scroll step was due'
  },
  {
    name: 'tile-taken-by-the-step',
    file: 'src/native/viewport.rs',
    before: '            tile::poll(&window, &state);\n',
    after: '',
    native: true,
    test: 'a_tile_that_finishes_between_two_steps_is_left_for_the_next_step',
    failure: 'no scroll step took the finished tile'
  },
  {
    name: 'tile-taken-by-the-poll-when-idle',
    file: 'src/native/tile/wait.rs',
    before: '        Some(at) => STEP_WAIT.checked_sub(at.elapsed()),',
    after: '        Some(_) => Some(STEP_WAIT),',
    native: true,
    test: 'a_tile_that_finishes_with_no_step_due_is_taken_by_the_poll',
    failure: 'did not finish the re-center'
  },
  {
    name: 'tile-wakes-when-the-wait-ends',
    file: 'src/native/tile/wait.rs',
    before: '    wake_when_wait_ends(window, state, left);\n',
    after: '',
    native: true,
    test: 'a_tile_left_for_a_step_that_never_comes_is_taken_when_the_wait_ends',
    failure: 'set no wake-up for the end of the wait'
  },
  {
    name: 'tile-wake-cancelled-by-the-poll',
    file: 'src/native/tile.rs',
    before: '        current.tile_wake.stop();\n',
    after: '',
    native: true,
    test: 'a_scroll_step_that_takes_a_tile_cancels_the_wake_up_for_the_end_of_the_wait',
    failure: 'the wake-up stays set after a step took the tile'
  },
  {
    name: 'tile-wake-covers-whole-milliseconds',
    file: 'src/native/tile/wait.rs',
    before: 'left.as_micros().div_ceil(1000)',
    after: '(left.as_micros() / 1000)',
    native: true,
    test: 'a_wake_up_is_set_for_a_whole_number_of_milliseconds_that_covers_the_wait',
    failure: 'is not the whole milliseconds that cover it'
  },
  {
    name: 'minimap-most-bands',
    file: 'src/minimap/plan.rs',
    before: '    return ((f64::from(limit) / rows).floor() as usize).max(1);',
    after: '    return ((f64::from(limit) / rows).ceil() as usize).max(1);',
    integration: 'minimap_scene',
    test: 'the_most_bands_fit_the_texture_limit_and_one_more_does_not',
    failure: 'pass the limit'
  },
  {
    name: 'minimap-chunk-rows',
    file: 'src/minimap/plan.rs',
    before: '        let end = (start + CHUNK_ROWS as usize).min(rows.end);',
    after: '        let end = rows.end;',
    integration: 'minimap_scene',
    test: 'a_tall_slice_is_cut_into_chunks_of_at_most_the_chunk_rows',
    failure: equality
  },
  // Measured on 2026-10-23: the test passes at chunks of up to 4096 rows (the renderer holds that many of the densest
  // text since selected rows are drawn once) and fails from 5120 rows, so the removed run takes a chunk as tall as the
  // widest slice.
  {
    name: 'minimap-chunk-size',
    file: 'src/minimap/plan.rs',
    before: 'pub const CHUNK_ROWS: u32 = 512;',
    after: 'pub const CHUNK_ROWS: u32 = 8192;',
    native: true,
    test: 'the_densest_chunks_are_not_blank',
    failure: 'nearly blank'
  },
  // Measured on 2026-10-23: two chunks drawn as one render keep their ink at 512 rows each and lose it at 4096 rows each
  // (8192 rows in one render), so this test is what keeps a margin of two inside the buffers.
  {
    name: 'minimap-chunk-margin',
    file: 'src/minimap/plan.rs',
    before: 'pub const CHUNK_ROWS: u32 = 512;',
    after: 'pub const CHUNK_ROWS: u32 = 4096;',
    native: true,
    test: 'the_chunk_size_leaves_a_factor_of_two_inside_what_one_render_holds',
    failure: 'one render of 8192 rows drew'
  },
  {
    name: 'minimap-chunk-cut',
    file: 'src/minimap/chunks.rs',
    before: '            if unit.reach.1 >= *low && unit.reach.0 <= *high {',
    after: '            if true {',
    integration: 'minimap_scene',
    test: 'the_cut_keeps_every_item_and_a_line_on_a_seam_in_both_chunks',
    failure: 'the cut keeps too much'
  },
  {
    name: 'minimap-chunk-copy-row',
    file: 'src/native/gpu_tile/slice.rs',
    before: '                row += chunk.height;',
    after: '                row += 0 * chunk.height;',
    native: true,
    test: 'a_picture_taller_than_the_device_limit_covers_the_whole_track',
    failure: 'is blank'
  },
  {
    name: 'minimap-slice-top',
    file: 'src/minimap/scene.rs',
    before: '        top: rows.start as u32,',
    after: '        top: 0,',
    integration: 'minimap_scene',
    test: 'a_long_text_shapes_only_the_lines_near_its_window',
    failure: equality
  },
  {
    name: 'minimap-window-key-selection',
    file: 'src/native/minimap/request.rs',
    before: '        selection,\n        find,\n',
    after: '        selection: {\n            let _ = selection;\n            None\n        },\n        find,\n',
    native: true,
    test: 'the_minimap_of_a_selection_is_the_average_of_the_windows_pixels',
    failure: 'the minimap picture did not follow the change'
  },
  {
    name: 'minimap-text-x',
    file: 'src/minimap/horizontal.rs',
    before: '    return (x / width).clamp(0.0, 1.0) * span;',
    after: '    return (x / width) * span;',
    integration: 'minimap',
    test: 'a_position_across_the_minimap_is_the_same_share_of_the_texts_width',
    failure: equality
  },
  {
    name: 'minimap-press-column',
    file: 'src/native/minimap/pointer.rs',
    before: '    let across = text_x(\n        x,',
    after: '    let across = text_x(\n        0.0,',
    native: true,
    test: 'a_press_across_the_minimap_puts_the_caret_in_the_same_share_of_the_text',
    failure: 'do not grow across the minimap'
  },
  {
    name: 'minimap-press-centers',
    file: 'src/native/minimap/pointer.rs',
    before: '.centered(top + CODE_ROW / 2.0)',
    after: '.centered(top)',
    native: true,
    test: 'a_press_outside_the_box_jumps_to_the_row_and_the_keyboard_still_works',
    failure: 'the pressed row is centered'
  },
  {
    name: 'minimap-press-caret',
    file: 'src/native/minimap/pointer.rs',
    before: '    current.document.select(position);\n',
    after: '',
    native: true,
    test: 'a_press_outside_the_box_jumps_to_the_row_and_the_keyboard_still_works',
    failure: 'the caret is on the pressed row'
  },
  {
    name: 'minimap-shift-anchor',
    file: 'src/native/minimap/pointer.rs',
    before: '    if !shift {\n        position.anchor = hit;\n    }\n',
    after: '    position.anchor = hit;\n',
    native: true,
    test: 'a_press_outside_the_box_jumps_to_the_row_and_the_keyboard_still_works',
    failure: 'a Shift press keeps the anchor'
  },
  {
    name: 'minimap-drag-scrolls',
    file: 'src/native/minimap/pointer.rs',
    before: '        scroll_to(&active, &drag_state, offset);\n',
    after: '        let _ = offset;\n',
    native: true,
    test: 'dragging_the_box_scrolls_continuously_and_release_ends_it',
    failure: 'step 1'
  },
  {
    name: 'minimap-release-ends-drag',
    file: 'src/native/minimap/pointer.rs',
    before: '        release_state.borrow_mut().minimap.drag = None;\n',
    after: '        let _ = &release_state;\n',
    native: true,
    test: 'dragging_the_box_scrolls_continuously_and_release_ends_it',
    failure: 'after the release the pointer moves nothing'
  },
  {
    name: 'minimap-wheel-reaches-view',
    file: 'ui/minimap.slint',
    before: '        moved => {\n            if !self.pressed { return; }\n            root.drag(self.mouse-y);\n        }\n',
    after: '        moved => {\n            if !self.pressed { return; }\n            root.drag(self.mouse-y);\n        }\n        scroll-event(event) => { return EventResult.accept; }\n',
    native: true,
    test: 'the_wheel_over_the_minimap_scrolls_like_the_wheel_over_the_text',
    failure: 'the wheel over the minimap scrolled by the same amount'
  },
  // Minimap worker boundary: the picture is valid for the text and never for the scroll offset, it is built on the
  // worker, and a stale picture is never shown.
  {
    name: 'minimap-built-on-worker',
    file: 'src/native/minimap_worker.rs',
    before: '.name(THREAD_NAME.to_string())',
    after: '.name("window".to_string())',
    native: true,
    test: 'the_picture_is_built_on_the_worker_and_stays_inside_its_bounds',
    failure: 'a picture was built off the worker'
  },
  {
    name: 'minimap-follows-measurement',
    file: 'src/native/minimap/pace.rs',
    before: '    if !steady {\n        return Plan::Wait(DEBOUNCE);',
    after: '    if true {\n        return Plan::Wait(DEBOUNCE);',
    native: true,
    test: 'each_kind_of_change_has_its_own_pace',
    failure: 'after the slide period are posted at once'
  },
  {
    name: 'minimap-resize-keeps-wait',
    file: 'src/native/minimap/pace.rs',
    before: '    if waiting {\n        return Plan::Keep;\n    }',
    after: '    if false {\n        return Plan::Keep;\n    }',
    native: true,
    test: 'each_kind_of_change_has_its_own_pace',
    failure: 'a wait that is already running is kept'
  },
  {
    name: 'minimap-throttle-period',
    file: 'src/native/minimap/pace.rs',
    before: '    if since >= period {\n        return Plan::Post;\n    }',
    after: '    if false {\n        return Plan::Post;\n    }',
    native: true,
    test: 'each_kind_of_change_has_its_own_pace',
    failure: 'overflow when subtracting durations'
  },
  {
    name: 'minimap-slide-period',
    file: 'src/native/minimap/pace.rs',
    before: '    let period = if scrolling { SLIDE } else { THROTTLE };',
    after: '    let period = THROTTLE;',
    native: true,
    test: 'each_kind_of_change_has_its_own_pace',
    failure: 'other bands inside the period'
  },
  {
    name: 'minimap-key-has-bands',
    file: 'src/native/minimap/request.rs',
    before: '        window: (bands.start, bands.end),',
    after: '        window: (0, 0),',
    native: true,
    test: 'a_scroll_posts_a_request_only_when_it_names_other_bands',
    failure: 'the picture of the scrolled bands did not arrive'
  },
  {
    name: 'minimap-scroll-notes-bands',
    file: 'src/native/viewport.rs',
    before: '            super::minimap::note(&window, &state);',
    after: '            let _ = &window;',
    native: true,
    test: 'a_scroll_posts_a_request_only_when_it_names_other_bands',
    failure: 'the picture of the scrolled bands did not arrive'
  },
  {
    name: 'minimap-slide-offset',
    file: 'src/minimap/geometry.rs',
    before: '        return overhang * (offset / range).clamp(0.0, 1.0);',
    after: '        return overhang * 0.0 * (offset / range).clamp(0.0, 1.0);',
    integration: 'minimap',
    test: 'a_long_text_keeps_the_scale_and_its_picture_slides_past_the_track',
    failure: 'slide(geometry.max_offset())'
  },
  {
    name: 'minimap-window-margin',
    file: 'src/minimap/geometry.rs',
    before: '            (((from - behind as f32 * step) / step).floor().max(0.0) as usize).min(total - 1);',
    after: '            ((from / step).floor().max(0.0) as usize).min(total - 1);',
    integration: 'minimap',
    test: 'the_bands_to_keep_surround_the_part_of_the_picture_in_view',
    failure: 'after the part in view'
  },
  // A fling outruns a picture that holds one band beyond the track: the window holds more bands on the side the view heads toward.
  {
    name: 'minimap-lead-down',
    file: 'src/minimap/plan.rs',
    before: '            Self::Down { .. } => (1, LEAD),',
    after: '            Self::Down { .. } => (1, 1),',
    integration: 'minimap',
    test: 'the_window_of_a_fling_reaches_ahead_on_the_side_it_heads_toward',
    failure: 'heading down holds the lead after the track'
  },
  {
    name: 'minimap-lead-up',
    file: 'src/minimap/plan.rs',
    before: '            Self::Up { .. } => (LEAD, 1),',
    after: '            Self::Up { .. } => (1, 1),',
    integration: 'minimap',
    test: 'the_window_of_a_fling_reaches_ahead_on_the_side_it_heads_toward',
    failure: 'heading up holds the lead before the track'
  },
  {
    name: 'minimap-heading-needs-a-band',
    file: 'src/minimap/plan.rs',
    before: '        if moved.abs() < step {\n            return self;\n        }',
    after: '        if false {\n            return self;\n        }',
    integration: 'minimap',
    test: 'a_heading_follows_the_track_by_whole_bands',
    failure: 'less than a band changes nothing'
  },
  {
    name: 'minimap-heading-followed',
    file: 'src/native/minimap/request.rs',
    before: '    follow(window, &mut current);\n',
    after: '',
    native: true,
    test: 'a_fling_is_given_a_picture_that_reaches_the_bands_it_is_heading_toward',
    failure: 'a fling down is not heading down'
  },
  {
    name: 'minimap-window-has-heading',
    file: 'src/native/minimap/request.rs',
    before: '        usize::MAX,\n        current.minimap.heading,',
    after: '        usize::MAX,\n        Heading::Still { at: 0.0 },',
    native: true,
    test: 'a_fling_is_given_a_picture_that_reaches_the_bands_it_is_heading_toward',
    failure: 'the picture of the scrolled bands did not arrive'
  },
  {
    name: 'minimap-stale-text',
    file: 'src/native/minimap/picture.rs',
    before: '&& reply.key.stamp == displayed(&current);',
    after: '&& true;',
    native: true,
    test: 'a_stale_picture_is_dropped_and_a_current_one_is_shown',
    failure: 'a stale picture was set'
  },
  {
    name: 'minimap-older-request',
    file: 'src/native/minimap/picture.rs',
    before: 'let fresh = reply.id >= current.minimap.shown_id &&',
    after: 'let fresh = true &&',
    native: true,
    test: 'a_stale_picture_is_dropped_and_a_current_one_is_shown',
    failure: 'a stale picture was set'
  },
  // Markdown: the palette entries, the injection of fenced languages, and the ink and face of each markup role.
  {
    name: 'markup-scope-heading',
    file: 'src/syntax.rs',
    before: '    "markup.heading",\n    "markup.bold",',
    after: '    "markup.headings",\n    "markup.bold",',
    integration: 'syntax_markdown',
    test: 'atx_headings_are_painted_as_headings',
    failure: 'expected markup.heading on'
  },
  {
    name: 'markup-raw-block-unpainted',
    file: 'src/syntax.rs',
    before: '    "markup.raw.inline",\n',
    after: '    "markup.raw",\n',
    integration: 'syntax_markdown',
    test: 'fenced_code_has_no_block_color',
    failure: 'a fenced block is painted with a block color'
  },
  {
    name: 'fence-language-injection',
    file: 'src/syntax_loader.rs',
    before: 'return self.helix.language_for_marker(marker);',
    after: 'return None;',
    integration: 'syntax_markdown',
    test: 'fenced_code_is_painted_in_its_own_language',
    failure: 'expected keyword on'
  },
  {
    name: 'markup-heading-ink',
    file: 'src/code_colors.rs',
    before: 'role == 12 || role == HEADING_ROLE || role == LINK_ROLE',
    after: 'role == 12 || role == LINK_ROLE',
    integration: 'syntax_markdown_faces',
    test: 'markdown_roles_are_painted_in_their_hue_in_both_schemes',
    failure: 'runs are not the hue'
  },
  {
    name: 'markup-weight-push',
    file: 'src/shaped_text/layout.rs',
    before: 'if face.weight_offset != 0.0 {',
    after: 'if false {',
    integration: 'syntax_markdown_faces',
    test: 'heavy_roles_shape_in_a_heavier_instance_of_the_roman_face',
    failure: 'makes one heavier run'
  },
  {
    name: 'markup-italic-push',
    file: 'src/shaped_text/layout.rs',
    before: 'if face.italic {',
    after: 'if false {',
    integration: 'syntax_markdown_faces',
    test: 'italic_roles_use_the_real_italic_face',
    failure: 'role 16: '
  },
  {
    name: 'markup-strike-weight',
    file: 'src/source_style.rs',
    before: 'weight_offset: -100.0,',
    after: 'weight_offset: 0.0,',
    integration: 'syntax_markdown_faces',
    test: 'struck_text_is_lighter_than_the_source_weight',
    failure: 'faces: '
  },
  // Hints of an answer: a hint that stands on a line the answer did not ask about is not part of the answer, or the
  // same hints are added to that line again with every answer (the block of line 96 of a test file four rows, then
  // eight, then twelve, in the code view and in the minimap).
  {
    name: 'hints-asked-lines',
    file: 'src/annotation/blocks.rs',
    before: '            if !asked.contains(&line) {',
    after: '            if false && !asked.contains(&line) {',
    integration: 'annotation',
    test: 'hints_outside_the_asked_lines_are_not_part_of_an_answer',
    failure: 'an answer that repeated the hint of an unasked line added it again'
  },
  {
    name: 'hints-asked-lines-window',
    file: 'src/annotation/blocks.rs',
    before: '            if !asked.contains(&line) {',
    after: '            if false && !asked.contains(&line) {',
    native: true,
    test: 'an_answer_for_other_lines_adds_no_hint_to_a_line_it_did_not_ask_about',
    failure: 'an answer for lines 2 and 3 added hints to line 0'
  },
];
// An optional comma-separated list reruns only the named guards, for example after adding one.
const only = process.env
  .usage_only ? new Set(process.env
    .usage_only
    .split(',')) : undefined;
const selected = only ? cases.filter(item => only.has(item.name)) : cases;
if (only && (selected.length !== only.size)) throw new Error(`Unknown guard name in: ${
   process.env
     .usage_only}`);
const results: RunResult[] = [];
const baselined = new Set<string>();

/**
 Runs one guard test in the disposable copy, records the result, and stops the script when it is not the expected one.

 @param item - guard whose test runs

 @param phase - baseline (unmodified), removed (guard removed, must fail), or restored (must pass again)

 @throws {Error} when the guard names no test kind or the run is not the expected result
 */
function run({
  item,
  phase
}: {
  item: Guard;
  phase: string;
}): void {
  if (!item.native && !item.lib && item.integration === undefined) throw new Error(`Guard ${item.name} names no native, library, or integration test`);
  // A case names a native test, a unit test of the library (`lib`), or an integration test file.
  let command = [
    'cargo',
    'test',
    '--offline',
    '--no-default-features',
    '--test',
    item.integration ?? '',
    item.test,
    '--',
    '--nocapture'
  ];
  if (item.native) command = [
    'cargo',
    'nextest',
    'run',
    '--offline',
    '--bin',
    'monochromatic-ide',
    '--filter-expr',
    `test(${  item.test
       })`
  ];
  if (item.lib) command = [
    'cargo',
    'test',
    '--offline',
    '--no-default-features',
    '--lib',
    item.test,
    '--',
    '--nocapture'
  ];
  const result = spawnSync(
    'podman',
    [
      'run',
      '--rm',
      '--network=none',
      ...containerLimits.limitArguments,
      '--security-opt',
      'label=disable',
      '--volume',
      `${source  }:/work`,
      '--volume',
      `${cache  }:/work/target`,
      '--volume',
      `${cargoHome  }:/cargo`,
      '--workdir',
      '/work',
      ...containerLimits.environmentArguments,
      '--env',
      'SLINT_EMIT_DEBUG_INFO=1',
      ...gpuArguments,
      '--env',
      'HELIX_RUNTIME=/work/target/debug/runtime',
      'localhost/monochromatic/ide',
      ...command,
    ],
    {
      encoding: 'utf8',
      maxBuffer: 16 * 1_024
        * 1_024
    }
  );
  const output = (result.stdout ?? '') + (result.stderr ?? '');
  writeFileSync(
    join(
      artifact,
      `${item.name  }-${
         phase
         }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  const failedAsExpected = (result.status !== 0) && output.includes(item.test)
    && output.includes(item.failure);
  const passedAsExpected = (result.status === 0) && output.includes(item.test)
    && /1 (?:test|passed)/.test(output);
  const accepted = phase === 'removed' ? failedAsExpected : passedAsExpected;
  results.push({
    name: item.name,
    phase,
    test: item.test,
    status: result.status,
    accepted
  });
  writeFileSync(
    join(
      artifact,
      'results.json'
    ),
    JSON.stringify(
      results,
      null,
      2
    )
  );
  if (!accepted) throw new Error(`Unexpected ${  phase  } result for ${  item.name  }; inspect ${  artifact}`);
  console.log(
    JSON.stringify(results.at(-1)),
  );
}
for (const item of selected) {
  // A restored run of an earlier case is already this test's unmodified baseline.
  if (!baselined.has(item.test)) {
    run({
      item,
      phase: 'baseline'
    });
    baselined.add(item.test);
  }
  // A guard whose rule is enforced at two places names the second place in `also`, and both are removed together.
  const edits = [
    item,
    ...(item.also ?? [])
  ].map(edit => ({
    path: join(
      source,
      edit.file
    ),
    original: readFileSync(
      join(
        source,
        edit.file
      ),
      'utf8'
    ),
    edit
  }));
  try {
    for (const {
      path,
      edit
    } of edits) {
      const current = readFileSync(
        path,
        'utf8'
      );
      const changed = replaceOne({
        text: current,
        before: edit.before,
        after: edit.after,
        occurrence: edit.occurrence
      });
      if (changed === current) throw new Error(`Mutation did not change ${  edit.file}`);
      writeFileSync(
        path,
        changed
      );
    }
    run({
      item,
      phase: 'removed'
    });
  } finally { for (const {
    path,
    original
  } of edits) writeFileSync(
    path,
    original
  ); }
  run({
    item,
    phase: 'restored'
  });
}
console.log(`Source guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
