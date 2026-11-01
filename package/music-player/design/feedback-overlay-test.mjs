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


//region Consumer verification uses disposable copies, never changes inspected publication evidence
const root = process.cwd();
const fixture = mkdtempSync(join(
  scratchBase,
  'feedback-overlay-review-'
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
const manifestName = 'feedback-overlay-witnesses.json';
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
  'feedback-overlay.template.html'
);
copyFileSync(
  join(
    root,
    'questions',
    'feedback-overlay.template.html'
  ),
  templatePath
);
const builder = join(
  fixture,
  'feedback-overlay.mjs'
);
copyFileSync(
  join(
    root,
    'feedback-overlay.mjs'
  ),
  builder
);
const mutation = process.argv[2];
if ((mutation !== undefined) && (mutation !== 'without-exact-cohort')
  && (mutation !== 'without-hold')) {
  throw new Error('Unknown overlay consumer test mutation.');
}
if (mutation !== undefined) {
  const source = readFileSync(
    builder,
    'utf8'
  );
  const guard = mutation === 'without-hold' ? 'capture.held !== true || ' :
    `if (JSON.stringify(Object.keys(images).sort()) !== JSON.stringify(expected.sort())) {\n  throw new Error('Overlay review requires exact panel, scene, theme and scale combinations.');\n}`;
  if (source.split(guard)
    .length
    !== 2) throw new Error('Exact overlay guard mutation target absent.');
  writeFileSync(
    builder,
    source.replace(
      guard,
      ''
    )
  );
}
function invoke(command) {
  return spawnSync(
    process.execPath,
    [
      builder,
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
    const run = invoke(command);
    if (run.status !== 0) throw new Error(`Positive overlay consumer failed: ${  run.stderr}`);
  }
}
function reject({
  input,
  diagnostic
}) {
  writeFileSync(
    join(
      evidence,
      manifestName
    ),
    JSON.stringify(input)
  );
  const run = invoke('build');
  if ((run.status === 0) || (!run.stderr
    .includes(diagnostic))) {
    throw new Error(`Expected overlay rejection absent: ${  diagnostic}`);
  }
  writeFileSync(
    join(
      evidence,
      manifestName
    ),
    JSON.stringify(manifest)
  );
}
try {
  positive();
  const differentInset = structuredClone(manifest);
  const resized = differentInset.witnesses[0];
  const imagePath = join(
    evidence,
    resized.file
  );
  const originalPng = readFileSync(imagePath);
  const inputPath = join(
    fixture,
    'inset-positive-source.png'
  );
  writeFileSync(
    inputPath,
    originalPng
  );
  resized.applicationRoot[1]++;
  resized.cropPixels
    .y++;
  resized.cropPixels
    .height--;
  const crop = spawnSync(
    'magick',
    [
      '-limit',
      'thread',
      '2',
      '-limit',
      'memory',
      '256MiB',
      inputPath,
      '-crop',
      `${resized.cropPixels
        .width}x${resized.cropPixels
          .height}+0+1`,
      '+repage',
      '-strip',
      '-define',
      'png:exclude-chunks=all',
      `PNG24:${  imagePath}`
    ],
    { encoding: 'utf8' }
  );
  if (crop.status !== 0) throw new Error(`Synthetic inset fixture failed: ${  crop.stderr}`);
  writeFileSync(
    join(
      evidence,
      manifestName
    ),
    JSON.stringify(differentInset)
  );
  positive();
  writeFileSync(
    imagePath,
    originalPng
  );
  writeFileSync(
    join(
      evidence,
      manifestName
    ),
    JSON.stringify(manifest)
  );
  const cropOrigin = structuredClone(manifest);
  cropOrigin.witnesses[0]
    .cropPixels
    .y++;
  reject({
    input: cropOrigin,
    diagnostic: 'measured native application bounds'
  });
  const absentBounds = structuredClone(manifest);
  delete absentBounds.witnesses[0]
    .applicationRoot;
  reject({
    input: absentBounds,
    diagnostic: 'measured native application bounds'
  });
  const wrongPanel = structuredClone(manifest);
  wrongPanel.witnesses[0]
    .physicalPixels[1]++;
  reject({
    input: wrongPanel,
    diagnostic: 'measured native application bounds'
  });
  const wrongRenderer = structuredClone(manifest);
  wrongRenderer.witnesses[0]
    .renderer = 'SwiftShader';
  reject({
    input: wrongRenderer,
    diagnostic: 'image geometry, hold or acquisition assertion differs'
  });
  const wrongImage = structuredClone(manifest);
  wrongImage.witnesses[0]
    .systemImageFingerprint = 'different-image';
  reject({
    input: wrongImage,
    diagnostic: 'image geometry, hold or acquisition assertion differs'
  });
  const badPath = structuredClone(manifest);
  badPath.witnesses[0]
    .file = '../outside.png';
  reject({
    input: badPath,
    diagnostic: 'outside the evidence boundary'
  });
  const mismatched = structuredClone(manifest);
  mismatched.witnesses[0]
    .scene = 'mismatch';
  reject({
    input: mismatched,
    diagnostic: 'filename and metadata disagree'
  });
  const missing = structuredClone(manifest);
  const target = missing.witnesses[0];
  const original = target.file;
  target.scene = 'unplanned';
  target.file = `feedback-overlay-${target.panel}-unplanned-${target.scheme}-s${target.fontScale * 100}.png`;
  copyFileSync(
    join(
      evidence,
      original
    ),
    join(
      evidence,
      target.file
    )
  );
  reject({
    input: missing,
    diagnostic: 'exact panel, scene, theme and scale combinations'
  });
  const missingHold = structuredClone(manifest);
  missingHold.witnesses[0]
    .held = false;
  reject({
    input: missingHold,
    diagnostic: 'image geometry, hold or acquisition assertion differs'
  });
  const unknownAcquisition = structuredClone(manifest);
  unknownAcquisition.witnesses[0]
    .freshHierarchyValidated = false;
  reject({
    input: unknownAcquisition,
    diagnostic: 'image geometry, hold or acquisition assertion differs'
  });
  const template = readFileSync(
    templatePath,
    'utf8'
  );
  writeFileSync(
    templatePath,
    template.replace(
      '<form id="review-form">',
      '<form id="review-form"><input type="radio" name="policy">'
    )
  );
  if (invoke('build')
    .status
    !== 0) throw new Error('Ballot fixture did not build.');
  const ballot = invoke('validate');
  if ((ballot.status === 0) || (!ballot.stderr
    .includes('not a policy ballot'))) throw new Error('Unexpected policy ballot accepted.');
  writeFileSync(
    templatePath,
    template
  );
  positive();
  const output = join(
    fixture,
    'questions',
    'feedback-overlay.html'
  );
  writeFileSync(
    output,
    readFileSync(
      output,
      'utf8'
    )
      .replace(
      'Design evidence only.',
      'Changed output.'
      )
  );
  if (invoke('validate')
    .status
    === 0) throw new Error('Changed overlay artifact accepted.');
  console.log('Overlay review positive, measured crop, physical panel, renderer, system image, path, metadata, exact cohort, hold, acquisition, ballot and output controls passed.');
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
