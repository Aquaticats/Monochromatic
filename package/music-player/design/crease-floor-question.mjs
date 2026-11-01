import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

/** Embed each separately rendered native Compose variant and state in one offline form. */
const root = process.cwd();
const question = join(
  root,
  'questions'
);
const template = join(
  question,
  'crease-floor.template.html'
);
const output = join(
  question,
  'crease-floor-review.html'
);
const installedApkSha256 = 'd301bebdd35d053ee63cbd9a8500949fb97b8be5dc26c9bb971be1493a3462b5';
/** Check each public capture's physical panel and remove any metadata-bearing source. */
function captureData({
  floor,
  state
}) {
  const name = `search-e2-complete-${floor}-${state}-inner-light-s200.png`;
  const png = readFileSync(join(
    question,
    'render',
    name
  ));
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (png.readUInt32BE(16) !== 2_076)
    || (png.readUInt32BE(20) !== 2_152)) {
    throw new Error(`${name}: expected the native 2076 x 2152 inner panel.`);
  }
  const chunks = [];
  for (let offset = 8; offset < png.length;) {
    const length = png.readUInt32BE(offset);
    const type = png.toString(
      'ascii',
      offset + 4,
      offset + 8
    );
    chunks.push(type);
    offset += length + 12;
    if (type === 'IEND') break;
  }
  if (chunks.some(type => [
    'eXIf',
    'iTXt',
    'tEXt',
    'zTXt',
    'iCCP',
    'tIME'
  ].includes(type))) {
    throw new Error(`${name}: unexpected text, profile or timestamp metadata.`);
  }
  return `data:image/png;base64,${png.toString('base64')}`;
}
/** Build one map of independent APK-captured states without an external image request. */
function expectedHtml() {
  const captures = Object.fromEntries([
    '7p5',
    '14',
    '20'
  ].map(floor => [
    floor,
    Object.fromEntries([
      'player',
      'empty',
      'results'
    ].map(state => [
      state,
      captureData({
        floor,
        state
      })
    ]))
  ]));
  const source = readFileSync(
    template,
    'utf8'
  );
  if ((source.split('__E2_CAPTURE_MAP__')
    .length
    !== 2) ||
    (source.split('__E2_APK_SHA__')
      .length
      !== 2)) {
    throw new Error('Native E2 form must have one capture slot and one APK provenance slot.');
  }
  return source.replace(
    '__E2_CAPTURE_MAP__',
    JSON.stringify(captures)
  )
    .replace(
    '__E2_APK_SHA__',
    installedApkSha256
    );
}
const action = process.argv[2];
if (action === 'build') {
  writeFileSync(
    output,
    expectedHtml()
  );
  console.log(`Built nine native player/Search E2 states in ${output}`);
} else if (action === 'validate') {
  const actual = readFileSync(
    output,
    'utf8'
  );
  if (actual !== expectedHtml()) throw new Error('E2 review differs from its six sanitized native captures.');
  for (const marker of [
    'P7.5 &gt; P14 &gt; P20',
    'Search closed',
    'data-preview="7p5"',
    'data-preview="14"',
    'data-preview="20"',
    'name="floor"',
    'Anything else you want settled or changed?',
    'Response path:',
    'Reset 100%',
    'prefers-color-scheme: dark',
    installedApkSha256
  ]) {
    if (!actual.includes(marker)) throw new Error(`Missing native E2 contract: ${marker}`);
  }
  if (actual.includes('<script src=') || actual.includes('<link rel="stylesheet"')) {
    throw new Error('E2 review must be self-contained.');
  }
  console.log('Validated nine independent physical-panel player/Search screenshots and the E2 form.');
} else {
  throw new Error('Expected build or validate.');
}
