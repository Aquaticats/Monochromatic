import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import {
  drawnTexts,
  expected,
  scenes
} from './template-editor-scenes.mjs';

//region Inspected native cohort, each view checked from its own image, its record and the grammar
// D88: nothing is compared with a recorded digest. What is checked about an image is read from the image,
// and what a view draws is checked against what the template reference yields for its authored state.
const question = join(
  process.cwd(),
  'questions'
);
const evidence = join(
  question,
  'evidence'
);
const manifest = JSON.parse(readFileSync(
  join(
    evidence,
    'template-editor-witnesses.json'
  ),
  'utf8'
));
// Each panel's size and the upper edge of its navigation area, in physical pixels, as the emulator reports them.
const panels = {
  inner: {
    size: [
      2_076,
      2_152
    ],
    navigationTop: 2_074
  },
  cover: {
    size: [
      1_080,
      2_424
    ],
    navigationTop: 2_365
  }
};
if ((manifest.schema !== 3) || (!Array.isArray(manifest.witnesses))) throw new Error('Template editor review manifest is not this study.');
const images = {};
const facts = {};
const keyboardTops = {
  inner: new Set(),
  cover: new Set()
};
for (const capture of manifest.witnesses) {
  const {file} = capture;
  if (!/^template-editor-(?:inner|cover)-[a-z-]+-(?:light|dark)-s(?:100|200)\.png$/.test(file)) {
    throw new Error('Template editor capture path is outside the allowed evidence boundary.');
  }
  const state = scenes.find(candidate => candidate.id === capture.scene);
  if ((!state) || (file !== `template-editor-${capture.panel}-${capture.scene}-${capture.scheme}-s${capture.fontScale * 100}.png`)) {
    throw new Error('Template editor capture filename and metadata disagree.');
  }
  const png = readFileSync(join(
    evidence,
    file
  ));
  const [physicalWidth, physicalHeight] = panels[capture.panel]
    .size;
  const crop = capture.cropPixels;
  // The status strip is removed and everything under it kept, the keyboard included.
  if ((png.subarray(
    0,
    8
  )
    .toString('hex')
    !== '89504e470d0a1a0a') ||
    (capture.physicalPixels[0] !== physicalWidth)
    || (capture.physicalPixels[1] !== physicalHeight)
    ||
    (crop.x !== 0)
    || (crop.width !== physicalWidth)
    || (!Number.isInteger(crop.y))
    || (crop.y < 1)
    ||
    ((crop.y + crop.height) !== physicalHeight)
    || (png.readUInt32BE(16) !== crop.width)
    || (png.readUInt32BE(20) !== crop.height)
    ||
    (capture.densityDpi !== 390)
    || (capture.freshHierarchyValidated !== true)
    || (capture.productionServicesObserved !== false)) {
    throw new Error(`${file}: template editor image geometry or acquisition assertion differs.`);
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
    ].includes(chunk)) throw new Error('Template editor PNG contains unintended metadata.');
    offset += length + 12;
  }
  const want = expected(state);
  if ((capture.state !== state.state) || (capture.position !== state.position)) {
    throw new Error(`${file}: recorded state or position differs from the scene.`);
  }
  // A state that holds focus is captured with the keyboard open, and no other state is.
  if ((capture.keyboardShown !== want.focused) || (want.focused && (!(Number.isInteger(capture.keyboardTop) &&
    (capture.keyboardTop > crop.y)
    && (capture.keyboardTop
      < panels[capture.panel]
        .navigationTop))))) {
    throw new Error(`${file}: keyboard state differs from the authored state.`);
  }
  if (want.focused) keyboardTops[capture.panel]
    .add(capture.keyboardTop);
  // The view ends at the keyboard when one is open and at the navigation area otherwise; what scrolls is in
  // view only inside the page's scrolling window, which starts under the header.
  const viewBottom = want.focused ? capture.keyboardTop : panels[capture.panel]
    .navigationTop;
  if ((capture.viewBottom !== viewBottom) || (!Number.isInteger(capture.viewTop))
    || (capture.viewTop <= crop.y)
    || (capture.viewTop >= viewBottom)) {
    throw new Error(`${file}: the edges of the visible page are absent or differ from the panel.`);
  }
  // Where a focused page rests is the platform's (D94); the study scrolls only an end scene, to show the page's end.
  const rule = state.position === 'end' ? 'end' : undefined;
  if ((capture.scrollRule
    ?.rule) !== rule) throw new Error(`${file}: the scroll rule applied differs from the scene.`);
  const called = drawnTexts(state);
  if ((!Array.isArray(capture.drawn)) || (JSON.stringify(capture.drawn
    .map(item => [
      item.role,
      item.text
    ])) !==
    JSON.stringify(called.map(item => [
      item.role,
      item.text
    ])))) {
    throw new Error(`${file}: recorded copy differs from what the template reference yields for this state.`);
  }
  if ((!Array.isArray(capture.uncalledPageTexts)) || (capture.uncalledPageTexts
    .length
    > 0)) {
    throw new Error(`${file}: the page draws text its state does not call for.`);
  }
  // What counts as in view is re-derived from each text's rectangle. A text that scrolls must lie strictly
  // inside the window: a rectangle touching an edge is cut there or flush against it, and the hierarchy
  // cannot tell which. The page title sits in the header, between the status strip and the window.
  for (const item of capture.drawn) {
    const fixed = item.role === 'page-title';
    const inView = item.bounds
      .some(bounds => fixed ? (bounds[1] >= crop.y) && (bounds[3] <= capture.viewTop) :
        (bounds[1] > capture.viewTop) && (bounds[3] < viewBottom));
    if (item.inView !== inView) throw new Error(`${file}: recorded visibility of ${item.role} differs from its rectangle.`);
    // The header does not scroll, so its title must be in view in every view of the editor.
    if (fixed && (!inView)) throw new Error(`${file}: ${item.role} is in the header but is not in view.`);
  }
  // A page scrolled to its end shows its last text.
  if ((state.position === 'end') && (!capture.drawn
    .at(-1)
    .inView)) throw new Error(`${file}: the page's end is not in view.`);
  const key = `${capture.panel}/${capture.scene}/${capture.scheme}/${capture.fontScale}`;
  if (images[key]) throw new Error('Duplicate template editor capture.');
  images[key] = {
    file,
    width: crop.width,
    height: crop.height,
    density: 390,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
  facts[key] = {
    keyboard: capture.keyboardShown,
    caretDrawn: capture.caretDrawn === true,
    scrolls: capture.scrollMax > 0,
    outOfView: capture.drawn
      .filter(item => (!item.inView) && (item.bounds
        .length
        === 0))
      .map(item => item.role),
    atEdge: capture.drawn
      .filter(item => (!item.inView) && (item.bounds
        .length
        > 0))
      .map(item => item.role)
  };
}
for (const panel of Object.keys(panels)) for (const scheme of [
  'light',
  'dark'
]) for (const scale of [
  1,
  2
]) for (const state of scenes) {
  if (!images[`${panel}/${state.id}/${scheme}/${scale}`]) {
    throw new Error(`Template editor review requires every authored scene under every condition: ${panel}/${state.id}/${scheme}/${scale}`);
  }
}
if (Object.keys(images)
  .length
  !== manifest.witnesses
    .length) throw new Error('Template editor review holds a view outside the authored cohort.');
// Every keyboard-open view of a panel met the same keyboard; a taller one would be the keyboard's own notice.
for (const panel of Object.keys(panels)) {
  if (keyboardTops[panel]
    .size
    !== 1) throw new Error(`Keyboard-open views of the ${panel} panel do not share one keyboard edge.`);
}
// The theme must not change what is in view: a light and dark pair of the same scene and size reports the same facts.
for (const key of Object.keys(facts)) {
  const [panel, scene, scheme, scale] = key.split('/');
  const other = facts[`${panel}/${scene}/dark/${scale}`];
  if ((scheme === 'light') && ((JSON.stringify(facts[key]
    .outOfView) !== JSON.stringify(other.outOfView)) ||
    (JSON.stringify(facts[key]
      .atEdge) !== JSON.stringify(other.atEdge)))) {
    throw new Error(`Light and dark views of one scene disagree about what is in view: ${  key}`);
  }
}
//endregion

//region Build and consumer validation
const template = readFileSync(
  join(
    question,
    'template-editor.template.html'
  ),
  'utf8'
);
for (const slot of [
  '__TEMPLATE_EDITOR_IMAGES__',
  '__TEMPLATE_EDITOR_FACTS__',
  '__TEMPLATE_EDITOR_STATES__'
]) {
  if (template.split(slot)
    .length
    !== 2) throw new Error(`Expected one slot ${  slot  }.`);
}
function embed(value) { return JSON.stringify(value)
  .replaceAll(
  '<',
  String.raw`\u003c`
  ); }
const states = Object.fromEntries(scenes.map(state => [
  state.id,
  expected(state)
]));
const html = template.replace(
  '__TEMPLATE_EDITOR_IMAGES__',
  () => embed(images)
)
  .replace(
  '__TEMPLATE_EDITOR_FACTS__',
  () => embed(facts)
  )
  .replace(
  '__TEMPLATE_EDITOR_STATES__',
  () => embed(states)
  );
const output = join(
  question,
  'template-editor.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log(`Built the offline template editor review with ${manifest.witnesses
    .length} inspected native views.`);
} else if (process.argv[2] === 'validate') {
  if (readFileSync(
    output,
    'utf8'
  ) !== html) throw new Error('Template editor review differs from template and checked evidence.');
  for (const marker of [
    'color-scheme: light dark',
    'Every state is authored',
    'Typing is not connected',
    'No production implementation is authorized',
    'id="final-notes"',
    'id="reply"',
    'Native pixels',
    'Reset 100% dp',
    'What is decided',
    '$tf(mi(len), m:ss)$ $mi(peak)$',
    'Which lines get a template',
    '$mi(track)$ of $mi(total)$ $mi(peak)$'
  ]) {
    if (!html.includes(marker)) throw new Error(`Template editor review is missing ${marker}.`);
  }
  // The page tells the published cohort's own capture story: it names how many visits contributed views, read
  // from the views' records, so a republished cohort cannot keep the previous cohort's account.
  const visits = new Set(manifest.witnesses
    .map(capture => capture.visit)).size;
  if (!html.includes(`${visits} visits contributed views`)) throw new Error(`Template editor review does not name its ${visits} visits.`);
  // Every authored scene has its figure, so a captured state cannot go unshown.
  for (const state of scenes) {
    if (template.split(`<figure data-scene="${state.id}">`)
      .length
      !== 2) throw new Error(`Template editor review shows no figure for ${state.id}.`);
  }
  // The editor (D89 to D96, D98) and the lines that get a template (D99) are decided:
  // the page shows them and asks nothing, so it holds no choice to make.
  if (/<input\b[^>]*type="(?:radio|checkbox)"|<select\b[^>]*\bname=|<[a-z]+\b[^>]*\srequired[\s>=]/i.test(html)) {
    throw new Error('Template editor review asks a question; the decided design is evidence only.');
  }
  // A template the page quotes must not use the conditional D92 removed.
  if (/\$[^$]*\bif\(/.test(html.replaceAll(
    /<script\b[\s\S]*?<\/script>/g,
    ''
  ))) throw new Error('Template editor review quotes a conditional.');
  if (((html.match(/<form\b/g) ?? []).length !== 1) ||
    /__TEMPLATE_EDITOR_[A-Z]+__|<script\b[^>]*\bsrc=|<link\b[^>]*\bhref=|<img\b[^>]*\bsrc="https?:/i.test(html)) {
    throw new Error('Template editor review must be one self-contained form.');
  }
  console.log('Validated the offline template editor review against its evidence and the template reference.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
