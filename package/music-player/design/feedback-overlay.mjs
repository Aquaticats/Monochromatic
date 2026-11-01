import { createHash } from 'node:crypto';
import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

//region Exact inspected authored cohort, native holds cannot become lifecycle evidence
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
    'feedback-overlay-witnesses.json'
  ),
  'utf8'
));
if ((manifest.schema !== 2) || (manifest.witnesses
  .length
  !== 48)) {
  throw new Error('Overlay manifest schema or inspected cohort differs.');
}
const expected = [];
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
        'missing',
        'undo',
        'combined',
        'trash-failed',
        'trash-pending',
        'detail-heavy'
      ]) {
        expected.push(`comparison/${panel}/${scene}/${scheme}/${scale}/initial`);
      }
    }
  }
}
const images = {};
for (const capture of manifest.witnesses) {
  const {
    file,
    panel,
    scene,
    scheme,
    fontScale
  } = capture;
  if (!/^feedback-overlay-(?:inner|cover)-[a-z-]+-(?:light|dark)-s(?:100|200)\.png$/.test(file)) {
    throw new Error('Overlay image path is outside the evidence boundary.');
  }
  if (file !== `feedback-overlay-${panel}-${scene}-${scheme}-s${fontScale * 100}.png`) {
    throw new Error('Overlay filename and metadata disagree.');
  }
  const png = readFileSync(join(
    evidence,
    file
  ));
  const hash = createHash('sha256')
    .update(png)
    .digest('hex');
  const width = panel === 'inner' ? 2_076 : 1_080;
  const physicalHeight = panel === 'inner' ? 2_152 : 2_424;
  const bounds = capture.applicationRoot;
  if ((!Array.isArray(bounds)) || (bounds.length !== 4)
    || (!bounds.every(Number.isInteger))
    ||
    (bounds[0] !== 0)
    || (bounds[2] !== width)
    || (bounds[1] < 1)
    || (bounds[1] >= (physicalHeight
      / 2))
    ||
    (bounds[3] <= bounds[1])
    || (bounds[3] > physicalHeight)
    ||
    (JSON.stringify(capture.physicalPixels) !== JSON.stringify([
      width,
      physicalHeight
    ]))
    ||
    (capture.cropPixels
      .x
      !== 0)
    || (capture.cropPixels
      .y
      !== bounds[1])) {
    throw new Error('Overlay crop must follow its measured native application bounds.');
  }
  const height = physicalHeight - bounds[1];
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (png.readUInt32BE(16) !== width)
    || (png.readUInt32BE(20) !== height)
    ||
    (capture.cropPixels
      .width
      !== width)
    || (capture.cropPixels
      .height
      !== height)
    ||
    (capture.densityDpi !== 390)
    || (capture.held !== true)
    || (capture.freshHierarchyValidated !== true)
    ||
    (capture.keyboardClosed !== true)
    || (capture.inspected !== true)
    || (png[24] !== 8)
    || (png[25] !== 2)
    ||
    (capture.renderer !== 'llvmpipe (LLVM 20.1.2, 256 bits)')
    ||
    (capture.rendererRequested !== 'host with renderD128 exposed')
    ||
    (capture.runtimeLibraryEnvironment !== 'Container-native Ubuntu runtime; no host /usr mount')
    ||
    (capture.systemImageFingerprint !== 'google/sdk_gphone16k_x86_64/emu64xa16k:17/CE2A.260420.050/16231978:user/dev-keys')) {
    throw new Error('Overlay image geometry, hold or acquisition assertion differs.');
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
    ].includes(chunk)) throw new Error('Overlay PNG contains unintended metadata.');
    offset += length + 12;
  }
  const key = `comparison/${panel}/${scene}/${scheme}/${fontScale}/initial`;
  if (images[key]) throw new Error('Duplicate overlay witness.');
  images[key] = {
    file,
    hash,
    width,
    height,
    density: 390,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
}
if (JSON.stringify(Object.keys(images)
  .sort()) !== JSON.stringify(expected.sort())) {
  throw new Error('Overlay review requires exact panel, scene, theme and scale combinations.');
}
//endregion

//region Reproducible self-contained consumer output, no policy ballot or implied storage action
const template = readFileSync(
  join(
    question,
    'feedback-overlay.template.html'
  ),
  'utf8'
);
if (template.split('__FEEDBACK_OVERLAY_IMAGES__')
  .length
  !== 2) throw new Error('Expected one overlay image slot.');
const html = template.replace(
  '__FEEDBACK_OVERLAY_IMAGES__',
  () => JSON.stringify(images)
    .replaceAll(
    '<',
    String.raw`\u003c`
    )
);
const output = join(
  question,
  'feedback-overlay.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log('Built self-contained D83 overlay evidence from exact inspected native holds.');
} else if (process.argv[2] === 'validate') {
  if (readFileSync(
    output,
    'utf8'
  ) !== html) throw new Error('Overlay output differs from its template and inspected evidence.');
  for (const marker of [
    'color-scheme: light dark',
    'D83 is settled; no new preference ballot',
    'Every source outcome is authored debug input',
    'explicitly held native poses',
    'not proof of automatic expiry',
    'Capture Android logs',
    'does not export a log file',
    'No production implementation is authorized',
    'id="final-notes"',
    'Native pixels',
    'Reset 100% dp'
  ]) {
    if (!html.includes(marker)) throw new Error(`Overlay review is missing ${  marker}`);
  }
  if (((html.match(/<form\b/g) ?? []).length !== 1) ||
    /__FEEDBACK_OVERLAY_IMAGES__|<script\b[^>]*\bsrc=|<link\b[^>]*\bhref=|<img\b[^>]*\bsrc="https?:/i.test(html)
    ||
    /<input\b[^>]*\btype="radio"|<textarea\b(?:[^"'<>]|"[^"]*"|'[^']*')*?\srequired(?:\s|=|>)/i.test(html)) {
    throw new Error('Overlay review must be offline evidence, not a policy ballot.');
  }
  console.log('Validated exact offline D83 overlay review and authored-hold boundary.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
