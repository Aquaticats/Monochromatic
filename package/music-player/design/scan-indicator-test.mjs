import { spawnSync } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, 'node_modules', '.monochromatic', 'music-player-design');
makeScratchBase(scratchBase, { recursive: true });


//region All consumer mutations occur in disposable copies of inspected evidence
const root = process.cwd();
const fixture = mkdtempSync(join(
  scratchBase,
  'scan-indicator-review-'
));
const evidence = join(
  fixture,
  'questions',
  'evidence'
);
mkdirSync(
  evidence,
  { recursive: true }
);
const manifestName = 'scan-indicator-witnesses.json';
const manifestPath = join(
  evidence,
  manifestName
);
const manifest = JSON.parse(readFileSync(
  join(
    root,
    'questions',
    'evidence',
    manifestName
  ),
  'utf8'
));
for (const file of [
  manifestName,
  ...manifest.witnesses
    .map(item => item.file)
]) {
  copyFileSync(
    join(
      root,
      'questions',
      'evidence',
      file
    ),
    join(
      evidence,
      file
    )
  );
}
const templatePath = join(
  fixture,
  'questions',
  'scan-indicator.template.html'
);
copyFileSync(
  join(
    root,
    'questions',
    'scan-indicator.template.html'
  ),
  templatePath
);
const template = readFileSync(
  templatePath,
  'utf8'
);
const builder = join(
  fixture,
  'scan-indicator.mjs'
);
const output = join(
  fixture,
  'questions',
  'scan-indicator.html'
);
const full = 'Analysing true peak · 412 of 1,218';
const limits = [
  '-limit',
  'thread',
  '2',
  '-limit',
  'memory',
  '256MiB'
];
function invoke({
  command,
  script = builder
}) {
  return spawnSync(
    process.execPath,
    [
      script,
      command
    ],
    {
      cwd: fixture,
      encoding: 'utf8',
      maxBuffer: 2 * 1_024
        * 1_024
    }
  );
}
function positive() {
  for (const command of [
    'build',
    'validate'
  ]) {
    const run = invoke({ command });
    if (run.status !== 0) throw new Error(`Positive scan-indicator consumer failed: ${  run.stderr}`);
  }
}
function pick({
  input,
  panel,
  scene,
  scale
}) {
  const found = input.witnesses
    .find(item => (item.panel === panel) && (item.scene === scene)
      && (item.fontScale === scale)
      && (item.scheme === 'light'));
  if (!found) throw new Error('Scan-indicator test witness absent.');
  return found;
}
function magick(args) {
  const run = spawnSync(
    'magick',
    [
      ...limits,
      ...args
    ],
    { encoding: 'utf8' }
  );
  if (run.status !== 0) throw new Error(`Synthetic image fixture failed: ${  run.stderr}`);
}
//endregion

//region One or more rejected inputs per builder rule
// `change` edits one witness of a fresh manifest copy; `image` rewrites that witness's PNG;
// `template` rewrites the page template; `afterBuild` runs between build and validate.
// A case names the single rule that must report it. Cases whose rule runs in `validate` build first.
const idle = {
  panel: 'cover',
  scene: 'idle',
  scale: 1
};
const clipped = {
  panel: 'cover',
  scene: 'running',
  scale: 2
};
const largeResume = {
  panel: 'inner',
  scene: 'paused',
  scale: 2
};
const cases = [
  {
    rule: 'manifest-schema',
    manifest: input => { input.schema = 2; }
  },
  {
    rule: 'manifest-witness-list',
    manifest: input => { input.witnesses = { length: 32 }; }
  },
  {
    rule: 'witness-record',
    manifest: input => { input.witnesses[5] = null; }
  },
  {
    rule: 'witness-path',
    at: idle,
    change: item => { item.file = '../outside.png'; }
  },
  {
    rule: 'witness-panel',
    at: clipped,
    change: item => { item.panel = ['cover']; }
  },
  {
    rule: 'witness-scheme',
    at: idle,
    change: item => { item.scheme = ['light']; }
  },
  {
    rule: 'witness-font-scale',
    at: largeResume,
    change: item => { item.fontScale = '2'; }
  },
  {
    rule: 'witness-scene',
    at: idle,
    change: item => { item.scene = ['idle']; }
  },
  {
    rule: 'witness-filename',
    at: idle,
    change: item => { item.scheme = 'dark'; }
  },
  {
    rule: 'root-rectangle',
    at: idle,
    change: item => { item.applicationRoot[1] += 0.5; }
  },
  {
    rule: 'root-rectangle',
    at: idle,
    change: item => { delete item.applicationRoot; }
  },
  {
    rule: 'root-width',
    at: idle,
    change: item => { item.applicationRoot[2]--; }
  },
  {
    rule: 'root-status-strip',
    at: idle,
    change: item => { item.applicationRoot[1] = 0; }
  },
  {
    rule: 'root-bottom',
    at: idle,
    change: item => { item.applicationRoot[3] = 200; }
  },
  {
    rule: 'root-bottom',
    at: idle,
    change: item => { item.applicationRoot[3] = 9_999; }
  },
  {
    rule: 'physical-pixels',
    at: idle,
    change: item => { item.physicalPixels = [
      2_076,
      2_152
    ]; }
  },
  {
    rule: 'crop-record',
    at: idle,
    change: item => { delete item.cropPixels; }
  },
  {
    rule: 'crop-origin',
    at: idle,
    change: item => { item.cropPixels
      .y++; }
  },
  {
    rule: 'crop-size',
    at: idle,
    change: item => { item.cropPixels
      .height--; }
  },
  {
    rule: 'png-signature',
    at: idle,
    image: bytes => Buffer.concat([
      Buffer.from([0x88]),
      bytes.subarray(1)
    ])
  },
  {
    rule: 'png-header-chunk',
    at: idle,
    image: bytes => bytes.subarray(
      0,
      20
    )
  },
  {
    rule: 'png-size',
    at: idle,
    image: (
      bytes,
      path
    ) => { magick([
      path,
      '-crop',
      '1079x2272+0+0',
      '+repage',
      '-strip',
      '-define',
      'png:exclude-chunks=all',
      `PNG24:${  path}`
    ]);
      return readFileSync(path); }
  },
  {
    rule: 'png-opaque-rgb',
    at: idle,
    image: (
      bytes,
      path
    ) => { magick([
      path,
      '-alpha',
      'on',
      '-strip',
      '-define',
      'png:exclude-chunks=all',
      `PNG32:${  path}`
    ]);
      return readFileSync(path); }
  },
  {
    rule: 'png-chunks',
    at: idle,
    image: bytes => Buffer.concat([
      bytes,
      Buffer.from(
        '0000ffff49444154',
        'hex'
      ),
      Buffer.from('account path')
    ])
  },
  {
    rule: 'png-chunks',
    at: idle,
    image: bytes => Buffer.concat([
      bytes.subarray(
        0,
        -12
      ),
      Buffer.from(
        '0000000474455874',
        'hex'
      ),
      Buffer.from('note'),
      Buffer.alloc(4),
      bytes.subarray(-12)
    ])
  },
  {
    rule: 'png-chunks',
    at: idle,
    image: bytes => bytes.subarray(
      0,
      -12
    )
  },
  {
    rule: 'density',
    at: idle,
    change: item => { item.densityDpi = 420; }
  },
  {
    rule: 'inspected',
    at: idle,
    change: item => { item.inspected = false; }
  },
  {
    rule: 'fresh-hierarchy',
    at: idle,
    change: item => { item.freshHierarchyValidated = false; }
  },
  {
    rule: 'keyboard-closed',
    at: idle,
    change: item => { item.keyboardClosed = false; }
  },
  {
    rule: 'stable-frames',
    at: idle,
    change: item => { item.stableAppFrames = false; }
  },
  {
    rule: 'retained-rgb',
    at: idle,
    change: item => { item.exactRetainedRgb = false; }
  },
  {
    rule: 'sanitized-opaque',
    at: idle,
    change: item => { item.opaque = false; }
  },
  {
    rule: 'sanitized-chunks',
    at: idle,
    change: item => { item.essentialPngChunksOnly = false; }
  },
  {
    rule: 'source-padding',
    at: clipped,
    change: item => { item.controlPaddingDp = 12; }
  },
  // Idle views carry no bar text, so only this rule can refuse a changed idle count.
  {
    rule: 'authored-state',
    at: idle,
    change: item => { item.state
      .done = 1; }
  },
  {
    rule: 'authored-state',
    at: largeResume,
    change: item => { item.state
      .phase = 'running'; }
  },
  {
    rule: 'system-image',
    at: idle,
    change: item => { item.systemImageFingerprint = 'different-system'; }
  },
  {
    rule: 'renderer',
    at: idle,
    change: item => { item.renderer = 'different-renderer'; }
  },
  {
    rule: 'player-rectangle',
    at: idle,
    change: item => { item.player = 'garbage'; }
  },
  {
    rule: 'player-origin',
    at: idle,
    change: item => { item.player[1]++; }
  },
  {
    rule: 'viewport-rectangle',
    at: idle,
    change: item => { item.viewport = 'garbage'; }
  },
  {
    rule: 'viewport-inside-player',
    at: idle,
    change: item => { item.viewport[3] = item.player[3] + 1; }
  },
  {
    rule: 'idle-bar',
    at: idle,
    change: item => { item.bar = [
      0,
      item.applicationRoot[3] - 137,
      item.applicationRoot[2],
      item.applicationRoot[3]
    ]; }
  },
  {
    rule: 'idle-control',
    at: idle,
    change: item => { item.control = [
      0,
      0,
      244,
      117
    ]; }
  },
  {
    rule: 'idle-bar-absent-flag',
    at: idle,
    change: item => { item.barAbsent = false; }
  },
  {
    rule: 'idle-status-lines',
    at: idle,
    change: item => { item.statusLines = 1; }
  },
  {
    rule: 'idle-status-ellipsis',
    at: idle,
    change: item => { item.statusEllipsized = true; }
  },
  {
    rule: 'idle-control-label',
    at: idle,
    change: item => { item.controlLabel = 'Pause'; }
  },
  {
    rule: 'idle-control-text-lines',
    at: idle,
    change: item => { item.controlTextLines = 3; }
  },
  {
    rule: 'idle-control-text-overflow',
    at: idle,
    change: item => { item.controlTextLayoutOverflow = false; }
  },
  {
    rule: 'idle-status-reading',
    at: idle,
    change: item => { item.inspectedVisibleStatus = full; }
  },
  {
    rule: 'idle-label-clearance',
    at: idle,
    change: item => { item.controlLabelClearPixels = {
      left: 1,
      right: 1
    }; }
  },
  // A shorter idle player with its viewport kept inside it is a reserved blank slot.
  {
    rule: 'idle-player-bottom',
    at: idle,
    change: item => { item.player[3] -= 137;
      item.viewport[3] -= 137; }
  },
  {
    rule: 'bar-rectangle',
    at: clipped,
    change: item => { item.bar = 'garbage'; }
  },
  {
    rule: 'bar-position',
    at: clipped,
    change: item => { item.bar[0] = 1; }
  },
  // Player and viewport move with the bar top, so only the height rule can refuse this taller bar.
  {
    rule: 'bar-height',
    at: clipped,
    change: item => { item.bar[1]--;
      item.player[3]--;
      item.viewport[3]--; }
  },
  {
    rule: 'player-above-bar',
    at: clipped,
    change: item => { item.player[3]--;
      item.viewport[3]--; }
  },
  {
    rule: 'control-rectangle',
    at: clipped,
    change: item => { item.control = null; }
  },
  {
    rule: 'control-width',
    at: clipped,
    change: item => { item.control[2]++; }
  },
  {
    rule: 'control-floor',
    at: clipped,
    change: item => { item.control[1] = item.control[3] - 116; }
  },
  {
    rule: 'control-inside-bar',
    at: clipped,
    change: item => { item.control[0] += 100;
      item.control[2] += 100; }
  },
  {
    rule: 'bar-absent-flag',
    at: clipped,
    change: item => { item.barAbsent = true; }
  },
  {
    rule: 'status-lines',
    at: clipped,
    change: item => { item.statusLines = 2; }
  },
  {
    rule: 'control-text-lines',
    at: clipped,
    change: item => { item.controlTextLines = 2; }
  },
  {
    rule: 'control-text-overflow',
    at: clipped,
    change: item => { item.controlTextLayoutOverflow = true; }
  },
  {
    rule: 'control-label',
    at: clipped,
    change: item => { item.controlLabel = 'Resume'; }
  },
  // Flag and reading agree with each other here, so only the panel-and-scale rule can refuse them.
  {
    rule: 'ellipsis-placement',
    at: largeResume,
    change: item => { item.statusEllipsized = true;
      item.inspectedVisibleStatus = 'Analysing true peak · 412…'; }
  },
  {
    rule: 'ellipsis-placement',
    at: clipped,
    change: item => { item.statusEllipsized = false;
      item.inspectedVisibleStatus = full; }
  },
  {
    rule: 'status-reading-text',
    at: largeResume,
    change: item => { item.inspectedVisibleStatus = 42; }
  },
  {
    rule: 'ellipsis-binding',
    at: clipped,
    change: item => { item.inspectedVisibleStatus = full; }
  },
  {
    rule: 'ellipsis-binding',
    at: largeResume,
    change: item => { item.inspectedVisibleStatus = 'Analysing true peak · 412…'; }
  },
  {
    rule: 'status-reading-authored',
    at: clipped,
    change: item => { item.inspectedVisibleStatus = `${full  }…`; }
  },
  {
    rule: 'status-reading-authored',
    at: clipped,
    change: item => { item.inspectedVisibleStatus = 'Analysing true peak · 999…'; }
  },
  {
    rule: 'status-reading-authored',
    at: clipped,
    change: item => { item.inspectedVisibleStatus = '…'; }
  },
  {
    rule: 'status-reading-authored',
    at: clipped,
    change: item => { item.inspectedVisibleStatus = '</p><script>alert(1)</script> & "quoted" \n…'; }
  },
  {
    rule: 'status-reading-authored',
    at: {
      panel: 'inner',
      scene: 'wide-count',
      scale: 1
    },
    change: item => { item.inspectedVisibleStatus = full; }
  },
  {
    rule: 'label-clearance-record',
    at: clipped,
    change: item => { delete item.controlLabelClearPixels; }
  },
  {
    rule: 'label-clearance-record',
    at: clipped,
    change: item => { item.controlLabelClearPixels = null; }
  },
  {
    rule: 'label-clearance-leading',
    at: clipped,
    change: item => { item.controlLabelClearPixels
      .left = -1; }
  },
  {
    rule: 'label-clearance-leading',
    at: clipped,
    change: item => { item.controlLabelClearPixels
      .left = 1.5; }
  },
  {
    rule: 'label-clearance-trailing',
    at: clipped,
    change: item => { item.controlLabelClearPixels
      .right = -1; }
  },
  {
    rule: 'witness-unique',
    manifest: input => { input.witnesses
      .push(structuredClone(input.witnesses[0])); }
  },
  {
    rule: 'exact-cohort',
    manifest: input => { input.witnesses
      .splice(
      input.witnesses
        .indexOf(pick({
          input,
          ...idle
        })),
      1
      ); }
  },
  {
    rule: 'template-slot',
    template: text => text.replace(
      '__SCAN_INDICATOR_IMAGES__',
      '{}'
    )
  },
  {
    rule: 'template-slot',
    template: text => text.replace(
      '__SCAN_INDICATOR_FINDINGS__',
      ''
    )
  },
  {
    rule: 'template-slot',
    template: text => text.replace(
      '__SCAN_INDICATOR_PROVENANCE__',
      ''
    )
  },
  {
    rule: 'output-current',
    phase: 'validate',
    afterBuild: () => { writeFileSync(
      output,
      readFileSync(
        output,
        'utf8'
      )
        .replace(
        'Design evidence only.',
        'Changed output.'
        )
    ); }
  },
  {
    rule: 'required-statement',
    phase: 'validate',
    template: text => text.replace(
      'Reset 100% dp',
      'Reset zoom'
    )
  },
  {
    rule: 'single-form',
    phase: 'validate',
    template: text => text.replace(
      '<dialog id="preview"',
      '<form></form><dialog id="preview"'
    )
  },
  {
    rule: 'single-inline-script',
    phase: 'validate',
    template: text => text.replace(
      '<dialog id="preview"',
      '<script src=extra.js></script><dialog id="preview"'
    )
  },
  {
    rule: 'no-unresolved-slot',
    phase: 'validate',
    template: text => text.replace(
      '<dialog id="preview"',
      '__SCAN_INDICATOR_EXTRA__<dialog id="preview"'
    )
  },
  {
    rule: 'no-external-reference',
    phase: 'validate',
    template: text => text.replace(
      '</style>',
      '</style><style>@import url(https://example.org/x.css);</style>'
    )
  },
  {
    rule: 'no-external-reference',
    phase: 'validate',
    template: text => text.replace(
      '<dialog id="preview"',
      '<img src="https://example.org/a.png" alt=""><dialog id="preview"'
    )
  },
  {
    rule: 'no-external-reference',
    phase: 'validate',
    template: text => text.replace(
      '</style>',
      '</style><link rel=stylesheet href=extra.css>'
    )
  },
  {
    rule: 'no-input-control',
    phase: 'validate',
    template: text => text.replace(
      '<form id="review-form">',
      '<form id="review-form"><input type=radio name=policy>'
    )
  },
  {
    rule: 'no-required-field',
    phase: 'validate',
    template: text => text.replace(
      '<textarea id="final-notes" name="notes"',
      '<textarea id="final-notes" name="notes" required'
    )
  },
  {
    rule: 'no-required-field',
    phase: 'validate',
    template: text => text.replace(
      '<form id="review-form">',
      '<form id="review-form"><select name="policy" required><option>A</option></select>'
    )
  },
  {
    rule: 'observation-fields-only',
    phase: 'validate',
    template: text => text.replace(
      '<form id="review-form">',
      '<form id="review-form"><select name="policy"><option>A</option></select>'
    )
  },
  {
    rule: 'observation-fields-only',
    phase: 'validate',
    template: text => text.replace(
      '<form id="review-form">',
      '<form id="review-form"><textarea name="policy"></textarea>'
    )
  },
  // A plausible whole number that the embedded pixels do not show.
  {
    rule: 'label-clearance-pixels',
    phase: 'validate',
    at: largeResume,
    change: item => { item.controlLabelClearPixels
      .left = 100_000; }
  },
  {
    rule: 'label-clearance-pixels',
    phase: 'validate',
    at: largeResume,
    change: item => { item.controlLabelClearPixels
      .right = 0; }
  },
];
function runCase(testCase) {
  const input = structuredClone(manifest);
  const undo = [];
  if (testCase.manifest) testCase.manifest(input);
  const item = testCase.at ? pick({
    input,
    ...testCase.at
  }) : undefined;
  if (testCase.image) {
    const path = join(
      evidence,
      item.file
    );
    const original = readFileSync(path);
    undo.push(() =>{  writeFileSync(
      path,
      original
    ); });
    const bytes = testCase.image(
      original,
      path
    );
    writeFileSync(
      path,
      bytes
    );
  }
  if (testCase.change) testCase.change(item);
  if (testCase.template) {
    const changed = testCase.template(template);
    if (changed === template) throw new Error(`Template case changed nothing: ${  testCase.rule}`);
    writeFileSync(
      templatePath,
      changed
    );
    undo.push(() =>{  writeFileSync(
      templatePath,
      template
    ); });
  }
  writeFileSync(
    manifestPath,
    JSON.stringify(input)
  );
  let run = invoke({ command: 'build' });
  if (testCase.phase === 'validate') {
    if (run.status !== 0) throw new Error(`Case for ${testCase.rule} must build before validation: ${run.stderr}`);
    if (testCase.afterBuild) testCase.afterBuild();
    run = invoke({ command: 'validate' });
  }
  for (const restore of undo.reverse()) restore();
  writeFileSync(
    manifestPath,
    JSON.stringify(manifest)
  );
  if ((run.status === 0) || (!run.stderr
    .includes(`[${testCase.rule}]`))) {
    throw new Error(`Expected scan-indicator rejection absent: ${testCase.rule}; got ${run.status === 0 ? 'acceptance' : run.stderr
      .trim()
      .split('\n')
      .find(line => line.includes('Error'))
      ?? 'another failure'}`);
  }
}
//endregion

//region Escaping is checked behind the rule that normally makes it unreachable
// The status rules confine manifest text to authored characters, so HTML escaping can never be seen
// through them. A second builder copy without that one rule shows the escape still holds on its own.
const relaxed = join(
  fixture,
  'scan-indicator-relaxed.mjs'
);
function ruleLine({
  source,
  rule
}) {
  const lines = source.split('\n')
    .filter(line => line.includes(`need({ rule: '${rule}',`));
  if (lines.length !== 1) throw new Error(`Exact scan-indicator rule line absent: ${  rule}`);
  return `${lines[0]  }\n`;
}
function escapingHolds() {
  const input = structuredClone(manifest);
  pick({
    input,
    ...clipped
  })
    .inspectedVisibleStatus = '</p><script>alert(1)</script> & "quoted"…';
  writeFileSync(
    manifestPath,
    JSON.stringify(input)
  );
  const run = invoke({
    command: 'build',
    script: relaxed
  });
  const built = run.status === 0 ? readFileSync(
    output,
    'utf8'
  ) : '';
  writeFileSync(
    manifestPath,
    JSON.stringify(manifest)
  );
  if (run.status !== 0) throw new Error(`Relaxed scan-indicator builder failed: ${  run.stderr}`);
  if ((!built.includes('&lt;/p&gt;&lt;script&gt;alert(1)&lt;/script&gt; &amp; &quot;quoted&quot;…')) || ((built.match(/<script\b/gi) ?? []).length !== 1)) {
    throw new Error('Expected scan-indicator rejection absent: html-escape; got unescaped manifest text in the page');
  }
}
//endregion

//region Normal run, rule listing, or one deleted rule
try {
  const source = readFileSync(
    join(
      root,
      'scan-indicator.mjs'
    ),
    'utf8'
  );
  const rules = [...new Set([...source.matchAll(/need\(\{ rule: '([a-z-]+)',/g)].map(match => match[1]))];
  const covered = new Set(cases.map(testCase => testCase.rule));
  const uncovered = rules.filter(rule => !covered.has(rule));
  const unknown = [...covered].filter(rule => !rules.includes(rule));
  if ((uncovered.length > 0) || (unknown.length > 0)) throw new Error(`Rules and cases differ: uncovered ${uncovered.join(', ') || 'none'}; unknown ${unknown.join(', ') || 'none'}`);
  const [mode, target] = process.argv
    .slice(2);
  if (mode === 'list') {
    console.log(JSON.stringify([
      ...rules,
      'html-escape'
    ]));
  } else {
    if ((mode !== undefined) && ((mode !== 'without') || (![
      ...rules,
      'html-escape'
    ].includes(target)))) throw new Error('Unknown scan-indicator consumer mutation.');
    const escape = `return text.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;').replaceAll('"', '&quot;');`;
    if (source.split(escape)
      .length
      !== 2) throw new Error('Exact scan-indicator escape body absent.');
    let subject = source;
    if (target === 'html-escape') subject = source.replace(
      escape,
      'return text;'
    );
    else if (target !== undefined) subject = source.replace(
      ruleLine({
        source,
        rule: target
      }),
      ''
    );
    writeFileSync(
      builder,
      subject
    );
    writeFileSync(
      relaxed,
      subject.replace(
        ruleLine({
          source,
          rule: 'status-reading-authored'
        }),
        ''
      )
    );
    positive();
    if (target === undefined) {
      const built = readFileSync(
        output,
        'utf8'
      );
      for (const statement of [
        'is ellipsized in 6 active views',
        'which show only “Analysing true peak · 412…” or “Analysing true peak · 9,9…”',
        'ink stays 1 to 2 physical pixels clear of the control outline',
        'and 5 on the trailing side, on both panels',
        'the native layout reports no overflow there',
        'Pause at 200% keeps at least 29 pixels clear',
        'both labels at 100% keep at least 57.',
        'prototype commit 25a95411750c5c5356631fc508513c42a81ad1c4',
        'APK SHA-256 c06ec80240e41641fee1fdec79313a8294b9595544740145f4fa6f2b1f77ddf1',
        'renderer Android Emulator OpenGL ES Translator (llvmpipe (LLVM 20.1.2, 256 bits))'
      ]) {
        if (!built.includes(statement)) throw new Error(`Generated statement absent: ${  statement}`);
      }
      // A changed measured inset is followed, not compared with a fixed crop.
      const changedInset = structuredClone(manifest);
      const view = changedInset.witnesses[0];
      const imagePath = join(
        evidence,
        view.file
      );
      const original = readFileSync(imagePath);
      view.applicationRoot[1]++;
      view.player[1]++;
      view.cropPixels
        .y++;
      view.cropPixels
        .height--;
      magick([
        imagePath,
        '-crop',
        `${view.cropPixels
          .width}x${view.cropPixels
            .height}+0+1`,
        '+repage',
        '-strip',
        '-define',
        'png:exclude-chunks=all',
        `PNG24:${  imagePath}`
      ]);
      writeFileSync(
        manifestPath,
        JSON.stringify(changedInset)
      );
      positive();
      writeFileSync(
        imagePath,
        original
      );
      writeFileSync(
        manifestPath,
        JSON.stringify(manifest)
      );
    }
    // With one rule deleted only that rule's cases run; each must then stop reporting the rule.
    for (const testCase of cases) if ((target === undefined) || (testCase.rule === target)) runCase(testCase);
    if ((target === undefined) || (target === 'html-escape')
      || (target === 'status-reading-authored')) escapingHolds();
    positive();
    console.log(`Scan-indicator consumer: ${rules.length} rules, ${cases.length} rejected inputs, escaping and positive controls passed.`);
  }
} finally {
  rmSync(
    fixture,
    {
      recursive: true,
      force: true
    }
  );
}
//endregion
