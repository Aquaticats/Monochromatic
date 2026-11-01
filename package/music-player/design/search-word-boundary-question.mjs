import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

// Embed only status-sanitized native Fold frames; leave raw capture metadata in private scratch.
const root = process.cwd();
const render = join(
  root,
  'questions',
  'render'
);
const templateFile = join(
  root,
  'questions',
  'archive',
  'search-word-boundary-deferred.template.html'
);
const outputFile = join(
  root,
  'questions',
  'archive',
  'search-word-boundary-deferred.html'
);
const captures = Object.fromEntries([
  'inner',
  'cover'
].map(panel => [
  panel,
  Object.fromEntries([
    'rankword',
    'rankany'
  ].map(variant => {
    const name = `search-word-boundary-review-${panel}-${variant}-s200.png`;
    const png = readFileSync(join(
      render,
      name
    ));
    const size = panel === 'inner' ? [
      2_076,
      2_152
    ] : [
      1_080,
      2_424
    ];
    if ((png.subarray(
      0,
      8
    )
      .toString('hex')
      !== '89504e470d0a1a0a') ||
      (png.readUInt32BE(16) !== size[0])
      || (png.readUInt32BE(20) !== size[1])) {
      throw new Error(`${name}: wrong physical Fold panel or non-PNG evidence`);
    }
    return [
      variant,
      `data:image/png;base64,${png.toString('base64')}`
    ];
  }))
]));
const command = process.argv[2];
if (command === 'build') {
  const template = readFileSync(
    templateFile,
    'utf8'
  );
  if (template.split('__WORD_BOUNDARY_CAPTURES__')
    .length
    !== 2) {
    throw new Error('Expected one embedded word-boundary capture slot.');
  }
  writeFileSync(
    outputFile,
    template.replace(
      '__WORD_BOUNDARY_CAPTURES__',
      JSON.stringify(captures)
    )
  );
  console.log(`Built the archived unselected word-boundary exploration in ${outputFile}.`);
} else if (command === 'validate') {
  const html = readFileSync(
    outputFile,
    'utf8'
  );
  const begin = html.indexOf('const captures = ');
  const end = html.indexOf(
    ';\nconst controls = ',
    begin
  );
  if ((begin === (-1)) || (end === (-1))
    ||
    (JSON.stringify(
      JSON.parse(html.slice(
        begin + 'const captures = '.length,
        end
      )),
    ) !== JSON.stringify(captures))) {
    throw new Error('Embedded word-boundary frames differ from native sanitized evidence.');
  }
  for (const marker of [
    'D59',
    'D60',
    'D61',
    'D62',
    'Scamper',
    'Dreamcam',
    'Reset 100%',
    'not a current question',
    'data-variant="rankword"',
    'data-variant="rankany"',
    'data-panel="inner"',
    'data-panel="cover"'
  ]) {
    if (!html.includes(marker)) throw new Error(`Word-boundary review missing ${marker}`);
  }
  if (html.includes('__WORD_BOUNDARY_CAPTURES__') || html.includes('<script src=')
    ||
    html.includes('<link rel="stylesheet"')) {
    throw new Error('Word-boundary review must be self-contained.');
  }
  console.log('Validated archived native word-boundary exploration without an active choice.');
} else {
  throw new Error('Expected build or validate.');
}
