#!/usr/bin/env node
// Observe committed sidebar-resize regressions failing after guard removal in a disposable package copy.
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
  'ide-sidebar-guard-'
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
console.log(`SIDEBAR_GUARD_ARTIFACT=${  artifact}`);

const drag = 'divider_drag_resizes_and_stops_at_both_bounds';
const resize = 'window_resize_shrinks_the_sidebar_and_restores_the_request';
const zone = 'divider_zone_takes_drags_on_five_columns_only';
const states = 'divider_states_change_line_weight_and_ink';
const stored = 'root.sidebar-requested-width = clamp(width, root.sidebar-minimum-width, root.sidebar-maximum-width);';

/**
 A guard: the edit that removes it and the test that must then fail.
 */
interface Guard {
  name: string;
  file: string;
  before: string;
  after: string;
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

const cases: Guard[] = [
  // The stored request: drag and keys cannot keep a width that cannot be shown now.
  {
    name: 'drag-upper-bound',
    file: 'ui/app.slint',
    before: stored,
    after: 'root.sidebar-requested-width = max(width, root.sidebar-minimum-width);',
    test: drag,
    failure: 'a drag stored a width above the maximum'
  },
  {
    name: 'drag-lower-bound',
    file: 'ui/app.slint',
    before: stored,
    after: 'root.sidebar-requested-width = min(width, root.sidebar-maximum-width);',
    test: drag,
    failure: 'a drag stored a width below the minimum'
  },
  // Any held button reports movement; only the left button records a drag origin.
  {
    name: 'left-button-only',
    file: 'ui/divider.slint',
    before: '            if !self.pressed { return; }\n',
    after: '',
    test: 'divider_ignores_plain_clicks_and_other_buttons_and_keeps_keyboard_focus',
    failure: 'a right-button drag resized the sidebar'
  },
  // The layout clamp: the source cell keeps its minimum and is never the cell that shrinks first.
  {
    name: 'source-minimum',
    file: 'ui/app.slint',
    before: '            min-width: root.source-minimum-width;\n',
    after: '',
    test: resize,
    failure: 'the sidebar did not shrink to keep the source minimum'
  },
  // The file label and a diagnostic can both prefer more width than the window; the label is checked first.
  {
    name: 'source-preferred',
    file: 'ui/app.slint',
    before: '            preferred-width: root.source-minimum-width;\n',
    after: '',
    test: resize,
    failure: 'narrowed the sidebar'
  },
  {
    name: 'sidebar-minimum',
    file: 'ui/app.slint',
    before: 'min-width: root.project-visible ? root.sidebar-minimum-width : 0px;',
    after: 'min-width: 0px;',
    test: resize,
    failure: 'the sidebar shrank below its minimum'
  },
  {
    name: 'maximum-floor',
    file: 'ui/app.slint',
    before: 'max(root.sidebar-minimum-width,\n        root.width - root.divider-width - root.source-minimum-width)',
    after: '(root.width - root.divider-width - root.source-minimum-width)',
    test: resize,
    failure: 'the reported maximum fell below the minimum'
  },
  // The pointer zone is the line's column and two columns on each side: no wider and no narrower.
  {
    name: 'zone-not-wider',
    file: 'ui/divider.slint',
    before: 'out property <length> reach: 2px;',
    after: 'out property <length> reach: 3px;',
    test: zone,
    failure: 'outside the zone, resized the sidebar'
  },
  {
    name: 'zone-not-narrower',
    file: 'ui/divider.slint',
    before: 'out property <length> reach: 2px;',
    after: 'out property <length> reach: 1px;',
    test: zone,
    failure: 'did not start a drag'
  },
  // Keyboard focus on the 1px line is marked by a color of its own and by a handle.
  {
    name: 'focus-color',
    file: 'ui/divider.slint',
    before: 'background: root.keyboard-focus ? Palette.accent-background : Palette.foreground;',
    after: 'background: Palette.foreground;',
    test: states,
    failure: 'keyboard focus does not have a line color of its own'
  },
  {
    name: 'focus-handle',
    file: 'ui/divider.slint',
    before: '        visible: root.keyboard-focus;\n',
    after: '        visible: false;\n',
    test: states,
    failure: 'keyboard focus did not draw its handle'
  },
  // Through element handles: the divider slider's increment action, and tree rows reporting the open file.
  {
    name: 'a11y-divider-increment',
    file: 'ui/divider.slint',
    before: 'accessible-action-increment => { root.resize(root.value + root.step); }',
    after: 'accessible-action-increment => { }',
    test: 'divider_is_a_slider_with_value_bounds_step_actions_and_keys',
    failure: 'the increment action did not set the sidebar width'
  },
  {
    name: 'a11y-tree-row-selected',
    file: 'ui/tree.slint',
    before: 'accessible-item-selected: entry.selected;',
    after: 'accessible-item-selected: false;',
    test: 'tree_rows_report_role_name_position_and_the_open_file_as_selected',
    failure: 'does not report selected = true'
  },
  // Answer of 2026-10-06: the keyboard-focus handle is 96px tall.
  {
    name: 'focus-handle-96',
    file: 'ui/divider.slint',
    before: 'height: min(96px, parent.height);',
    after: 'height: min(48px, parent.height);',
    test: 'divider_states_change_line_weight_and_ink',
    failure: 'so it is shorter than 96px'
  },
  // Tree rows carry their names once; the hidden slot badge's shortcut moves into the row description.
  {
    name: 'a11y-tree-row-text-hidden',
    file: 'ui/tree.slint',
    before: '                    // The row carries the name; its text is not read a second time.\n                    accessible-role: none;\n',
    after: '',
    test: 'tree_rows_report_role_name_position_and_the_open_file_as_selected',
    failure: 'is named by 2 elements in its list'
  },
  {
    name: 'a11y-tree-badge-shortcut',
    file: 'ui/tree.slint',
    before: ': (entry.recency == "" ? "Source file" : "Source file, Ctrl+" + entry.recency);',
    after: ': "Source file";',
    test: 'tree_rows_report_role_name_position_and_the_open_file_as_selected',
    failure: 'does not name its shortcut'
  },
  // Without a project the pointer half takes no input over the source.
  {
    name: 'hidden-grip',
    file: 'ui/app.slint',
    before: '        visible: root.project-visible;\n        x: project-tree.width - self.reach;',
    after: '        x: project-tree.width - self.reach;',
    test: 'hidden_project_has_no_divider',
    failure: 'the first window pixel is not source without a project'
  },
  // Without a project there is no divider to focus.
  {
    name: 'hidden-divider',
    file: 'ui/app.slint',
    before: '            visible: root.project-visible;\n            min-width: root.project-visible ? root.divider-width : 0px;',
    after: '            min-width: root.project-visible ? root.divider-width : 0px;',
    test: 'hidden_project_has_no_divider',
    failure: 'reached the divider of a hidden project'
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
      'cargo',
      'nextest',
      'run',
      '--offline',
      '--bin',
      'monochromatic-ide',
      '--filter-expr',
      `test(${  item.test
         })`,
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
  const path = join(
    source,
    item.file
  );
  const original = readFileSync(
    path,
    'utf8'
  );
  try {
    const start = original.indexOf(item.before);
    if (start === (-1)) throw new Error(`Mutation source anchor is absent: ${  item.before}`);
    if (original.includes(
      item.before,
      start
        + item.before
          .length
    )) throw new Error(`Mutation source anchor is ambiguous: ${  item.before}`);
    writeFileSync(
      path,
      original.slice(
        0,
        start
      ) + item.after
        + original.slice(start
          + item.before
            .length)
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
console.log(`Sidebar guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
