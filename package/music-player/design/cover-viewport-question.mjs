import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

/** Embed only checked, sanitized physical-panel captures in the self-contained design question. */
const root = process.cwd();
const question = join(
  root,
  'questions'
);
const template = join(
  question,
  'cover-viewport.template.html'
);
const output = join(
  question,
  'cover-viewport-review.html'
);
const images = {
  __CONTROL_IMAGE__: 'search-cover-viewport-control-s200.png',
  __REFINEMENT_IMAGE__: 'search-cover-viewport-refinement-s200.png',
};
/** Verify a public capture's panel geometry and lack of text-bearing metadata before embedding. */
function imageData(name) {
  const bytes = readFileSync(join(
    question,
    'render',
    name
  ));
  if ((bytes.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (bytes.readUInt32BE(16) !== 1_080)
    || (bytes.readUInt32BE(20) !== 2_424)) {
    throw new Error(`${name}: expected a sanitized physical cover PNG.`);
  }
  const chunks = [];
  for (let offset = 8; offset < bytes.length;) {
    const length = bytes.readUInt32BE(offset);
    const type = bytes.toString(
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
    throw new Error(`${name}: metadata would enter the review.`);
  }
  return `data:image/png;base64,${bytes.toString('base64')}`;
}
/** Construct the one review artifact from the accepted A boundary and two viewport variants. */
function expectedHtml() {
  return Object.entries(images)
    .reduce(
    (
      html,
      [marker, name]
    ) => {
      if (html.split(marker)
        .length
        !== 2) throw new Error(`${marker}: missing or repeated slot.`);
      return html.replace(
        marker,
        imageData(name)
      );
    },
    readFileSync(
      template,
      'utf8'
    )
    );
}
const action = process.argv[2];
if (action === 'build') {
  writeFileSync(
    output,
    expectedHtml()
  );
  console.log(`Built ${output}`);
} else if (action === 'validate') {
  const rendered = readFileSync(
    output,
    'utf8'
  );
  if (rendered !== expectedHtml()) throw new Error('Review differs from current public sanitized captures.');
  for (const marker of [
    'Same-fixture end-of-list comparison',
    'R &gt; C',
    'not Gboard',
    'Response path:',
    'name="coverViewport"',
    'Anything else you want settled or changed?',
    'Reset 100%',
    'prefers-color-scheme: dark',
    'The form only prepares text'
  ]) {
    if (!rendered.includes(marker)) throw new Error(`Missing review contract: ${marker}`);
  }
  if (rendered.includes('<script src=') || rendered.includes('<link rel="stylesheet"')) {
    throw new Error('Review must be self-contained.');
  }
  console.log('Validated two sanitized physical-panel captures and self-contained cover question.');
} else {
  throw new Error('Expected build or validate.');
}
