import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

// Embed only the sanitized aligned native evidence, never raw status-bearing captures.
const root = process.cwd();
const render = join(
  root,
  'questions',
  'render'
);
const templatePath = join(
  root,
  'questions',
  'archive',
  'search-ranking-before-dm.template.html'
);
const artifactPath = join(
  root,
  'questions',
  'archive',
  'search-ranking-before-dm.html'
);
const captures = Object.fromEntries([
  'inner',
  'cover'
].map(panel => [
  panel,
  Object.fromEntries([
    'rankmixed',
    'rankfolders',
    'ranktracks'
  ].map(rank => [
    rank,
    Object.fromEntries([
      'direct',
      'parenthits'
    ].map(scope => [
      scope,
      Object.fromEntries([
        'top',
        'end'
      ].map(position => {
        const suffix = position === 'end' ? '-end' : '';
        const name = `search-rank-accent-review-${panel}-${rank}-${scope}${suffix}-s200.png`;
        const png = readFileSync(join(
          render,
          name
        ));
        const dimensions = panel === 'inner' ? [
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
          (png.readUInt32BE(16) !== dimensions[0])
          || (png.readUInt32BE(20) !== dimensions[1])) {
          throw new Error(`${name}: unexpected physical Fold panel or non-PNG evidence`);
        }
        return [
          position,
          `data:image/png;base64,${png.toString('base64')}`
        ];
      }))
    ]))
  ]))
]));
const highlightCaptures = Object.fromEntries([
  'inner',
  'cover'
].map(panel => [
  panel,
  Object.fromEntries([
    'light',
    'dark'
  ].map(scheme => {
    const name = `search-selected-accent-review-${panel}-results-${scheme}-s200.png`;
    const png = readFileSync(join(
      render,
      name
    ));
    const dimensions = panel === 'inner' ? [
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
      (png.readUInt32BE(16) !== dimensions[0])
      || (png.readUInt32BE(20) !== dimensions[1])) {
      throw new Error(`${name}: unexpected selected Search highlight panel`);
    }
    return [
      scheme,
      `data:image/png;base64,${png.toString('base64')}`
    ];
  }))
]));
const command = process.argv[2];
if (command === 'build') {
  const template = readFileSync(
    templatePath,
    'utf8'
  );
  if ((template.split('__RANK_CAPTURE_MAP__')
    .length
    !== 2) ||
    (template.split('__HIGHLIGHT_CAPTURE_MAP__')
      .length
      !== 2)) {
    throw new Error('The ranking form requires one ranking and one highlight capture slot.');
  }
  writeFileSync(
    artifactPath,
    template.replace(
      '__RANK_CAPTURE_MAP__',
      JSON.stringify(captures)
    )
      .replace(
      '__HIGHLIGHT_CAPTURE_MAP__',
      JSON.stringify(highlightCaptures)
      )
  );
  console.log(`Built the historical D/P and M/F/T Search comparison in ${artifactPath}.`);
} else if (command === 'validate') {
  const html = readFileSync(
    artifactPath,
    'utf8'
  );
  const begin = html.indexOf('const captures = ');
  const end = html.indexOf(
    ';\nconst highlightCaptures = ',
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
    throw new Error('Embedded ranking rasters differ from the sanitized aligned Fold sources.');
  }
  const highlightBegin = html.indexOf('const highlightCaptures = ');
  const highlightEnd = html.indexOf(
    ';\nconst scopeInputs = ',
    highlightBegin
  );
  if ((highlightBegin === (-1)) || (highlightEnd === (-1))
    ||
    (JSON.stringify(
      JSON.parse(html.slice(
        highlightBegin + 'const highlightCaptures = '.length,
        highlightEnd
      )),
    ) !== JSON.stringify(highlightCaptures))) {
    throw new Error('Embedded light/dark OS-accent highlights differ from sanitized native sources.');
  }
  for (const required of [
    'D58',
    'D59',
    'OKLCH',
    'Scope D',
    'Scope P',
    'Order M',
    'Order F',
    'Order T',
    'Ranking: D &gt; P',
    'Ranking: M &gt; F &gt; T',
    'exact',
    'Reset 100%',
    'D/M selected',
    'Historical',
    'data-preview="inner"',
    'data-preview="cover"'
  ]) {
    if (!html.includes(required)) throw new Error(`Ranking review missing ${required}`);
  }
  if (html.includes('__RANK_CAPTURE_MAP__') || html.includes('__HIGHLIGHT_CAPTURE_MAP__')
    ||
    html.includes('<script src=')
    || html.includes('<link rel="stylesheet"')) {
    throw new Error('Ranking form is not self-contained.');
  }
  console.log('Validated archived native ranking evidence and recorded D/M selection.');
} else {
  throw new Error('Expected build or validate.');
}
