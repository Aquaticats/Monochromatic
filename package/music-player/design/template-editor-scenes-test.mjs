import assert from 'node:assert/strict';
import { templates } from './template-reference.mjs';
import {
  drawnTexts,
  expected,
  library,
  scenes,
  standIns
} from './template-editor-scenes.mjs';

//region What each authored state draws, written out so a change in the reference or the scenes is noticed
const drawn = Object.fromEntries(scenes.map(state => [
  state.id,
  expected(state)
]));
assert.deepEqual(
  Object.keys(drawn),
  [
    'list',
    'default',
    'help',
    'unknown-field',
    'open-formula',
    'custom',
    'custom-end',
    'empty-library',
    'playing'
  ]
);
// Plain substitution (D93): the space before the empty peak stays in the second row's line.
const libraryRows = [
  {
    title: 'Another Xronixle',
    supporting: '4:35 −1.2 dBTP'
  },
  {
    title: 'Burning Aquamarine',
    supporting: '5:12 '
  }
];
for (const id of [
  'list',
  'default',
  'help',
  'unknown-field',
  'open-formula'
]) assert.deepEqual(
  drawn[id]
    .previewRows,
  libraryRows,
  id
);
// A user's own separator stays when the peak is empty.
const changedRows = [
  {
    title: 'Another Xronixle',
    supporting: 'FLAC · 4:35 · −1.2 dBTP'
  },
  {
    title: 'Burning Aquamarine',
    supporting: 'FLAC · 5:12 · '
  }
];
assert.deepEqual(
  drawn.custom
    .previewRows,
  changedRows
);
assert.deepEqual(
  drawn['custom-end']
    .previewRows,
  changedRows
);
assert.deepEqual(
  drawn['empty-library']
    .previewRows,
  [
    {
      title: 'Track title',
      supporting: '3:20 −1.0 dBTP'
    },
    {
      title: 'Track not analysed yet',
      supporting: '3:20 '
    }
  ]
);
// The playing track's line counts the track's place in its folder; the space before the empty peak stays too.
assert.deepEqual(
  drawn.playing
    .previewRows,
  [
    {
      title: 'Another Xronixle',
      supporting: '1 of 16 −1.2 dBTP'
    },
    {
      title: 'Burning Aquamarine',
      supporting: '2 of 16 '
    }
  ]
);
// Settings lists both templates, each with what its default yields for the first file (the D97 version).
assert.deepEqual(
  drawn.list
    .listEntries,
  [
    {
      title: 'Track rows',
      supporting: '4:35 −1.2 dBTP'
    },
    {
      title: 'Playing track',
      supporting: '1 of 16 −1.2 dBTP'
    }
  ]
);
assert.deepEqual(
  Object.values(drawn)
    .map(state => `${state.pageTitle  }|${
       state.kind}`),
  [
    ...Array(8)
      .fill('Track rows|track'),
    'Playing track|playing'
  ]
);
assert.equal(
  drawn.list
    .page,
  'list'
);
assert.deepEqual(
  drawn.help
    .help,
  {
    signature: 'tf(seconds, [format])',
    parameter: 'format',
    description: "Format: h, m and s for hours, minutes and seconds; a doubled letter pads with a zero; text between apostrophes is kept. Without it, m:ss."
  }
);
assert.equal(
  drawn.help
    .caret,
  15
);
assert.deepEqual(
  drawn['unknown-field']
    .errors,
  ['mi: unknown field peek']
);
assert.equal(
  drawn['unknown-field']
    .caret,
  drawn['unknown-field']
    .template
    .length
);
assert.deepEqual(
  drawn['open-formula']
    .errors,
  ['formula: the $ at character 1 has no closing $']
);
assert.equal(
  drawn.default
    .previewNote,
  'From your library. The second file is not analysed yet.'
);
assert.equal(
  drawn['unknown-field']
    .previewNote,
  'Rows keep the last valid template.'
);
// A library is always open (D95); stand-ins stand for one that holds no track.
assert.equal(
  drawn['empty-library']
    .previewNote,
  'Your library has no tracks yet. These are sample values.'
);
assert.equal(
  drawn.playing
    .previewNote,
  'From your library. The second file is not analysed yet.'
);
assert.deepEqual(
  drawn.playing
    .fields
    .slice(7)
    .map(field => `${field.label  }|${  field.insert  }|${  field.value}`),
  [
    'Place in folder|mi(track)|1',
    'Tracks in folder|mi(total)|16'
  ]
);
assert.deepEqual(
  drawn.default
    .fields
    .map(field => `${field.label  }|${  field.insert  }|${  field.value}`),
  [
    'Title|mi(title)|Another Xronixle',
    'File name|mi(file)|かめりあ(Camellia) - Another Xronixle',
    'Extension|mi(ext)|flac',
    'Folder|mi(folder)|Camellia',
    'Path|mi(path)|Camellia/かめりあ(Camellia) - Another Xronixle.flac',
    'Duration|mi(len)|275',
    'True peak|mi(peak)|−1.2 dBTP'
  ]
);
//endregion

//region Invariants every state keeps
for (const state of Object.values(drawn)) {
  // Mistakes take the place of typing help, and help needs a caret.
  assert.ok(
    !((state.errors
      .length
      > 0) && state.help),
    state.id
  );
  assert.equal(
    state.valid,
    state.errors
      .length
      === 0,
    state.id
  );
  assert.ok(
    (state.help === undefined) || state.focused,
    state.id
  );
  assert.equal(
    state.resetEnabled,
    state.template
      !== templates[state.kind]
        .defaultTemplate,
    state.id
  );
  assert.equal(
    state.previewRows
      .length,
    2,
    state.id
  );
  assert.ok(
    (state.caret === undefined) || ((state.caret >= 0) && (state.caret
      <= state.template
        .length)),
    state.id
  );
  assert.ok(
    !state.template
      .includes('|'),
    state.id
  );
  // No template of the study uses the conditional D92 removed.
  assert.ok(
    !state.template
      .includes('if('),
    state.id
  );
  // A scene under another position draws the same authored state: nothing the grammar decides differs.
  const base = drawn[state.state];
  assert.ok(
    base && (base.position === 'top'),
    state.id
  );
  for (const key of [
    'page',
    'kind',
    'pageTitle',
    'template',
    'caret',
    'focused',
    'valid',
    'previewRows',
    'previewNote',
    'errors',
    'help',
    'fields',
    'resetEnabled',
    'listEntries'
  ]) {
    assert.deepEqual(
      state[key],
      base[key],
      `${state.id  } ${
         key}`
    );
  }
}
assert.deepEqual(
  Object.values(drawn)
    .filter(state => state.focused)
    .map(state => state.id),
  [
    'help',
    'unknown-field',
    'open-formula'
  ]
);
assert.deepEqual(
  Object.values(drawn)
    .filter(state => state.position === 'end')
    .map(state => state.id),
  ['custom-end']
);
// Each preview pair shows one file with a true peak and one without, so the empty field is always on screen.
for (const pair of [
  library,
  standIns
]) assert.deepEqual(
  pair.map(track => track.peak === undefined),
  [
    false,
    true
  ]
);
// The page's whole copy, in reading order: the list page is short, an editor page has its sections in a fixed order.
assert.deepEqual(
  drawnTexts(scenes[0])
    .map(item => `${item.role  }=${  item.text}`),
  [
    'page-title=Settings',
    'section-templates=Templates',
    'entry-title-0=Track rows',
    'entry-supporting-0=4:35 −1.2 dBTP',
    'entry-title-1=Playing track',
    'entry-supporting-1=1 of 16 −1.2 dBTP'
  ]
);
for (const state of scenes.slice(1)) {
  const roles = drawnTexts(state)
    .map(item => item.role);
  assert.deepEqual(
    roles.filter(role => (!/^(preview-supporting|error|help|field-(name|value|insert))-/u.test(role)) && (!role.startsWith("preview-title-"))),
    [
      'page-title',
      'section-preview',
      'preview-note',
      'field-label',
      'template',
      'section-fields',
      ...(expected(state)
        .resetEnabled ? ['reset'] : [])
    ],
    state.id
  );
  assert.equal(
    roles.filter(role => role.startsWith('field-name-'))
      .length,
    state.kind === 'playing' ? 9 : 7,
    state.id
  );
  assert.equal(
    new Set(roles).size,
    roles.length,
    state.id
  );
}
// Both preview rows draw a second line; the unanalysed one holds the duration and the space before the empty peak.
assert.deepEqual(
  drawnTexts(scenes[1])
    .filter(item => item.role
      .startsWith('preview-'))
    .map(item => `${item.role  }=${  item.text}`),
  [
    'preview-title-0=Another Xronixle',
    'preview-supporting-0=4:35 −1.2 dBTP',
    'preview-title-1=Burning Aquamarine',
    'preview-supporting-1=5:12 ',
    'preview-note=From your library. The second file is not analysed yet.'
  ]
);
assert.equal(
  drawnTexts(scenes.find(state => state.id === 'unknown-field'))
    .find(item => item.role === 'error-0')
    .text,
  'mi: unknown field peek'
);
// Reset is on the page exactly for the states whose template is not the default.
assert.deepEqual(
  scenes.filter(state => drawnTexts(state)
    .some(item => item.role === 'reset'))
    .map(state => state.id),
  [
    'unknown-field',
    'open-formula',
    'custom',
    'custom-end'
  ]
);
//endregion
console.log(`Template editor scenes: ${  scenes.length  } states and their drawn text passed.`);
