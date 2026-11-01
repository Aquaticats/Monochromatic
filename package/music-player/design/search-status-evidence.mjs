import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

/** Embed only the manifest's sanitized captures whose pixel size is their physical panel. */
const root = process.cwd();
const question = join(
  root,
  'questions'
);
const templatePath = join(
  question,
  'search-status-evidence.template.html'
);
const outputPath = join(
  question,
  'search-status-evidence.html'
);
const manifest = JSON.parse(readFileSync(
  join(
    question,
    'evidence',
    'search-status-review-manifest.json'
  ),
  'utf8'
));
if ((manifest.disposableAvd !== 'Fold_No_Hardware_Probe') ||
  (!manifest.keyboardClosedOnly)
  || (!manifest.statusStripGeneric)
  || (manifest.captures
    .length
    !== 24)) {
  throw new Error('Static Search evidence provenance is missing or outside the disposable scope.');
}
const captures = {};
for (const item of manifest.captures) {
  const png = readFileSync(join(
    question,
    item.file
  ));
  const expected = item.panel === 'inner' ? [
    2_076,
    2_152,
    136
  ] : [
    1_080,
    2_424,
    151
  ];
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (png.readUInt32BE(16) !== expected[0])
    || (png.readUInt32BE(20) !== expected[1])
    ||
    (item.physicalPixels
      .join('x')
      !== expected.slice(
        0,
        2
      )
        .join('x'))
    ||
    (item.statusMaskedTopPixels !== expected[2])
    ||
    (![
      'empty',
      'none',
      'unavailable'
    ].includes(item.state))
    ||
    (![
      'light',
      'dark'
    ].includes(item.scheme))
    ||
    (![
      1,
      2
    ].includes(item.fontScale))) {
    throw new Error(`${item.file}: static status evidence failed its physical-panel check.`);
  }
  captures[item.panel] ??= {};
  captures[item.panel][item.state] ??= {};
  captures[item.panel][item.state][item.scheme] ??= {};
  const key = String(item.fontScale * 100);
  if (captures[item.panel][item.state][item.scheme][key]) {
    throw new Error(`${item.file}: duplicate static status capture combination.`);
  }
  captures[item.panel][item.state][item.scheme][key] = `data:image/png;base64,${png.toString('base64')}`;
}
for (const panel of [
  'inner',
  'cover'
]) {
  for (const state of [
    'empty',
    'none',
    'unavailable'
  ]) {
    for (const scheme of [
      'light',
      'dark'
    ]) {
      for (const scale of [
        '100',
        '200'
      ]) {
        if (!captures[panel]?.[state]?.[scheme]?.[scale]) {
          throw new Error(`${panel}/${state}/${scheme}/${scale}: missing static status capture.`);
        }
      }
    }
  }
}
const command = process.argv[2];
const template = readFileSync(
  templatePath,
  'utf8'
);
if (template.split('__SEARCH_STATUS_CAPTURE_MAP__')
  .length
  !== 2) {
  throw new Error('Static Search evidence template must have exactly one capture-map slot.');
}
if (command === 'build') {
  writeFileSync(
    outputPath,
    template.replace(
      '__SEARCH_STATUS_CAPTURE_MAP__',
      JSON.stringify(captures)
    )
  );
  console.log('Built self-contained selected-layout Search status evidence.');
} else if (command === 'validate') {
  const html = readFileSync(
    outputPath,
    'utf8'
  );
  const begin = html.indexOf('const captures = ');
  const end = html.indexOf(
    ';\nconst state = ',
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
    throw new Error('Embedded static statuses differ from the checked sanitized captures.');
  }
  for (const marker of [
    'selected Search A review',
    'No query entered',
    'Nonmatching query: zzq',
    'Fixed unavailable marker',
    'Source 2076 × 2152',
    'Source 1080 × 2424',
    'color-scheme: light dark',
    'Reset 100%',
    'data-preview="inner"',
    'data-preview="cover"',
    'not a preference questionnaire',
    'reply in this chat'
  ]) {
    if (!html.includes(marker)) throw new Error(`Static Search evidence is missing ${marker}.`);
  }
  if (html.includes('__SEARCH_STATUS_CAPTURE_MAP__') || html.includes('<script src=')
    ||
    html.includes('<link rel="stylesheet"')) {
    throw new Error('Static Search evidence has an unresolved slot or external code dependency.');
  }
  console.log('Validated the self-contained static Search status evidence.');
} else {
  throw new Error('Expected build or validate.');
}
