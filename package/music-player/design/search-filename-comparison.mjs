import { createHash } from 'node:crypto';
import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

//region Provenance, only committed sanitized images enter the offline review
const root = process.cwd();
const question = join(
  root,
  'questions'
);
const evidenceRoot = join(
  question,
  'evidence'
);
const comparison = JSON.parse(readFileSync(
  join(
    evidenceRoot,
    'search-filename-comparison-witnesses.json'
  ),
  'utf8'
));
const baseline = JSON.parse(readFileSync(
  join(
    evidenceRoot,
    'search-filename-actual-player-witnesses.json'
  ),
  'utf8'
));
if ((comparison.witnesses
  .length
  !== 70) || (baseline.captures
    .length
    !== 32)) {
  throw new Error('Filename review cohort differs from the inspected study.');
}
const images = {};
for (const capture of [
  ...comparison.witnesses,
  ...baseline.captures
]) {
  const file = capture.file ?? capture.image;
  if (!/^search-filename-(?:comparison|actual-player)-[a-z0-9-]+\.png$/.test(file)) {
    throw new Error('Filename review capture name is outside the allowed evidence boundary.');
  }
  const png = readFileSync(join(
    evidenceRoot,
    file
  ));
  const hash = createHash('sha256')
    .update(png)
    .digest('hex');
  const dimensions = capture.cropPixels;
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (png.readUInt32BE(16) !== dimensions.width)
    || (png.readUInt32BE(20) !== dimensions.height)
    ||
    (![
      'inner',
      'cover'
    ].includes(capture.panel))
    || (![
      'light',
      'dark'
    ].includes(capture.scheme))
    ||
    (![
      1,
      2
    ].includes(capture.fontScale))) {
    throw new Error(`${file}: filename review capture dimension or environment failed.`);
  }
  for (let offset = 8; offset < png.length;) {
    const length = png.readUInt32BE(offset);
    const chunk = png.subarray(
      offset + 4,
      offset + 8
    )
      .toString('ascii');
    if (![
      'IHDR',
      'IDAT',
      'IEND'
    ].includes(chunk)) throw new Error(`${file}: unexpected PNG metadata.`);
    offset += length + 12;
  }
  const source = file.includes('-actual-player-') ? 'baseline' : 'comparison';
  const position = capture.position ?? 'initial';
  const scene = source === 'baseline' ? `${capture.scene}-${capture.selection}` : capture.scene;
  const key = `${source}/${capture.panel}/${scene}/${capture.scheme}/${capture.fontScale}/${position}`;
  const endMarker = position === 'scrolled' ? '-end' : '';
  const expectedFile = `search-filename-${source === 'baseline' ? 'actual-player' : 'comparison'}-${capture.panel}-${scene}${endMarker}-${capture.scheme}-s${capture.fontScale * 100}.png`;
  if ((file !== expectedFile) || (![
    'initial',
    'scrolled'
  ].includes(position))) {
    throw new Error(`${file}: filename and capture metadata disagree.`);
  }
  if (images[key]) throw new Error(`Duplicate filename review capture: ${key}.`);
  images[key] = {
    file,
    hash,
    width: dimensions.width,
    height: dimensions.height,
    density: 390,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
}
for (const panel of [
  'inner',
  'cover'
]) {
  for (const scheme of [
    'light',
    'dark'
  ]) {
    for (const scale of [
      1,
      2
    ]) {
      for (const scene of [
        'placementfull',
        'placementsupport',
        'literalfull',
        'literalsupport',
        'visibilityfull',
        'visibilityconditional',
        'visibilitysupportfull',
        'visibilitysupportconditional'
      ]) {
        if (!images[`comparison/${panel}/${scene}/${scheme}/${scale}/initial`]) {
          throw new Error(`Missing comparison combination: ${panel}/${scene}/${scheme}/${scale}.`);
        }
      }
      for (const scene of [
        'long-none',
        'long-first',
        'long-second',
        'short-none'
      ]) {
        if (!images[`baseline/${panel}/${scene}/${scheme}/${scale}/initial`]) {
          throw new Error(`Missing actual-renderer combination: ${panel}/${scene}/${scheme}/${scale}.`);
        }
      }
    }
  }
}
//endregion

//region Build and validation, template and committed output must contain the same checked cohort
const expectedKeys = [];
for (const panel of [
  'inner',
  'cover'
]) {
  for (const scheme of [
    'light',
    'dark'
  ]) {
    for (const scale of [
      1,
      2
    ]) {
      for (const scene of [
        'placementfull',
        'placementsupport',
        'literalfull',
        'literalsupport',
        'visibilityfull',
        'visibilityconditional',
        'visibilitysupportfull',
        'visibilitysupportconditional'
      ]) {
        expectedKeys.push(`comparison/${panel}/${scene}/${scheme}/${scale}/initial`);
      }
      for (const scene of [
        'long-none',
        'long-first',
        'long-second',
        'short-none'
      ]) {
        expectedKeys.push(`baseline/${panel}/${scene}/${scheme}/${scale}/initial`);
      }
    }
  }
}
for (const scheme of [
  'light',
  'dark'
]) {
  for (const [panel, scene] of [
    [
      'inner',
      'literalfull'
    ],
    [
      'inner',
      'literalsupport'
    ],
    [
      'cover',
      'literalsupport'
    ]
  ]) {
    expectedKeys.push(`comparison/${panel}/${scene}/${scheme}/2/scrolled`);
  }
}
if (JSON.stringify(Object.keys(images)
  .sort()) !== JSON.stringify(expectedKeys.sort())) {
  throw new Error('Filename review must contain the exact initial, baseline and six scrolled combinations.');
}
const templatePath = join(
  question,
  'search-filename-comparison.template.html'
);
const outputPath = join(
  question,
  'search-filename-comparison.html'
);
const template = readFileSync(
  templatePath,
  'utf8'
);
if (template.split('__FILENAME_REVIEW_IMAGES__')
  .length
  !== 2) {
  throw new Error('Filename review template must contain exactly one image-map slot.');
}
// Encode at the HTML script boundary so a data key cannot terminate the script element.
const serialized = JSON.stringify(images)
  .replaceAll(
  '<',
  String.raw`\u003c`
  );
const html = template.replace(
  '__FILENAME_REVIEW_IMAGES__',
  () => serialized
);
const command = process.argv[2];
if (command === 'build') {
  writeFileSync(
    outputPath,
    html
  );
  console.log('Built self-contained filename evidence review with 102 verified images and optional observations.');
} else if (command === 'validate') {
  if (readFileSync(
    outputPath,
    'utf8'
  ) !== html) throw new Error('Filename review differs from template and checked evidence.');
  for (const marker of [
    'color-scheme: light dark',
    'Fixed-policy question withdrawn',
    'Supporting text is user-configurable through templates in Settings',
    'not template-engine output, implemented presets or Settings functionality',
    'Default templates and their interaction with required distinguishing information remain undesigned',
    'Cam Outside.mp3',
    'not indexed files',
    'id="final-notes"',
    'id="reply"',
    'Reset 100% dp',
    'Native pixels',
    'Unknown scope is not proof of uniqueness',
    'not native accessibility acceptance',
    'no production Search implementation',
    'Optional evidence observations',
    'No filename policy is selected'
  ]) {
    if (!html.includes(marker)) throw new Error(`Filename review is missing ${marker}.`);
  }
  if (/__FILENAME_REVIEW_IMAGES__|<script\b[^>]*\bsrc=|<link\b[^>]*\bhref=|<img\b[^>]*\bsrc="https?:/i.test(html) ||
    ((html.match(/<form\b/g) ?? []).length !== 1)) {
    throw new Error('Filename review is not a self-contained evidence-only form with optional observations.');
  }
  if (/<input\b[^>]*\btype="radio"|name="placement"|name="visibility"|data\.get\('(placement|visibility)'\)|<textarea\b(?:[^"'<>]|"[^"]*"|'[^']*')*?\srequired(?:\s|=|>)/i.test(html)) {
    throw new Error('Filename evidence review cannot contain policy votes or required observations.');
  }
  console.log('Validated exact offline evidence review, optional observations and native images.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
