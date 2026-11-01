// Placement and estimator variants: (1) does the even pass-1 grid alias against musical
// structure (test stratified golden-ratio jitter)? (2) do wider zoom expansions help?
// (3) does heard-clipping density (fraction of heard bins at digital full scale) predict
// the residual under-read, enabling a margin split inside the lossy bucket?
import {
  db,
  gainErrors,
  loadSafePaths,
  loadTracks,
  quantile
} from './corpus.mjs';

const SHORT_MAX = 90;
const CEILING_DB = -1;
const TOO_LOUD = 0.5;
const BUDGET = 229_215.18996182326;
const GOLDEN = (Math.sqrt(5) - 1) / 2;

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const safe = await loadSafePaths(new URL(
  '../out/metadata.jsonl',
  import.meta.url
).pathname);
const shorts = tracks.filter((t) => t.dur <= SHORT_MAX);
const longs = tracks.filter((t) => t.dur > SHORT_MAX);
const shortSecs = shorts.reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
);
const longSecs = longs.reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
);
const cTotal = ((BUDGET - shortSecs) / longSecs) - 0.0001;

function makeHeap(bins) {
  const h = [];
  function less(
    a,
    b
  ) { return bins[a] < bins[b]; }
  function push(i) {
    h.push(i);
    let c = h.length - 1;
    while (c > 0) {
      const p = (c - 1) >> 1;
      if (less(
        h[p],
        h[c]
      )) {
        [h[p], h[c]] = [
          h[c],
          h[p]
        ];
        c = p;
      } else break;
    }
  }
  function pop() {
    const t = h[0];
    const l = h.pop();
    if (h.length > 0) {
      h[0] = l;
      let p = 0;
      for (;;) {
        const a = (2 * p) + 1;
        const b = a + 1;
        let m = p;
        if ((a < h.length) && less(
          h[m],
          h[a]
        )) m = a;
        if ((b < h.length) && less(
          h[m],
          h[b]
        )) m = b;
        if (m === p) break;
        [h[p], h[m]] = [
          h[m],
          h[p]
        ];
        p = m;
      }
    }
    return t;
  }
  return {
    push,
    pop,
    size: () => h.length
  };
}

/** Grid pass-1 (the shipped shape). */
function gridIndices(
  n,
  coverage
) {
  const count = Math.max(
    1,
    Math.round(coverage * n)
  );
  const span = n - 1;
  return Array.from(
    { length: count },
    (
      _,
      i
    ) =>
      count <= 1 ? Math.floor(span / 2) : Math.round((i / (count - 1)) * span)
  );
}

/** Stratified golden-ratio jitter: one sample per stratum at a rotating offset. */
function jitterIndices(
  n,
  coverage
) {
  const count = Math.max(
    1,
    Math.round(coverage * n)
  );
  const stratum = n / count;
  const out = [];
  for (let k = 0; k < count; k += 1) {
    const offset = (k * GOLDEN) % 1;
    out.push(Math.min(
      n - 1,
      Math.floor((k + offset) * stratum)
    ));
  }
  return out;
}

/** Zoom with pluggable pass-1 and expansion radius. */
function zoom(
  t,
  pass1,
  radius
) {
  const {bins} = t;
  const n = bins.length;
  const budgetBins = Math.max(
    1,
    Math.floor(cTotal * n)
  );
  const decoded = new Uint8Array(n);
  const heap = makeHeap(bins);
  let used = 0;
  let peak = 0;
  function decode(i) {
    decoded[i] = 1;
    used += 1;
    if (bins[i] > peak) peak = bins[i];
    heap.push(i);
  }
  for (const i of pass1) {
    if (used >= budgetBins) break;
    if (!decoded[i]) decode(i);
  }
  while ((used < budgetBins) && (heap.size() > 0)) {
    const i = heap.pop();
    for (let d = 1; d <= radius; d += 1) {
      if (((i - d) >= 0) && (!decoded[i - d])
        && (used < budgetBins)) decode(i - d);
      if (((i + d) < n) && (!decoded[i + d])
        && (used < budgetBins)) decode(i + d);
    }
  }
  return {
    peak,
    used,
    decoded
  };
}

function report(
  label,
  probeFn
) {
  let decoded = shortSecs;
  const rows = [];
  for (const t of longs) {
    const {
      peak,
      used
    } = probeFn(t);
    decoded += used * t.binSecs;
    rows.push({
      fullDb: db(t.full),
      probeDb: db(peak),
      safe: safe.has(t.path)
    });
  }
  const loud = rows.filter((r) => r.fullDb > CEILING_DB);
  const urs = loud.map((r) => r.fullDb - r.probeDb);
  const pctl = [
    0.9,
    0.99,
    1
  ].map((f) => `p${f * 100}=${quantile(
    urs,
    f
  )
    .toFixed(2)}`)
    .join(' ');
  const parts = [
    0.4,
    0.5
  ].map((m) => {
    const clamps = loud.filter((r) => (r.fullDb - r.probeDb
      - m) > TOO_LOUD)
      .length;
    let qs = 0;
    for (const r of rows) {
      qs += gainErrors({
        fullDb: r.fullDb,
        probeDb: r.probeDb,
        marginDb: m,
        ceilingDb: CEILING_DB
      })
        .quietDb;
    }
    return `m=${m}: clamps=${clamps} avg=${(qs / tracks.length).toFixed(3)}`;
  });
  console.log(`${label}: decoded=${decoded.toFixed(0)}s ${decoded <= BUDGET ? 'IN' : 'OVER'} | ${pctl} | ${parts.join(' | ')}`);
}

report(
  'grid r=1 (answer)',
  (t) => zoom(
    t,
    gridIndices(
      t.bins
        .length,
      0.1
    ),
    1
  )
);
report(
  'jitter r=1',
  (t) => zoom(
    t,
    jitterIndices(
      t.bins
        .length,
      0.1
    ),
    1
  )
);
report(
  'grid r=2',
  (t) => zoom(
    t,
    gridIndices(
      t.bins
        .length,
      0.1
    ),
    2
  )
);
report(
  'jitter r=2',
  (t) => zoom(
    t,
    jitterIndices(
      t.bins
        .length,
      0.1
    ),
    2
  )
);
report(
  'grid r=3',
  (t) => zoom(
    t,
    gridIndices(
      t.bins
        .length,
      0.1
    ),
    3
  )
);

// Heard-clipping density: does the fraction of heard bins at/above near-full-scale
// predict the residual under-read (within the non-lossless bucket)?
{
  const CLIP_LINEAR = 0.985;
  const feats = [];
  for (const t of longs) {
    if (db(t.full) <= CEILING_DB) continue;
    const {
      peak,
      decoded
    } = zoom(
      t,
      gridIndices(
        t.bins
          .length,
        0.1
      ),
      1
    );
    let heard = 0;
    let clipped = 0;
    for (let i = 0; i
      < t.bins
        .length; i += 1) {
      if (!decoded[i]) continue;
      heard += 1;
      if (t.bins[i] >= CLIP_LINEAR) clipped += 1;
    }
    feats.push({
      ur: db(t.full) - db(peak),
      clipDensity: clipped / heard,
      safe: safe.has(t.path)
    });
  }
  const lossy = feats.filter((f) => !f.safe);
  function spearman(
    xs,
    ys
  ) {
    function rank(vs) {
      const order = vs.map((
        v,
        i
      ) => [
        v,
        i
      ])
        .sort((
          a,
          b
        ) => a[0] - b[0]);
      const ranks = new Array(vs.length);
      order.forEach((
        [, i],
        r
      ) => {
        ranks[i] = r;
      });
      return ranks;
    }
    const rx = rank(xs);
    const ry = rank(ys);
    const n = xs.length;
    const mx = (n - 1) / 2;
    let cov = 0;
    let vx = 0;
    let vy = 0;
    for (let i = 0; i < n; i += 1) {
      cov += (rx[i] - mx) * (ry[i] - mx);
      vx += (rx[i] - mx) ** 2;
      vy += (ry[i] - mx) ** 2;
    }
    return cov / Math.sqrt(vx * vy);
  }
  console.log(`clipDensity spearman vs ur (lossy, n=${lossy.length}): ${spearman(
    lossy.map((f) => f.clipDensity),
    lossy.map((f) => f.ur)
  )
    .toFixed(3)}`);
  // The clamp tail's clip density vs the bucket's.
  const tail = lossy.filter((f) => f.ur > 1);
  console.log(
    `clipDensity median: lossy all=${quantile(
      lossy.map((f) => f.clipDensity),
      0.5
    )
      .toFixed(3)} tail(ur>1)=${quantile(
        tail.map((f) => f.clipDensity),
        0.5
      )
        .toFixed(3)}`,
  );
}
