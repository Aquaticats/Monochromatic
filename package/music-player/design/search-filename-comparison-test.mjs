import { spawnSync } from 'node:child_process';
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import { mkdirSync as makeScratchBase } from 'node:fs';

// Scratch lives inside the package's gitignored node_modules/.monochromatic, never the system temporary directory,
// whose backing device is unknowable.
const scratchBase = join(import.meta.dirname, 'node_modules', '.monochromatic', 'music-player-design');
makeScratchBase(scratchBase, { recursive: true });


//region Disposable consumer fixture, never mutate committed evidence while proving validators
const root = process.cwd();
const fixture = mkdtempSync(join(
  scratchBase,
  'filename-review-'
));
const evidence = join(
  fixture,
  'questions',
  'evidence'
);
mkdirSync(
  evidence,
  { recursive: true }
);
const manifestFile = 'search-filename-comparison-witnesses.json';
const comparison = JSON.parse(readFileSync(
  join(
    root,
    'questions',
    'evidence',
    manifestFile
  ),
  'utf8'
));
const baselineFile = 'search-filename-actual-player-witnesses.json';
const baseline = JSON.parse(readFileSync(
  join(
    root,
    'questions',
    'evidence',
    baselineFile
  ),
  'utf8'
));
for (const file of [
  manifestFile,
  baselineFile,
  ...comparison.witnesses
    .map(item => item.file),
  ...baseline.captures
    .map(item => item.image)
]) {
  copyFileSync(
    join(
      root,
      'questions',
      'evidence',
      file
    ),
    join(
      evidence,
      file
    )
  );
}
copyFileSync(
  join(
    root,
    'questions',
    'search-filename-comparison.template.html'
  ),
  join(
    fixture,
    'questions',
    'search-filename-comparison.template.html'
  )
);
const builderPath = join(
  fixture,
  'search-filename-comparison.mjs'
);
copyFileSync(
  join(
    root,
    'search-filename-comparison.mjs'
  ),
  builderPath
);
const mutation = process.argv[2];
if ((mutation !== undefined) && (![
  'without-exact-cohort',
  'without-evidence-only'
].includes(mutation))) {
  throw new Error('Unknown filename review test mutation.');
}
if (mutation === 'without-exact-cohort') {
  const guard = `if (JSON.stringify(Object.keys(images).sort()) !== JSON.stringify(expectedKeys.sort())) {\n  throw new Error('Filename review must contain the exact initial, baseline and six scrolled combinations.');\n}`;
  const source = readFileSync(
    builderPath,
    'utf8'
  );
  if (source.split(guard)
    .length
    !== 2) throw new Error('Exact-cohort mutation target differs.');
  writeFileSync(
    builderPath,
    source.replace(
      guard,
      ''
    )
  );
}
if (mutation === 'without-evidence-only') {
  const source = readFileSync(
    builderPath,
    'utf8'
  );
  const diagnostic = source.indexOf('    throw new Error(\'Filename evidence review cannot contain policy votes');
  const start = source.lastIndexOf(
    '\n  if (',
    diagnostic
  );
  const end = source.indexOf(
    '\n  console.log(\'Validated exact offline evidence review',
    diagnostic
  );
  if ((start === (-1)) || (end === (-1))) throw new Error('Evidence-only mutation target differs.');
  writeFileSync(
    builderPath,
    source.slice(
      0,
      start
    ) + source.slice(end)
  );
}
function invoke(command) {
  return spawnSync(
    process.execPath,
    [
      join(
        fixture,
        'search-filename-comparison.mjs'
      ),
      command
    ],
    {
      cwd: fixture,
      encoding: 'utf8',
      maxBuffer: 2 * 1_024
        * 1_024,
    }
  );
}
function writeComparison(data) { writeFileSync(
  join(
    evidence,
    manifestFile
  ),
  JSON.stringify(data)
); }
function expectRejection({
  data,
  diagnostic
}) {
  writeComparison(data);
  const result = invoke('build');
  if ((result.status === 0) || (!result.stderr
    .includes(diagnostic))) {
    throw new Error(`Expected filename review rejection absent: ${diagnostic}; status ${result.status}.`);
  }
  writeComparison(comparison);
}
function expectTemplateRejection({
  original,
  changed,
  diagnostic
}) {
  const path = join(
    fixture,
    'questions',
    'search-filename-comparison.template.html'
  );
  const template = readFileSync(
    path,
    'utf8'
  );
  if (!template.includes(original)) throw new Error('Template test target is absent.');
  writeFileSync(
    path,
    template.replace(
      original,
      changed
    )
  );
  if (invoke('build')
    .status
    !== 0) throw new Error('Changed template could not build for validation.');
  const result = invoke('validate');
  if ((result.status === 0) || (!result.stderr
    .includes(diagnostic))) {
    throw new Error(`Expected evidence-only review rejection absent: ${diagnostic}; status ${result.status}.`);
  }
  writeFileSync(
    path,
    template
  );
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      !== 0)) throw new Error('Restored template failed.');
}
try {
  const positive = invoke('build');
  if ((positive.status !== 0) || (invoke('validate')
    .status
    !== 0)) throw new Error('Disposable positive review failed.');
  const quotedPath = join(
    fixture,
    'questions',
    'search-filename-comparison.template.html'
  );
  const quotedOriginal = readFileSync(
    quotedPath,
    'utf8'
  );
  writeFileSync(
    quotedPath,
    quotedOriginal.replace(
      'id="final-notes" name="notes"',
      'id="final-notes" name="notes" data-example="required now"'
    )
  );
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      !== 0)) throw new Error('Quoted required word misread as attribute.');
  writeFileSync(
    quotedPath,
    quotedOriginal
  );
  if (invoke('build')
    .status
    !== 0) throw new Error('Quoted-value positive control did not restore.');
  const traversal = structuredClone(comparison);
  traversal.witnesses[0]
    .file = '../outside.png';
  expectRejection({
    data: traversal,
    diagnostic: 'outside the allowed evidence boundary'
  });
  // The digest case that used to reach this condition is gone with D88; a wrong recorded width reaches it directly.
  const wrongWidth = structuredClone(comparison);
  wrongWidth.witnesses[0]
    .cropPixels
    .width += 1;
  expectRejection({
    data: wrongWidth,
    diagnostic: 'filename review capture dimension or environment failed'
  });
  const mismatchedName = structuredClone(comparison);
  mismatchedName.witnesses[0]
    .scene = 'another-scene';
  expectRejection({
    data: mismatchedName,
    diagnostic: 'filename and capture metadata disagree'
  });
  const missingScroll = structuredClone(comparison);
  const target = missingScroll.witnesses
    .find(item => item.position === 'scrolled');
  const original = target.file;
  target.scene = 'unusedscroll';
  target.file = `search-filename-comparison-${target.panel}-unusedscroll-end-${target.scheme}-s200.png`;
  copyFileSync(
    join(
      evidence,
      original
    ),
    join(
      evidence,
      target.file
    )
  );
  expectRejection({
    data: missingScroll,
    diagnostic: 'exact initial, baseline and six scrolled combinations'
  });
  const evidenceDiagnostic = 'cannot contain policy votes or required observations';
  expectTemplateRejection({
    original: '<form id="review-form">',
    changed: '<form id="review-form"><input type="radio" name="placement">',
    diagnostic: evidenceDiagnostic
  });
  expectTemplateRejection({
    original: 'id="final-notes" name="notes"',
    changed: 'id="final-notes" name="notes" required',
    diagnostic: evidenceDiagnostic
  });
  expectTemplateRejection({
    original: "  const notes = String(data.get('notes')).trim();",
    changed: "  data.get('visibility');\n  const notes = String(data.get('notes')).trim();",
    diagnostic: evidenceDiagnostic
  });
  expectTemplateRejection({
    original: 'Supporting text is user-configurable through templates in Settings',
    changed: 'Fixed supporting text',
    diagnostic: 'missing Supporting text is user-configurable through templates in Settings'
  });
  const output = join(
    fixture,
    'questions',
    'search-filename-comparison.html'
  );
  writeFileSync(
    output,
    readFileSync(
      output,
      'utf8'
    )
      .replace(
      'Fixed-policy question withdrawn.',
      'Changed review.'
      )
  );
  const changed = invoke('validate');
  if ((changed.status === 0) || (!changed.stderr
    .includes('differs from template and checked evidence'))) {
    throw new Error('Changed output was not rejected.');
  }
  console.log('Filename review disposable positive, traversal, dimension, metadata, exact-scroll and changed-output checks passed.');
} finally {
  rmSync(
    fixture,
    {
      recursive: true,
      force: true
    }
  );
}
//endregion
