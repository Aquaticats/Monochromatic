#!/usr/bin/env node
// Observe the committed live color-scheme and accent regressions failing after guard removal in a disposable package copy.
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
  'ide-theme-guard-'
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
console.log(`THEME_GUARD_ARTIFACT=${  artifact}`);

const scheme = 'live_color_scheme_repaints_source_overlays_tree_find_bar_and_divider';
const accent = 'accent_color_change_keeps_the_source_image_current';
const rows = 'selected_rows_use_the_ink_chosen_from_the_fill_with_measured_contrast';
const stale = 'the live source image differs from a cold render';

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
 What a run needs of a guard; the baseline and restored runs of a test have no edit and no expected failure.
 */
interface RunItem {
  name: string;
  test: string;
  failure?: string;
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
  // The markup notices a scheme flip; without it nothing asks native code to repaint the source image.
  {
    name: 'scheme-change-handler',
    file: 'ui/app.slint',
    before: '    changed dark-scheme => { root.theme-changed(); }\n',
    after: '',
    test: scheme,
    failure: stale
  },
  // Native code repaints when told; a handler that does nothing leaves the old pixels in place.
  {
    name: 'theme-repaint',
    file: 'src/native/render.rs',
    before: '            render(&active, &theme_state);\n',
    after: '',
    test: scheme,
    failure: stale
  },
  // Syntax ink variants follow the scheme only through this raster input.
  {
    name: 'syntax-scheme-input',
    file: 'src/native/render.rs',
    before: 'dark: window.get_dark_scheme(),',
    after: 'dark: true,',
    test: scheme,
    failure: 'light: keyword ink for this scheme is missing'
  },
  // Selected-text ink is chosen from the drawn fill; the palette's own ink is black in the dark scheme.
  {
    name: 'selected-ink-from-fill',
    file: 'src/native/render.rs',
    before: 'selected: legible_ink(\n            rgba(window.get_selection_fill().color()),\n            rgba(window.get_selected_foreground().color()),\n        ),',
    after: 'selected: rgba(window.get_selected_foreground().color()),',
    test: scheme,
    failure: 'dark: selected glyphs are not painted with the ink chosen from the selection fill'
  },
  // An ink that depends on the accent-tinted fill itself goes stale, because only a scheme flip repaints.
  // Selected rows take the same chosen ink; the palette's own, their fallback, is black in the dark scheme.
  {
    name: 'row-ink-from-fill',
    file: 'src/native/render.rs',
    before: '    window.set_selected_row_ink(brush(colors.selected));\n',
    after: '',
    test: rows,
    failure: 'selected row text is not drawn in the ink chosen from the selection fill'
  },
  // The hover and focus tint of a selected row moves its fill away from the ink, never toward it.
  {
    name: 'row-tint-away-from-ink',
    file: 'src/native/render.rs',
    before: 'let tint = if luminance(colors.selected) > 0.5 {',
    after: 'let tint = if luminance(colors.selected) < 0.5 {',
    test: rows,
    failure: 'tint lowered the contrast of the selected row'
  },
  {
    name: 'accent-dependent-ink',
    file: 'src/native/render.rs',
    before: 'selected: legible_ink(\n            rgba(window.get_selection_fill().color()),\n            rgba(window.get_selected_foreground().color()),\n        ),',
    after: 'selected: rgba(window.get_selection_fill().color()),',
    test: accent,
    failure: stale
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
  item: RunItem;
  phase: string;
}): void {
  const {test} = item;
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
      `test(${  test  })`,
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
      `${item.name  }-${  phase  }.log`
    ),
    output
  );
  if (result.error) throw result.error;
  const failedAsExpected = (result.status !== 0) && output.includes(test)
    && output.includes(item.failure ?? '');
  const passedAsExpected = (result.status === 0) && output.includes(test)
    && /1 (?:test|passed)/.test(output);
  const accepted = phase === 'removed' ? failedAsExpected : passedAsExpected;
  results.push({
    name: item.name,
    phase,
    test,
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
// One unmodified run per test is the baseline for every guard of that test.
for (const test of new Set(selected.map(item => item.test))) run({
  item: {
    name: `unmodified-${  test}`,
    test
  },
  phase: 'baseline'
});
for (const item of selected) {
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
}
// One restored run per test after the last guard shows the copy builds and passes again with every guard back.
for (const test of new Set(selected.map(item => item.test))) run({
  item: {
    name: `restored-${  test}`,
    test
  },
  phase: 'restored'
});
console.log(`Theme guard controls passed: ${  artifact  } (${  selected.length  } of ${  cases.length  } guards)`);
