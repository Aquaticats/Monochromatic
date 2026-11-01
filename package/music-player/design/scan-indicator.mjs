import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  readFileSync,
  writeFileSync
} from 'node:fs';
import { join } from 'node:path';

//region Rule helper and pinned artifact, not a scan policy or real analysis result
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
  apkSha256: 'c06ec80240e41641fee1fdec79313a8294b9595544740145f4fa6f2b1f77ddf1',
  prototypeCommit: '25a95411750c5c5356631fc508513c42a81ad1c4',
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
  if (!holds) throw new Error(`Scan-indicator [${rule}] ${detail}.`);
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
// Encoded at the HTML interpolation point; the status rules already limit these strings to authored text.
function escapeHtml(text) {
  return text.replaceAll(
    '&',
    '&amp;'
  )
    .replaceAll(
    '<',
    '&lt;'
    )
    .replaceAll(
    '>',
    '&gt;'
    )
    .replaceAll(
    '"',
    '&quot;'
    );
}
//endregion

//region Exact inspected native scan cohort
const manifest = JSON.parse(readFileSync(
  join(
    evidence,
    'scan-indicator-witnesses.json'
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
const states = {
  idle: {
    phase: 'idle',
    done: 0,
    total: 1_218
  },
  running: {
    phase: 'running',
    done: 412,
    total: 1_218
  },
  paused: {
    phase: 'paused',
    done: 412,
    total: 1_218
  },
  'wide-count': {
    phase: 'running',
    done: 9_999_998,
    total: 9_999_999
  },
};
const expected = [];
for (const panel of [
  'inner',
  'cover'
]) for (const scene of Object.keys(states)) {
  for (const scheme of [
    'light',
    'dark'
  ]) for (const scale of [
    1,
    2
  ]) {
    expected.push(`comparison/${panel}/${scene}/${scheme}/${scale}/initial`);
  }
}
const images = {};
const active = [];
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
    scene,
    scheme,
    fontScale
  } = capture;
  const path = /^scan-indicator-(?:inner|cover)-[a-z-]+-(?:light|dark)-s(?:100|200)\.png$/;
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
    rule: 'witness-scene',
    holds: ((typeof scene) === 'string') && Object.hasOwn(
      states,
      scene
    ),
    detail: `${at  }scene is not an authored scene`
  });
  const named = `scan-indicator-${panel}-${scene}-${scheme}-s${fontScale * 100}.png`;
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
    rule: 'source-padding',
    holds: capture.controlPaddingDp === 0,
    detail: `${at  }control does not use the accepted source padding`
  });
  const authored = JSON.stringify(states[scene]);
  need({
    rule: 'authored-state',
    holds: JSON.stringify(capture.state) === authored,
    detail: `${at  }state is not the authored state of its scene`
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

  //region Measured player, bar and control geometry in physical screenshot pixels
  const {
    player,
    viewport,
    bar,
    control
  } = capture;
  need({
    rule: 'player-rectangle',
    holds: rectangle(player),
    detail: `${at  }player is not an integer rectangle`
  });
  const fromRoot = (player[0] === 0) && (player[1] === bounds[1])
    && (player[2] === width);
  need({
    rule: 'player-origin',
    holds: fromRoot,
    detail: `${at  }player does not start at the application root across the panel width`
  });
  need({
    rule: 'viewport-rectangle',
    holds: rectangle(viewport),
    detail: `${at  }track viewport is not an integer rectangle`
  });
  need({
    rule: 'viewport-inside-player',
    holds: inside({
      inner: viewport,
      outer: player
    }),
    detail: `${at  }track viewport leaves the player`
  });
  let statusNote;
  if (scene === 'idle') {
    need({
      rule: 'idle-bar',
      holds: bar === null,
      detail: `${at  }idle view records a bar`
    });
    need({
      rule: 'idle-control',
      holds: control === null,
      detail: `${at  }idle view records a control`
    });
    need({
      rule: 'idle-bar-absent-flag',
      holds: capture.barAbsent === true,
      detail: `${at  }idle view does not record the bar as absent`
    });
    need({
      rule: 'idle-status-lines',
      holds: capture.statusLines === 0,
      detail: `${at  }idle view records status lines`
    });
    need({
      rule: 'idle-status-ellipsis',
      holds: capture.statusEllipsized === false,
      detail: `${at  }idle view records a status ellipsis`
    });
    need({
      rule: 'idle-control-label',
      holds: capture.controlLabel === null,
      detail: `${at  }idle view records a control label`
    });
    need({
      rule: 'idle-control-text-lines',
      holds: capture.controlTextLines === null,
      detail: `${at  }idle view records control text lines`
    });
    need({
      rule: 'idle-control-text-overflow',
      holds: capture.controlTextLayoutOverflow === null,
      detail: `${at  }idle view records control text overflow`
    });
    need({
      rule: 'idle-status-reading',
      holds: capture.inspectedVisibleStatus === null,
      detail: `${at  }idle view records a status reading`
    });
    need({
      rule: 'idle-label-clearance',
      holds: capture.controlLabelClearPixels === null,
      detail: `${at  }idle view records label clearance`
    });
    need({
      rule: 'idle-player-bottom',
      holds: player[3] === bounds[3],
      detail: `${at  }idle player leaves a reserved slot above the application root bottom`
    });
    statusNote = 'no scan bar and no reserved slot';
  } else {
    // 56dp bar, 100dp control and the 48dp control floor at the measured 390dpi density.
    need({
      rule: 'bar-rectangle',
      holds: rectangle(bar),
      detail: `${at  }bar is not an integer rectangle`
    });
    const atBottom = (bar[0] === 0) && (bar[2] === width)
      && (bar[3] === bounds[3]);
    need({
      rule: 'bar-position',
      holds: atBottom,
      detail: `${at  }bar does not span the panel width at the application root bottom`
    });
    need({
      rule: 'bar-height',
      holds: (bar[3] - bar[1]) === 137,
      detail: `${at  }bar is not 137px high`
    });
    need({
      rule: 'player-above-bar',
      holds: player[3] === bar[1],
      detail: `${at  }player does not end where the bar begins`
    });
    need({
      rule: 'control-rectangle',
      holds: rectangle(control),
      detail: `${at  }control is not an integer rectangle`
    });
    need({
      rule: 'control-width',
      holds: (control[2] - control[0]) === 244,
      detail: `${at  }control is not 244px wide`
    });
    need({
      rule: 'control-floor',
      holds: (control[3] - control[1]) >= 117,
      detail: `${at  }control is below the 117px layout floor`
    });
    need({
      rule: 'control-inside-bar',
      holds: inside({
        inner: control,
        outer: bar
      }),
      detail: `${at  }control leaves the bar`
    });
    need({
      rule: 'bar-absent-flag',
      holds: capture.barAbsent === false,
      detail: `${at  }active view records the bar as absent`
    });
    need({
      rule: 'status-lines',
      holds: capture.statusLines === 1,
      detail: `${at  }status is not one line`
    });
    need({
      rule: 'control-text-lines',
      holds: capture.controlTextLines === 1,
      detail: `${at  }control text is not one line`
    });
    need({
      rule: 'control-text-overflow',
      holds: capture.controlTextLayoutOverflow === false,
      detail: `${at  }native layout reports control text overflow`
    });
    const label = capture.state
      .phase
      === 'paused' ? 'Resume' : 'Pause';
    need({
      rule: 'control-label',
      holds: capture.controlLabel === label,
      detail: `${at  }control label does not match the authored phase`
    });
    const clipped = (panel === 'cover') && (fontScale === 2);
    need({
      rule: 'ellipsis-placement',
      holds: capture.statusEllipsized === clipped,
      detail: `${at  }status ellipsis flag does not match cover at 200% text`
    });
    const visible = capture.inspectedVisibleStatus;
    need({
      rule: 'status-reading-text',
      holds: (typeof visible) === 'string',
      detail: `${at  }inspected status reading is not text`
    });
    need({
      rule: 'ellipsis-binding',
      holds: visible.endsWith('…') === capture.statusEllipsized,
      detail: `${at  }inspected reading and the native ellipsis flag disagree`
    });
    const {
      done,
      total
    } = capture.state;
    const fullStatus = `Analysing true peak · ${done.toLocaleString('en-US')} of ${total.toLocaleString('en-US')}`;
    const prefix = (visible.length >= 2) && (visible.length <= fullStatus.length)
      && fullStatus.startsWith(visible.slice(
        0,
        -1
      ));
    const readable = capture.statusEllipsized ? prefix : visible === fullStatus;
    need({
      rule: 'status-reading-authored',
      holds: readable,
      detail: `${at  }inspected reading is not the authored status or a proper prefix of it`
    });
    const clear = capture.controlLabelClearPixels;
    need({
      rule: 'label-clearance-record',
      holds: ((typeof clear) === 'object') && (clear !== null),
      detail: `${at  }label clearance record is absent`
    });
    need({
      rule: 'label-clearance-leading',
      holds: Number.isInteger(clear.left) && (clear.left >= 0),
      detail: `${at  }leading label clearance is not a whole pixel count`
    });
    need({
      rule: 'label-clearance-trailing',
      holds: Number.isInteger(clear.right) && (clear.right >= 0),
      detail: `${at  }trailing label clearance is not a whole pixel count`
    });
    statusNote = `${capture.statusEllipsized ? `status ellipsized, showing “${visible}”` : 'status shows both counts in full' 
      } · ${capture.controlLabel} label ${Math.min(
        clear.left,
        clear.right
      )} px clear of the control outline`;
    active.push(capture);
  }
  //endregion
  const key = `comparison/${panel}/${scene}/${scheme}/${fontScale}/initial`;
  need({
    rule: 'witness-unique',
    holds: !Object.hasOwn(
      images,
      key
    ),
    detail: `${at  }repeats another witness`
  });
  images[key] = {
    file,
    hash,
    width,
    height,
    density: 390,
    statusNote,
    source: `data:image/png;base64,${png.toString('base64')}`
  };
}
const exact = JSON.stringify(Object.keys(images)
  .sort()) === JSON.stringify(expected.sort());
need({
  rule: 'exact-cohort',
  holds: exact,
  detail: 'review requires exact panel, scene, theme and scale combinations'
});
//endregion

//region Label clearance re-measured from the embedded images, so the stated pixels are not only asserted
// A pixel is ink when any channel differs from the control's interior background by more than 24 of 255.
// On each row crossed by the label, the clear gap is the background run between an outline run and the
// nearest label run. A run much wider than the outline stroke means label ink merged into the outline.
function measuredClearance(capture) {
  const [left, top, right, bottom] = capture.control;
  const width = right - left;
  const height = bottom - top;
  const rgb = execFileSync(
    'magick',
    [
      '-limit',
      'thread',
      '2',
      '-limit',
      'memory',
      '256MiB',
      join(
        evidence,
        capture.file
      ),
      '-crop',
      `${width}x${height}+${left
        - capture.cropPixels
          .x}+${top
            - capture.cropPixels
              .y}`,
      '+repage',
      '-depth',
      '8',
      'rgb:-'
    ],
    { maxBuffer: 8 * 1_024
      * 1_024 }
  );
  if (rgb.length !== (width
    * height
    * 3)) return {
      left: null,
      right: null,
      merged: true
    };
  const sample = ((8 * width) + Math.floor(width / 2)) * 3;
  let clearLeft = Number.POSITIVE_INFINITY;
  let clearRight = Number.POSITIVE_INFINITY;
  let merged = false;
  for (let row = 0; row < height; row++) {
    const runs = [];
    let start = -1;
    for (let x = 0; x <= width; x++) {
      const offset = ((row * width) + x) * 3;
      const ink = (x < width) && [
        0,
        1,
        2
      ].some(channel => Math.abs(rgb[offset + channel] - rgb[sample + channel]) > 24);
      if (ink && (start < 0)) start = x;
      if ((!ink) && (start >= 0)) {
        runs.push([
          start,
          x - 1
        ]);
        start = -1;
      }
    }
    const middle = (row > (height
      * 0.25)) && (row < (height
        * 0.75));
    if ((runs.length === 2) && middle
      && runs.every(run => (run[1] - run[0]) <= (width / 4))
      && runs.some(run => ((run[1] - run[0]) + 1) > 12)) merged = true;
    if ((runs.length < 3) || (runs[0][0] > 12)
      || (runs.at(-1)[1] < (width
        - 13))) continue;
    if ((((runs[0][1] - runs[0][0]) + 1) > 12) && middle) merged = true;
    else clearLeft = Math.min(
      clearLeft,
      runs[1][0] - runs[0][1]
        - 1
    );
    if ((((runs.at(-1)[1] - runs.at(-1)[0]) + 1) > 12) && middle) merged = true;
    else clearRight = Math.min(
      clearRight,
      runs.at(-1)[0] - runs.at(-2)[1]
        - 1
    );
  }
  return {
    left: clearLeft,
    right: clearRight,
    merged
  };
}
function clearanceMatchesPixels(capture) {
  const measured = measuredClearance(capture);
  return (!measured.merged) && (measured.left
    === capture.controlLabelClearPixels
      .left)
    && (measured.right
      === capture.controlLabelClearPixels
        .right);
}
//endregion

//region Inspection findings stated from validated data, so the page cannot drift from its evidence, then the stated provenance
function range(values) {
  const low = Math.min(...values);
  const high = Math.max(...values);
  return low === high ? String(low) : `${low} to ${high}`;
}
function leastClear(captures) {
  return Math.min(...captures.map(capture => Math.min(
    capture.controlLabelClearPixels
      .left,
    capture.controlLabelClearPixels
      .right
  )));
}
const ellipsized = active.filter(capture => capture.statusEllipsized);
const shown = [...new Set(ellipsized.map(capture => capture.inspectedVisibleStatus))].sort();
const resumeLarge = active.filter(capture => (capture.controlLabel === 'Resume') && (capture.fontScale === 2));
const pauseLarge = active.filter(capture => (capture.controlLabel === 'Pause') && (capture.fontScale === 2));
const normal = active.filter(capture => capture.fontScale === 1);
const shownText = shown.map(text => `“${escapeHtml(text)}”`)
  .join(' or ');
const leading = range(resumeLarge.map(capture => capture.controlLabelClearPixels
  .left));
const trailing = range(resumeLarge.map(capture => capture.controlLabelClearPixels
  .right));
const findings = `<p id="inspection-findings" class="note">Inspection found two things worth opening.
On the cover panel at 200% text the status label is ellipsized in ${ellipsized.length} active views,
which show only ${shownText}; the hidden counts are not presented as visible.
At 200% text the Resume label's ink stays ${leading} physical pixels clear of the control outline
on the leading side and ${trailing} on the trailing side, on both panels;
the native layout reports no overflow there.
Pause at 200% keeps at least ${leastClear(pauseLarge)} pixels clear,
and both labels at 100% keep at least ${leastClear(normal)}.</p>`;
const provenance = `<p id="provenance" class="digest">Artifact: prototype commit ${artifact.prototypeCommit} ·
APK SHA-256 ${artifact.apkSha256} · renderer ${artifact.renderer} ·
system image ${artifact.systemImageFingerprint} · capture container image ${artifact.containerImageId}.</p>`;
//endregion

//region Offline review preserves the accepted indicator and keeps observations optional
const template = readFileSync(
  join(
    questions,
    'scan-indicator.template.html'
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
    holds: template.split(`__SCAN_INDICATOR_${slot}__`)
      .length
      === 2,
    detail: `template needs exactly one ${slot} slot`
  });
}
const html = template.replace(
  '__SCAN_INDICATOR_FINDINGS__',
  () => findings
)
  .replace(
  '__SCAN_INDICATOR_PROVENANCE__',
  () => provenance
  )
  .replace(
  '__SCAN_INDICATOR_IMAGES__',
  () => JSON.stringify(images)
    .replaceAll(
    '<',
    String.raw`\u003c`
    )
  );
const output = join(
  questions,
  'scan-indicator.html'
);
if (process.argv[2] === 'build') {
  writeFileSync(
    output,
    html
  );
  console.log('Built offline native scan-indicator evidence from exact inspected witnesses.');
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
    'D26 is settled; no new preference ballot',
    'Every progress value is authored debug input',
    'single-line ellipsis',
    'id="inspection-findings"',
    'not presented as visible',
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
    holds: !/__SCAN_INDICATOR_[A-Z_]+__/.test(html),
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
  for (const capture of active) {
    need({
      rule: 'label-clearance-pixels',
      holds: clearanceMatchesPixels(capture),
      detail: `${capture.file}: stated label clearance differs from the embedded pixels`
    });
  }
  console.log('Validated exact offline native scan-indicator evidence, authored-state limits and label clearance pixels.');
} else {
  throw new Error('Expected build or validate.');
}
//endregion
