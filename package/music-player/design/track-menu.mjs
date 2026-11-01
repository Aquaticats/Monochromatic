import { createHash } from 'node:crypto';
import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

//region Exact inspected native menu cohort, not a menu policy or real operation result
const questions = join(
  process.cwd(),
  'questions'
);
const evidence = join(
  questions,
  'evidence'
);
const manifest = JSON.parse(readFileSync(
  join(
    evidence,
    'track-menu-witnesses.json'
  ),
  'utf8'
));
if ((manifest.schema !== 1) || (manifest.witnesses
  .length
  !== 24)) {
  throw new Error('Track-menu manifest schema or inspected cohort differs.');
}
const expected = [];
for (const panel of [
  'inner',
  'cover'
]) for (const scene of [
  'ordinary',
  'lower',
  'long-name'
]) {
  for (const scheme of [
    'light',
    'dark'
  ]) for (const scale of [
    1,
    2
  ]) {
    expected.push(`comparison/${panel}/${scene}/${scheme}/${scale}/initial`);
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
  if (!/^track-menu-(?:inner|cover)-[a-z-]+-(?:light|dark)-s(?:100|200)\.png$/.test(file)) {
    throw new Error('Track-menu image path is outside the evidence boundary.');
  }
  if (file !== `track-menu-${panel}-${scene}-${scheme}-s${fontScale * 100}.png`) {
    throw new Error('Track-menu filename and metadata disagree.');
  }
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
    throw new Error('Track-menu crop must follow its measured application bounds.');
  }
  const height = physicalHeight - bounds[1];
  const png = readFileSync(join(
    evidence,
    file
  ));
  const hash = createHash('sha256')
    .update(png)
    .digest('hex');
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (png.readUInt32BE(16) !== width)
    || (png.readUInt32BE(20) !== height)
    || (png[24] !== 8)
    || (png[25] !== 2)
    ||
    (capture.cropPixels
      .width
      !== width)
    || (capture.cropPixels
      .height
      !== height)
    || (capture.densityDpi !== 390)
    ||
    (capture.inspected !== true)
    || (capture.freshHierarchyValidated !== true)
    || (capture.keyboardClosed !== true)
    ||
    (capture.nativeLongPress !== true)
    || (capture.allActionsInitiallyVisible !== true)
    ||
    (capture.actionLayoutFloorVerified !== true)
    || (capture.heading
      .lines
      !== 1)
    ||
    (capture.heading
      .overflow
      !== (scene === 'long-name'))
    || (capture.targetIndex !== (scene === 'lower' ? 8 : 1))
    ||
    (capture.systemImageFingerprint !== 'google/sdk_gphone16k_x86_64/emu64xa16k:17/CE2A.260420.050/16231978:user/dev-keys')
    ||
    (capture.renderer !== 'Android Emulator OpenGL ES Translator (llvmpipe (LLVM 20.1.2, 256 bits))')) {
    throw new Error('Track-menu image, native input, heading or acquisition assertion differs.');
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
    ].includes(chunk)) throw new Error('Track-menu PNG contains unintended metadata.');
    offset += length + 12;
  }
  const key = `comparison/${panel}/${scene}/${scheme}/${fontScale}/initial`;
  if (images[key]) throw new Error('Duplicate track-menu witness.');
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
  throw new Error('Track-menu review requires exact panel, scene, theme and scale combinations.');
}
//endregion

//region Offline review preserves the accepted menu and keeps observations optional
const template = readFileSync(
  join(
    questions,
    'track-menu.template.html'
  ),
  'utf8'
);
if (template.split('__TRACK_MENU_IMAGES__')
  .length
  !== 2) throw new Error('Expected one track-menu image slot.');
const html = template.replace(
  '__TRACK_MENU_IMAGES__',
  () => JSON.stringify(images)
    .replaceAll(
    '<',
    String.raw`\u003c`
    )
);
const output = join(
  questions,
  'track-menu.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log('Built offline native track-menu evidence from exact inspected witnesses.');
} else if (process.argv[2] === 'validate') {
  if (readFileSync(
    output,
    'utf8'
  ) !== html) throw new Error('Track-menu output differs from its template and evidence.');
  for (const marker of [
    'color-scheme: light dark',
    'D7 is settled; no new preference ballot',
    'Every track and operation is authored debug input',
    'single-line ellipsis',
    'intent only',
    'No production implementation is authorized',
    'id="final-notes"',
    'Native pixels',
    'Reset 100% dp'
  ]) {
    if (!html.includes(marker)) throw new Error(`Track-menu review is missing ${  marker}`);
  }
  if (((html.match(/<form\b/g) ?? []).length !== 1) ||
    /__TRACK_MENU_IMAGES__|<script\b[^>]*\bsrc=|<link\b[^>]*\bhref=|<img\b[^>]*\bsrc="https?:/i.test(html)
    ||
    /<input\b[^>]*\btype="radio"|<textarea\b(?:[^"'<>]|"[^"]*"|'[^']*')*?\srequired(?:\s|=|>)/i.test(html)) {
    throw new Error('Track-menu review must remain offline evidence, not a policy ballot.');
  }
  console.log('Validated exact offline native track-menu evidence and operation limits.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
