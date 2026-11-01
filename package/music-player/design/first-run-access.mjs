import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

//region Inspected native cohort, each view checked from its own image and record
// D88: nothing is compared with a recorded digest. What is checked about an image is read from the image.
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
    'first-run-access-witnesses.json'
  ),
  'utf8'
));
const panels = {
  inner: [
    2_076,
    2_152
  ],
  cover: [
    1_080,
    2_424
  ]
};
// Whether the authored state carries the analysis explanation (D84: automatic, no choice offered).
// A library is always open (D95), so there is no state without a source; access declined stays (D100).
const scenes = {
  declined: true,
  'system-no-audio': false,
  'folder-no-audio': false
};
// The declined state shows the open library as unreadable and offers both ways on (D100).
const declinedActions = [
  'Allow access',
  'Open a folder',
  'Settings'
];
const withdrawn = 'choose whether to analyse';
if ((manifest.schema !== 2)
  || ((typeof manifest.analysisText) !== 'string')
  || (manifest.analysisText
    .length
    === 0)
  ||
  manifest.analysisText
    .includes(withdrawn)
  || (!Array.isArray(manifest.witnesses))) {
  throw new Error('First-run review manifest is not the rebuilt study or still offers the withdrawn analysis choice.');
}
const images = {};
const views = {};
for (const capture of manifest.witnesses) {
  const {file} = capture;
  if (!/^first-run-access-(?:inner|cover)-[a-z-]+-(?:light|dark)-s(?:100|200)(?:-end)?\.png$/.test(file)) {
    throw new Error('First-run capture path is outside the allowed evidence boundary.');
  }
  const suffix = capture.position === 'scrolled' ? '-end' : '';
  const expectedFile = `first-run-access-${capture.panel}-${capture.scene}-${capture.scheme}-s${capture.fontScale * 100}${suffix}.png`;
  if ((file !== expectedFile) || (![
    'initial',
    'scrolled'
  ].includes(capture.position))
    || (!(capture.scene in scenes))) {
    throw new Error('First-run capture filename and metadata disagree.');
  }
  const png = readFileSync(join(
    evidence,
    file
  ));
  const [physicalWidth, physicalHeight] = panels[capture.panel];
  const crop = capture.cropPixels;
  // The status strip is removed and the navigation area kept, so the crop starts under the strip and ends at the panel's end.
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
    || (capture.keyboardClosed !== true)
    || (capture.freshHierarchyValidated !== true)) {
    throw new Error(`${file}: first-run image geometry or acquisition assertion differs.`);
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
    ].includes(chunk)) throw new Error('First-run PNG contains unintended metadata.');
    offset += length + 12;
  }
  if ((!Array.isArray(capture.texts))
    || capture.texts
      .some(text => ((typeof text) !== 'string') || text.includes(withdrawn))) {
    throw new Error(`${file}: first-run drawn text offers the withdrawn analysis choice.`);
  }
  if ((!scenes[capture.scene])
    && capture.texts
      .includes(manifest.analysisText)) {
    throw new Error(`${file}: first-run drawn text explains analysis in a state that has none.`);
  }
  if ((capture.scene === 'declined') && (capture.position === 'initial')
    && declinedActions.some(label => !capture.texts
      .includes(label))) {
    throw new Error(`${file}: the declined state does not draw both ways on and Settings.`);
  }
  const key = `comparison/${capture.panel}/${capture.scene}/${capture.scheme}/${capture.fontScale}/${capture.position}`;
  if (images[key]) throw new Error('Duplicate first-run review capture.');
  images[key] = {
    file,
    width: crop.width,
    height: crop.height,
    density: 390,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
  views[key] = capture;
}
// Every authored state on both panels, in both themes, at both text sizes; the kept end views follow from the captures.
for (const panel of Object.keys(panels)) for (const scheme of [
  'light',
  'dark'
]) for (const scale of [
  1,
  2
]) for (const scene of Object.keys(scenes)) {
  const base = `comparison/${panel}/${scene}/${scheme}/${scale}/`;
  const first = views[`${base  }initial`];
  const end = views[`${base  }scrolled`];
  if (!first) throw new Error(`First-run review requires every authored state under every condition: ${  base}`);
  const moved = first.dragTest
    ?.movedAppPixels;
  if (((typeof moved) !== 'boolean') || (moved !== (end !== undefined))) {
    throw new Error(`First-run end view and the recorded drag outcome disagree: ${  base}`);
  }
  // The hierarchy marks a container scrollable only when its content overflows; it must tell the same story as the drag.
  if (first.scrollableContainerReported !== moved) {
    throw new Error(`First-run drag outcome and the hierarchy disagree about scrolling: ${  base}`);
  }
  if (end && ((end.scrollProof
    ?.appRgbChanged
    !== true) || (!Number.isFinite(end.scrollProof
      .verticalDisplacement))
    ||
    (end.scrollProof
      .verticalDisplacement
      === 0))) {
    throw new Error(`First-run end view lacks changed pixels and a displaced body: ${  base}`);
  }
  // The explanation may begin under the first view at large text; it must be drawn in one of the kept views.
  if (scenes[scene] && (![
    first,
    end
  ].some(view => view?.texts
    .includes(manifest.analysisText)))) {
    throw new Error(`First-run analysis explanation is in neither kept view: ${  base}`);
  }
}
if (Object.keys(images)
  .length
  !== manifest.witnesses
    .length) throw new Error('First-run review holds a view outside the authored cohort.');
const total = manifest.witnesses
  .length;
const ends = manifest.witnesses
  .filter(capture => capture.position === 'scrolled')
  .length;
//endregion

//region Build and consumer validation, authored states never become implicit policy selection
const template = readFileSync(
  join(
    question,
    'first-run-access.template.html'
  ),
  'utf8'
);
if (template.split('__FIRST_RUN_REVIEW_IMAGES__')
  .length
  !== 2) throw new Error('Expected one first-run image slot.');
const serialized = JSON.stringify(images)
  .replaceAll(
  '<',
  String.raw`\u003c`
  );
const html = template.replace(
  '__FIRST_RUN_REVIEW_IMAGES__',
  () => serialized
);
const output = join(
  question,
  'first-run-access.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log(`Built offline first-run state evidence with ${total} inspected native views, ${ends} of them end views.`);
} else if (process.argv[2] === 'validate') {
  if (readFileSync(
    output,
    'utf8'
  ) !== html) throw new Error('First-run review differs from template and checked evidence.');
  for (const marker of [
    'color-scheme: light dark',
    'Existing requirements, no cosmetic-policy ballot',
    'Every state is authored debug input',
    'not real discovery or permission evaluation',
    'No production implementation is authorized',
    'Open a folder',
    'in-app Settings',
    'automatic and not optional',
    'no choice about analysis',
    'automatic WorkManager initialization',
    'Allow access',
    'The declined state, as approved',
    'id="final-notes"',
    'id="reply"',
    'Native pixels',
    'Reset 100% dp'
  ]) {
    if (!html.includes(marker)) throw new Error(`First-run review is missing ${marker}.`);
  }
  if (html.includes(withdrawn)) throw new Error('First-run review still states the withdrawn analysis choice.');
  // The page tells its own cohort's capture story: it names how many visits contributed views, read from the records.
  const visits = new Set(manifest.witnesses
    .map(capture => capture.visit)).size;
  if (!html.includes(`${visits} visits contributed views`)) throw new Error(`First-run review does not name its ${visits} visits.`);
  // Every figure belongs to an authored state, and every authored state has its first-view figure.
  const figured = [...template.matchAll(/<figure data-scene="([a-z-]+)"/gu)].map(match => match[1]);
  for (const scene of figured) if (!(scene in scenes)) throw new Error(`First-run review shows a figure for ${scene}, which is not an authored state.`);
  for (const scene of Object.keys(scenes)) if (!template.includes(`<figure data-scene="${scene}">`)) throw new Error(`First-run review shows no figure for ${scene}.`);
  if (((html.match(/<form\b/g) ?? []).length !== 1) ||
    /__FIRST_RUN_REVIEW_IMAGES__|<script\b[^>]*\bsrc=|<link\b[^>]*\bhref=|<img\b[^>]*\bsrc="https?:/i.test(html)
    ||
    /<input\b[^>]*\btype="radio"|name="placement"|name="visibility"|<textarea\b(?:[^"'<>]|"[^"]*"|'[^']*')*?\srequired(?:\s|=|>)/i.test(html)) {
    throw new Error('First-run review must be self-contained evidence, not a policy ballot.');
  }
  console.log('Validated offline first-run review, optional observations and the absence of the withdrawn analysis choice.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
