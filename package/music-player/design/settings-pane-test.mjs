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
  'settings-pane-review-'
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
const manifestName = 'settings-pane-witnesses.json';
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
// The published Search images the builder compares each retained left half with.
const searchFiles = [
  'light-s100',
  'dark-s100',
  'light-s200',
  'dark-s200'
].map(name => `search-filename-comparison-inner-literalfull-${name}.png`);
for (const file of [
  manifestName,
  ...searchFiles,
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
  'settings-pane.template.html'
);
copyFileSync(
  join(
    root,
    'questions',
    'settings-pane.template.html'
  ),
  templatePath
);
const template = readFileSync(
  templatePath,
  'utf8'
);
const builder = join(
  fixture,
  'settings-pane.mjs'
);
const output = join(
  fixture,
  'questions',
  'settings-pane.html'
);
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
    if (run.status !== 0) throw new Error(`Positive Settings-pane consumer failed: ${  run.stderr}`);
  }
}
function pick({
  input,
  panel,
  view,
  scale,
  position,
  scheme = 'light'
}) {
  const found = input.witnesses
    .find(item => (item.panel === panel) && (item.view === view)
      && (item.fontScale === scale)
      && (item.position === position)
      && (item.scheme === scheme));
  if (!found) throw new Error('Settings-pane test witness absent.');
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
function evidenceBytes(at) {
  return readFileSync(join(
    evidence,
    pick({
      input: manifest,
      ...at
    })
      .file
  ));
}
//endregion

//region One or more rejected inputs per builder rule
// `change` edits one witness of a fresh manifest copy; `image` rewrites that witness's PNG;
// `rename` copies that PNG under a new name; `template` rewrites the page template;
// `afterBuild` runs between build and validate.
// A case names the single rule that must report it. Cases whose rule runs in `validate` build first.
const closed = {
  panel: 'cover',
  view: 'closed',
  scale: 1,
  position: 'none'
};
const fits = {
  panel: 'inner',
  view: 'accepted',
  scale: 1,
  position: 'start'
};
const other = {
  panel: 'inner',
  view: 'inverse',
  scale: 1,
  position: 'start'
};
const large = {
  panel: 'inner',
  view: 'accepted',
  scale: 2,
  position: 'start'
};
const cover = {
  panel: 'cover',
  view: 'inverse',
  scale: 1,
  position: 'start'
};
// Pastes one block of the same image over another place in it, keeping the sanitized PNG form.
// Offsets are image pixels of the unfolded 100% light crop, whose status strip is 136px.
function paste({
  path,
  from,
  size,
  to
}) {
  magick([
    path,
    '(',
    '+clone',
    '-crop',
    `${size}+${from}`,
    '+repage',
    ')',
    '-geometry',
    `+${  to}`,
    '-composite',
    '-strip',
    '-define',
    'png:exclude-chunks=all',
    `PNG24:${  path}`
  ]);
  return readFileSync(path);
}
const cases = [
  {
    rule: 'manifest-schema',
    manifest: input => { input.schema = 2; }
  },
  {
    rule: 'manifest-witness-list',
    manifest: input => { input.witnesses = { length: 24 }; }
  },
  {
    rule: 'witness-record',
    manifest: input => { input.witnesses[5] = null; }
  },
  {
    rule: 'witness-path',
    at: closed,
    change: item => { item.file = '../outside.png'; }
  },
  {
    rule: 'witness-panel',
    at: closed,
    change: item => { item.panel = ['cover']; }
  },
  {
    rule: 'witness-scheme',
    at: closed,
    change: item => { item.scheme = ['light']; }
  },
  {
    rule: 'witness-font-scale',
    at: large,
    change: item => { item.fontScale = '2'; }
  },
  {
    rule: 'witness-view',
    at: closed,
    change: item => { item.view = ['closed']; }
  },
  {
    rule: 'witness-position',
    at: fits,
    change: item => { item.position = 'middle'; }
  },
  // No column scrolls, so an end-of-column position is not an authored view either.
  {
    rule: 'witness-position',
    at: fits,
    change: item => { item.position = 'end'; }
  },
  {
    rule: 'witness-position-view',
    at: closed,
    change: item => { item.position = 'start'; }
  },
  {
    rule: 'witness-position-view',
    at: fits,
    change: item => { item.position = 'none'; }
  },
  {
    rule: 'witness-filename',
    at: closed,
    change: item => { item.scheme = 'dark'; }
  },
  {
    rule: 'root-rectangle',
    at: closed,
    change: item => { item.applicationRoot[1] += 0.5; }
  },
  {
    rule: 'root-rectangle',
    at: closed,
    change: item => { delete item.applicationRoot; }
  },
  {
    rule: 'root-width',
    at: closed,
    change: item => { item.applicationRoot[2]--; }
  },
  {
    rule: 'root-status-strip',
    at: closed,
    change: item => { item.applicationRoot[1] = 0; }
  },
  {
    rule: 'root-bottom',
    at: closed,
    change: item => { item.applicationRoot[3] = 200; }
  },
  {
    rule: 'root-bottom',
    at: closed,
    change: item => { item.applicationRoot[3] = 9_999; }
  },
  {
    rule: 'physical-pixels',
    at: closed,
    change: item => { item.physicalPixels = [
      2_076,
      2_152
    ]; }
  },
  {
    rule: 'crop-record',
    at: closed,
    change: item => { delete item.cropPixels; }
  },
  {
    rule: 'crop-origin',
    at: closed,
    change: item => { item.cropPixels
      .y++; }
  },
  {
    rule: 'crop-size',
    at: closed,
    change: item => { item.cropPixels
      .height--; }
  },
  {
    rule: 'png-signature',
    at: closed,
    image: bytes => Buffer.concat([
      Buffer.from([0x88]),
      bytes.subarray(1)
    ])
  },
  {
    rule: 'png-header-chunk',
    at: closed,
    image: bytes => bytes.subarray(
      0,
      20
    )
  },
  {
    rule: 'png-size',
    at: closed,
    image: (
      bytes,
      path
    ) => { magick([
      path,
      '-crop',
      '1079x2000+0+0',
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
    at: closed,
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
    at: closed,
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
    at: closed,
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
    at: closed,
    image: bytes => bytes.subarray(
      0,
      -12
    )
  },
  {
    rule: 'density',
    at: closed,
    change: item => { item.densityDpi = 420; }
  },
  {
    rule: 'inspected',
    at: closed,
    change: item => { item.inspected = false; }
  },
  {
    rule: 'fresh-hierarchy',
    at: closed,
    change: item => { item.freshHierarchyValidated = false; }
  },
  {
    rule: 'keyboard-closed',
    at: closed,
    change: item => { item.keyboardClosed = false; }
  },
  {
    rule: 'stable-frames',
    at: closed,
    change: item => { item.stableAppFrames = false; }
  },
  {
    rule: 'retained-rgb',
    at: closed,
    change: item => { item.exactRetainedRgb = false; }
  },
  {
    rule: 'sanitized-opaque',
    at: closed,
    change: item => { item.opaque = false; }
  },
  {
    rule: 'sanitized-chunks',
    at: closed,
    change: item => { item.essentialPngChunksOnly = false; }
  },
  // The closed player draws no switch, so only this rule can refuse a changed closed state.
  {
    rule: 'authored-state',
    at: closed,
    change: item => { item.state
      .strip = false; }
  },
  {
    rule: 'authored-state',
    at: fits,
    change: item => { item.state
      .resume = false; }
  },
  // D84 removed the analysis row, so a reinstated analysis field is not an authored state either.
  {
    rule: 'authored-state',
    at: fits,
    change: item => { item.state
      .analyse = false; }
  },
  {
    rule: 'system-image',
    at: closed,
    change: item => { item.systemImageFingerprint = 'different-system'; }
  },
  {
    rule: 'renderer',
    at: closed,
    change: item => { item.renderer = 'different-renderer'; }
  },
  {
    rule: 'witness-unique',
    manifest: input => { input.witnesses
      .push(structuredClone(input.witnesses[0])); }
  },
  {
    rule: 'closed-player-rectangle',
    at: closed,
    change: item => { item.player = 'garbage'; }
  },
  {
    rule: 'closed-player-root',
    at: closed,
    change: item => { item.player[1]++; }
  },
  {
    rule: 'closed-player-root',
    at: closed,
    change: item => { item.player[3] -= 137; }
  },
  {
    rule: 'closed-no-pane',
    at: closed,
    change: item => { item.pane = [
      0,
      0,
      1_080,
      2_424
    ]; }
  },
  {
    rule: 'closed-no-pane',
    at: closed,
    change: item => { item.scrollMaxPixels = 0; }
  },
  {
    rule: 'closed-only-player',
    at: fits,
    change: item => { item.player = [
      0,
      136,
      2_076,
      2_074
    ]; }
  },
  {
    rule: 'pane-rectangle',
    at: fits,
    change: item => { item.pane = 'garbage'; }
  },
  {
    rule: 'pane-placement',
    at: fits,
    change: item => { item.pane[0]++; }
  },
  {
    rule: 'pane-placement',
    at: cover,
    change: item => { item.pane = [
      540,
      0,
      1_080,
      2_424
    ]; }
  },
  {
    rule: 'left-half',
    at: fits,
    change: item => { item.left = null; }
  },
  {
    rule: 'left-half',
    at: cover,
    change: item => { item.left = [
      0,
      0,
      1_038,
      2_152
    ]; }
  },
  {
    rule: 'header-rectangle',
    at: fits,
    change: item => { item.header = null; }
  },
  {
    rule: 'header-position',
    at: fits,
    change: item => { item.header[0]++; }
  },
  {
    rule: 'header-position',
    at: fits,
    change: item => { item.header[1]++; }
  },
  {
    rule: 'header-height',
    at: fits,
    change: item => { item.header[3]++; }
  },
  {
    rule: 'back-rectangle',
    at: fits,
    change: item => { item.back = 'garbage'; }
  },
  {
    rule: 'back-floor',
    at: fits,
    change: item => { item.back[2] = item.back[0] + 116; }
  },
  {
    rule: 'back-floor',
    at: fits,
    change: item => { item.back[3] = item.back[1] + 116; }
  },
  {
    rule: 'back-inside-header',
    at: fits,
    change: item => { item.back[1] += 100;
      item.back[3] += 100; }
  },
  {
    rule: 'title-rectangle',
    at: fits,
    change: item => { item.title = null; }
  },
  {
    rule: 'title-inside-header',
    at: fits,
    change: item => { item.title[3] = item.header[3] + 1; }
  },
  {
    rule: 'back-before-title',
    at: fits,
    change: item => { item.title[0] = item.back[2] - 1; }
  },
  {
    rule: 'title-one-line',
    at: fits,
    change: item => { item.titleLines = 2; }
  },
  {
    rule: 'title-one-line',
    at: fits,
    change: item => { item.titleOverflow = true; }
  },
  {
    rule: 'viewport-rectangle',
    at: fits,
    change: item => { item.viewport = 'garbage'; }
  },
  {
    rule: 'viewport-position',
    at: fits,
    change: item => { item.viewport[1] += 10; }
  },
  {
    rule: 'viewport-position',
    at: fits,
    change: item => { item.viewport[3]--; }
  },
  {
    rule: 'scroll-extent',
    at: fits,
    change: item => { item.scrollMaxPixels = -1; }
  },
  {
    rule: 'scroll-extent',
    at: fits,
    change: item => { item.scrollMaxPixels = 1.5; }
  },
  {
    rule: 'rows-list',
    at: fits,
    change: item => { item.rows = item.rows
      .slice(
      0,
      1
      ); }
  },
  {
    rule: 'rows-list',
    at: fits,
    change: item => { item.rows
      .push(structuredClone(item.rows[1])); }
  },
  {
    rule: 'visible-record',
    at: fits,
    change: item => { item.visible = null; }
  },
  {
    rule: 'visible-record',
    at: fits,
    change: item => { item.visible
      .rows = []; }
  },
  {
    rule: 'row-record',
    at: fits,
    change: item => { item.rows[1] = null; }
  },
  {
    rule: 'row-identity',
    at: fits,
    change: item => { item.rows[0]
      .id = 'resume-where-left-off';
      item.rows[1]
        .id = 'strip-common-prefixes'; }
  },
  {
    rule: 'row-rectangles',
    at: fits,
    change: item => { item.rows[0]
      .switch = null; }
  },
  {
    rule: 'row-spans-page',
    at: fits,
    change: item => { item.rows[0]
      .row[0]++; }
  },
  {
    rule: 'row-floor',
    at: fits,
    change: item => { item.rows[0]
      .row[3] = item.rows[0]
        .row[1]
        + 116; }
  },
  {
    rule: 'row-order',
    at: fits,
    change: item => { item.rows[1]
      .row[1] -= 5; }
  },
  {
    rule: 'row-order',
    at: fits,
    change: item => { item.rows[1]
      .row[1] += 10; }
  },
  // A start view whose rows sit above the viewport top is not at the column start.
  {
    rule: 'row-order',
    at: large,
    change: item => { item.rows[0]
      .row[1] -= 40; }
  },
  {
    rule: 'switch-size',
    at: fits,
    change: item => { item.rows[0]
      .switch[2]++; }
  },
  {
    rule: 'switch-size',
    at: fits,
    change: item => { item.rows[1]
      .switch[3]--; }
  },
  {
    rule: 'row-parts-inside',
    at: fits,
    change: item => { item.rows[0]
      .title[0] = item.rows[0]
        .row[0]
        - 1; }
  },
  {
    rule: 'title-above-supporting',
    at: fits,
    change: item => { item.rows[0]
      .supporting[1] = item.rows[0]
        .title[3]
        - 1; }
  },
  {
    rule: 'text-clear-of-switch',
    at: fits,
    change: item => { item.rows[0]
      .supporting[2] = item.rows[0]
        .switch[0]
        + 1; }
  },
  {
    rule: 'row-text-lines',
    at: fits,
    change: item => { item.rows[0]
      .supportingLines = 0; }
  },
  {
    rule: 'row-text-lines',
    at: fits,
    change: item => { item.rows[0]
      .titleLines = '1'; }
  },
  {
    rule: 'row-text-overflow',
    at: fits,
    change: item => { item.rows[0]
      .titleOverflow = true; }
  },
  {
    rule: 'row-text-overflow',
    at: fits,
    change: item => { item.rows[1]
      .supportingOverflow = true; }
  },
  {
    rule: 'visible-row-record',
    at: fits,
    change: item => { item.visible
      .rows[0]
      .id = 'analyse-in-background'; }
  },
  {
    rule: 'visible-row-part',
    at: fits,
    change: item => { item.visible
      .rows[0]
      .row[3]++; }
  },
  {
    rule: 'visible-switch-part',
    at: fits,
    change: item => { item.visible
      .rows[0]
      .switch = [
        0,
        0,
        10,
        10
      ]; }
  },
  {
    rule: 'fold-connector',
    at: fits,
    change: item => { item.rows[0]
      .title[0] = 1_092; }
  },
  {
    rule: 'fold-connector',
    at: fits,
    change: item => { item.rows[1]
      .supporting[0] = 1_092; }
  },
  {
    rule: 'scroll-geometry',
    at: fits,
    change: item => { item.scrollMaxPixels = 7; }
  },
  // A last row reaching past the navigation area overhangs the viewport, so a zero extent is inconsistent.
  {
    rule: 'scroll-geometry',
    at: large,
    change: item => { item.rows[1]
      .row[3] += 2_000; }
  },
  // A consistent 10px overhang: the extent agrees with the geometry, so only the no-scroll rule refuses it.
  {
    rule: 'column-fits',
    at: fits,
    change: item => { item.rows[1]
      .row[3] = (item.applicationRoot[3] + 10) - (item.rows[1]
        .row[1]
        - item.rows[0]
          .row[3]);
      item.scrollMaxPixels = 10; }
  },
  {
    rule: 'rows-shown',
    at: fits,
    change: item => { item.visible
      .rows[1]
      .row = null; }
  },
  {
    rule: 'exact-cohort',
    manifest: input => { input.witnesses
      .splice(
      input.witnesses
        .indexOf(pick({
          input,
          ...closed
        })),
      1
      ); }
  },
  // Per-view rules still hold here, so only the cross-view rule can refuse these.
  {
    rule: 'layout-stable',
    at: other,
    change: item => { item.rows[1]
      .titleLines = 3; }
  },
  {
    rule: 'layout-stable',
    at: {
      ...fits,
      scheme: 'dark'
    },
    change: item => { item.rows[0]
      .supportingLines = 5; }
  },
  {
    rule: 'template-slot',
    template: text => text.replace(
      '__SETTINGS_PANE_IMAGES__',
      '{}'
    )
  },
  {
    rule: 'template-slot',
    template: text => text.replace(
      '__SETTINGS_PANE_FINDINGS__',
      ''
    )
  },
  {
    rule: 'template-slot',
    template: text => text.replace(
      '__SETTINGS_PANE_PROVENANCE__',
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
    rule: 'required-statement',
    phase: 'validate',
    template: text => text.replace(
      'it was not separately chosen',
      'it was chosen'
    )
  },
  // D86 withdrew both rows; a page that drops the withdrawal notice would present them as current.
  {
    rule: 'required-statement',
    phase: 'validate',
    template: text => text.replace(
      'It is not a current design to review',
      'It is the current design'
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
      '__SETTINGS_PANE_EXTRA__<dialog id="preview"'
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
  // Valid images of the same size whose drawn switches are not the recorded state.
  {
    rule: 'switch-position-pixels',
    phase: 'validate',
    at: fits,
    image: () => evidenceBytes(other)
  },
  {
    rule: 'switch-position-pixels',
    phase: 'validate',
    at: fits,
    image: () => evidenceBytes({
      panel: 'inner',
      view: 'closed',
      scale: 1,
      position: 'none'
    })
  },
  // Row text pasted into the empty page below the rows, where D85's removed closing sentence used to be.
  {
    rule: 'nothing-after-rows',
    phase: 'validate',
    at: fits,
    image: (
      bytes,
      path
    ) => paste({
      path,
      from: '1500+250',
      size: '300x100',
      to: '1500+1500'
    })
  },
  // Empty header ground pasted over the title, then over the Back glyph: every rectangle still holds, the ink does not.
  {
    rule: 'title-contrast-pixels',
    phase: 'validate',
    at: fits,
    image: (
      bytes,
      path
    ) => paste({
      path,
      from: '1700+54',
      size: '300x69',
      to: '1269+54'
    })
  },
  {
    rule: 'back-contrast-pixels',
    phase: 'validate',
    at: fits,
    image: (
      bytes,
      path
    ) => paste({
      path,
      from: '1700+30',
      size: '117x117',
      to: '1132+30'
    })
  },
  // Part of the Settings page pasted over the folder browser on the retained left half.
  {
    rule: 'left-half-search',
    phase: 'validate',
    at: fits,
    image: (
      bytes,
      path
    ) => paste({
      path,
      from: '1500+900',
      size: '300x300',
      to: '200+400'
    })
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
  if (testCase.rename) {
    const renamed = testCase.rename(item.file);
    if (renamed === item.file) throw new Error(`Rename case changed nothing: ${  testCase.rule}`);
    copyFileSync(
      join(
        evidence,
        item.file
      ),
      join(
        evidence,
        renamed
      )
    );
    undo.push(() =>{  rmSync(join(
      evidence,
      renamed
    )); });
    item.file = renamed;
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
    throw new Error(`Expected Settings-pane rejection absent: ${testCase.rule}; got ${run.status === 0 ? 'acceptance' : run.stderr
      .trim()
      .split('\n')
      .find(line => line.includes('Error'))
      ?? 'another failure'}`);
  }
}
//endregion

//region Script-data encoding is checked behind the rules that normally make it unreachable
// The path and filename rules confine image names to plain characters, so the `<` encoding of the
// embedded image table can never be seen through them. A second builder copy without those two
// rules shows that a name which would otherwise end the script's parsing stays inert data.
const relaxed = join(
  fixture,
  'settings-pane-relaxed.mjs'
);
const hostile = 'settings-pane-<!--<script>.png';
function ruleLine({
  source,
  rule
}) {
  const lines = source.split('\n')
    .filter(line => line.includes(`need({ rule: '${rule}',`));
  if (lines.length !== 1) throw new Error(`Exact Settings-pane rule line absent: ${  rule}`);
  return `${lines[0]  }\n`;
}
function encodingHolds() {
  const input = structuredClone(manifest);
  const item = pick({
    input,
    ...closed
  });
  copyFileSync(
    join(
      evidence,
      item.file
    ),
    join(
      evidence,
      hostile
    )
  );
  item.file = hostile;
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
  rmSync(join(
    evidence,
    hostile
  ));
  writeFileSync(
    manifestPath,
    JSON.stringify(manifest)
  );
  if (run.status !== 0) throw new Error(`Relaxed Settings-pane builder failed: ${  run.stderr}`);
  if ((!built.includes(String.raw`settings-pane-\u003c!--\u003cscript>.png`)) || built.includes(hostile)
    || ((built.match(/<script\b/gi) ?? []).length !== 1)) {
    throw new Error('Expected Settings-pane rejection absent: script-encoding; got an unencoded image name in the page script');
  }
}
//endregion

//region Normal run, rule listing, or one deleted rule
try {
  const source = readFileSync(
    join(
      root,
      'settings-pane.mjs'
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
      'script-encoding'
    ]));
  } else {
    if ((mode !== undefined) && ((mode !== 'without') || (![
      ...rules,
      'script-encoding'
    ].includes(target)))) throw new Error('Unknown Settings-pane consumer mutation.');
    const encoding = `.replace('__SETTINGS_PANE_IMAGES__', () => JSON.stringify(images).replaceAll('<', '\\\\u003c'));`;
    if (source.split(encoding)
      .length
      !== 2) throw new Error('Exact Settings-pane script encoding absent.');
    let subject = source;
    if (target === 'script-encoding') subject = source.replace(
      encoding,
      `.replace('__SETTINGS_PANE_IMAGES__', () => JSON.stringify(images));`
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
          rule: 'witness-path'
        }),
        ''
      )
        .replace(
        ruleLine({
          source,
          rule: 'witness-filename'
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
        'id="inspection-findings"',
        'against a 117 pixel floor',
        'Neither a switch position nor the theme changes any measured rectangle.',
        'prototype commit 67eae28d1f31689bcc9f132e4db97ec87426e572',
        'APK SHA-256 5f3a23911f2a5c32859a78e533704298bfb5afa042cf968f4fc82691ef35e9ab',
        'renderer Android Emulator OpenGL ES Translator (llvmpipe (LLVM 20.1.2, 256 bits))'
      ]) {
        if (!built.includes(statement)) throw new Error(`Generated statement absent: ${  statement}`);
      }
      // A changed measured inset is followed, not compared with a fixed crop.
      const changedInset = structuredClone(manifest);
      const view = pick({
        input: changedInset,
        ...closed
      });
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
    if ((target === undefined) || [
      'script-encoding',
      'witness-path',
      'witness-filename'
    ].includes(target)) encodingHolds();
    positive();
    console.log(`Settings-pane consumer: ${rules.length} rules, ${cases.length} rejected inputs, script encoding and positive controls passed.`);
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
