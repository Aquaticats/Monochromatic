#!/usr/bin/env node
// Observe committed inlay-hint and diagnostic regressions failing after guard removal in a disposable package copy.
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
  'ide-annotation-guard-'
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
console.log(`ANNOTATION_GUARD_ARTIFACT=${  artifact}`);

/**
 The fields every guard has: the edit that removes it and the test that must then fail.
 */
interface GuardBase {
  name: string;
  file: string;
  before: string;
  after: string;
  test: string;
  failure: string;
}

/**
 A guard whose test is a unit test of the window binary, run by nextest.
 */
interface NativeGuard extends GuardBase {
  native: true;
  integration?: undefined;
}

/**
 A guard whose test is in one integration test file.
 */
interface IntegrationGuard extends GuardBase {
  native?: undefined;
  integration: string;
}

/**
 A guard: the edit that removes it and the one test that must then fail.
 */
type Guard = NativeGuard | IntegrationGuard;

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
 `text` with the one occurrence of `before` replaced by `after`.

 @param text - source of the file to mutate

 @param before - anchor that the guard exists to protect; absence means the guard moved with its code

 @param after - replacement that removes the guard

 @returns the edited text

 @throws {Error} when `before` is absent or occurs more than once, since either way the guard may be aimed at the wrong code
 */
function replaceOne({
  text,
  before,
  after
}: {
  text: string;
  before: string;
  after: string;
}): string {
  const start = text.indexOf(before);
  if (start === (-1)) throw new Error(`Mutation source anchor is absent: ${  before}`);
  if (text.indexOf(
    before,
    start + before.length
  ) >= 0) throw new Error(`Mutation source anchor is not unique: ${  before}`);
  return text.slice(
    0,
    start
  ) + after
    + text.slice(start + before.length);
}
const equality = 'assertion `left == right` failed';
const cases: Guard[] = [
  // Stale snapshots: hints and diagnostics are painted only for the displayed file generation and revision.
  {
    name: 'hint-stamp',
    file: 'src/annotation/blocks.rs',
    before: 'let labelled = self.labelled == Some(displayed);',
    after: 'let labelled = self.labelled.is_some();',
    integration: 'annotation',
    test: 'stale_snapshots_paint_nothing_and_show_no_card',
    failure: 'a stale hint was painted'
  },
  {
    name: 'hint-stamp-native',
    file: 'src/annotation/blocks.rs',
    before: 'let labelled = self.labelled == Some(displayed);',
    after: 'let labelled = self.labelled.is_some();',
    native: true,
    test: 'stale_snapshots_disappear_after_reload_and_file_switch',
    failure: 'stale hints were painted after reload'
  },
  {
    name: 'diagnostic-stamp',
    file: 'src/annotation.rs',
    before: 'return snapshot.stamp == stamp;',
    after: 'return snapshot.stamp.file == snapshot.stamp.file || stamp.file == 0;',
    integration: 'annotation',
    test: 'stale_snapshots_paint_nothing_and_show_no_card',
    failure: 'a stale message was painted'
  },
  {
    name: 'diagnostic-stamp-native',
    file: 'src/annotation.rs',
    before: 'return snapshot.stamp == stamp;',
    after: 'return snapshot.stamp.file == snapshot.stamp.file || stamp.file == 0;',
    native: true,
    test: 'stale_snapshots_disappear_after_reload_and_file_switch',
    failure: 'stale markers were painted after reload'
  },
  // The Language poll's entry points: a snapshot of other text is refused, and accepted diagnostics are indexed.
  {
    name: 'accept-hints-stamp',
    file: 'src/annotation.rs',
    before: 'snapshot: Arc<HintsSnapshot>,\n    ) -> bool {\n        if snapshot.stamp != displayed {',
    after: 'snapshot: Arc<HintsSnapshot>,\n    ) -> bool {\n        if snapshot.stamp.file != displayed.file {',
    integration: 'annotation',
    test: 'accepting_stores_only_snapshots_of_the_displayed_text',
    failure: 'hints for an earlier revision were stored'
  },
  {
    name: 'accept-diagnostics-stamp',
    file: 'src/annotation.rs',
    before: 'snapshot: Arc<DiagnosticsSnapshot>,\n    ) -> bool {\n        if snapshot.stamp != displayed {',
    after: 'snapshot: Arc<DiagnosticsSnapshot>,\n    ) -> bool {\n        if snapshot.stamp.file != displayed.file {',
    integration: 'annotation',
    test: 'accepting_stores_only_snapshots_of_the_displayed_text',
    failure: 'diagnostics for an earlier revision were stored'
  },
  {
    name: 'accept-diagnostics-index',
    file: 'src/annotation.rs',
    before: '        self.problems = problems;\n        self.reach = reach;\n',
    after: '        let _unused = (problems, reach);\n',
    integration: 'annotation',
    test: 'accepting_stores_only_snapshots_of_the_displayed_text',
    failure: 'accepted diagnostics were not indexed'
  },
  {
    name: 'accessor-stamp',
    file: 'src/annotation.rs',
    before: 'let held = self.hints.as_deref()?;\n        // The snapshot is lent only while it describes the displayed text.\n        if held.stamp == displayed {',
    after: 'let held = self.hints.as_deref()?;\n        // The snapshot is lent only while it describes the displayed text.\n        if held.stamp.file == displayed.file {',
    integration: 'annotation',
    test: 'accepting_stores_only_snapshots_of_the_displayed_text',
    failure: 'hints were handed out for another revision'
  },
  {
    name: 'accept-native',
    file: 'src/annotation.rs',
    before: 'snapshot: Arc<HintsSnapshot>,\n    ) -> bool {\n        if snapshot.stamp != displayed {',
    after: 'snapshot: Arc<HintsSnapshot>,\n    ) -> bool {\n        if snapshot.stamp.file != displayed.file {',
    native: true,
    test: 'accepted_snapshots_show_after_one_render',
    failure: 'hints for another revision were accepted'
  },
  // Hints of one revision accumulate by the line range each answer asked about.
  {
    name: 'hint-range-replaced',
    file: 'src/annotation/blocks.rs',
    before: '        for line in stale {\n            self.labels.remove(&line);\n            self.placed.remove(&line);\n        }\n',
    after: '',
    integration: 'annotation',
    test: 'hints_of_one_revision_accumulate_by_line_range',
    failure: 'an answer did not replace the hints of its own lines'
  },
  {
    name: 'hint-revision-dropped',
    file: 'src/annotation/blocks.rs',
    before: '            self.labels.clear();\n            self.placed.clear();\n',
    after: '',
    integration: 'annotation',
    test: 'hints_of_one_revision_accumulate_by_line_range',
    failure: 'hints of the previous revision were kept'
  },
  // Space held across a reload: for one text only, hint space until hints for the line arrive, all of it until its time.
  {
    name: 'held-text-only',
    file: 'src/annotation/blocks.rs',
    before: 'if self.held_for == Some(displayed) {',
    after: 'if self.held_for.is_some() {',
    integration: 'annotation',
    test: 'held_space_keeps_block_heights_until_hints_return_or_time_passes',
    failure: 'held space was applied to another text'
  },
  {
    name: 'hint-space-released',
    file: 'src/annotation/blocks.rs',
    before: '            if asked.contains(&part.line) {\n                part.hints = 0.0;\n            }\n',
    after: '',
    integration: 'annotation',
    test: 'held_space_keeps_block_heights_until_hints_return_or_time_passes',
    failure: 'hint space was not given up when hints for the line arrived'
  },
  {
    name: 'hold-expiry',
    file: 'src/annotation/blocks.rs',
    before: 'let due = match self.held_until {\n            Some(until) => now >= until,\n            None => false,\n        };',
    after: 'let due = false;',
    integration: 'annotation',
    test: 'held_space_keeps_block_heights_until_hints_return_or_time_passes',
    failure: 'space was not given up when its time had passed'
  },
  {
    name: 'hold-on-reload',
    file: 'src/native/reload.rs',
    before: '        rows::hold(&mut current, carried, window.window().scale_factor());',
    after: '        rows::refresh(&mut current, window.window().scale_factor());',
    native: true,
    test: 'a_reload_holds_row_space_until_annotations_return',
    failure: 'the reload changed where lines are'
  },
  {
    name: 'hold-expiry-render',
    file: 'src/native/reload.rs',
    before: '        if due {\n            render(&active_window, &state);\n        }\n',
    after: '',
    native: true,
    test: 'stale_snapshots_disappear_after_reload_and_file_switch',
    failure: 'held space was never given up'
  },
  // Bounded windows: only the diagnostics of the materialized rows are underlined, and ranges starting above them are found.
  {
    name: 'diagnostic-window',
    file: 'src/annotation.rs',
    before: 'if problem.mark.start >= end {',
    after: 'if false {',
    native: true,
    test: 'only_visible_annotation_changes_repaint',
    failure: 'a change outside the materialized rows repainted the source'
  },
  {
    name: 'reach-index',
    file: 'src/annotation.rs',
    before: '        let from = first_reaching(&self.reach, start);',
    after: '        let from = self\n            .problems\n            .partition_point(|problem| return problem.mark.start < start);',
    integration: 'annotation',
    test: 'diagnostic_window_finds_ranges_starting_above_it',
    failure: equality
  },
  {
    name: 'omitted-severity',
    file: 'src/annotation/severity.rs',
    before: 'return severity.unwrap_or(Severity::Warning);',
    after: 'return severity.unwrap_or(Severity::Error);',
    integration: 'annotation',
    test: 'caret_problems_include_range_ends_and_order_by_severity',
    failure: equality
  },
  {
    name: 'worst-first',
    file: 'src/annotation.rs',
    before: '            problems.sort_by_key(severity_order);\n',
    after: '',
    native: true,
    test: 'caret_problems_follow_the_caret_and_are_spelled_out_in_full',
    failure: 'messages above a line are listed worst first, then in source order'
  },
  // Message rows: a runaway message and a pile of messages are capped with a count.
  {
    name: 'message-cap',
    file: 'src/virtual_row.rs',
    before: 'if total > MESSAGE_ROWS && index + 1 == MESSAGE_ROWS {',
    after: 'if false {',
    integration: 'virtual_row',
    test: 'a_runaway_message_is_cut_and_counts_what_is_left_out',
    failure: equality
  },
  {
    name: 'pile-cap',
    file: 'src/virtual_row.rs',
    before: 'if index == LINE_MESSAGES {',
    after: 'if false {',
    integration: 'virtual_row',
    test: 'a_pile_of_messages_is_capped_with_a_count',
    failure: equality
  },
  // Frame stamp: visible annotations and their inks are paint inputs; a frame whose lines did not change is moved, not repainted.
  {
    name: 'stamp-annotations',
    file: 'src/source_frame.rs',
    before: '\n            && (Arc::ptr_eq(&self.annotations, &other.annotations)\n                || self.annotations == other.annotations)',
    after: '',
    integration: 'annotation_frame',
    test: 'visible_annotations_and_inks_are_paint_inputs',
    failure: 'a different hint label must repaint'
  },
  {
    name: 'stamp-annotations-native',
    file: 'src/source_frame.rs',
    before: '\n            && (Arc::ptr_eq(&self.annotations, &other.annotations)\n                || self.annotations == other.annotations)',
    after: '',
    native: true,
    test: 'annotations_leave_reading_geometry_copy_and_find_unchanged',
    failure: 'the injected diagnostics were not marked (positive control)'
  },
  {
    name: 'stamp-inks',
    file: 'src/source_frame.rs',
    before: '\n            && self.annotation_colors == other.annotation_colors;',
    after: ';',
    integration: 'annotation_frame',
    test: 'visible_annotations_and_inks_are_paint_inputs',
    failure: 'a different annotation ink must repaint'
  },
  {
    name: 'rows-wait-for-scrolling',
    file: 'src/native/rows.rs',
    before: 'if still != offset && same_text && !force && (moving || scrolling(current)) {',
    after: 'if false && moving {',
    native: true,
    test: 'rows_above_the_view_wait_until_scrolling_has_stopped',
    failure: 'rows above the view moved the offset while the reader was scrolling'
  },
  {
    name: 'rebase-frame',
    file: 'src/native/render.rs',
    before: '            shaped, row_map, ..\n        } = &mut *current;\n        if let Some(view) = shaped.as_mut()\n            && view.rebase(row_map)\n        {\n            matches_changed = true;\n        }\n    } else if measured_here.is_none() && same_inputs',
    after: '            shaped, row_map, ..\n        } = &mut *current;\n        if let Some(view) = shaped.as_mut()\n            && false\n        {\n            matches_changed = true;\n        }\n    } else if measured_here.is_none() && same_inputs',
    native: true,
    test: 'rows_arriving_above_the_view_move_no_visible_pixel',
    failure: 'a visible pixel moved when'
  },
  // Geometry: terminator runs, point runs, overlap order, hints and messages at their pixel x, packing, and line styles.
  {
    name: 'terminator-run',
    file: 'src/annotation_layout.rs',
    before: 'if mark.start <= row_end && mark.end > row_end {',
    after: 'if false {',
    integration: 'annotation_layout',
    test: 'multi_line_range_marks_every_row_including_an_empty_line',
    failure: equality
  },
  {
    name: 'point-run',
    file: 'src/annotation_layout.rs',
    before: 'if mark.start == mark.end {',
    after: 'if false {',
    integration: 'annotation_layout',
    test: 'point_ranges_get_a_terminator_wide_run',
    failure: equality
  },
  {
    name: 'overlap-order',
    file: 'src/annotation_layout.rs',
    before: 'underlines.sort_by_key(paint_order);',
    after: '',
    integration: 'annotation_layout',
    test: 'overlapping_ranges_draw_the_worst_severity_last',
    failure: equality
  },
  {
    name: 'hint-at-its-position',
    file: 'src/annotation_layout.rs',
    before: 'let at = row.caret_x(label.position, scale);',
    after: 'let at: f32 = 0.0;',
    integration: 'annotation_layout',
    test: 'hints_stand_at_the_exact_pixel_x_of_their_position',
    failure: 'does not stand above it'
  },
  {
    name: 'message-at-its-diagnostic',
    file: 'src/annotation_layout.rs',
    before: 'let mut x = row.caret_x(block.messages[first].start, scale);',
    after: 'let mut x: f32 = 0.0;',
    integration: 'annotation_layout',
    test: 'message_rows_start_at_their_diagnostic_and_sit_tight_on_the_code_row',
    failure: equality
  },
  {
    name: 'hint-gap',
    file: 'src/annotation_layout.rs',
    before: 'if rows == 0 || index > 0 || x < end + HINT_GAP {',
    after: 'if rows == 0 || index > 0 {',
    integration: 'annotation_layout',
    test: 'overlapping_hints_take_a_new_row_and_only_the_current_row_is_considered',
    failure: equality
  },
  {
    name: 'rows-above-the-code-row',
    file: 'src/tile_scene.rs',
    before: 'let top = view.map.code_top(text.line) - text.rise;',
    after: 'let top = view.map.code_top(text.line);',
    native: true,
    test: 'virtual_rows_paint_above_the_code_row_and_never_in_it',
    failure: 'pixels of its ink'
  },
  {
    name: 'line-styles',
    file: 'src/tile_scene/underline.rs',
    before: 'return Some(vec![4.0 * unit, (period - 4.0) * unit]);',
    after: 'return None;',
    native: true,
    test: 'each_severity_has_its_own_line_style_in_its_ink',
    failure: 'warning dashes'
  },
  {
    name: 'selected-underline-ink',
    file: 'src/tile_scene/frame.rs',
    before: '(severity_ink, colors.selected),',
    after: '(severity_ink, severity_ink),',
    native: true,
    test: 'selected_underlines_take_the_selected_ink',
    failure: 'the selected underline is not in the selected ink'
  },
  // The vertical mapping: a block belongs to the code row beneath it, blocks take their height, and the view is kept still.
  {
    name: 'block-belongs-beneath',
    file: 'src/row_map.rs',
    before: '        return Place {\n            line,\n            in_block: bounded < code,\n        };',
    after: '        return Place {\n            line: if bounded < code { line.saturating_sub(1) } else { line },\n            in_block: bounded < code,\n        };',
    integration: 'row_map',
    test: 'every_pixel_belongs_to_one_line_and_blocks_to_the_row_beneath',
    failure: equality
  },
  {
    name: 'block-belongs-beneath-native',
    file: 'src/row_map.rs',
    before: '        return Place {\n            line,\n            in_block: bounded < code,\n        };',
    after: '        return Place {\n            line: if bounded < code { line.saturating_sub(1) } else { line },\n            in_block: bounded < code,\n        };',
    native: true,
    test: 'virtual_rows_are_not_source_text',
    failure: 'did not act on that line'
  },
  {
    name: 'blocks-take-their-height',
    file: 'src/native/rows.rs',
    before: 'raised.push((block.line, block.height()));',
    after: 'raised.push((block.line, 0.0));',
    native: true,
    test: 'rows_arriving_in_view_move_only_lines_beneath_them',
    failure: 'changed its pixels'
  },
  {
    name: 'view-kept-still',
    file: 'src/native/rows.rs',
    before: 'let still = anchored(&current.row_map, &layout.map, offset).min(limit(&layout.map, height));',
    after: 'let still = offset.min(limit(&layout.map, height));',
    native: true,
    test: 'rows_arriving_above_the_view_move_no_visible_pixel',
    failure: 'the offset did not follow the rows that arrived above the view'
  },
  {
    name: 'top-stays-top',
    file: 'src/native/rows.rs',
    before: '    if offset <= 0.0 {\n        return 0.0;\n    }\n',
    after: '',
    native: true,
    test: 'rows_arriving_above_the_view_move_no_visible_pixel',
    failure: equality
  },
  // Soft wrap: an underline across a wrap break is drawn on the visual row it marks, and selected ink follows it.
  {
    name: 'underline-visual-row',
    file: 'src/annotation_layout.rs',
    before: 'offset: rect.y - row.top,',
    after: 'offset: 0.0,',
    integration: 'wrap_rows',
    test: 'underlines_across_a_wrap_break_stand_on_their_own_rows',
    failure: 'is not drawn on its own visual row'
  },
  // Soft wrap: hints are packed against the broken line and held inside the text area, messages break to it.
  {
    name: 'hint-broken-line',
    file: 'src/annotation/blocks.rs',
    before: 'Some(width) => shaper.wrapped_row(document, line, scale, width),',
    after: 'Some(_) => shaper.row(document, line, scale),',
    integration: 'wrap_annotations',
    test: 'assembled_hints_stand_where_the_broken_line_draws_them',
    failure: equality
  },
  {
    name: 'hint-width-repack',
    file: 'src/annotation/blocks.rs',
    before: '                let repack = match self.placed.get(&line) {\n                    None => true,\n                    Some(known) => known.wrap != wrap && focus.contains(&line),',
    after: '                let repack = match self.placed.get(&line) {\n                    None => true,\n                    Some(_) => false,',
    integration: 'wrap_annotations',
    test: 'assembled_hints_stand_where_the_broken_line_draws_them',
    failure: 'another width packs the hint again'
  },
  {
    name: 'hint-edge',
    file: 'src/annotation_layout.rs',
    before: 'x = x.min((limit - piece_width).max(0.0));',
    after: 'x = x.max(0.0);',
    integration: 'wrap_annotations',
    test: 'a_hint_that_would_pass_the_text_area_ends_at_its_edge',
    failure: 'the last hint ends at the right edge'
  },
  {
    name: 'label-pieces',
    file: 'src/annotation_layout.rs',
    before: 'Some(limit) if width > limit => shaper.wrap_row_text(&label.text, scale, limit),',
    after: 'Some(_) if false => Vec::new(),',
    integration: 'wrap_annotations',
    test: 'a_label_wider_than_the_text_area_breaks_into_rows',
    failure: 'the label needs several rows'
  },
  {
    name: 'message-break',
    file: 'src/annotation/blocks.rs',
    before: 'if !(own + widths[index] <= limit) {',
    after: 'if false {',
    integration: 'wrap_annotations',
    test: 'message_rows_break_to_the_text_area',
    failure: 'a narrow area takes more rows'
  },
  {
    name: 'message-rebreak',
    file: 'src/annotation/blocks.rs',
    before: '                        let refit = match self.wrapped.get(&line) {\n                            None => true,\n                            Some(known) => known.wrap != wrap && focus.contains(&line),',
    after: '                        let refit = match self.wrapped.get(&line) {\n                            None => true,\n                            Some(_) => false,',
    integration: 'wrap_annotations',
    test: 'message_rows_break_to_the_text_area',
    failure: 'a narrow area takes more rows'
  },
  {
    name: 'message-measured-once',
    file: 'src/annotation/blocks.rs',
    before: 'let widths = match self.natural.entry(line) {',
    after: 'self.natural.remove(&line);\n                            let widths = match self.natural.entry(line) {',
    integration: 'wrap_annotations',
    test: 'a_new_width_does_only_the_work_it_needs',
    failure: 'no message row is measured twice'
  },
  {
    name: 'message-cap-broken',
    file: 'src/annotation/blocks.rs',
    before: 'if end - first > MESSAGE_ROWS {',
    after: 'if false {',
    integration: 'wrap_annotations',
    test: 'a_message_keeps_its_row_cap_at_a_narrow_width',
    failure: 'the cap holds after the rows are broken'
  },
  {
    name: 'message-edge',
    file: 'src/annotation_layout.rs',
    before: 'x = x.min((limit - reach).max(0.0));',
    after: 'x = x.max(0.0);',
    integration: 'wrap_annotations',
    test: 'a_message_far_to_the_right_moves_left_to_stay_inside_the_text_area',
    failure: 'a message row passes the text area'
  },
  {
    name: 'deferred-width-stays-on-worker',
    file: 'src/native/render.rs',
    before: 'shown.width != requested.width && differs_only_in_annotations',
    after: 'false && differs_only_in_annotations',
    native: true,
    test: 'a_new_width_left_to_the_worker_keeps_the_displayed_tile_until_the_workers_tile_arrives',
    failure: 'painted a tile for the resize'
  },
  {
    name: 'discarded-tile-asked-again',
    file: 'src/native/tile.rs',
    before: '        if remapped || wanted {',
    after: '        if remapped {',
    native: true,
    test: 'a_tile_discarded_while_the_window_waits_for_one_is_asked_for_again',
    failure: 'the discarded tile was not asked for again'
  },
  {
    name: 'repack-in-focus-only',
    file: 'src/annotation/blocks.rs',
    before: 'if repack {',
    after: 'if repack || self.placed.get(&line).is_some_and(|known| return known.wrap != wrap) {',
    integration: 'wrap_annotations',
    test: 'a_new_width_applies_to_the_lines_near_the_view_first',
    failure: 'only the hinted line near the view is packed again'
  },
  {
    name: 'refit-in-focus-only',
    file: 'src/annotation/blocks.rs',
    before: 'if refit {',
    after: 'if refit || self.wrapped.get(&line).is_some_and(|known| return known.wrap != wrap) {',
    integration: 'wrap_annotations',
    test: 'a_new_width_applies_to_the_lines_near_the_view_first',
    failure: 'a far line keeps its rows'
  },
  {
    name: 'stale-packing-dropped',
    file: 'src/annotation/blocks.rs',
    before: '        if !labelled {\n            self.placed.clear();\n        }\n',
    after: '',
    integration: 'wrap_annotations',
    test: 'a_new_width_applies_to_the_lines_near_the_view_first',
    failure: 'stale_in(Some(500.0)'
  },
  {
    name: 'stale-flag-in-key',
    file: 'src/native/rows.rs',
    before: '        current\n            .annotations\n            .stale_in(wrap_of(current), &focus_of(current)),\n',
    after: '        false,\n',
    native: true,
    test: 'hints_far_from_the_view_are_packed_again_when_the_view_comes_near',
    failure: 'the far line was not packed for the new width'
  },
  {
    name: 'message-line-rejoined',
    file: 'src/annotation/blocks.rs',
    before: 'while to < rows.len() && rows[to].gap.is_some() {',
    after: 'while false {',
    integration: 'wrap_annotations',
    test: 'a_message_line_is_broken_again_as_a_whole',
    failure: 'is a short row'
  },
  {
    name: 'cut-word-joined-whole',
    file: 'src/annotation/blocks.rs',
    before: 'joined.push_str(row.gap.unwrap_or(" "));',
    after: 'joined.push_str(" ");',
    integration: 'wrap_annotations',
    test: 'a_message_line_is_broken_again_as_a_whole',
    failure: 'the word was split by a blank'
  },
  {
    name: 'rows-wait-for-scroll-step',
    file: 'src/native/rows.rs',
    before: 'let moving = (offset - current.offset).abs() > 0.01;',
    after: 'let moving = false;',
    native: true,
    test: 'rows_above_the_view_wait_until_scrolling_has_stopped',
    failure: 'change handler had not run yet'
  },
  // Native: the gutter letter names the worst severity starting on a line, and its column never moves the text.
  {
    name: 'gutter-letter',
    file: 'src/native/rows.rs',
    before: 'mark = i32::from(rank(worst.severity)) + 1;',
    after: 'let _unused = worst;\n                mark = 0;',
    native: true,
    test: 'gutter_letters_show_the_worst_severity_in_front_of_the_line_number',
    failure: 'marks handed to the window'
  },
  {
    name: 'gutter-worst',
    file: 'src/native/rows.rs',
    before: '&& let Some(worst) = block.messages.first()',
    after: '&& let Some(worst) = block.messages.last()',
    native: true,
    test: 'gutter_letters_show_the_worst_severity_in_front_of_the_line_number',
    failure: 'marks handed to the window'
  },
  {
    name: 'gutter-column-fixed',
    file: 'ui/app.slint',
    before: 'out property <length> gutter-width: 6px + root.mark-advance + root.mark-gap + root.number-width + root.number-gap;',
    after: 'out property <length> gutter-width: 6px + (root.row-marks[0] > 0 ? root.mark-advance + root.mark-gap : 0px) + root.number-width + root.number-gap;',
    native: true,
    test: 'gutter_letters_show_the_worst_severity_in_front_of_the_line_number',
    failure: 'moved sideways when diagnostics arrived'
  },
  // Native: the letter stands the same gap before every line's own number, and the number column follows the file.
  {
    name: 'gutter-letter-beside-number',
    file: 'ui/app.slint',
    before: 'x: number.x - root.mark-gap - self.width;',
    after: 'x: 6px;',
    native: true,
    test: 'the_letter_stands_the_same_gap_before_one_two_and_three_digit_numbers',
    failure: 'the gap before a two-digit number differs'
  },
  {
    name: 'gutter-digits-follow-file',
    file: 'ui/app.slint',
    before: 'text: max(root.line-count, 100);',
    after: 'text: 100;',
    native: true,
    test: 'the_gutter_gains_a_digit_past_999_lines_and_the_pointer_follows',
    failure: 'widens the gutter by one digit'
  },
];
// An optional comma-separated list reruns only the named guards, for example after adding one.
const only = process.env
  .usage_only ? new Set(process.env
    .usage_only
    .split(',')) : undefined;
const selected = only ? cases.filter(item => only.has(item.name)) : cases;
if (only && (selected.length !== only.size)) throw new Error(`Unknown guard name in: ${  process.env
  .usage_only}`);
const results: RunResult[] = [];
const baselined = new Set();

/**
 Runs one guard test in the disposable copy, records the result, and stops the script when it is not the expected one.

 @param item - guard whose test runs

 @param phase - baseline (unmodified), removed (guard removed, must fail), or restored (must pass again)

 @throws {Error} when the run is not the expected result
 */
function run({
  item,
  phase
}: {
  item: Guard;
  phase: string;
}): void {
  const command = item.native
    ? [
      'cargo',
      'nextest',
      'run',
      '--offline',
      '--bin',
      'monochromatic-ide',
      '--filter-expr',
      `test(${  item.test
         })`
    ]
    : [
      'cargo',
      'test',
      '--offline',
      '--no-default-features',
      '--test',
      item.integration,
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
  if (!accepted) throw new Error(`Unexpected ${  phase
     } result for ${
     item.name
     }; inspect ${
     artifact}`);
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
  const path = join(
    source,
    item.file
  );
  const original = readFileSync(
    path,
    'utf8'
  );
  try {
    const changed = replaceOne({
      text: original,
      before: item.before,
      after: item.after
    });
    if (changed === original) throw new Error(`Mutation did not change ${  item.file}`);
    writeFileSync(
      path,
      changed
    );
    run({
      item,
      phase: 'removed'
    });
  } finally { writeFileSync(
    path,
    original
  ); }
  run({
    item,
    phase: 'restored'
  });
}
console.log(`Annotation guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
