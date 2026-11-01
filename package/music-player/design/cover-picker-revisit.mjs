import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

//region Inspected native cohort, each view checked from its own image and record
// D88: nothing is compared with a recorded digest. What is checked about an image is read from the image.
const question = join(
  process.cwd(),
  'questions'
);
const evidence = join(
  question,
  'evidence'
);
const manifest = JSON.parse(readFileSync(
  join(
    evidence,
    'cover-picker-revisit-witnesses.json'
  ),
  'utf8'
));
// The folded cover only (D46 concerns it): 1080 by 2424 physical pixels.
const panel = [
  1_080,
  2_424
];
// P4 is the temporary choice (D46); P5 is the agent's version, P4 whose trigger shows that the picker is open.
const variants = [
  'p4',
  'p5'
];
const expectedTexts = [
  'Camellia',
  'Open',
  'C418'
];
if ((manifest.schema !== 1) || (!Array.isArray(manifest.witnesses))) throw new Error('Cover picker review manifest is not the expected study.');
const images = {};
const views = {};
for (const capture of manifest.witnesses) {
  const {file} = capture;
  if (!/^cover-picker-revisit-p[45]-(?:light|dark)-s(?:100|200)\.png$/.test(file)) {
    throw new Error('Cover picker capture path is outside the allowed evidence boundary.');
  }
  const expectedFile = `cover-picker-revisit-${capture.variant}-${capture.scheme}-s${capture.fontScale * 100}.png`;
  if ((file !== expectedFile) || (!variants.includes(capture.variant))) throw new Error('Cover picker capture filename and metadata disagree.');
  const png = readFileSync(join(
    evidence,
    file
  ));
  const crop = capture.cropPixels;
  // The status strip is removed and the navigation area kept, so the crop starts under the strip and ends at the panel's end.
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (capture.physicalPixels[0] !== panel[0])
    || (capture.physicalPixels[1] !== panel[1])
    ||
    (crop.x !== 0)
    || (crop.width !== panel[0])
    || (!Number.isInteger(crop.y))
    || (crop.y < 1)
    ||
    ((crop.y + crop.height) !== panel[1])
    || (png.readUInt32BE(16) !== crop.width)
    || (png.readUInt32BE(20) !== crop.height)
    ||
    (capture.densityDpi !== 390)
    || (capture.keyboardClosed !== true)
    || (capture.stableAppFrames !== true)) {
    throw new Error(`${file}: cover picker image geometry or acquisition assertion differs.`);
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
    ].includes(chunk)) throw new Error('Cover picker PNG contains unintended metadata.');
    offset += length + 12;
  }
  if ((!Array.isArray(capture.texts)) || expectedTexts.some(text => !capture.texts
    .some(drawn => drawn.includes(text)))) {
    throw new Error(`${file}: cover picker drawn text lacks the trigger, the Open action or the first folder.`);
  }
  const key = `comparison/${capture.variant}/${capture.scheme}/${capture.fontScale}`;
  if (images[key]) throw new Error('Duplicate cover picker review capture.');
  images[key] = {
    file,
    width: crop.width,
    height: crop.height,
    density: 390,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
  views[key] = capture;
}
// Every candidate in both themes at both text sizes.
for (const variant of variants) for (const scheme of [
  'light',
  'dark'
]) for (const scale of [
  1,
  2
]) {
  if (!views[`comparison/${variant}/${scheme}/${scale}`]) throw new Error(`Cover picker review requires every candidate under every condition: ${variant}/${scheme}/${scale}`);
}
// The title of both candidates is drawn at one place: P5 must not move it when the picker opens.
// The container's offset is in dp, so rounding to physical pixels can differ by one pixel; two is the bound.
for (const scheme of [
  'light',
  'dark'
]) for (const scale of [
  1,
  2
]) {
  const closed = views[`comparison/p4/${scheme}/${scale}`]
    .titleBounds;
  const open = views[`comparison/p5/${scheme}/${scale}`]
    .titleBounds;
  if ((!Array.isArray(closed)) || (closed.length !== 4)
    || (!Array.isArray(open))
    || open.some((
      value,
      index
    ) => Math.abs(value - closed[index]) > 2)) {
    throw new Error(`Cover picker title moves between P4 and P5 under ${scheme}/${scale}.`);
  }
}
const total = manifest.witnesses
  .length;
//endregion

//region Build and consumer validation, the candidates never become an implicit selection
const template = readFileSync(
  join(
    question,
    'cover-picker-revisit.template.html'
  ),
  'utf8'
);
if (template.split('__COVER_PICKER_REVIEW_IMAGES__')
  .length
  !== 2) throw new Error('Expected one cover picker image slot.');
const serialized = JSON.stringify(images)
  .replaceAll(
  '<',
  String.raw`\u003c`
  );
const html = template.replace(
  '__COVER_PICKER_REVIEW_IMAGES__',
  () => serialized
);
const output = join(
  question,
  'cover-picker-revisit.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log(`Built offline cover picker evidence with ${total} inspected native views.`);
} else if (process.argv[2] === 'validate') {
  if (readFileSync(
    output,
    'utf8'
  ) !== html) throw new Error('Cover picker review differs from template and checked evidence.');
  for (const marker of [
    'color-scheme: light dark',
    'Design evidence only',
    'P4, the current temporary choice',
    'P5, the agent\'s version',
    'Ranking: P5 &gt; P4',
    'No production implementation is authorized',
    'id="final-notes"',
    'id="reply"',
    'Native pixels',
    'Reset 100% dp'
  ]) {
    if (!html.includes(marker)) throw new Error(`Cover picker review is missing ${marker}.`);
  }
  // The page tells its own cohort's capture story: it names how many visits contributed views, read from the records.
  const visits = new Set(manifest.witnesses
    .map(capture => capture.visit)).size;
  if (!html.includes(`${visits} ${visits === 1 ? 'visit' : 'visits'} contributed views`)) throw new Error(`Cover picker review does not name its ${visits} visits.`);
  // Every figure belongs to a candidate, and every candidate has its figure.
  const figured = [...template.matchAll(/<figure data-variant="([a-z0-9]+)"/gu)].map(match => match[1]);
  for (const variant of figured) if (!variants.includes(variant)) throw new Error(`Cover picker review shows a figure for ${variant}, which is not a candidate.`);
  for (const variant of variants) if (figured.filter(name => name === variant)
    .length
    !== 1) throw new Error(`Cover picker review shows no single figure for ${variant}.`);
  if (((html.match(/<form\b/g) ?? []).length !== 1) ||
    /__COVER_PICKER_REVIEW_IMAGES__|<script\b[^>]*\bsrc=|<link\b[^>]*\bhref=|<img\b[^>]*\bsrc="https?:/i.test(html)
    ||
    /<input\b[^>]*\btype="radio"|<textarea\b(?:[^"'<>]|"[^"]*"|'[^']*')*?\srequired(?:\s|=|>)/i.test(html)) {
    throw new Error('Cover picker review must be self-contained evidence, not a ballot.');
  }
  console.log('Validated offline cover picker review, optional observations and the absence of a ballot.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
