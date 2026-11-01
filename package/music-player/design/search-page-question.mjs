// Preserve and validate historical captures of the rejected unfolded Search split.
// Its blank-strip checks reproduce a withdrawn interpretation of E2, not a design gate.
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
  'archive',
  'search-rejected-fold-review.template.html'
);
const outputPath = join(
  root,
  'questions',
  'archive',
  'search-rejected-fold-review.html'
);
const renderPath = join(
  root,
  'questions',
  'render'
);
const evidencePath = join(
  root,
  'questions',
  'evidence'
);
const stages = [
  'player',
  'open-empty',
  'open-results',
  'open-none',
  'open-unavailable'
];
const panels = [
  'cover',
  'inner'
];
const modes = [
  'dark',
  'light'
];
const groups = {
  COVER: stages.map((stage) => ({
    stage,
    panel: 'cover',
    text: '100'
  })),
  INNER: stages.map((stage) => ({
    stage,
    panel: 'inner',
    text: '100'
  })),
  STRESS: panels.flatMap((panel) => [
    'player',
    'open-results'
  ].map((stage) => ({
    stage,
    panel,
    text: '200'
  }))),
};
const scenes = Object.values(groups)
  .flat();
const titles = {
  player: 'Player · Search button',
  'open-empty': 'Search page · empty query',
  'open-results': 'Search page · sample results',
  'open-none': 'Search page · no results',
  'open-unavailable': 'Search page · library unavailable',
};
const dimensions = {
  cover: [
    1_080,
    2_424
  ],
  inner: [
    2_076,
    2_152
  ]
};

/** Names one exact captured frame without a positional screenshot index. */
function basename({
  panel,
  stage,
  mode,
  text
}) {
  return `fold-search-${panel}-${stage}-${mode}-s${text}`;
}

/** Reads an opaque native screenshot for the requested panel, state and scheme. */
function image({
  panel,
  stage,
  mode,
  text
}) {
  return readFileSync(join(
    renderPath,
    `${basename({
      panel,
      stage,
      mode,
      text
    })}.png`
  ));
}

/** Embeds both native color schemes in one active-design card. */
function card({
  panel,
  stage,
  text
}) {
  const [width, height] = dimensions[panel];
  const light = `data:image/png;base64,${image({
    panel,
    stage,
    mode: 'light',
    text
  })
    .toString('base64')}`;
  const dark = `data:image/png;base64,${image({
    panel,
    stage,
    mode: 'dark',
    text
  })
    .toString('base64')}`;
  const label = `${panel === 'cover' ? 'Folded cover' : 'Unfolded inner'} · ${titles[stage]} · ${text}% text`;
  return `<figure class="capture" data-panel="${panel}" data-stage="${stage}" data-text="${text}" data-label="${label}">
    <h3>${label}</h3>
    <picture><source media="(prefers-color-scheme: dark)" srcset="${dark}"><img src="${light}" width="${width}" height="${height}" alt="Native Pixel 9 Pro Fold ${label}; displayed scheme follows system setting"></picture>
    <figcaption class="small">${width} × ${height} physical px, native Compose with Android system bars. Preview with device chassis at design dp.</figcaption>
    <div class="preview-actions"><button type="button" data-scheme="dark" aria-label="Preview ${label} in dark mode">Preview dark</button><button type="button" data-scheme="light" aria-label="Preview ${label} in light mode">Preview light</button></div>
  </figure>`;
}

/** Produces a self-contained file with no network, font, script or image dependencies. */
function build() {
  const template = readFileSync(
    templatePath,
    'utf8'
  );
  const filled = Object.entries(groups)
    .reduce(
    (
      markup,
      [group, entries]
    ) => {
      const placeholder = `__${group}_CARDS__`;
      if (!markup.includes(placeholder)) throw new Error(`Fold Search template lost ${placeholder}.`);
      return markup.replace(
        placeholder,
        entries.map(card)
          .join('\n')
      );
    },
    template
    );
  const dark = image({
    panel: 'cover',
    stage: 'player',
    mode: 'dark',
    text: '100'
  })
    .toString('base64');
  const light = image({
    panel: 'cover',
    stage: 'player',
    mode: 'light',
    text: '100'
  })
    .toString('base64');
  if ((!filled.includes('__FLOW_DARK__')) || (!filled.includes('__FLOW_LIGHT__'))) {
    throw new Error('Fold Search template lost its native player starting state.');
  }
  writeFileSync(
    outputPath,
    filled.replace(
      '__FLOW_DARK__',
      `data:image/png;base64,${dark}`
    )
      .replace(
      '__FLOW_LIGHT__',
      `data:image/png;base64,${light}`
      )
  );
  console.log('Built self-contained Fold-native Search review.');
}

/** Reads physical pixels at specified points without scaling the screenshot. */
function pixels({
  scene,
  mode,
  points
}) {
  const path = join(
    renderPath,
    `${basename({
      ...scene,
      mode
    })}.png`
  );
  return execFileSync(
    'magick',
    [
      path,
      '-format',
      points.map(([x, y]) => `%[hex:p{${x},${y}}]`)
        .join(' '),
      'info:'
    ],
    { encoding: 'utf8' }
  )
    .trim()
    .split(' ');
}

/** Checks only the rejected study's obsolete blank-strip premise for historical integrity. */
function assertHistoricalBlankConnector({
  xml,
  scene,
  mode
}) {
  const nodes = [...xml.matchAll(/<node\b[^>]*>/g)].map((match) => match[0]);
  const crossing = nodes.filter((node) => node.includes('package="dev.monochromatic.musicplayer"') &&
    (/text="[^"]+"/.test(node) || /content-desc="[^"]+"/.test(node)
      ||
      node.includes('clickable="true"')
      || node.includes('focusable="true"')
      || node.includes('scrollable="true"')))
    .find((node) => {
      const bounds = node.match(/bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]"/);
      if (!bounds) throw new Error(`${basename({
        ...scene,
        mode
      })}: semantic app node has no parseable bounds: ${node.slice(
        0,
        350
      )}`);
      return (Number(bounds[1]) < 1_068) && (Number(bounds[3]) > 1_009);
    });
  if (crossing) throw new Error(`${basename({
    ...scene,
    mode
  })}: app content or hit target crosses Fold connector: ${crossing.slice(
    0,
    350
  )}`);
  const width = 59;
  const top = 136;
  const height = 1_938;
  const path = join(
    renderPath,
    `${basename({
      ...scene,
      mode
    })}.png`
  );
  const rgba = execFileSync(
    'magick',
    [
      path,
      '-crop',
      `${width}x${height}+1009+${top}`,
      '+repage',
      '-depth',
      '8',
      'rgba:-'
    ],
    { maxBuffer: 1_000_000 }
  );
  if (rgba.length !== (width
    * height
    * 4)) throw new Error(`${basename({
    ...scene,
    mode
  })}: incomplete connector pixels.`);
  const expected = mode === 'dark' ? 0 : 255;
  for (let offset = 0; offset < rgba.length; offset += 4) {
    if ((rgba[offset] !== expected) || (rgba[offset + 1] !== expected)
      || (rgba[offset + 2] !== expected)
      || (rgba[offset + 3] !== 255)) {
      const pixel = offset / 4;
      throw new Error(`${basename({
        ...scene,
        mode
      })}: connector x=[1009,1068) is not the accepted ${mode} fill at (${1_009 + (pixel % width)},${top + Math.floor(pixel / width)}).`);
    }
  }
}

/** Checks the captured native header, divider, and distinct body. */
function assertHeader({
  scene,
  mode
}) {
  if (scene.stage === 'player') return;
  const dividerY = scene.panel === 'inner' ? 312 : 329;
  const [header, divider, body] = pixels({
    scene,
    mode,
    points: [
      [
        200,
        200
      ],
      [
        200,
        dividerY
      ],
      [
        scene.panel === 'inner' ? 200 : 800,
        420
      ]
    ]
  });
  const expectedHeader = mode === 'dark' ? '1A1A1FFF' : 'E7E7F1FF';
  const expectedBody = mode === 'dark' ? '000000FF' : 'FFFFFFFF';
  if ((header !== expectedHeader) || (divider === header)
    || (divider === body)
    || (body !== expectedBody)) {
    throw new Error(`${basename({
      ...scene,
      mode
    })}: native one-bar header, divider or page body changed: ${[
      header,
      divider,
      body
    ].join(',')}.`);
  }
}

/** Verifies captured roles and Android state before treating rasters as native evidence. */
function assertRoles() {
  for (const panel of panels) for (const mode of modes) {
    const {
      android,
      roles
    } = JSON.parse(readFileSync(
      join(
        evidencePath,
        `fold-search-${panel}-roles-${mode}.json`
      ),
      'utf8'
    ));
    if ((android.api !== 37) || (android.deviceState !== (panel === 'inner' ? '2' : '0'))
      ||
      (android.night !== mode)
      || (android.densityDpi !== 390)
      ||
      (android.displayPixels
        .join('x')
        !== dimensions[panel]
          .join('x'))
      ||
      (roles.surface_container_lowest !== (mode === 'dark' ? '#000000' : '#FFFFFF'))) {
      throw new Error(`${panel}/${mode}: Android device state, screen size or accepted connector role differs from the native capture.`);
    }
  }
}

/** Checks every embedded image, role record, visible hierarchy and connector pixel. */
function validate() {
  const html = readFileSync(
    outputPath,
    'utf8'
  );
  if ([
    '__FLOW_DARK__',
    '__FLOW_LIGHT__',
    '__COVER_CARDS__',
    '__INNER_CARDS__',
    '__STRESS_CARDS__'
  ].some((placeholder) => html.includes(placeholder))) {
    throw new Error('Fold Search review still has a placeholder.');
  }
  if (/<script\s+[^>]*src=|<link\s+[^>]*href=/i.test(html) || /navigator\.clipboard|clipboardData|execCommand\(\s*['"]copy/.test(html)) {
    throw new Error('Fold Search review depends on external resources or a clipboard API.');
  }
  if (html.includes('Slint design captures') || html.includes('Expanded desktop window')
    || html.includes('Narrow desktop width')
    || html.includes('I/G/R choices to make')) {
    throw new Error('Obsolete desktop-sized Search review resurfaced as target evidence.');
  }
  const figures = html.split('<figure class="capture"')
    .slice(1)
    .map((part) => part.split('</figure>')[0]);
  if (figures.length !== scenes.length) throw new Error('Fold Search review lost an active native state.');
  assertRoles();
  for (const [index, scene] of scenes.entries()) {
    const figure = figures[index];
    const [width, height] = dimensions[scene.panel];
    if ((!figure.includes(`data-panel="${scene.panel}" data-stage="${scene.stage}" data-text="${scene.text}"`)) ||
      (!figure.includes(`width="${width}" height="${height}"`))
      || (!figure.includes('data-scheme="dark"'))
      || (!figure.includes('data-scheme="light"'))) {
      throw new Error(`${scene.panel}/${scene.stage}/${scene.text}: review card has the wrong panel, scheme or physical frame.`);
    }
    const urls = [...figure.matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/g)].map((match) => match[1]);
    if (urls.length !== 2) throw new Error(`${scene.panel}/${scene.stage}: expected both native schemes.`);
    for (const [mode, encoded] of modes.map((
      mode,
      offset
    ) => [
      mode,
      urls[offset]
    ])) {
      const raw = image({
        ...scene,
        mode
      });
      const embedded = Buffer.from(
        encoded,
        'base64'
      );
      if ((!embedded.equals(raw)) || (embedded.subarray(
        0,
        8
      )
        .toString('hex')
        !== '89504e470d0a1a0a')
        ||
        (embedded.readUInt32BE(16) !== width)
        || (embedded.readUInt32BE(20) !== height)) {
        throw new Error(`${basename({
          ...scene,
          mode
        })}: review image does not match its opaque native frame.`);
      }
      const metadata = JSON.parse(readFileSync(
        join(
          evidencePath,
          `${basename({
            ...scene,
            mode
          })}.meta.json`
        ),
        'utf8'
      ));
      if ((metadata.deviceState !== (scene.panel === 'inner' ? '2' : '0')) ||
        (metadata.physicalPixels
          .join('x')
          !== dimensions[scene.panel]
            .join('x'))
        ||
        (metadata.densityDpi !== 390)
        || (metadata.night !== mode)
        ||
        (metadata.fontScale !== (scene.text === '200' ? 2 : 1))) {
        throw new Error(`${basename({
          ...scene,
          mode
        })}: capture metadata does not prove its panel, scheme and font scale.`);
      }
      const xml = readFileSync(
        join(
          evidencePath,
          `${basename({
            ...scene,
            mode
          })}.xml`
        ),
        'utf8'
      );
      if ((!xml.includes('package="dev.monochromatic.musicplayer"')) || (!xml.includes('content-desc="Search music"'))) {
        throw new Error(`${basename({
          ...scene,
          mode
        })}: native hierarchy lost the Search control.`);
      }
      const stateText = {
        player: 'content-desc="Search music"',
        'open-empty': 'text="Search your music"',
        'open-results': 'text="Results for “cam”"',
        'open-none': 'text="No results for “zzq”"',
        'open-unavailable': 'text="Library unavailable"',
      }[scene.stage];
      if ((!xml.includes(stateText)) || ((scene.stage !== 'player') && (!xml.includes('content-desc="Back to player"')))
        ||
        ([
          'open-results',
          'open-none'
        ].includes(scene.stage) && (!xml.includes('content-desc="Clear search"')))) {
        throw new Error(`${basename({
          ...scene,
          mode
        })}: native hierarchy does not show its labelled page state.`);
      }
      if (scene.panel === 'inner') assertHistoricalBlankConnector({
        xml,
        scene,
        mode
      });
      assertHeader({
        scene,
        mode
      });
    }
  }
  const urls = [...html.matchAll(/data:image\/png;base64,([A-Za-z0-9+/=]+)/g)].map((match) => match[1]);
  if ((urls.length !== ((scenes.length
    * 2)
    + 2)) || (new Set(urls).size !== (scenes.length
      * 2))) {
    throw new Error('Fold Search review lost its exact native player flow or repeated a state.');
  }
  for (const text of [
    'D47',
    'D48',
    'D49',
    '24dp connector',
    'id="panel"',
    'id="open-page"',
    'id="back-player"',
    'id="sample-query"',
    'id="unavailable"',
    'id="correction"',
    'preview.showModal()',
    'Reset 100%',
    'width="1080" height="2424"',
    'frameWidth: 907',
    'frameWidth: 502.4',
    'previewStage.scrollTop = 0',
    'previewStage.scrollLeft = 0',
    'showStage(\'open-empty\')',
    'showStage(\'player\')',
    'backPlayer.focus()',
    'openPage.focus()'
  ]) {
    if (!html.includes(text)) throw new Error(`Fold Search review is missing ${text}.`);
  }
  console.log('Withdrawn Fold Search artifact matches its historical captures; no design acceptance implied.');
}

if (process.argv[2] === 'build') build();
else if (process.argv[2] === 'validate') validate();
else throw new Error('Usage: node search-page-question.mjs build|validate');
