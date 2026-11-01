import {
  execFileSync,
  spawnSync
} from 'node:child_process';
import {
  mkdirSync,
  readFileSync,
  writeFileSync
} from 'node:fs';
import {
  dirname,
  join,
  resolve
} from 'node:path';
import { fileURLToPath } from 'node:url';

/** Design-only raw capture root, supplied only for regeneration. */
const input = process.env
  .MUSIC_PLAYER_GBOARD_INPUT;
/** Destination belongs to this design package, never to the active AVD. */
const destination = join(
  import.meta.dirname,
  'questions/evidence'
);
/** One measured source per keyboard state, not candidate choices. */
const captures = [
  {
    id: 'gboard-inner-split-light-s100',
    image: 'fold-disposable-inner-gboard-results-100.png',
    xml: 'fold-disposable-inner-results-100.xml',
    panel: 'inner',
    theme: 'light',
    scale: 1,
    version: 175_753_756,
    kind: 'pass-inner',
    keyboard: [
      0,
      1_352,
      2_076,
      2_152
    ]
  },
  {
    id: 'gboard-inner-split-light-s200',
    image: 'fold-disposable-inner-gboard-results-200.png',
    xml: 'fold-disposable-inner-results-200.xml',
    panel: 'inner',
    theme: 'light',
    scale: 2,
    version: 175_753_756,
    kind: 'pass-inner',
    keyboard: [
      0,
      1_352,
      2_076,
      2_152
    ]
  },
  {
    id: 'gboard-cover-full-light-s100',
    image: 'fold-disposable-cover-gboard-results-100.png',
    xml: 'fold-disposable-cover-results-100.xml',
    panel: 'cover',
    theme: 'light',
    scale: 1,
    version: 175_753_756,
    kind: 'pass-cover',
    keyboard: [
      0,
      1_605,
      1_080,
      2_424
    ]
  },
  {
    id: 'gboard-cover-full-light-s200',
    image: 'fold-disposable-cover-gboard-results-200.png',
    xml: 'fold-disposable-cover-results-200.xml',
    panel: 'cover',
    theme: 'light',
    scale: 2,
    version: 175_753_756,
    kind: 'pass-cover',
    keyboard: [
      0,
      1_605,
      1_080,
      2_424
    ]
  },
  {
    id: 'gboard-inner-split-dark-s200',
    image: 'fold-disposable-inner-gboard-results-dark-200.png',
    xml: 'fold-disposable-inner-results-dark-200.xml',
    panel: 'inner',
    theme: 'dark',
    scale: 2,
    version: 175_753_756,
    kind: 'pass-inner',
    keyboard: [
      0,
      1_352,
      2_076,
      2_152
    ]
  },
  {
    id: 'gboard-cover-full-dark-s200',
    image: 'fold-disposable-cover-gboard-results-dark-200.png',
    xml: 'fold-disposable-cover-results-dark-200.xml',
    panel: 'cover',
    theme: 'dark',
    scale: 2,
    version: 175_753_756,
    kind: 'pass-cover',
    keyboard: [
      0,
      1_605,
      1_080,
      2_424
    ]
  },
  {
    id: 'gboard-inner-updated-split-dark-s200',
    image: 'fold-disposable-inner-updated-gboard-results-dark-200.png',
    xml: 'fold-disposable-inner-updated-results-dark-200.xml',
    panel: 'inner',
    theme: 'dark',
    scale: 2,
    version: 175_981_944,
    kind: 'pass-inner',
    keyboard: [
      0,
      1_352,
      2_076,
      2_152
    ]
  },
  {
    id: 'gboard-cover-updated-full-dark-s200',
    image: 'fold-disposable-cover-updated-gboard-results-dark-200.png',
    xml: 'fold-disposable-cover-updated-results-dark-200.xml',
    panel: 'cover',
    theme: 'dark',
    scale: 2,
    version: 175_981_944,
    kind: 'pass-cover',
    keyboard: [
      0,
      1_605,
      1_080,
      2_424
    ]
  },
  {
    id: 'gboard-inner-font-banner-light-s200',
    image: 'fold-disposable-inner-gboard-empty-200.png',
    xml: 'fold-disposable-inner-gboard-empty-200.xml',
    panel: 'inner',
    theme: 'light',
    scale: 2,
    version: 175_753_756,
    kind: 'banner',
    keyboard: [
      0,
      1_140,
      2_076,
      2_152
    ]
  },
  {
    id: 'gboard-inner-floating-overlap-light-s200',
    image: 'gboard-float-font200.png',
    xml: 'gboard-floating-font200.xml',
    panel: 'inner',
    theme: 'light',
    scale: 2,
    version: 175_981_944,
    kind: 'floating-inner',
    keyboard: [
      274,
      310,
      1_180,
      1_081
    ]
  },
  {
    id: 'gboard-cover-floating-overlap-light-s100',
    image: 'gboard-cover-typed.png',
    xml: 'gboard-cover-typed.xml',
    panel: 'cover',
    theme: 'light',
    scale: 1,
    version: 175_981_944,
    kind: 'floating-cover',
    keyboard: [
      0,
      304,
      830,
      1_025
    ]
  },
];
/** Exact debug APK installed on both AVDs, compared by pulling the original's installed APK. */
const apkSha256 = 'd895072b4f232181c1f24d9db0bfd6b3af7cafae2e25f869cbe21b0039e81f53';
/** Only synthetic text and app-owned controls may enter durable metadata. */
const labels = new Set([
  'cam',
  'Camellia',
  'Another Xronixle',
  'Search your music',
  'Track position',
  'Previous track',
  'Pause',
  'Next track',
  'Repeat track',
  'Play in order',
  'Shuffle Camellia',
  'Shuffle all folders',
  'Folder · opens this folder',
  'Track · Camellia · reveals track'
]);
/** The actual 48dp minimum at this measured 390dpi is 117px. */
const minimumModeHeight = (48 * 390) / 160;

/** Extract one UI Automator attribute without treating XML as a new relation language. */
function attribute({
  node,
  name
}) {
  const marker = `${name}="`;
  const start = node.indexOf(marker);
  return start === (-1) ? '' : node.slice(
    start + marker.length,
    node.indexOf(
      '"',
      start + marker.length
    )
  );
}

/** Retain only known synthetic app information, never Gboard suggestions or system identifiers. */
function curatedNodes(xml) {
  return xml.split('<node ')
    .slice(1)
    .filter((node) => node.includes('package="dev.monochromatic.musicplayer"'))
    .map((node) => ({
      text: attribute({
        node,
        name: 'text'
      }),
      description: attribute({
        node,
        name: 'content-desc'
      }),
      type: attribute({
        node,
        name: 'class'
      }),
      bounds: JSON.parse(attribute({
        node,
        name: 'bounds'
      })
        .replace(
        '][',
        ','
        )),
      focused: attribute({
        node,
        name: 'focused'
      }) === 'true',
    }))
    .filter((node) => labels.has(node.text) || labels.has(node.description)
      || (node.type === 'android.widget.EditText'));
}

/** Find one expected app label within the required physical pane. */
function requiredNode({
  nodes,
  label,
  leftAtLeast = 0,
  rightAtMost = Number.POSITIVE_INFINITY
}) {
  const node = nodes.find((candidate) =>
    ((candidate.text === label) || (candidate.description === label))
      && (candidate.bounds[0] >= leftAtLeast)
      && (candidate.bounds[2] <= rightAtMost));
  if (!node) throw new Error(`Missing ${label} in expected pane.`);
  return node;
}

/** A floating keyboard can hide app text even when its bottom IME inset is zero. */
function overlaps({
  first,
  second
}) {
  return (first[0] < second[2]) && (second[0] < first[2])
    && (first[1] < second[3])
    && (second[1] < first[3]);
}

/** Check distinct passing and failing states against their measured keyboard bounds. */
function verifyGeometry({
  capture,
  nodes
}) {
  const {
    keyboard,
    kind
  } = capture;
  if ((kind !== 'banner') && (kind !== 'floating-inner')) {
    const editor = requiredNode({
      nodes,
      label: 'cam'
    });
    if (!editor.focused) throw new Error(`${capture.id}: cam editor lost focus.`);
  }
  if (kind === 'pass-inner') {
    requiredNode({
      nodes,
      label: 'Camellia',
      leftAtLeast: 1_093
    });
    requiredNode({
      nodes,
      label: 'Another Xronixle',
      leftAtLeast: 1_093
    });
    for (const label of [
      'Track position',
      'Previous track',
      'Pause',
      'Next track',
      'Repeat track',
      'Play in order',
      'Shuffle Camellia',
      'Shuffle all folders'
    ]) {
      const node = requiredNode({
        nodes,
        label,
        rightAtMost: 983
      });
      if (node.bounds[3] > (keyboard[1]
        - 8)) throw new Error(`${capture.id}: ${label} reaches keyboard.`);
    }
    const finalMode = requiredNode({
      nodes,
      label: 'Shuffle all folders',
      rightAtMost: 983
    });
    if ((finalMode.bounds[3] - finalMode.bounds[1]) < minimumModeHeight) {
      throw new Error(`${capture.id}: final mode target is clipped.`);
    }
  } else if (kind === 'pass-cover') {
    for (const label of [
      'Camellia',
      'Another Xronixle'
    ]) {
      const node = requiredNode({
        nodes,
        label
      });
      if (node.bounds[3] > (keyboard[1]
        - 8)) throw new Error(`${capture.id}: ${label} reaches keyboard.`);
    }
  } else if (kind === 'banner') {
    requiredNode({
      nodes,
      label: 'Search your music'
    });
    const finalMode = requiredNode({
      nodes,
      label: 'Shuffle all folders',
      rightAtMost: 983
    });
    if ((finalMode.bounds[3] !== keyboard[1])
      || ((finalMode.bounds[3] - finalMode.bounds[1]) >= minimumModeHeight)) {
      throw new Error(`${capture.id}: banner no longer demonstrates final mode clipping.`);
    }
  } else if (kind === 'floating-inner') {
    const title = requiredNode({
      nodes,
      label: 'Another Xronixle',
      rightAtMost: 983
    });
    if (!overlaps({
      first: title.bounds,
      second: keyboard
    })) {
      throw new Error(`${capture.id}: floating keys no longer cover deck title.`);
    }
  } else if (kind === 'floating-cover') {
    for (const label of [
      'Camellia',
      'Another Xronixle'
    ]) {
      const result = requiredNode({
        nodes,
        label
      });
      if (!overlaps({
        first: result.bounds,
        second: keyboard
      })) {
        throw new Error(`${capture.id}: floating keys no longer cover ${label}.`);
      }
    }
  } else {
    throw new Error(`Unexpected evidence kind: ${kind}`);
  }
}

/** Check image dimensions, alpha, and the anonymized status-bar corner. */
function verifyImage({
  capture,
  file
}) {
  const expected = capture.panel === 'inner' ? '2076 2152' : '1080 2424';
  const result = execFileSync(
    'magick',
    [
      'identify',
      '-format',
      '%w %h %[opaque]',
      file
    ],
    {
      encoding: 'utf8',
    }
  )
    .trim();
  if (result !== `${expected} True`) throw new Error(`${capture.id}: unexpected image shape: ${result}`);
  const corner = execFileSync(
    'magick',
    [
      file,
      '-format',
      '%[fx:p{0,0}.r] %[fx:p{0,0}.g] %[fx:p{0,0}.b]',
      'info:'
    ],
    {
      encoding: 'utf8',
    }
  )
    .trim();
  if (corner !== (capture.theme === 'dark' ? '0 0 0' : '1 1 1')) {
    throw new Error(`${capture.id}: status area was not anonymized: ${corner}`);
  }
}

/** Build sanitized physical-pixel evidence and curated, privacy-bounded geometry records. */
function build() {
  if (!input) throw new Error('Set MUSIC_PLAYER_GBOARD_INPUT to the private raw capture directory.');
  const font = execFileSync(
    'fc-match',
    [
      'sans-serif',
      '--format',
      '%{file}'
    ],
    {
      encoding: 'utf8',
    }
  )
    .trim();
  if (!font) throw new Error('No installed font for the anonymized status clock.');
  mkdirSync(
    destination,
    { recursive: true }
  );
  for (const capture of captures) {
    const nodes = curatedNodes(readFileSync(
      resolve(
        input,
        capture.xml
      ),
      'utf8'
    ));
    verifyGeometry({
      capture,
      nodes
    });
    const output = join(
      destination,
      `${capture.id}.png`
    );
    const inner = capture.panel === 'inner';
    const statusEdge = inner ? 570 : 350;
    const statusHeight = inner ? 135 : 151;
    const surface = capture.theme === 'dark' ? '#000000' : '#ffffff';
    const ink = capture.theme === 'dark' ? '#ffffff' : '#171820';
    const result = spawnSync(
      'magick',
      [
        resolve(
          input,
          capture.image
        ),
        '-fill',
        surface,
        '-draw',
        `rectangle 0,0 ${statusEdge},${statusHeight}`,
        '-font',
        font,
        '-pointsize',
        String(capture.scale === 2 ? 57 : 37),
        '-fill',
        ink,
        '-annotate',
        `+${inner ? 149 : 61}+${capture.scale === 2 ? 93 : 81}`,
        '9:41',
        '-strip',
        output
      ],
      { encoding: 'utf8' }
    );
    if (result.error) throw result.error;
    if ((result.status !== 0)
      || result.stderr
        .trim()) {
      throw new Error(`${capture.id}: ImageMagick status ${result.status}: ${result.stderr
        .trim()}`);
    }
    const avd = capture.kind
      .startsWith('floating') ? 'Pixel_9_Pro_Fold' : 'Fold_No_Hardware_Probe';
    writeFileSync(
      join(
        destination,
        `${capture.id}.json`
      ),
      `${JSON.stringify(
        {
      sourceImage: capture.image,
      sourceHierarchy: capture.xml,
      image: `${capture.id}.png`,
      avd,
      panel: capture.panel,
      theme: capture.theme,
      fontScale: capture.scale,
      keyboardMode: capture.kind,
      keyboardBoundsPx: capture.keyboard,
      densityDpi: 390,
      gboardVersionCode: capture.version,
      apkSha256,
      appNodes: nodes,
    },
        null,
        2
      )}\n`
    );
    verifyImage({
      capture,
      file: output
    });
    console.log(`${capture.id}: sanitized and measured`);
  }
}

/** Validate committed evidence without requiring private raw captures in CI. */
function validate() {
  for (const capture of captures) {
    const record = JSON.parse(readFileSync(
      join(
        destination,
        `${capture.id}.json`
      ),
      'utf8'
    ));
    if ((record.image !== `${capture.id}.png`)
      || (record.keyboardMode !== capture.kind)
      || (JSON.stringify(record.keyboardBoundsPx) !== JSON.stringify(capture.keyboard))) {
      throw new Error(`${capture.id}: evidence provenance changed.`);
    }
    verifyGeometry({
      capture,
      nodes: record.appNodes
    });
    verifyImage({
      capture,
      file: join(
        destination,
        record.image
      )
    });
  }
  console.log('Validated sanitized real-Gboard geometry evidence.');
}

if (process.argv[2] === 'build') build();
else if (process.argv[2] === 'validate') validate();
else throw new Error('Usage: node gboard-evidence.mjs build|validate');
