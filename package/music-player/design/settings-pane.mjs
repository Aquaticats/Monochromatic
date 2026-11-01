import { createHash } from 'node:crypto';
import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';
import { inflateSync } from 'node:zlib';

//region Rule helper and pinned artifact, not a stored preference or a working setting
const questions = join(
  process.cwd(),
  'questions'
);
const evidence = join(
  questions,
  'evidence'
);
// The APK digest, prototype commit and container image id are only stated in the page: D88 locks
// nothing by hash, so no rule compares them. The system image and renderer are still required.
const artifact = {
  apkSha256: '5f3a23911f2a5c32859a78e533704298bfb5afa042cf968f4fc82691ef35e9ab',
  prototypeCommit: '67eae28d1f31689bcc9f132e4db97ec87426e572',
  containerImageId: '4b8805002ee369c81b7826b941afc52c0c9678a0d0e427a0fefd1684b78b5f94',
  systemImageFingerprint: 'google/sdk_gphone16k_x86_64/emu64xa16k:17/CE2A.260420.050/16231978:user/dev-keys',
  renderer: 'Android Emulator OpenGL ES Translator (llvmpipe (LLVM 20.1.2, 256 bits))',
};
// Every rule is one `need` statement on one line with a unique name. The consumer test deletes
// exactly that line and expects its own case to stop reporting the name, so no rule goes unexercised.
function need({
  rule,
  holds,
  detail
}) {
  if (!holds) throw new Error(`Settings-pane [${rule}] ${detail}.`);
}
function rectangle(value) {
  return Array.isArray(value) && (value.length === 4)
    && value.every(Number.isInteger)
    && (value[2] > value[0])
    && (value[3] > value[1]);
}
function inside({
  inner,
  outer
}) {
  return (inner[0] >= outer[0]) && (inner[1] >= outer[1])
    && (inner[2] <= outer[2])
    && (inner[3] <= outer[3]);
}
function same(
  first,
  second
) {
  return JSON.stringify(first) === JSON.stringify(second);
}
// A clipped part is absent, or an integer rectangle inside its whole element.
function part({
  visible,
  whole
}) {
  return (visible === null) || (rectangle(visible) && inside({
    inner: visible,
    outer: whole
  }));
}
// Header, image data and end chunks only, each within the file, with nothing after the end chunk.
function essentialChunksOnly(png) {
  let offset = 8;
  while ((offset + 12) <= png.length) {
    const length = png.readUInt32BE(offset);
    const type = png.subarray(
      offset + 4,
      offset + 8
    )
      .toString('ascii');
    const next = offset + 12
      + length;
    if ((![
      'IHDR',
      'IDAT',
      'IEND'
    ].includes(type)) || (next > png.length)) return false;
    if (type === 'IEND') return (length === 0) && (next === png.length);
    offset = next;
  }
  return false;
}
//endregion

//region Exact inspected native Settings cohort
const manifest = JSON.parse(readFileSync(
  join(
    evidence,
    'settings-pane-witnesses.json'
  ),
  'utf8'
));
need({
  rule: 'manifest-schema',
  holds: manifest.schema === 1,
  detail: 'manifest schema is not 1'
});
need({
  rule: 'manifest-witness-list',
  holds: Array.isArray(manifest.witnesses),
  detail: 'manifest has no witness list'
});
// D11's row order, and the switch field each row draws. D84 removed the third, analysis row.
const rowFields = [
  [
    'strip-common-prefixes',
    'strip'
  ],
  [
    'resume-where-left-off',
    'resume'
  ]
];
const states = {
  closed: {
    opened: false,
    strip: true,
    resume: true
  },
  accepted: {
    opened: true,
    strip: true,
    resume: true
  },
  inverse: {
    opened: true,
    strip: false,
    resume: false
  },
};
const images = {};
const opened = [];
for (const [index, capture] of manifest.witnesses
  .entries()) {
  const at = `witness ${index + 1}: `;
  need({
    rule: 'witness-record',
    holds: ((typeof capture) === 'object') && (capture !== null),
    detail: `${at  }is not a record`
  });
  const {
    file,
    panel,
    view,
    position,
    scheme,
    fontScale
  } = capture;
  const path = /^settings-pane-(?:inner|cover)-[a-z-]+-(?:light|dark)-s(?:100|200)\.png$/;
  need({
    rule: 'witness-path',
    holds: ((typeof file) === 'string') && path.test(file),
    detail: `${at  }image path is outside the evidence boundary`
  });
  // Strict types: a template literal would coerce `"2"` or `["cover"]` and let later comparisons disagree.
  need({
    rule: 'witness-panel',
    holds: (panel === 'inner') || (panel === 'cover'),
    detail: `${at  }panel is not inner or cover`
  });
  need({
    rule: 'witness-scheme',
    holds: (scheme === 'light') || (scheme === 'dark'),
    detail: `${at  }theme is not light or dark`
  });
  need({
    rule: 'witness-font-scale',
    holds: (fontScale === 1) || (fontScale === 2),
    detail: `${at  }font scale is not the number 1 or 2`
  });
  need({
    rule: 'witness-view',
    holds: ((typeof view) === 'string') && Object.hasOwn(
      states,
      view
    ),
    detail: `${at  }view is not an authored view`
  });
  // With two rows the column fits in every captured condition, so no end-of-column view exists.
  need({
    rule: 'witness-position',
    holds: (position === 'none') || (position === 'start'),
    detail: `${at  }scroll position is not none or start`
  });
  need({
    rule: 'witness-position-view',
    holds: (view === 'closed') === (position === 'none'),
    detail: `${at  }only the closed player has no scroll position`
  });
  const named = `settings-pane-${panel}-${view}-${scheme}-s${fontScale * 100}.png`;
  need({
    rule: 'witness-filename',
    holds: file === named,
    detail: `${at  }filename and metadata disagree`
  });

  //region Crop follows the measured application root
  const width = panel === 'inner' ? 2_076 : 1_080;
  const physicalHeight = panel === 'inner' ? 2_152 : 2_424;
  const bounds = capture.applicationRoot;
  need({
    rule: 'root-rectangle',
    holds: rectangle(bounds),
    detail: `${at  }application root is not an integer rectangle`
  });
  need({
    rule: 'root-width',
    holds: (bounds[0] === 0) && (bounds[2] === width),
    detail: `${at  }application root does not span the panel width`
  });
  need({
    rule: 'root-status-strip',
    holds: (bounds[1] >= 1) && (bounds[1] < (physicalHeight
      / 2)),
    detail: `${at  }application root has no status strip above it`
  });
  need({
    rule: 'root-bottom',
    holds: (bounds[3] > (physicalHeight
      / 2)) && (bounds[3] <= physicalHeight),
    detail: `${at  }application root bottom is outside the lower half of the panel`
  });
  const panelPixels = JSON.stringify([
    width,
    physicalHeight
  ]);
  need({
    rule: 'physical-pixels',
    holds: JSON.stringify(capture.physicalPixels) === panelPixels,
    detail: `${at  }physical size is not this panel`
  });
  const height = physicalHeight - bounds[1];
  const crop = capture.cropPixels;
  need({
    rule: 'crop-record',
    holds: ((typeof crop) === 'object') && (crop !== null),
    detail: `${at  }crop record is absent`
  });
  need({
    rule: 'crop-origin',
    holds: (crop.x === 0) && (crop.y === bounds[1]),
    detail: `${at  }crop does not start at the measured application root`
  });
  need({
    rule: 'crop-size',
    holds: (crop.width === width) && (crop.height === height),
    detail: `${at  }crop size is not the panel below the status strip`
  });
  //endregion

  //region Displayed bytes are the hashed, sanitized image
  const png = readFileSync(join(
    evidence,
    file
  ));
  const hash = createHash('sha256')
    .update(png)
    .digest('hex');
  need({
    rule: 'png-signature',
    holds: png.subarray(
      0,
      8
    )
      .toString('hex')
      === '89504e470d0a1a0a',
    detail: `${at  }image is not a PNG`
  });
  const headerFirst = (png.length >= 33) && (png.readUInt32BE(8) === 13)
    && (png.subarray(
      12,
      16
    )
      .toString('ascii')
      === 'IHDR');
  need({
    rule: 'png-header-chunk',
    holds: headerFirst,
    detail: `${at  }image does not start with a header chunk`
  });
  need({
    rule: 'png-size',
    holds: (png.readUInt32BE(16) === width) && (png.readUInt32BE(20) === height),
    detail: `${at  }image size is not the crop size`
  });
  need({
    rule: 'png-opaque-rgb',
    holds: (png[24] === 8) && (png[25] === 2),
    detail: `${at  }image is not 8-bit opaque RGB`
  });
  need({
    rule: 'png-chunks',
    holds: essentialChunksOnly(png),
    detail: `${at  }image carries metadata, trailing bytes or a malformed chunk`
  });
  //endregion

  //region Acquisition and authored state
  need({
    rule: 'density',
    holds: capture.densityDpi === 390,
    detail: `${at  }density is not 390dpi`
  });
  need({
    rule: 'inspected',
    holds: capture.inspected === true,
    detail: `${at  }image was not inspected`
  });
  need({
    rule: 'fresh-hierarchy',
    holds: capture.freshHierarchyValidated === true,
    detail: `${at  }hierarchy was not validated as fresh`
  });
  need({
    rule: 'keyboard-closed',
    holds: capture.keyboardClosed === true,
    detail: `${at  }keyboard was not recorded closed`
  });
  need({
    rule: 'stable-frames',
    holds: capture.stableAppFrames === true,
    detail: `${at  }app frames were not recorded stable`
  });
  need({
    rule: 'retained-rgb',
    holds: capture.exactRetainedRgb === true,
    detail: `${at  }retained RGB was not proven exact`
  });
  need({
    rule: 'sanitized-opaque',
    holds: capture.opaque === true,
    detail: `${at  }sanitized image was not recorded opaque`
  });
  need({
    rule: 'sanitized-chunks',
    holds: capture.essentialPngChunksOnly === true,
    detail: `${at  }sanitized image was not recorded metadata-free`
  });
  need({
    rule: 'authored-state',
    holds: same(
      capture.state,
      states[view]
    ),
    detail: `${at  }state is not the authored state of its view`
  });
  need({
    rule: 'system-image',
    holds: capture.systemImageFingerprint === artifact.systemImageFingerprint,
    detail: `${at  }system image differs`
  });
  need({
    rule: 'renderer',
    holds: capture.renderer === artifact.renderer,
    detail: `${at  }renderer differs`
  });
  //endregion

  const key = `comparison/${panel}/${view}/${scheme}/${fontScale}/${position}`;
  need({
    rule: 'witness-unique',
    holds: !Object.hasOwn(
      images,
      key
    ),
    detail: `${at  }repeats another witness`
  });
  let fitNote;
  if (view === 'closed') {
    //region Closed player, the baseline both Back paths return to
    const { player } = capture;
    need({
      rule: 'closed-player-rectangle',
      holds: rectangle(player),
      detail: `${at  }closed player is not an integer rectangle`
    });
    need({
      rule: 'closed-player-root',
      holds: same(
        player,
        bounds
      ),
      detail: `${at  }closed player does not fill the application root`
    });
    const absent = [
      'pane',
      'header',
      'back',
      'title',
      'viewport',
      'left',
      'rows',
      'visible',
      'scrollMaxPixels',
      'titleLines',
      'titleOverflow'
    ];
    need({
      rule: 'closed-no-pane',
      holds: absent.every(name => capture[name] === null),
      detail: `${at  }closed player records Settings geometry`
    });
    fitNote = 'player with Settings closed';
    //endregion
  } else {
    //region Measured page, header and rows in physical screenshot pixels; D85 removed the closing sentence
    // 72dp header, 48dp floors and the 52 by 32dp switch at the measured 390dpi density.
    const {
      pane,
      header,
      back,
      title,
      viewport,
      left,
      rows,
      visible
    } = capture;
    need({
      rule: 'closed-only-player',
      holds: capture.player === null,
      detail: `${at  }open Settings view records a closed player`
    });
    need({
      rule: 'pane-rectangle',
      holds: rectangle(pane),
      detail: `${at  }Settings page is not an integer rectangle`
    });
    const placed = same(
      pane,
      panel === 'inner' ? [
        1_038,
        0,
        2_076,
        2_152
      ] : [
        0,
        0,
        1_080,
        2_424
      ]
    );
    need({
      rule: 'pane-placement',
      holds: placed,
      detail: `${at  }Settings page is not the unfolded right half or the full cover`
    });
    need({
      rule: 'left-half',
      holds: same(
        left,
        panel === 'inner' ? [
          0,
          0,
          1_038,
          2_152
        ] : null
      ),
      detail: `${at  }retained player half is not the unfolded left half`
    });
    need({
      rule: 'header-rectangle',
      holds: rectangle(header),
      detail: `${at  }header is not an integer rectangle`
    });
    const headed = (header[0] === pane[0]) && (header[2] === pane[2])
      && (header[1] === bounds[1]);
    need({
      rule: 'header-position',
      holds: headed,
      detail: `${at  }header does not span the page at the application root top`
    });
    need({
      rule: 'header-height',
      holds: (header[3] - header[1]) === 176,
      detail: `${at  }header is not 176px high`
    });
    need({
      rule: 'back-rectangle',
      holds: rectangle(back),
      detail: `${at  }Back target is not an integer rectangle`
    });
    need({
      rule: 'back-floor',
      holds: ((back[2] - back[0]) >= 117) && ((back[3] - back[1]) >= 117),
      detail: `${at  }Back target is below the 117px layout floor`
    });
    need({
      rule: 'back-inside-header',
      holds: inside({
        inner: back,
        outer: header
      }),
      detail: `${at  }Back target leaves the header`
    });
    need({
      rule: 'title-rectangle',
      holds: rectangle(title),
      detail: `${at  }page title is not an integer rectangle`
    });
    need({
      rule: 'title-inside-header',
      holds: inside({
        inner: title,
        outer: header
      }),
      detail: `${at  }page title leaves the header`
    });
    need({
      rule: 'back-before-title',
      holds: back[2] <= title[0],
      detail: `${at  }Back target overlaps the page title`
    });
    need({
      rule: 'title-one-line',
      holds: (capture.titleLines === 1) && (capture.titleOverflow === false),
      detail: `${at  }page title is not one unclipped line`
    });
    need({
      rule: 'viewport-rectangle',
      holds: rectangle(viewport),
      detail: `${at  }scroll viewport is not an integer rectangle`
    });
    const below = (viewport[0] === pane[0]) && (viewport[2] === pane[2])
      && (viewport[1] >= header[3])
      && ((viewport[1] - header[3]) <= 3)
      && (viewport[3] === pane[3]);
    need({
      rule: 'viewport-position',
      holds: below,
      detail: `${at  }scroll viewport does not fill the page below the header`
    });
    const scroll = capture.scrollMaxPixels;
    need({
      rule: 'scroll-extent',
      holds: Number.isInteger(scroll) && (scroll >= 0),
      detail: `${at  }scroll extent is not a whole pixel count`
    });
    need({
      rule: 'rows-list',
      holds: Array.isArray(rows) && (rows.length === 2),
      detail: `${at  }page does not record two rows`
    });
    need({
      rule: 'visible-record',
      holds: ((typeof visible) === 'object') && (visible !== null)
        && Array.isArray(visible.rows)
        && (visible.rows
          .length
          === 2),
      detail: `${at  }visible-part record is absent`
    });
    let previousBottom = viewport[1];
    const information = [title];
    const shownRows = [];
    for (const [rowIndex, row] of rows.entries()) {
      const on = `${at}row ${rowIndex + 1}: `;
      need({
        rule: 'row-record',
        holds: ((typeof row) === 'object') && (row !== null),
        detail: `${on  }is not a record`
      });
      need({
        rule: 'row-identity',
        holds: row.id === rowFields[rowIndex][0],
        detail: `${on  }is not in the accepted row order`
      });
      const drawn = [
        'row',
        'title',
        'supporting',
        'switch'
      ].every(name => rectangle(row[name]));
      need({
        rule: 'row-rectangles',
        holds: drawn,
        detail: `${on  }row, text or switch is not an integer rectangle`
      });
      need({
        rule: 'row-spans-page',
        holds: (row.row[0] === pane[0]) && (row.row[2] === pane[2]),
        detail: `${on  }does not span the page`
      });
      need({
        rule: 'row-floor',
        holds: (row.row[3] - row.row[1]) >= 117,
        detail: `${on  }is below the 117px layout floor`
      });
      need({
        rule: 'row-order',
        holds: (row.row[1] >= previousBottom) && ((row.row[1] - previousBottom) <= 3),
        detail: `${on  }does not follow the previous row or the content start`
      });
      previousBottom = row.row[3];
      const switchSize = ((row.switch[2] - row.switch[0]) === 127) && ((row.switch[3] - row.switch[1]) === 78);
      need({
        rule: 'switch-size',
        holds: switchSize,
        detail: `${on  }switch is not 127 by 78px`
      });
      const contained = [
        'title',
        'supporting',
        'switch'
      ].every(name => inside({
        inner: row[name],
        outer: row.row
      }));
      need({
        rule: 'row-parts-inside',
        holds: contained,
        detail: `${on  }text or switch leaves its row`
      });
      need({
        rule: 'title-above-supporting',
        holds: row.supporting[1] >= row.title[3],
        detail: `${on  }supporting text overlaps its title`
      });
      const clear = (row.title[2] <= row.switch[0]) && (row.supporting[2] <= row.switch[0]);
      need({
        rule: 'text-clear-of-switch',
        holds: clear,
        detail: `${on  }text runs under its switch`
      });
      const counted = Number.isInteger(row.titleLines) && (row.titleLines >= 1)
        && Number.isInteger(row.supportingLines)
        && (row.supportingLines >= 1);
      need({
        rule: 'row-text-lines',
        holds: counted,
        detail: `${on  }text line counts are not whole positive numbers`
      });
      need({
        rule: 'row-text-overflow',
        holds: (row.titleOverflow === false) && (row.supportingOverflow === false),
        detail: `${on  }native layout reports text overflow`
      });
      const seen = visible.rows[rowIndex];
      need({
        rule: 'visible-row-record',
        holds: ((typeof seen) === 'object') && (seen !== null)
          && (seen.id === row.id),
        detail: `${on  }visible part belongs to another row`
      });
      need({
        rule: 'visible-row-part',
        holds: part({
          visible: seen.row,
          whole: row.row
        }),
        detail: `${on  }visible row is not part of its row`
      });
      need({
        rule: 'visible-switch-part',
        holds: part({
          visible: seen.switch,
          whole: row.switch
        }),
        detail: `${on  }visible switch is not part of its switch`
      });
      information.push(
        row.title,
        row.supporting,
        row.switch
      );
      shownRows.push({
        whole: same(
          seen.row,
          row.row
        ) && (row.row[3] <= bounds[3]),
        switchWhole: same(
          seen.switch,
          row.switch
        )
      });
    }
    // E2: information stays off the fold connector, which ends at x 1093 on the unfolded panel.
    const offConnector = (panel === 'cover') || information.every(rect => rect[0] >= 1_093);
    need({
      rule: 'fold-connector',
      holds: offConnector,
      detail: `${at  }information starts inside the fold connector`
    });
    // The column ends one divider below its last row, plus the navigation inset under the application root.
    // The divider is read from the gap between the two rows rather than assumed, since 1dp is a fractional pixel count here.
    const divider = rows[1]
      .row[1]
      - rows[0]
        .row[3];
    const overhang = (rows[1]
      .row[3]
      + divider
      + (pane[3] - bounds[3])) - viewport[3];
    need({
      rule: 'scroll-geometry',
      holds: scroll === Math.max(
        0,
        overhang
      ),
      detail: `${at  }scroll extent is not what the measured column overhangs its viewport`
    });
    need({
      rule: 'column-fits',
      holds: scroll === 0,
      detail: `${at  }column scrolls, and this review has no end-of-column view to show what it hides`
    });
    need({
      rule: 'rows-shown',
      holds: shownRows.every(row => row.whole),
      detail: `${at  }a row is not wholly shown above the navigation area`
    });
    const heights = rows.map(row => row.row[3] - row.row[1]);
    // Empty page between the last row's divider and the navigation area.
    const spare = bounds[3]
      - rows[1]
        .row[3]
      - divider;
    fitNote = `rows ${heights.join(', ')} px high · whole column fits without scrolling · ${spare} px of empty page below it`;
    opened.push({
      capture,
      key,
      heights,
      spare,
      shownRows
    });
    //endregion
  }
  images[key] = {
    file,
    hash,
    width,
    height,
    density: 390,
    fitNote,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
}
//endregion

//region Cohort shape: every environment has its three views
const expected = [];
for (const panel of [
  'inner',
  'cover'
]) for (const scheme of [
  'light',
  'dark'
]) for (const scale of [
  1,
  2
]) {
  expected.push(`comparison/${panel}/closed/${scheme}/${scale}/none`);
  for (const view of [
    'accepted',
    'inverse'
  ]) {
    expected.push(`comparison/${panel}/${view}/${scheme}/${scale}/start`);
  }
}
need({
  rule: 'exact-cohort',
  holds: same(
    Object.keys(images)
      .sort(),
    expected.sort()
  ),
  detail: 'review requires exact panel, view, theme, scale and scroll-position combinations'
});
// Whole-element layout, which neither a switch position nor a theme may change.
function layout(capture) {
  const {
    pane,
    header,
    back,
    title,
    viewport,
    scrollMaxPixels
  } = capture;
  return [
    pane,
    header,
    back,
    title,
    viewport,
    scrollMaxPixels,
    capture.rows
      .map(row => [
        row.row,
        row.title,
        row.supporting,
        row.switch,
        row.titleLines,
        row.supportingLines
      ])
  ];
}
for (const item of opened) {
  const {
    panel,
    fontScale,
    position
  } = item.capture;
  const reference = opened.find(other => other.key === `comparison/${panel}/accepted/light/${fontScale}/${position}`);
  need({
    rule: 'layout-stable',
    holds: (reference !== undefined) && same(
      layout(reference.capture),
      layout(item.capture)
    ),
    detail: `${item.capture
      .file}: a switch position or theme changed the layout`
  });
}
//endregion

//region Switch positions re-measured from the embedded images, so a drawn state is not only asserted
// On the 52dp-wide Material switch the handle is centred 16dp from the leading edge when off and
// 36dp when on; 4dp from that edge is track in both positions. On the vertical centre line, the
// handle centre whose colour differs from the track by more than 24 of 255 in a channel holds the handle.
// Decodes an 8-bit, non-interlaced RGB PNG in process: inflate the joined image-data chunks, then
// undo each scanline's filter (PNG specification section 9). Any other format returns null, so a
// rule reading it fails rather than guessing. Spawning a decoder per image dominated validation time.
function paeth({
  left,
  up,
  upLeft
}) {
  const estimate = (left + up) - upLeft;
  const toLeft = Math.abs(estimate - left);
  const toUp = Math.abs(estimate - up);
  const toUpLeft = Math.abs(estimate - upLeft);
  if ((toLeft <= toUp) && (toLeft <= toUpLeft)) return left;
  return toUp <= toUpLeft ? up : upLeft;
}
function decodeRgb(png) {
  if ((png.length < 33) || (png[24] !== 8)
    || (png[25] !== 2)
    || (png[26] !== 0)
    || (png[27] !== 0)
    || (png[28] !== 0)) return null;
  const width = png.readUInt32BE(16);
  const height = png.readUInt32BE(20);
  const parts = [];
  for (let offset = 8; (offset + 12) <= png.length;) {
    const length = png.readUInt32BE(offset);
    if (png.subarray(
      offset + 4,
      offset + 8
    )
      .toString('ascii')
      === 'IDAT') parts.push(png.subarray(
        offset + 8,
        offset + 8
          + length
      ));
    offset += 12 + length;
  }
  const raw = inflateSync(Buffer.concat(parts));
  const stride = width * 3;
  if (raw.length !== (height
    * (stride + 1))) return null;
  const rgb = Buffer.alloc(height * stride);
  for (let row = 0; row < height; row++) {
    const filter = raw[row * (stride + 1)];
    const source = (row * (stride + 1)) + 1;
    const target = row * stride;
    for (let x = 0; x < stride; x++) {
      const left = x >= 3 ? rgb[(target + x) - 3] : 0;
      const up = row > 0 ? rgb[(target - stride) + x] : 0;
      const upLeft = (row > 0) && (x >= 3) ? rgb[((target - stride) + x) - 3] : 0;
      let predicted;
      if (filter === 0) predicted = 0;
      else if (filter === 1) predicted = left;
      else if (filter === 2) predicted = up;
      else if (filter === 3) predicted = (left + up) >> 1;
      else if (filter === 4) predicted = paeth({
        left,
        up,
        upLeft
      });
      else return null;
      rgb[target + x] = (raw[source + x] + predicted) & 255;
    }
  }
  return {
    width,
    height,
    rgb
  };
}
// Each embedded image is decoded once; every region below is sliced from that copy.
const decodedImages = new Map();
function decoded(file) {
  if (!decodedImages.has(file)) decodedImages.set(
    file,
    decodeRgb(
      readFileSync(join(
        evidence,
        file
      )),
    )
  );
  return decodedImages.get(file);
}
// RGB bytes of one image region, or null when the image does not hold the whole region.
function imageRegion({
  file,
  left,
  top,
  width,
  height
}) {
  const image = decoded(file);
  if ((image === null) || (left < 0)
    || (top < 0)
    || ((left + width) > image.width)
    || ((top + height) > image.height)) return null;
  const region = Buffer.alloc(width * height
    * 3);
  for (let row = 0; row < height; row++) {
    const start = (((top + row) * image.width) + left) * 3;
    image.rgb
      .copy(
      region,
      row * width
        * 3,
      start,
      start + (width * 3)
      );
  }
  return region;
}
// The same region addressed by a measured screenshot rectangle, moved into the cropped image.
function measuredRegion({
  capture,
  rectangle: bounds
}) {
  const [left, top, right, bottom] = bounds;
  const width = right - left;
  const height = bottom - top;
  return {
    width,
    height,
    rgb: imageRegion({
      file: capture.file,
      left: left
        - capture.cropPixels
          .x,
      top: top
        - capture.cropPixels
          .y,
      width,
      height
    })
  };
}
function drawnSwitch({
  capture,
  rectangle: bounds
}) {
  const {
    width,
    height,
    rgb
  } = measuredRegion({
    capture,
    rectangle: bounds
  });
  if (rgb === null) return null;
  const middle = Math.floor(height / 2);
  function differs(dp) {
    const sample = ((middle * width) + Math.round((width * dp) / 52)) * 3;
    const track = ((middle * width) + Math.round((width * 4) / 52)) * 3;
    return [
      0,
      1,
      2
    ].some(channel => Math.abs(rgb[sample + channel] - rgb[track + channel]) > 24);
  }
  const handleLeading = differs(16);
  const handleTrailing = differs(36);
  if (handleLeading === handleTrailing) return null;
  return handleTrailing;
}
// D85 removed the closing sentence, so nothing is drawn after the rows: from the last row's divider down
// to the application root bottom, above the gesture handle, the page is one uniform colour.
function nothingAfterRows(item) {
  const { capture } = item;
  const last = capture.rows
    .at(-1)
    .row;
  const divider = capture.rows[1]
    .row[1]
    - capture.rows[0]
      .row[3];
  // The divider's layout box is 2px, but its 1dp stroke is 2.4375px wide at 390dpi and tints one more
  // pixel row; the retained images show that row at about half the divider's contrast.
  const top = last[3] + divider
    + 1;
  if (top >= capture.applicationRoot[3]) return true;
  const { rgb } = measuredRegion({
    capture,
    rectangle: [
      capture.pane[0],
      top,
      capture.pane[2],
      capture.applicationRoot[3]
    ]
  });
  if (rgb === null) return false;
  for (let offset = 3; offset < rgb.length; offset += 3) {
    if ((rgb[offset] !== rgb[0]) || (rgb[offset + 1] !== rgb[1])
      || (rgb[offset + 2] !== rgb[2])) return false;
  }
  return true;
}
function switchesMatchPixels(item) {
  return item.capture
    .rows
    .every((
      row,
      index
    ) => {
      if (!item.shownRows[index]
        .switchWhole) return true;
      return drawnSwitch({
        capture: item.capture,
        rectangle: row.switch
      })
        === item.capture
          .state[rowFields[index][1]];
    });
}
//endregion

//region Header legibility re-measured from the embedded images, because rectangles cannot show a colour defect
// WCAG relative luminance of one pixel, and the strongest contrast any pixel of a header rectangle has
// against the header ground at that rectangle's trailing top corner, which holds no glyph.
function luminance({
  rgb,
  offset
}) {
  const linear = [
    0,
    1,
    2
  ].map(channel => {
    const value = rgb[offset + channel] / 255;
    return value <= 0.04045 ? value / 12.92 : ((value + 0.055) / 1.055) ** 2.4;
  });
  return (0.2126 * linear[0]) + (0.7152 * linear[1])
    + (0.0722 * linear[2]);
}
function strongestContrast({
  capture,
  rectangle: bounds
}) {
  const {
    width,
    rgb
  } = measuredRegion({
    capture,
    rectangle: bounds
  });
  if (rgb === null) return 0;
  const ground = luminance({
    rgb,
    offset: (((2 * width) + width) - 3) * 3
  });
  let strongest = 1;
  for (let offset = 0; offset < rgb.length; offset += 3) {
    const ink = luminance({
      rgb,
      offset
    });
    strongest = Math.max(
      strongest,
      (Math.max(
        ink,
        ground
      ) + 0.05) / (Math.min(
        ink,
        ground
      ) + 0.05)
    );
  }
  return strongest;
}
//endregion

//region Retained left half compared with the published accepted Search page, which keeps the same half
// D51's accepted Search page keeps the folder browser and deck on the unfolded left half. These are
// its published images. Shades a step or two apart between capture sessions are the
// same drawn element, so a pixel counts as different only above 24 of 255 in a channel.
const searchEvidence = {
  'light/1': { file: 'search-filename-comparison-inner-literalfull-light-s100.png' },
  'dark/1': { file: 'search-filename-comparison-inner-literalfull-dark-s100.png' },
  'light/2': { file: 'search-filename-comparison-inner-literalfull-light-s200.png' },
  'dark/2': { file: 'search-filename-comparison-inner-literalfull-dark-s200.png' },
};
function leftHalfDifference(capture) {
  const region = {
    left: 0,
    top: 0,
    width: 1_038,
    height: capture.applicationRoot[3] - capture.applicationRoot[1]
  };
  const settings = imageRegion({
    file: capture.file,
    ...region
  });
  const search = imageRegion({
    file: searchEvidence[`${capture.scheme}/${capture.fontScale}`]
      .file,
    ...region
  });
  if ((settings === null) || (search === null)) return Number.POSITIVE_INFINITY;
  let differing = 0;
  for (let offset = 0; offset < settings.length; offset += 3) {
    if ((Math.abs(settings[offset] - search[offset]) > 24) || (Math.abs(settings[offset + 1] - search[offset + 1]) > 24)
      || (Math.abs(settings[offset + 2] - search[offset + 2]) > 24)) differing++;
  }
  return differing;
}
//endregion

//region Inspection findings stated from validated data, so the page cannot drift from its evidence, then the stated provenance
function range(values) {
  const low = Math.min(...values);
  const high = Math.max(...values);
  return low === high ? String(low) : `${low} to ${high}`;
}
const starts = opened.filter(item => item.capture
  .position
  === 'start');
const normal = starts.filter(item => item.capture
  .fontScale
  === 1);
const large = starts.filter(item => item.capture
  .fontScale
  === 2);
const findings = `<p id="inspection-findings" class="note">Inspection and measurement found this.
The whole column fits without scrolling in every condition.
Below the last row, ${range(normal.map(item => item.spare))} physical pixels of empty page remain at 100% text
and ${range(large.map(item => item.spare))} at 200%, above the navigation area.
Rows are ${range(normal.flatMap(item => item.heights))} physical pixels high at 100% text
and ${range(large.flatMap(item => item.heights))} at 200%, against a 117 pixel floor;
no row title or supporting line reports overflow.
Row titles take at most ${Math.max(...normal.flatMap(item => item.capture
  .rows
  .map(row => row.titleLines)))} line at 100% text
and ${Math.max(...large.flatMap(item => item.capture
  .rows
  .map(row => row.titleLines)))} at 200%.
Supporting lines wrap to at most ${Math.max(...normal.flatMap(item => item.capture
  .rows
  .map(row => row.supportingLines)))} lines at 100%
and ${Math.max(...large.flatMap(item => item.capture
  .rows
  .map(row => row.supportingLines)))} at 200%.
Neither a switch position nor the theme changes any measured rectangle.</p>`;
const provenance = `<p id="provenance" class="digest">Artifact: prototype commit ${artifact.prototypeCommit} ·
APK SHA-256 ${artifact.apkSha256} · renderer ${artifact.renderer} ·
system image ${artifact.systemImageFingerprint} · capture container image ${artifact.containerImageId}.</p>`;
//endregion

//region Offline review preserves the accepted pane and keeps observations optional
const template = readFileSync(
  join(
    questions,
    'settings-pane.template.html'
  ),
  'utf8'
);
for (const slot of [
  'IMAGES',
  'FINDINGS',
  'PROVENANCE'
]) {
  need({
    rule: 'template-slot',
    holds: template.split(`__SETTINGS_PANE_${slot}__`)
      .length
      === 2,
    detail: `template needs exactly one ${slot} slot`
  });
}
const html = template.replace(
  '__SETTINGS_PANE_FINDINGS__',
  () => findings
)
  .replace(
  '__SETTINGS_PANE_PROVENANCE__',
  () => provenance
  )
  .replace(
  '__SETTINGS_PANE_IMAGES__',
  () => JSON.stringify(images)
    .replaceAll(
    '<',
    String.raw`\u003c`
    )
  );
const output = join(
  questions,
  'settings-pane.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log('Built offline native Settings-pane evidence from exact inspected witnesses.');
} else if (process.argv[2] === 'validate') {
  need({
    rule: 'output-current',
    holds: readFileSync(
      output,
      'utf8'
    ) === html,
    detail: 'output differs from its template and evidence'
  });
  for (const marker of [
    'color-scheme: light dark',
    'Withdrawn by D86; no new preference ballot',
    'It is not a current design to review',
    'Every switch position is authored debug state',
    'it was not separately chosen',
    'id="inspection-findings"',
    'Settings has no analysis switch',
    'Also read by eye',
    'id="provenance"',
    'No production implementation is authorized',
    'id="final-notes"',
    'Native pixels',
    'Reset 100% dp'
  ]) {
    need({
      rule: 'required-statement',
      holds: html.includes(marker),
      detail: `review is missing ${  marker}`
    });
  }
  need({
    rule: 'single-form',
    holds: (html.match(/<form\b/gi) ?? []).length === 1,
    detail: 'review must hold exactly one form'
  });
  const scripts = ((html.match(/<script\b/gi) ?? []).length === 1) && (!/<script\b[^>]*\bsrc\b/i.test(html));
  need({
    rule: 'single-inline-script',
    holds: scripts,
    detail: 'review must hold exactly one inline script'
  });
  need({
    rule: 'no-unresolved-slot',
    holds: !/__SETTINGS_PANE_[A-Z_]+__/.test(html),
    detail: 'review holds an unresolved slot'
  });
  // Base64 image data cannot contain `:`, `(` or `@`, so these patterns only ever match authored markup.
  need({
    rule: 'no-external-reference',
    holds: !/https?:\/\/|@import|url\(|<link\b/i.test(html),
    detail: 'review must stay offline, without external references'
  });
  need({
    rule: 'no-input-control',
    holds: !/<input\b/i.test(html),
    detail: 'review must not hold input controls; observations use one optional text area'
  });
  const required = /<(?:textarea|select)\b(?:[^"'<>]|"[^"]*"|'[^']*')*?\srequired(?:\s|=|>)/i.test(html);
  need({
    rule: 'no-required-field',
    holds: !required,
    detail: 'review must not require an answer'
  });
  const fields = ((html.match(/<textarea\b/gi) ?? []).length === 2) && ((html.match(/<select\b/gi) ?? []).length === 3);
  need({
    rule: 'observation-fields-only',
    holds: fields,
    detail: 'review must keep one observation field, one prepared reply and three viewing selects'
  });
  for (const item of opened) {
    const at = `${item.capture
      .file  }: `;
    need({
      rule: 'switch-position-pixels',
      holds: switchesMatchPixels(item),
      detail: `${at  }a drawn switch position differs from its authored state`
    });
    need({
      rule: 'nothing-after-rows',
      holds: nothingAfterRows(item),
      detail: `${at  }something is drawn between the last row and the application root bottom`
    });
    // 4.5 to 1 for text and 3 to 1 for a graphic are the WCAG 2 contrast minimums.
    need({
      rule: 'title-contrast-pixels',
      holds: strongestContrast({
        capture: item.capture,
        rectangle: item.capture
          .title
      }) >= 4.5,
      detail: `${at  }page title does not reach 4.5 to 1 against the header`
    });
    need({
      rule: 'back-contrast-pixels',
      holds: strongestContrast({
        capture: item.capture,
        rectangle: item.capture
          .back
      }) >= 3,
      detail: `${at  }Back glyph does not reach 3 to 1 against the header`
    });
    const sameHalf = (item.capture
      .panel
      === 'cover') || (leftHalfDifference(item.capture) === 0);
    need({
      rule: 'left-half-search',
      holds: sameHalf,
      detail: `${at  }retained left half differs from the published accepted Search page`
    });
  }
  console.log('Validated exact offline native Settings-pane evidence, authored-state limits, drawn switch positions, the empty page after the rows, header contrast and the retained left half.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
