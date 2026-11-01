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
  'track-menu-review-'
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
const manifestName = 'track-menu-witnesses.json';
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
  'track-menu.template.html'
);
copyFileSync(
  join(
    root,
    'questions',
    'track-menu.template.html'
  ),
  templatePath
);
const builder = join(
  fixture,
  'track-menu.mjs'
);
copyFileSync(
  join(
    root,
    'track-menu.mjs'
  ),
  builder
);
const mutation = process.argv[2];
if ((mutation !== undefined) && (mutation !== 'without-exact-cohort')
  && (mutation !== 'without-native-input')) throw new Error('Unknown track-menu consumer mutation.');
if (mutation !== undefined) {
  const source = readFileSync(
    builder,
    'utf8'
  );
  const guard = mutation === 'without-native-input' ? 'capture.nativeLongPress !== true || ' :
    `if (JSON.stringify(Object.keys(images).sort()) !== JSON.stringify(expected.sort())) {\n  throw new Error('Track-menu review requires exact panel, scene, theme and scale combinations.');\n}`;
  if (source.split(guard)
    .length
    !== 2) throw new Error('Exact track-menu guard target absent.');
  writeFileSync(
    builder,
    source.replace(
      guard,
      ''
    )
  );
}
function invoke(command) { return spawnSync(
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
); }
function positive() {
  for (const command of [
    'build',
    'validate'
  ]) {
    const run = invoke(command);
    if (run.status !== 0) throw new Error(`Positive track-menu consumer failed: ${  run.stderr}`);
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
    .includes(diagnostic))) throw new Error(`Expected track-menu rejection absent: ${  diagnostic}`);
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
  const changedInset = structuredClone(manifest);
  const view = changedInset.witnesses[0];
  const imagePath = join(
    evidence,
    view.file
  );
  const original = readFileSync(imagePath);
  const sourcePath = join(
    fixture,
    'inset-source.png'
  );
  writeFileSync(
    sourcePath,
    original
  );
  view.applicationRoot[1]++;
  view.cropPixels
    .y++;
  view.cropPixels
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
      sourcePath,
      '-crop',
      `${view.cropPixels
        .width}x${view.cropPixels
          .height}+0+1`,
      '+repage',
      '-strip',
      '-define',
      'png:exclude-chunks=all',
      `PNG24:${  imagePath}`
    ],
    { encoding: 'utf8' }
  );
  if (crop.status !== 0) throw new Error(`Synthetic changed-inset fixture failed: ${  crop.stderr}`);
  writeFileSync(
    join(
      evidence,
      manifestName
    ),
    JSON.stringify(changedInset)
  );
  positive();
  writeFileSync(
    imagePath,
    original
  );
  writeFileSync(
    join(
      evidence,
      manifestName
    ),
    JSON.stringify(manifest)
  );
  for (const [field, value] of [
    [
      'nativeLongPress',
      false
    ],
    [
      'allActionsInitiallyVisible',
      false
    ],
    [
      'targetIndex',
      99
    ],
    [
      'renderer',
      'different-renderer'
    ]
  ]) {
    const input = structuredClone(manifest);
    input.witnesses[0][field] = value;
    reject({
      input,
      diagnostic: 'image, native input, heading or acquisition assertion differs'
    });
  }
  const heading = structuredClone(manifest);
  heading.witnesses[0]
    .heading
    .lines = 8;
  reject({
    input: heading,
    diagnostic: 'image, native input, heading or acquisition assertion differs'
  });
  const badCrop = structuredClone(manifest);
  badCrop.witnesses[0]
    .cropPixels
    .y++;
  reject({
    input: badCrop,
    diagnostic: 'measured application bounds'
  });
  const badPath = structuredClone(manifest);
  badPath.witnesses[0]
    .file = '../outside.png';
  reject({
    input: badPath,
    diagnostic: 'outside the evidence boundary'
  });
  const missing = structuredClone(manifest);
  const target = missing.witnesses
    .find(item => item.scene === 'ordinary');
  const prior = target.file;
  target.scene = 'unplanned';
  target.file = `track-menu-${target.panel}-unplanned-${target.scheme}-s${target.fontScale * 100}.png`;
  copyFileSync(
    join(
      evidence,
      prior
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
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      === 0)) throw new Error('Policy ballot boundary failed.');
  writeFileSync(
    templatePath,
    template
  );
  positive();
  const output = join(
    fixture,
    'questions',
    'track-menu.html'
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
    === 0) throw new Error('Changed track-menu output accepted.');
  console.log('Track-menu consumer artifact, native-input, heading, target, crop, provenance, cohort and no-ballot controls passed.');
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
