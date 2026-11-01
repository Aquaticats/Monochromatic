import {
  defaultTemplate,
  errorLines,
  evaluateTemplate,
  helpAt,
  templates
} from './template-reference.mjs';

//region Authored states of the template editor study, with what each must draw
// Two templates are studied: the track row's supporting line (D35, D81) and the playing track's line. Every text a state draws is
// computed here from the reference, so the native fixture's authored copy can be checked against
// what the grammar yields. The preview scrolls with the page (D91) and the page rests where the
// platform puts a focused field (D94). A library is always open, and stand-ins stand for an empty one (D95).
// The second template, for the playing track, is the agent's version under D97 and awaits approval.
// Design evidence only; no production implementation is authorized.

/** Two files of the open library in the study, in a folder of 16: one analysed, one not analysed yet. */
export const library = [
  {
    title: 'Another Xronixle',
    file: 'かめりあ(Camellia) - Another Xronixle',
    ext: 'flac',
    folder: 'Camellia',
    path: 'Camellia/かめりあ(Camellia) - Another Xronixle.flac',
    len: 275,
    peak: '−1.2 dBTP',
    track: 1,
    total: 16
  },
  {
    title: 'Burning Aquamarine',
    file: 'かめりあ(Camellia) - Burning Aquamarine',
    ext: 'flac',
    folder: 'Camellia',
    path: 'Camellia/かめりあ(Camellia) - Burning Aquamarine.flac',
    len: 312,
    peak: undefined,
    track: 2,
    total: 16
  },
];

/** Stand-in rows for the preview while the open library holds no track (D90, D95). */
export const standIns = [
  {
    title: 'Track title',
    file: 'File name',
    ext: 'flac',
    folder: 'Folder',
    path: 'Folder/File name.flac',
    len: 200,
    peak: '−1.0 dBTP',
    track: 1,
    total: 10
  },
  {
    title: 'Track not analysed yet',
    file: 'File name',
    ext: 'flac',
    folder: 'Folder',
    path: 'Folder/File name.flac',
    len: 200,
    peak: undefined,
    track: 2,
    total: 10
  },
];

/**
 `caret` is a position in the template text and means the field has focus and the keyboard is open;
 without it the field is not focused. `|` in `typed` marks that position and is not part of the template.
 `state` names the authored native state a scene draws; `position` is `end` only for a scene captured with
 the page scrolled to its end, which is how the capture shows the end of the page, not a behaviour.
 `kind` names the template the editor edits: `track` for track rows, `playing` for the playing track.
 */
function scene({
  id,
  state = id,
  position = 'top',
  page = 'editor',
  kind = 'track',
  typed,
  libraryEmpty = false
}) {
  const caret = typed.indexOf('|');
  return {
    id,
    state,
    position,
    page,
    kind,
    template: typed.replace(
      '|',
      ''
    ),
    caret: caret === (-1) ? undefined : caret,
    libraryEmpty
  };
}
// A changed template whose separator stays when the peak is empty (D93), so the review shows what that means.
const changed = '$tc(up, mi(ext))$ · $tf(mi(len), m:ss)$ · $mi(peak)$';
export const scenes = [
  scene({
    id: 'list',
    page: 'list',
    typed: defaultTemplate
  }),
  scene({
    id: 'default',
    typed: defaultTemplate
  }),
  scene({
    id: 'help',
    typed: defaultTemplate.replace(
      'm:ss',
      'm:|ss'
    )
  }),
  scene({
    id: 'unknown-field',
    typed: '$tf(mi(len), m:ss)$ $mi(peek)$|'
  }),
  scene({
    id: 'open-formula',
    typed: '$tf(mi(len), m:ss)|'
  }),
  scene({
    id: 'custom',
    typed: changed
  }),
  scene({
    id: 'custom-end',
    state: 'custom',
    position: 'end',
    typed: changed
  }),
  scene({
    id: 'empty-library',
    typed: defaultTemplate,
    libraryEmpty: true
  }),
  scene({
    id: 'playing',
    kind: 'playing',
    typed: templates.playing
      .defaultTemplate
  }),
];

/** Every text the state draws that the grammar decides. */
export function expected(state) {
  const definition = templates[state.kind];
  const tracks = state.libraryEmpty ? standIns : library;
  const results = tracks.map(track => evaluateTemplate({
    text: state.template,
    track,
    fields: definition.fields
  }));
  const valid = results.every(result => result.valid);
  // While the template is invalid the rows keep the last valid template, which in the study is the default (D90).
  const shown = valid ? results : tracks.map(track => evaluateTemplate({
    text: definition.defaultTemplate,
    track,
    fields: definition.fields
  }));
  const errors = valid ? [] : errorLines(results[0]
    .errors);
  // Mistakes take the place of typing help under the field; help is shown only for a template that applies.
  const help = valid && (state.caret !== undefined) ? helpAt({
    text: state.template,
    caret: state.caret,
    fields: definition.fields
  }) : undefined;
  return {
    id: state.id,
    state: state.state,
    position: state.position,
    page: state.page,
    kind: state.kind,
    pageTitle: definition.label,
    template: state.template,
    caret: state.caret,
    focused: state.caret !== undefined,
    libraryEmpty: state.libraryEmpty,
    valid,
    previewRows: tracks.map((
      track,
      index
    ) => ({
      title: track.title,
      supporting: shown[index]
        .text
    })),
    previewNote: state.libraryEmpty ? 'Your library has no tracks yet. These are sample values.' :
      valid ? 'From your library. The second file is not analysed yet.' : 'Rows keep the last valid template.',
    errors,
    help: help === undefined ? undefined : {
      signature: help.signature,
      parameter: help.parameter,
      description: help.description
    },
    fields: definition.fields
      .map(field => ({
        label: field.label,
        insert: `mi(${  field.mode
           })`,
        value: String(tracks[0][field.mode] ?? '')
      })),
    resetEnabled: state.template !== definition.defaultTemplate,
    // Settings lists every template with the line its default yields for the first file of the library.
    listEntries: Object.values(templates)
      .map(template => ({
        title: template.label,
        supporting: evaluateTemplate({
          text: template.defaultTemplate,
          track: library[0],
          fields: template.fields
        })
          .text
      })),
  };
}

/** Every text a state's page draws, each with the role it plays, in reading order. */
export function drawnTexts(state) {
  const want = expected(state);
  if (want.page === 'list') {
    return [
      {
        role: 'page-title',
        text: 'Settings'
      },
      {
        role: 'section-templates',
        text: 'Templates'
      },
      ...want.listEntries
        .flatMap((
          entry,
          index
        ) => [
          {
            role: `entry-title-${  index}`,
            text: entry.title
          },
          {
            role: `entry-supporting-${  index}`,
            text: entry.supporting
          }
        ])
    ];
  }
  return [
    {
      role: 'page-title',
      text: want.pageTitle
    },
    {
      role: 'section-preview',
      text: 'Preview'
    },
    ...want.previewRows
      .flatMap((
        row,
        index
      ) => [
        {
          role: `preview-title-${  index}`,
          text: row.title
        },
        ...(row.supporting === '' ? [] : [{
          role: `preview-supporting-${  index}`,
          text: row.supporting
        }])
      ]),
    {
      role: 'preview-note',
      text: want.previewNote
    },
    {
      role: 'field-label',
      text: 'Template'
    },
    {
      role: 'template',
      text: want.template
    },
    ...want.errors
      .map((
        line,
        index
      ) => ({
        role: `error-${  index}`,
        text: line
      })),
    ...(want.help ? [
      {
        role: 'help-signature',
        text: want.help
          .signature
      },
      {
        role: 'help-description',
        text: want.help
          .description
      }
    ] : []),
    {
      role: 'section-fields',
      text: 'Fields'
    },
    ...want.fields
      .flatMap(field => [
        {
          role: `field-name-${  field.insert}`,
          text: field.label
        },
        {
          role: `field-value-${  field.insert}`,
          text: field.value === '' ? 'No value yet' : field.value
        },
        {
          role: `field-insert-${  field.insert}`,
          text: field.insert
        }
      ]),
    ...(want.resetEnabled ? [{
      role: 'reset',
      text: 'Reset to default'
    }] : [])
  ];
}
//endregion

if (process.argv[1]
  ?.endsWith('template-editor-scenes.mjs')) {
  console.log(JSON.stringify(
    scenes.map(expected),
    null,
    2
  ));
}
