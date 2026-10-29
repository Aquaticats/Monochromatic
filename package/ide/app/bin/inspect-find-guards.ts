#!/usr/bin/env node
// Observe committed in-file find regressions failing after guard removal in a disposable package copy.
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
  'ide-find-guard-'
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
console.log(`FIND_GUARD_ARTIFACT=${  artifact}`);

/**
 A guard: the edit that removes it and the test that must then fail.
 */
interface Guard {
  name: string;
  file: string;
  before: string;
  after: string;
  occurrence?: number | undefined;
  integration?: string;
  native?: boolean;
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
const clearCell = 'find_clear_cell_is_48px_and_every_part_of_it_clears';
const clearEdges = 'find_clear_cell_ends_at_its_edges_and_hides_without_focus_or_text';
const cases: Guard[] = [
  // Reply identity: each component of the three-part tag is removed separately.
  {
    name: 'identity-file',
    file: 'src/find_worker.rs',
    before: 'return self.file == wanted.file',
    after: 'return true',
    test: 'reply_for_another_file_generation_is_rejected',
    failure: 'a reply for another file generation was accepted'
  },
  {
    name: 'identity-revision',
    file: 'src/find_worker.rs',
    before: '&& self.revision == wanted.revision',
    after: '&& true',
    test: 'reply_for_another_content_revision_is_rejected',
    failure: 'a reply for another content revision was accepted'
  },
  {
    name: 'identity-query',
    file: 'src/find_worker.rs',
    before: '&& self.query == wanted.query',
    after: '&& true',
    test: 'reply_for_another_query_is_rejected',
    failure: 'a reply for another query was accepted'
  },
  {
    name: 'cancel-wants-nothing',
    file: 'src/find_worker.rs',
    before: '        self.wanted = None;\n',
    after: '',
    test: 'reply_after_cancel_is_rejected',
    failure: 'a reply was accepted after cancellation'
  },
  // The same comparison through a real worker thread: latest request, reload, and navigation.
  {
    name: 'latest-request-wins',
    file: 'src/find_worker.rs',
    before: '&& self.query == wanted.query',
    after: '&& true',
    integration: 'find_worker',
    test: 'newest_request_wins_over_running_and_waiting_requests',
    failure: 'a reply for a superseded query was returned'
  },
  {
    name: 'reload-retag',
    file: 'src/find_worker.rs',
    before: '&& self.revision == wanted.revision',
    after: '&& true',
    integration: 'find_worker',
    test: 'reload_retags_requests_with_the_new_revision',
    failure: equality
  },
  {
    name: 'navigation-retag',
    file: 'src/find_worker.rs',
    before: 'return self.file == wanted.file',
    after: 'return true',
    integration: 'find_worker',
    test: 'navigation_retags_requests_with_the_new_file_generation',
    failure: 'a reply for the previous file was returned'
  },
  {
    name: 'navigable-query',
    file: 'src/find_worker.rs',
    before: '&& self.query == wanted.query',
    after: '&& true',
    integration: 'find_navigation',
    test: 'results_are_navigable_only_for_their_exact_identity',
    failure: 'matches from a superseded query were navigable'
  },
  // Painting validity: accepted results describe exactly one file generation and revision.
  {
    name: 'paint-file',
    file: 'src/find_navigation.rs',
    before: 'if self.identity.file == file && ',
    after: 'if ',
    integration: 'find_navigation',
    test: 'results_are_painted_only_for_their_file_generation_and_revision',
    failure: 'matches from another file generation were painted'
  },
  {
    name: 'paint-revision',
    file: 'src/find_navigation.rs',
    before: ' && self.identity.revision == revision {',
    after: ' {',
    integration: 'find_navigation',
    test: 'results_are_painted_only_for_their_file_generation_and_revision',
    failure: 'matches from another content revision were painted'
  },
  // Matching semantics and bounds.
  {
    name: 'empty-query',
    file: 'src/find.rs',
    before: 'if query.is_empty() {',
    after: 'if false {',
    integration: 'find',
    test: 'empty_query_matches_nothing',
    failure: equality
  },
  {
    name: 'literal-escape',
    file: 'src/find.rs',
    before: 'RegexBuilder::new(&escape(query))',
    after: 'RegexBuilder::new(query)',
    integration: 'find',
    test: 'regex_metacharacters_are_literal',
    failure: equality
  },
  {
    name: 'case-folding',
    file: 'src/find.rs',
    before: 'builder.case_insensitive(true);',
    after: 'builder.case_insensitive(false);',
    integration: 'find_reference',
    test: 'plain_matcher_differs_from_browser_reference_only_in_pinned_cases',
    failure: "the plain matcher's differences from the browser reference changed"
  },
  {
    name: 'match-limit',
    file: 'src/find.rs',
    before: 'if ranges.len() == limit {',
    after: 'if false {',
    integration: 'find',
    test: 'match_count_is_bounded_and_reports_truncation',
    failure: 'matches beyond the limit were retained'
  },
  {
    name: 'query-limit',
    file: 'src/find.rs',
    before: 'if length > MAX_FIND_QUERY_CHARS {',
    after: 'if false {',
    integration: 'find',
    test: 'query_length_is_bounded_with_a_diagnostic',
    failure: 'an over-long query was accepted'
  },
  {
    name: 'source-limit',
    file: 'src/find.rs',
    before: 'if bytes > source_bytes {',
    after: 'if false {',
    integration: 'find',
    test: 'source_size_is_bounded_with_a_diagnostic',
    failure: 'source above the bound was searched'
  },
  // Paint inputs.
  {
    name: 'active-excluded',
    file: 'src/find_paint.rs',
    before: 'if active == Some(*range) {',
    after: 'if false {',
    integration: 'find_paint',
    test: 'active_match_is_excluded_from_other_match_rectangles',
    failure: 'the active match was also drawn as an ordinary match'
  },
  {
    name: 'find-match-visual-rows',
    file: 'src/shaped_row.rs',
    before: 'y: self.top + self.visual_row_offset(row, scale),',
    after: 'y: self.top,',
    integration: 'wrap',
    test: 'range_yields_one_rectangle_per_visual_row',
    failure: 'has a rectangle at'
  },
  {
    name: 'frame-stamp',
    file: 'src/source_frame.rs',
    before: '\n            && (Arc::ptr_eq(&self.matches, &other.matches) || self.matches == other.matches)',
    after: '',
    integration: 'source_frame',
    test: 'find_matches_invalidate_and_equal_lists_reuse',
    failure: 'adding matches must invalidate the frame'
  },
  // Native bar: recompute triggers, focus, overlay interplay, and lifecycle.
  {
    name: 'reload-recompute',
    file: 'src/native/find/tick.rs',
    before: 'revision: state.document.revision(),',
    after: 'revision: 0,',
    native: true,
    test: 'native_find_recomputes_after_external_reload_and_follows_selection_correspondence',
    failure: 'matches were not recomputed for the reloaded revision'
  },
  {
    name: 'file-recompute',
    file: 'src/native/find/tick.rs',
    before: 'file: state.file_generation,',
    after: 'file: 1,',
    native: true,
    test: 'native_find_recomputes_for_a_switched_file_and_keeps_input_focus',
    failure: 'matches were not recomputed for the new file'
  },
  {
    name: 'seek-only-on-edit',
    file: 'src/native/find/tick.rs',
    before: 'if find.seek\n        && let Some(index)',
    after: 'if let Some(index)',
    native: true,
    test: 'native_find_recomputes_after_external_reload_and_follows_selection_correspondence',
    failure: 'the selection must follow the replaced region, not jump to a later match'
  },
  {
    name: 'closed-bar-stays-idle',
    file: 'src/native/find/tick.rs',
    before: 'if !find.open || !find.available {',
    after: 'if !find.available {',
    native: true,
    test: 'native_find_opens_types_steps_wraps_reveals_and_closes',
    failure: 'highlights returned after closing'
  },
  {
    name: 'empty-text-clears',
    file: 'src/native/find/tick.rs',
    before: 'if find.query.is_empty() {',
    after: 'if false {',
    native: true,
    test: 'native_find_reports_refused_text_and_clears_for_empty_text',
    failure: 'empty find text must show no count'
  },
  {
    name: 'closed-bar-edit',
    file: 'src/native/find/session.rs',
    before: 'if !find.open {',
    after: 'if false {',
    native: true,
    test: 'native_find_reports_refused_text_and_clears_for_empty_text',
    failure: 'an edit reached the closed find bar'
  },
  {
    name: 'closed-bar-dismiss',
    file: 'src/native/find/session.rs',
    before: 'if !find.open {',
    after: 'if false {',
    occurrence: 1,
    native: true,
    test: 'native_find_reports_refused_text_and_clears_for_empty_text',
    failure: 'dismissing a closed find bar moved keyboard focus'
  },
  {
    name: 'find-input-focus',
    file: 'src/native/navigation/open.rs',
    before: 'if window.get_find_has_focus() {',
    after: 'if false {',
    native: true,
    test: 'native_find_recomputes_for_a_switched_file_and_keeps_input_focus',
    failure: 'a file open moved keyboard focus out of the find input'
  },
  {
    name: 'no-file-ignored',
    file: 'src/native/find/session.rs',
    before: 'if !window.get_source_available() {',
    after: 'if false {',
    native: true,
    test: 'native_find_reports_refused_text_and_clears_for_empty_text',
    failure: 'Ctrl+F opened the find bar without a displayed file'
  },
  {
    name: 'search-overlay-modal',
    file: 'src/native/find/session.rs',
    before: 'if window.get_search_open() {',
    after: 'if false {',
    native: true,
    test: 'native_find_and_search_overlay_close_topmost_first',
    failure: 'Ctrl+F took keyboard focus from the modal search overlay'
  },
  // The find box's clear control: a 48px cell that is a click target everywhere, reports the edit,
  // and is shown only with text and keyboard focus.
  {
    name: 'clear-cell-size',
    file: 'ui/query-input.slint',
    before: 'private property <length> clear-size: 48px;',
    after: 'private property <length> clear-size: 16px;',
    native: true,
    test: clearCell,
    failure: 'below 48 by 48'
  },
  {
    name: 'clear-target-fills-cell',
    file: 'ui/query-input.slint',
    before: 'clear-touch := TouchArea {\n                    width: 100%;\n                    height: 100%;',
    after: 'clear-touch := TouchArea {\n                    x: 16px;\n                    y: 16px;\n                    width: 16px;\n                    height: 16px;',
    native: true,
    test: clearCell,
    failure: 'down the clear cell did not clear the find text'
  },
  {
    name: 'clear-reports-edit',
    file: 'ui/query-input.slint',
    before: '        root.edited("");\n',
    after: '',
    native: true,
    test: clearCell,
    failure: 'clearing did not update the find results'
  },
  {
    name: 'clear-needs-text',
    file: 'ui/query-input.slint',
    before: 'root.text != "" && root.enabled && input.has-focus;',
    after: 'root.enabled && input.has-focus;',
    native: true,
    test: clearEdges,
    failure: 'the clear control is shown for empty find text'
  },
  {
    name: 'clear-needs-focus',
    file: 'ui/query-input.slint',
    before: 'root.text != "" && root.enabled && input.has-focus;',
    after: 'root.text != "" && root.enabled;',
    native: true,
    test: clearEdges,
    failure: 'the clear control is shown without keyboard focus in the box'
  },
  // Through element handles, as assistive tools reach the box: setting the value runs find, and the
  // clear control is a button.
  {
    name: 'a11y-set-value-edits',
    file: 'ui/query-input.slint',
    before: '        root.text = value;\n        root.edited(value);\n',
    after: '        root.text = value;\n',
    native: true,
    test: 'find_box_and_clear_control_expose_role_label_value_and_actions',
    failure: "setting the find box's value did not run find"
  },
  {
    name: 'a11y-clear-role',
    file: 'ui/query-input.slint',
    before: '                accessible-role: button;',
    after: '                accessible-role: AccessibleRole.text;',
    native: true,
    test: 'find_box_and_clear_control_expose_role_label_value_and_actions',
    failure: "the find box's clear control is not a button"
  },
  // Answer of 2026-10-06: selected find text takes the ink chosen from the fill, passed through the find bar.
  {
    name: 'find-box-ink-passed',
    file: 'ui/app.slint',
    before: '                error-message: root.find-error;\n                selected-ink: root.selected-row-ink;\n',
    after: '                error-message: root.find-error;\n',
    native: true,
    test: 'find_box_selection_uses_the_ink_chosen_from_the_fill',
    failure: 'dark: selected find text is not drawn in light ink'
  },
  {
    name: 'find-bar-ink-passed',
    file: 'ui/find.slint',
    before: '                selected-ink: root.selected-ink;\n',
    after: '',
    native: true,
    test: 'find_box_selection_uses_the_ink_chosen_from_the_fill',
    failure: 'dark: selected find text is not drawn in light ink'
  },
  {
    name: 'escape-topmost',
    file: 'ui/app.slint',
    before: 'root.find-open && !root.search-open {',
    after: 'root.find-open {',
    native: true,
    test: 'native_find_and_search_overlay_close_topmost_first',
    failure: 'Escape did not close the topmost search overlay'
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
      `test(${  item.test  })`
    ]
    : [
      'cargo',
      'test',
      '--offline',
      '--no-default-features',
      ...(item.integration ? [
        '--test',
        item.integration
      ] : ['--lib']),
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
      after: item.after,
      occurrence: item.occurrence
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
console.log(`Find guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
