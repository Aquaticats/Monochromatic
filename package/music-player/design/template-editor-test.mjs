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
  'template-editor-review-'
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
const manifestFile = 'template-editor-witnesses.json';
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
  'template-editor.template.html'
);
copyFileSync(
  join(
    root,
    'questions',
    'template-editor.template.html'
  ),
  templatePath
);
// The builder computes what each state draws from the scenes and the reference, so both travel with it.
for (const file of [
  'template-editor-scenes.mjs',
  'template-reference.mjs'
]) copyFileSync(
  join(
    root,
    file
  ),
  join(
    fixture,
    file
  )
);
const builder = join(
  fixture,
  'template-editor.mjs'
);
copyFileSync(
  join(
    root,
    'template-editor.mjs'
  ),
  builder
);
// Each named guard can be deleted from the disposable builder; the cases that prove it must then fail this test.
const guards = {
  'without-header-title': 'fixed && !inView',
  'without-cohort': 'if (!images[`${panel}/${state.id}/${scheme}/${scale}`]) {',
  'without-keyboard-edge': 'keyboardTops[panel].size !== 1',
  'without-view-edges': 'capture.viewBottom !== viewBottom',
};
const removed = process.argv[2];
if (removed !== undefined) {
  if (!(removed in guards)) throw new Error('Unknown template editor review test mutation.');
  const source = readFileSync(
    builder,
    'utf8'
  );
  if (source.split(guards[removed])
    .length
    !== 2) throw new Error(`Expected template editor guard absent: ${  removed}`);
  writeFileSync(
    builder,
    source.replace(
      guards[removed],
      () => removed === 'without-cohort' ? 'if (false) {' : 'false'
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
    throw new Error(`Expected template editor review rejection absent: ${diagnostic}; status ${run.status}; ${run.stderr
      .slice(
        0,
        300
      )}`);
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
// Positive fixtures the cases need: a typing view, a view at rest and the view scrolled to the end.
function index(found) {
  const at = manifest.witnesses
    .findIndex(found);
  if (at === (-1)) throw new Error('Required positive fixture absent.');
  return at;
}
const typing = index(item => (item.scene === 'unknown-field') && (item.panel === 'inner')
  && (item.scheme === 'light')
  && (item.fontScale === 1));
const resting = index(item => (item.scene === 'default') && (item.panel === 'inner')
  && (item.scheme === 'light')
  && (item.fontScale === 1));
const ended = index(item => (item.scene === 'custom-end') && (item.panel === 'cover')
  && (item.scheme === 'light')
  && (item.fontScale === 2));
function role(
  view,
  name
) {
  const item = view.drawn
    .find(candidate => candidate.role === name);
  if (!item) throw new Error(`Required role absent from fixture: ${  name}`);
  return item;
}
try {
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      !== 0)) throw new Error('Positive template editor consumer failed.');
  reject({
    change: input => { input.witnesses[0]
      .file = '../outside.png'; },
    diagnostic: 'outside the allowed evidence boundary'
  });
  reject({
    change: input => { input.witnesses[0]
      .scene = 'different'; },
    diagnostic: 'filename and metadata disagree'
  });
  reject({
    change: input => { input.witnesses[0]
      .freshHierarchyValidated = false; },
    diagnostic: 'image geometry or acquisition assertion differs'
  });
  reject({
    change: input => { input.witnesses[0]
      .cropPixels
      .y += 1; },
    diagnostic: 'image geometry or acquisition assertion differs'
  });
  reject({
    change: input => { input.witnesses[ended]
      .position = 'top'; },
    diagnostic: 'recorded state or position differs from the scene'
  });
  reject({
    change: input => { input.witnesses[typing]
      .keyboardShown = false; },
    diagnostic: 'keyboard state differs from the authored state'
  });
  reject({
    change: input => { input.witnesses[resting]
      .keyboardShown = true; },
    diagnostic: 'keyboard state differs from the authored state'
  });
  // A view at rest ends at the navigation area, not at the screen's lower edge; a view that claims more would call cut text visible.
  reject({
    change: input => { input.witnesses[resting]
      .viewBottom = input.witnesses[resting]
        .physicalPixels[1]; },
    diagnostic: 'edges of the visible page are absent or differ from the panel'
  });
  reject({
    change: input => { input.witnesses[typing]
      .viewBottom += 40; },
    diagnostic: 'edges of the visible page are absent or differ from the panel'
  });
  // Where a focused page rests is the platform's (D94): a recorded scroll of the study's own on a typing view is refused.
  reject({
    change: input => { input.witnesses[typing]
      .scrollRule = {
        rule: 'lines',
        target: 0,
        viewport: 1
      }; },
    diagnostic: 'scroll rule applied differs from the scene'
  });
  reject({
    change: input => { input.witnesses[ended]
      .scrollRule = null; },
    diagnostic: 'scroll rule applied differs from the scene'
  });
  reject({
    change: input => { role(
      input.witnesses[typing],
      'error-0'
    )
      .text = 'mi: unknown field peak'; },
    diagnostic: 'recorded copy differs from what the template reference yields'
  });
  reject({
    change: input => { input.witnesses[typing]
      .drawn
      .pop(); },
    diagnostic: 'recorded copy differs from what the template reference yields'
  });
  reject({
    change: input => { input.witnesses[typing]
      .uncalledPageTexts = ['Save']; },
    diagnostic: 'draws text its state does not call for'
  });
  // A text recorded as in view although its rectangle touches the keyboard's edge.
  reject({
    change: input => { const item = role(
      input.witnesses[typing],
      'error-0'
    );
      item.bounds = [[
        item.bounds[0][0],
        item.bounds[0][1],
        item.bounds[0][2],
        input.witnesses[typing]
          .viewBottom
      ]]; },
    diagnostic: 'recorded visibility of error-0 differs from its rectangle'
  });
  reject({
    change: input => { role(
      input.witnesses[resting],
      'preview-title-0'
    )
      .inView = false; },
    diagnostic: 'recorded visibility of preview-title-0 differs from its rectangle'
  });
  // The header does not scroll: a page title with no rectangle is refused even when recorded as out of view.
  reject({
    change: input => { const item = role(
      input.witnesses[resting],
      'page-title'
    );
      item.bounds = [];
      item.inView = false; },
    diagnostic: 'page-title is in the header but is not in view'
  });
  reject({
    change: input => { const item = input.witnesses[ended]
      .drawn
      .at(-1);
      item.bounds = [];
      item.inView = false; },
    diagnostic: "the page's end is not in view"
  });
  reject({
    change: input => { input.witnesses
      .splice(
      ended,
      1
      ); },
    diagnostic: 'requires every authored scene under every condition'
  });
  // One pixel is enough to differ from the panel's other views and too little to change what this view keeps in view.
  reject({
    change: input => { input.witnesses[typing]
      .keyboardTop -= 1;
      input.witnesses[typing]
        .viewBottom -= 1; },
    diagnostic: 'do not share one keyboard edge'
  });
  if (invoke('build')
    .status
    !== 0) throw new Error('Restored evidence build failed.');
  // The page shows the decided design and asks nothing; it must keep its decided list and quote no conditional.
  const template = readFileSync(
    templatePath,
    'utf8'
  );
  // Counted as they run, so the printed number cannot drift from the cases.
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
    writeFileSync(
      templatePath,
      template
    );
    refusedPages += 1;
  }
  refusedPage({
    change: page => page.replace(
      '</form>',
      '<input type="radio" name="layout" value="flow"></form>'
    ),
    diagnostic: 'asks a question; the decided design is evidence only'
  });
  refusedPage({
    change: page => page.replace(
      '<textarea id="final-notes"',
      '<textarea required id="final-notes"'
    ),
    diagnostic: 'asks a question; the decided design is evidence only'
  });
  refusedPage({
    change: page => page.replace(
      '<li>A mistake is named under the field',
      '<li><code>$if(mi(peak), a)$</code></li><li>A mistake is named under the field'
    ),
    diagnostic: 'quotes a conditional'
  });
  refusedPage({
    change: page => page.replace(
      'What is decided',
      'Background'
    ),
    diagnostic: 'is missing What is decided'
  });
  refusedPage({
    change: page => page.replace(
      '</form>',
      '</form><form></form>'
    ),
    diagnostic: 'must be one self-contained form'
  });
  // Which lines get a template (D99) must stay on the page, and every captured scene must have its figure.
  refusedPage({
    change: page => page.replace(
      'Which lines get a template',
      'Also built'
    ),
    diagnostic: 'is missing Which lines get a template'
  });
  refusedPage({
    change: page => page.replace(
      '<figure data-scene="playing">',
      '<figure data-scene="playing-old">'
    ),
    diagnostic: 'shows no figure for playing'
  });
  // A page still telling an earlier cohort's capture story names a visit count its views do not have.
  refusedPage({
    change: page => page.replace(
      /\d+ visits contributed views/u,
      '3 visits contributed views'
    ),
    diagnostic: 'does not name its'
  });
  if ((invoke('build')
    .status
    !== 0) || (invoke('validate')
      .status
      !== 0)) throw new Error('Restored template did not validate.');
  const output = join(
    fixture,
    'questions',
    'template-editor.html'
  );
  writeFileSync(
    output,
    readFileSync(
      output,
      'utf8'
    )
      .replace(
      'Every state is authored',
      'Changed output'
      )
  );
  if (invoke('validate')
    .status
    === 0) throw new Error('Changed committed artifact was not rejected.');
  console.log(`Template editor review consumer: positive, ${rejected} rejected manifests, ${refusedPages} rejected pages and a changed-output check passed.`);
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
