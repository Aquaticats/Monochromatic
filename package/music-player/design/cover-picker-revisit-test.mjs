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


//region Disposable consumer inputs, committed native evidence is never mutated
const root = process.cwd();
const fixture = mkdtempSync(join(
  scratchBase,
  'cover-picker-review-'
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
const manifestFile = 'cover-picker-revisit-witnesses.json';
const manifest = JSON.parse(readFileSync(
  join(
    root,
    'questions',
    'evidence',
    manifestFile
  ),
  'utf8'
));
for (const file of [
  manifestFile,
  ...manifest.witnesses
    .map(item => item.file)
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
const templatePath = join(
  fixture,
  'questions',
  'cover-picker-revisit.template.html'
);
copyFileSync(
  join(
    root,
    'questions',
    'cover-picker-revisit.template.html'
  ),
  templatePath
);
const builder = join(
  fixture,
  'cover-picker-revisit.mjs'
);
copyFileSync(
  join(
    root,
    'cover-picker-revisit.mjs'
  ),
  builder
);
// Each named guard can be deleted from the disposable builder; the cases that prove it must then fail this test.
const guards = {
  'without-title-position': `open.some((value, index) => Math.abs(value - closed[index]) > 2)`,
  'without-cohort': `if (!views[\`comparison/\${variant}/\${scheme}/\${scale}\`]) throw new Error(\`Cover picker review requires every candidate under every condition: \${variant}/\${scheme}/\${scale}\`);`,
  'without-drawn-text': `expectedTexts.some(text => !capture.texts.some(drawn => drawn.includes(text)))`,
  'without-visit-count': "!html.includes(`${visits} ${visits === 1 ? 'visit' : 'visits'} contributed views`)",
};
const removed = process.argv[2];
if (removed !== undefined) {
  if (!(removed in guards)) throw new Error('Unknown cover picker review test mutation.');
  const source = readFileSync(
    builder,
    'utf8'
  );
  if (source.split(guards[removed])
    .length
    !== 2) throw new Error(`Expected cover picker guard absent: ${  removed}`);
  writeFileSync(
    builder,
    source.replace(
      guards[removed],
      () => removed === 'without-cohort' ? '' : 'false'
    )
  );
}
function invoke(command) {
  return spawnSync(
    process.execPath,
    [
      builder,
      command
    ],
    {
      cwd: fixture,
      encoding: 'utf8',
      maxBuffer: 2 * 1_024
        * 1_024
    }
  );
}
let rejected = 0;
function reject({
  change,
  diagnostic
}) {
  const input = structuredClone(manifest);
  change(input);
  writeFileSync(
    join(
      evidence,
      manifestFile
    ),
    JSON.stringify(input)
  );
  const run = invoke('build');
  if ((run.status === 0) || (!run.stderr
    .includes(diagnostic))) {
    throw new Error(`Expected cover picker review rejection absent: ${diagnostic}; status ${run.status}.`);
  }
  writeFileSync(
    join(
      evidence,
      manifestFile
    ),
    JSON.stringify(manifest)
  );
  rejected += 1;
}
const first = manifest.witnesses
  .findIndex(item => (item.variant === 'p5') && (item.scheme === 'dark')
    && (item.fontScale === 1));
if (first === (-1)) throw new Error('Required positive fixture absent.');
try {
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      !== 0)) throw new Error('Positive cover picker consumer failed.');
  reject({
    change: input => { input.witnesses[0]
      .file = '../outside.png'; },
    diagnostic: 'outside the allowed evidence boundary'
  });
  reject({
    change: input => { input.witnesses[0]
      .variant = input.witnesses[0]
        .variant
        === 'p4' ? 'p5' : 'p4'; },
    diagnostic: 'filename and metadata disagree'
  });
  reject({
    change: input => { input.witnesses[0]
      .keyboardClosed = false; },
    diagnostic: 'image geometry or acquisition assertion differs'
  });
  reject({
    change: input => { input.witnesses[0]
      .cropPixels
      .y += 1; },
    diagnostic: 'image geometry or acquisition assertion differs'
  });
  reject({
    change: input => { input.witnesses[0]
      .texts = input.witnesses[0]
        .texts
        .filter(text => !text.includes('C418')); },
    diagnostic: 'lacks the trigger, the Open action or the first folder'
  });
  reject({
    change: input => { input.witnesses
      .splice(
      first,
      1
      ); },
    diagnostic: 'requires every candidate under every condition'
  });
  reject({
    change: input => { input.witnesses[first]
      .titleBounds = [
        0,
        0,
        1,
        1
      ]; },
    diagnostic: 'title moves between P4 and P5'
  });
  reject({
    change: input => { delete input.witnesses[first]
      .titleBounds; },
    diagnostic: 'title moves between P4 and P5'
  });
  reject({
    change: input => { input.schema = 2; },
    diagnostic: 'not the expected study'
  });
  if (invoke('build')
    .status
    !== 0) throw new Error('Restored evidence build failed.');
  const template = readFileSync(
    templatePath,
    'utf8'
  );
  writeFileSync(
    templatePath,
    template.replace(
      '<form id="review-form">',
      '<form id="review-form"><input type="radio" name="design">'
    )
  );
  if (invoke('build')
    .status
    !== 0) throw new Error('Ballot fixture build failed.');
  const ballot = invoke('validate');
  if ((ballot.status === 0) || (!ballot.stderr
    .includes('not a ballot'))) throw new Error('Ballot was not rejected.');
  // Pages that keep another cohort's story, show a figure that is no candidate, omit one, or lose the ranking.
  let refusedPages = 0;
  function refusedPage({
    change,
    diagnostic
  }) {
    const changed = change(template);
    if (changed === template) throw new Error(`Template mutation changed nothing: ${  diagnostic}`);
    writeFileSync(
      templatePath,
      changed
    );
    if (invoke('build')
      .status
      !== 0) throw new Error(`Mutated template did not build: ${  diagnostic}`);
    const run = invoke('validate');
    if ((run.status === 0) || (!run.stderr
      .includes(diagnostic))) throw new Error(`Mutated page was not rejected: ${  diagnostic}`);
    refusedPages += 1;
  }
  refusedPage({
    change: page => page.replace(
      /\d+ visits? contributed views/u,
      '99 visits contributed views'
    ),
    diagnostic: 'does not name its'
  });
  refusedPage({
    change: page => page.replace(
      '<div class="gallery">',
      '<div class="gallery"><figure data-variant="p9"></figure>'
    ),
    diagnostic: 'which is not a candidate'
  });
  refusedPage({
    change: page => page.replace(
      '<figure data-variant="p5">',
      '<figure data-variant="p4">'
    ),
    diagnostic: 'shows no single figure for p4'
  });
  refusedPage({
    change: page => page.replace(
      'Ranking: P5 &gt; P4',
      'Order'
    ),
    diagnostic: 'is missing Ranking'
  });
  writeFileSync(
    templatePath,
    template
  );
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      !== 0)) throw new Error('Restored template did not validate.');
  const output = join(
    fixture,
    'questions',
    'cover-picker-revisit.html'
  );
  writeFileSync(
    output,
    readFileSync(
      output,
      'utf8'
    )
      .replace(
      'Design evidence only.',
      'Changed output.'
      )
  );
  if (invoke('validate')
    .status
    === 0) throw new Error('Changed committed artifact was not rejected.');
  console.log(`Cover picker review consumer: positive, ${rejected} rejected manifests, ballot, ${refusedPages} rejected pages and changed-output checks passed.`);
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
