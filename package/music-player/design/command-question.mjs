// Build and verify the self-contained native-Slint command-bar design round.
// The cover-P form and the withdrawn first command matrix remain historical evidence.
import { execFileSync } from 'node:child_process';
import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

const root = process.cwd();
const templatePath = join(
  root,
  'questions',
  'current.template.html'
);
const outputPath = join(
  root,
  'questions',
  'current.html'
);
const renderPath = join(
  root,
  'questions',
  'render'
);
const slots = [
  'IN_APP',
  'GLOBAL',
  'SEARCH',
  'WIDE',
  'STRESS',
  'EDGE'
];
const groups = {
  IN_APP: [
    1,
    2,
    3
  ].map((variant) => ({
    kind: 'i',
    variant,
    size: 'compact'
  })),
  GLOBAL: [
    1,
    2
  ].map((variant) => ({
    kind: 'g',
    variant,
    size: 'compact'
  })),
  SEARCH: [
    1,
    2
  ].map((variant) => ({
    kind: 'r',
    variant,
    size: 'compact'
  })),
  WIDE: [
    1,
    2,
    3
  ].map((variant) => ({
    kind: 'i',
    variant,
    size: 'wide'
  })),
  STRESS: [{
    kind: 'i',
    variant: 1,
    size: 'stress'
  }],
  EDGE: [
    1,
    2
  ].map((variant) => ({
    kind: 'e',
    variant,
    size: 'compact'
  })),
};
const labels = {
  i: 'In-app placement',
  g: 'Global invocation',
  r: 'Search relationship',
  e: 'Edge state'
};
const modes = [
  'dark',
  'light'
];

/** Names one native Slint raster from its decision axis and viewport. */
function basename({
  kind,
  variant,
  mode,
  size
}) {
  return `command-md3-${kind}${variant}-${mode}-${size}`;
}

/** Reads the exact native PNG that will be embedded in one picture source. */
function encodedImage(scene) {
  return readFileSync(join(
    renderPath,
    `${basename(scene)}.png`
  ))
    .toString('base64');
}

/** Produces a native-pixel preview pair for one independent candidate or stress state. */
function cardHtml(scene) {
  const dark = `data:image/png;base64,${encodedImage({
    ...scene,
    mode: 'dark'
  })}`;
  const light = `data:image/png;base64,${encodedImage({
    ...scene,
    mode: 'light'
  })}`;
  const label = `${scene.kind
    .toUpperCase()}${scene.variant} · ${labels[scene.kind]}${scene.size === 'compact' ? '' : ` · ${scene.size}`}`;
  const width = scene.size === 'wide' ? 1_100 : (scene.size === 'stress' ? 360 : 480);
  const height = scene.size === 'compact' ? 600 : 640;
  return `<figure class="capture${scene.size === 'wide' ? ' wide' : ''}" data-label="${label}">
    <h3>${label}</h3>
    <picture><source media="(prefers-color-scheme: dark)" srcset="${dark}"><img src="${light}" width="${width}" height="${height}" alt="Native Slint ${label}, scheme follows system setting"></picture>
    <figcaption class="small">Native Slint · ${width} × ${height}px · thumbnail follows system scheme.</figcaption>
    <div class="preview-actions"><button type="button" data-scheme="dark">Preview dark</button><button type="button" data-scheme="light">Preview light</button></div>
  </figure>`;
}

/** Replaces each independent question and state-probe slot with embedded native rasters. */
function build() {
  const template = readFileSync(
    templatePath,
    'utf8'
  );
  const html = slots.reduce(
    (
      markup,
      slot
    ) => {
      const placeholder = `__${slot}_CARDS__`;
      if (!markup.includes(placeholder)) throw new Error(`Command template lost ${placeholder}.`);
      return markup.replace(
        placeholder,
        groups[slot]
          .map(cardHtml)
          .join('\n')
      );
    },
    template
  );
  if (!html.includes('__MD3_REFERENCE__')) throw new Error('Command template lost the baseline header reference.');
  const reference = readFileSync(join(
    root,
    'questions',
    'evidence',
    'command-md3-baseline-header-comparison.png'
  ))
    .toString('base64');
  writeFileSync(
    outputPath,
    html.replace(
      '__MD3_REFERENCE__',
      `data:image/png;base64,${reference}`
    )
  );
  console.log('Built self-contained separable command-bar questionnaire.');
}

/** Checks the Slint panel color at a quiet pixel inside the compact docked candidate. */
function panelPixel(mode) {
  const file = join(
    renderPath,
    `${basename({
      kind: 'i',
      variant: 1,
      mode,
      size: 'compact'
    })}.png`
  );
  return execFileSync(
    'magick',
    [
      file,
      '-format',
      '%[pixel:p{20,175}]',
      'info:'
    ],
    { encoding: 'utf8' }
  )
    .trim();
}

/** Rejects stale rasters, missing schemes, clipboard use, and collapsed decision axes. */
function validate() {
  const html = readFileSync(
    outputPath,
    'utf8'
  );
  if (slots.some((slot) => html.includes(`__${slot}_CARDS__`)) || html.includes('__MD3_REFERENCE__')) throw new Error('Command questionnaire retains a capture placeholder.');
  if (/<script\s+[^>]*src=|<link\s+[^>]*href=/i.test(html) || /navigator\.clipboard|clipboardData|execCommand\(\s*['"]copy/.test(html)) {
    throw new Error('Command questionnaire depends on an external resource or clipboard API.');
  }
  const scenes = Object.values(groups)
    .flat();
  const urls = [...html.matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/g)].map((match) => match[1]);
  const reference = readFileSync(join(
    root,
    'questions',
    'evidence',
    'command-md3-baseline-header-comparison.png'
  ))
    .toString('base64');
  if ((urls.length !== ((scenes.length
    * 2)
    + 1)) || (new Set(urls).size !== ((scenes.length
      * 2)
      + 1))) {
    throw new Error(`Command questionnaire carries ${urls.length} data URLs and ${new Set(urls).size} distinct rasters; expected ${(scenes.length * 2) + 1} each.`);
  }
  const referenceImage = Buffer.from(
    reference,
    'base64'
  );
  if ((urls.filter((url) => url === reference)
    .length
    !== 1) ||
    (referenceImage.subarray(
      0,
      8
    )
      .toString('hex')
      !== '89504e470d0a1a0a')
    ||
    (referenceImage.readUInt32BE(16) !== 924)
    || (referenceImage.readUInt32BE(20) !== 108)
    ||
    (!html.includes('width="924" height="108" alt="Baseline MD3 docked Search header'))) {
    throw new Error('Command questionnaire lost its exact baseline header comparison.');
  }
  for (const scene of scenes) {
    for (const mode of modes) {
      const key = basename({
        ...scene,
        mode
      });
      const encoded = encodedImage({
        ...scene,
        mode
      });
      if (urls.filter((url) => url === encoded)
        .length
        !== 1) throw new Error(`${key}: embedded raster missing or stale.`);
      const image = Buffer.from(
        encoded,
        'base64'
      );
      if (image.subarray(
        0,
        8
      )
        .toString('hex')
        !== '89504e470d0a1a0a') throw new Error(`${key}: not a PNG.`);
      const expected = scene.size === 'wide' ? [
        1_100,
        640
      ] : (scene.size === 'stress' ? [
        360,
        640
      ] : [
        480,
        600
      ]);
      if ((image.readUInt32BE(16) !== expected[0]) || (image.readUInt32BE(20) !== expected[1])) throw new Error(`${key}: wrong native dimensions.`);
    }
  }
  const figures = html.split('<figure class="capture')
    .slice(1)
    .map((chunk) => chunk.split('</figure>')[0]);
  if (figures.length !== scenes.length) throw new Error('Command questionnaire lost a native comparison card.');
  for (const [index, scene] of scenes.entries()) {
    const figure = figures[index];
    const expectedCode = `${scene.kind
      .toUpperCase()}${scene.variant}`;
    const captures = [...figure.matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/g)].map((match) => match[1]);
    const expectedWidth = scene.size === 'wide' ? 1_100 : (scene.size === 'stress' ? 360 : 480);
    const expectedHeight = scene.size === 'compact' ? 600 : 640;
    if ((!figure.includes(`data-label="${expectedCode} ·`)) || (captures.length !== 2)
      ||
      (captures[0] !== encodedImage({
        ...scene,
        mode: 'dark'
      }))
      || (captures[1] !== encodedImage({
        ...scene,
        mode: 'light'
      }))
      ||
      (!figure.includes(`width="${expectedWidth}" height="${expectedHeight}" alt="Native Slint`))
      ||
      (!figure.includes('data-scheme="dark"'))
      || (!figure.includes('data-scheme="light"'))) {
      throw new Error(`${expectedCode} ${scene.size}: the card does not disclose its matching native dark/light captures.`);
    }
  }
  for (const [name, values] of Object.entries({
    inapp: [
      'I1',
      'I2',
      'I3'
    ],
    global: [
      'G1',
      'G2'
    ],
    search: [
      'R1',
      'R2'
    ]
  })) {
    for (const value of values) {
      if (html.split(`name="${name}" value="${value}" required`)
        .length
        !== 2) {
        throw new Error(`Command questionnaire lost required ${name}/${value} choice.`);
      }
    }
  }
  if (/<input[^>]*type="radio"[^>]*checked/.test(html)) throw new Error('Command questionnaire must not preselect a choice.');
  for (const text of [
    'name="inapp"',
    'name="global"',
    'name="search"',
    'name="correction"',
    'In-app ranking: I1 &gt; I2 &gt; I3',
    'Global ranking: G2 &gt; G1',
    'Search relationship ranking: R1 &gt; R2',
    'D21',
    'D25',
    'off by default',
    'baseline MD3',
    '28px outer shape',
    '56px header and divider',
    '72px header',
    'preview.showModal()',
    'await previewImage.decode()',
    'returnTarget?.focus()',
    'stepZoom(-0.25)'
  ]) {
    if (!html.includes(text)) throw new Error(`Command questionnaire is missing ${text}.`);
  }
  if (((html.match(/type="radio"/g) ?? []).length !== 7) || ((html.match(/<fieldset>/g) ?? []).length !== 3)) {
    throw new Error('Command questionnaire must ask three separable questions with seven radio choices.');
  }
  if (((html.match(/class="capture/g) ?? []).length !== scenes.length) || ((html.match(/data-scheme="dark"/g) ?? []).length !== scenes.length)
    || ((html.match(/data-scheme="light"/g) ?? []).length !== scenes.length)) {
    throw new Error('Command questionnaire must disclose every native scene in both schemes.');
  }
  if ((panelPixel('dark') !== 'srgba(30,31,38,1)') || (panelPixel('light') !== 'srgba(231,231,241,1)')) {
    throw new Error('Command native panels lost the measured high-container reference.');
  }
  function sample({
    mode,
    point
  }) {
    const file = join(
      renderPath,
      `${basename({
        kind: 'i',
        variant: 1,
        mode,
        size: 'compact'
      })}.png`
    );
    return execFileSync(
      'magick',
      [
        file,
        '-format',
        `%[pixel:p{${point}}]`,
        'info:'
      ],
      { encoding: 'utf8' }
    )
      .trim();
  }
  for (const [mode, divider, focus] of [
    [
      'dark',
      'srgba(115,117,127,1)',
      'srgba(186,197,238,1)'
    ],
    [
      'light',
      'srgba(121,122,132,1)',
      'srgba(78,94,139,1)'
    ],
  ]) {
    if ((sample({
      mode,
      point: '200,148'
    }) !== divider) || (sample({
      mode,
      point: '24,223'
    }) !== focus)) {
      throw new Error(`${mode} command study lost its baseline divider or keyboard-focus boundary.`);
    }
  }
  console.log('Separable desktop command-bar questionnaire is valid.');
}

if (process.argv[2] === 'build') build();
else if (process.argv[2] === 'validate') validate();
else throw new Error('Usage: node command-question.mjs build|validate');
