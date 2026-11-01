import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

/** Embed the verified, sanitized inner-panel capture into an offline design question. */
const root = process.cwd();
const question = join(
  root,
  'questions'
);
const templateFile = join(
  question,
  'floating-results.template.html'
);
const outputFile = join(
  question,
  'floating-results-review.html'
);
const capture = readFileSync(join(
  question,
  'render',
  'search-floating-inner-results-light-s200.png'
));
if ((capture.subarray(
  0,
  8
)
  .toString('hex')
  !== '89504e470d0a1a0a') ||
  (capture.readUInt32BE(16) !== 2_076)
  || (capture.readUInt32BE(20) !== 2_152)) {
  throw new Error('Floating result capture must use the physical inner-panel resolution.');
}
const chunks = [];
for (let offset = 8; offset < capture.length;) {
  const length = capture.readUInt32BE(offset);
  const type = capture.toString(
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
  throw new Error('Floating capture contains private metadata.');
}
const embedded = `data:image/png;base64,${capture.toString('base64')}`;
const template = readFileSync(
  templateFile,
  'utf8'
);
if (template.split('__FLOATING_IMAGE__')
  .length
  !== 2) throw new Error('Capture slot must occur once.');
const expected = template.replace(
  '__FLOATING_IMAGE__',
  embedded
);
const action = process.argv[2];
if (action === 'build') {
  writeFileSync(
    outputFile,
    expected
  );
  console.log(`Built ${outputFile}`);
} else if (action === 'validate') {
  const actual = readFileSync(
    outputFile,
    'utf8'
  );
  if (actual !== expected) throw new Error('Review differs from the public sanitized physical-panel capture.');
  for (const marker of [
    'A &gt; B',
    'not a built screen',
    'name="floatingResults"',
    'Anything else you want settled or changed?',
    'Response path:',
    'Reset 100%',
    'prefers-color-scheme: dark',
    'new IME experiments'
  ]) {
    if (!actual.includes(marker)) throw new Error(`Missing floating-results review contract: ${marker}`);
  }
  if (actual.includes('<script src=') || actual.includes('<link rel="stylesheet"')) {
    throw new Error('Review must not depend on external resources.');
  }
  console.log('Validated self-contained floating result exception question.');
} else {
  throw new Error('Expected build or validate.');
}
