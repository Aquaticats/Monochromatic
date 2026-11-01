#!/usr/bin/env node
// Observe committed search regressions failing after guard removal in a disposable package copy.
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
  'ide-search-guard-'
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
console.log(`SEARCH_GUARD_ARTIFACT=${  artifact}`);

/**
 The fields every search guard has.
 */
interface GuardBase {
  name: string;
  native?: boolean;
  integration?: string;
  test: string;
}

/**
 A guard that removes one anchored piece of source.
 */
interface ReplaceGuard extends GuardBase {
  file: string;
  before: string;
  after: string;
  occurrence?: number | undefined;
  alter?: undefined;
  control?: undefined;
  failure: string;
}

/**
 A guard whose removal is a function of the file text, for source that no single anchor describes.
 */
interface AlterGuard extends GuardBase {
  file: string;
  alter: (text: string) => string;
  control?: undefined;
  failure: string;
}

/**
 A lifecycle control: the unmodified test runs and nothing is removed.
 */
interface ControlGuard extends GuardBase {
  control: true;
  failure?: undefined;
}

/**
 A search guard: a replacement, a computed alteration, or a control.
 */
type Guard = ReplaceGuard | AlterGuard | ControlGuard;

/**
 A container run recorded in `results.json` and echoed as one JSON line.
 */
interface RunResult {
  name: string;
  phase: string;
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
const cases: Guard[] = [
  {
    name: 'reply-generation',
    file: 'src/search_worker.rs',
    before: 'if current.generation == self.generation {',
    after: 'if true {',
    test: 'unread_reply_identity_must_equal_the_current_generation',
    failure: 'a stale reply crossed the current generation boundary'
  },
  {
    name: 'result-count-limit',
    file: 'src/search_collect.rs',
    before: 'if hits.len() == maximum {',
    after: 'if false {',
    integration: 'search_worker',
    test: 'both_result_streams_stop_at_their_approved_caps',
    failure: 'assertion `left == right` failed'
  },
  {
    name: 'record-limit',
    file: 'src/search_io.rs',
    before: 'if bytes.len() > MAX_SEARCH_RECORD {',
    after: 'if false {',
    test: 'exact_record_limit_has_an_observed_boundary',
    failure: 'record(&mut rejected'
  },
  {
    name: 'scope-containment',
    file: 'src/search_worker/request.rs',
    before: 'workspace.resolve(selected)?',
    after: 'fs::canonicalize(workspace.root().join(selected))?',
    integration: 'search_scope',
    test: 'invalid_scope_is_an_error_and_a_later_valid_scope_recovers',
    failure: 'result.results.paths.is_err()'
  },
  {
    name: 'eof-cancellation',
    file: 'src/search_process.rs',
    alter: text => {
      const start = text.indexOf('        let waited = tokio::select! {');
      const end = text.indexOf(
        '        if let Some(status) = waited {',
        start
      );
      if ((start === (-1)) || (end < start)) throw new Error('EOF wait cancellation boundary is absent');
      return `${text.slice(
        0,
        start
      )  }        let waited = Some(child.wait().await);\n${
       text.slice(end)}`;
    },
    test: 'cancellation_after_stdout_eof_still_reaps_the_running_child',
    failure: 'EOF must retain cancellation during process wait'
  },
  // Whole-model replacement recreates Slint's repeated rows; this is a lifecycle control, not a custom guard.
  {
    name: 'model-replacement-click',
    native: true,
    test: 'replacement_query_cancels_a_held_result_click',
    control: true
  },
  {
    name: 'pending-open-focus',
    file: 'src/native/navigation/open.rs',
    before: 'if !window.get_search_open() {',
    after: 'if true {',
    occurrence: 1,
    native: true,
    test: 'pending_file_open_does_not_steal_search_input_focus',
    failure: 'asynchronous source install stole query focus'
  },
  // The search box's clear control reports the emptied query, so the results are removed with it.
  {
    name: 'clear-reports-edit',
    file: 'ui/query-input.slint',
    before: '        root.edited("");\n',
    after: '',
    native: true,
    test: 'search_clear_cell_is_48px_clears_the_query_and_results_and_keeps_focus',
    failure: 'clearing the query left search results'
  },
  // Through element handles: the clear control's default action clears, and result rows report selection.
  {
    name: 'a11y-clear-default-action',
    file: 'ui/query-input.slint',
    before: 'accessible-action-default => { root.clear(); }',
    after: '',
    native: true,
    test: 'search_box_and_clear_control_expose_role_label_value_and_actions',
    failure: "the clear control's default action did not clear the search box"
  },
  {
    name: 'a11y-result-selected',
    file: 'ui/search.slint',
    before: 'accessible-item-selected: selected;',
    after: 'accessible-item-selected: false;',
    native: true,
    test: 'search_results_report_role_name_position_and_the_selected_result',
    failure: 'does not report selected = true'
  },
  // Answers of 2026-10-06. Selected box text takes the ink chosen from the fill, which the panel passes on.
  {
    name: 'box-selected-ink',
    file: 'ui/query-input.slint',
    before: 'selection-foreground-color: root.enabled ? root.selected-ink',
    after: 'selection-foreground-color: root.enabled ? Palette.accent-foreground',
    native: true,
    test: 'search_box_selection_uses_the_ink_chosen_from_the_fill_in_both_schemes',
    failure: 'dark: selected text is not drawn in light ink'
  },
  {
    name: 'search-box-ink-passed',
    file: 'ui/search.slint',
    before: '                selected-ink: root.selected-ink;\n                accessible-description: root.result-detail;\n',
    after: '                accessible-description: root.result-detail;\n',
    native: true,
    test: 'search_box_selection_uses_the_ink_chosen_from_the_fill_in_both_schemes',
    failure: 'dark: selected text is not drawn in light ink'
  },
  // The clear plate covers the whole 48px cell, and its boundary is translucent, so the box's border shows through.
  {
    name: 'plate-whole-cell',
    file: 'ui/query-input.slint',
    before: '                    width: 100%;\n                    height: 100%;\n                    border-radius: 4px;\n',
    after: '                    width: 32px;\n                    height: 32px;\n                    border-radius: 4px;\n',
    native: true,
    test: 'search_clear_control_marks_hover_and_press_by_fill_and_boundary',
    failure: 'hover did not fill the whole cell with a plate and a one-pixel boundary'
  },
  {
    name: 'plate-translucent',
    file: 'ui/query-input.slint',
    before: 'border-color: Palette.foreground.with-alpha(clear-touch.pressed ? 0.8 : 0.5);',
    after: 'border-color: Palette.foreground;',
    native: true,
    test: 'search_clear_control_marks_hover_and_press_by_fill_and_boundary',
    failure: "under hover the box's border does not show through the plate's boundary"
  },
  // Result rows carry their names once, and the query box describes the result count.
  {
    name: 'a11y-result-text-hidden',
    file: 'ui/search.slint',
    before: '                        // The row carries its path and detail as name and description; its texts are not read again.\n                        accessible-role: none;\n',
    after: '',
    native: true,
    test: 'search_results_report_role_name_position_and_the_selected_result',
    failure: 'is named by 2 elements in its list'
  },
  {
    name: 'a11y-search-count',
    file: 'ui/search.slint',
    before: '                accessible-description: root.result-detail;\n',
    after: '',
    native: true,
    test: 'search_box_description_counts_the_results',
    failure: "after setting the query the search box's description is not the expected count text"
  },
  {
    name: 'same-file-focus',
    file: 'src/native/navigation/open.rs',
    before: 'if !window.get_search_open() {',
    after: 'if true {',
    native: true,
    test: 'pending_file_open_does_not_steal_search_input_focus',
    failure: 'same-file request stole query focus'
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
      maxBuffer: 8 * 1_024
        * 1_024
    }
  );
  const output = (result.stdout ?? '') + (result.stderr ?? '');
  writeFileSync(
    join(
      artifact,
      `${item.name  }-${  phase  }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  const failedAsExpected = (result.status !== 0) && output.includes(item.test)
    && output.includes(item.failure ?? '');
  const passedAsExpected = (result.status === 0) && output.includes(item.test)
    && /1 (?:test|passed)/.test(output);
  const accepted = phase === 'removed' ? failedAsExpected : passedAsExpected;
  results.push({
    name: item.name,
    phase,
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
  run({
    item,
    phase: 'baseline'
  });
  if (item.control) continue;
  const path = join(
    source,
    item.file
  );
  const original = readFileSync(
    path,
    'utf8'
  );
  try {
    const changed = item.alter ? item.alter(original) : replaceOne({
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
console.log(`Search guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
