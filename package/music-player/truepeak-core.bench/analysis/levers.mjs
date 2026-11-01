// The remaining non-opaque levers, measured: (1) does any observable heard-statistic
// predict the zoom's residual under-read (enabling a per-track margin formula)?
// (2) does global budget reallocation (early-stop provably-safe quiet tracks, pour the
// savings into everyone else's climb) improve the measures? (3) does weighting pass-1
// away from track openings (crests skew late) help?
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
// The largest needle prominence ever observed in the corpus (probe -4.5 dBTP hiding a
// -0.5 dBTP crest, from the plan's stage-two evidence); the provable-safety early stop
// assumes no unheard crest exceeds the heard max by more than this.
const MAX_NEEDLE_DB = 4.6;

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

/** Pass-1 indices, optionally position-weighted (density multiplier per position fraction). */
function pass1Indices(
  n,
  coverage,
  weightFn
) {
  const count = Math.max(
    1,
    Math.round(coverage * n)
  );
  if (!weightFn) {
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
  // Inverse-CDF placement: integrate the weight, place samples at equal mass steps.
  const cdf = new Float64Array(n);
  let acc = 0;
  for (let i = 0; i < n; i += 1) {
    acc += weightFn(i / n);
    cdf[i] = acc;
  }
  const out = [];
  let j = 0;
  for (let k = 0; k < count; k += 1) {
    const target = ((k + 0.5) / count) * acc;
    while ((j < (n
      - 1)) && (cdf[j] < target)) j += 1;
    out.push(j);
  }
  return out;
}

/** Zoom with an injectable pass-1 and per-track bin budget. */
function zoomWith(
  t,
  pass1,
  budgetBins
) {
  const {bins} = t;
  const n = bins.length;
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
    if ((i > 0) && (!decoded[i - 1])
      && (used < budgetBins)) decode(i - 1);
    if ((i < (n
      - 1)) && (!decoded[i + 1])
      && (used < budgetBins)) decode(i + 1);
  }
  return {
    peak,
    used,
    decoded
  };
}

function report(
  label,
  rows,
  decoded,
  marginFor
) {
  const loud = rows.filter((r) => r.fullDb > CEILING_DB);
  const clamped = loud.filter((r) => (r.fullDb - r.probeDb
    - marginFor(r)) > TOO_LOUD);
  let quietSum = 0;
  let quietWorst = 0;
  for (const r of rows) {
    const { quietDb } = gainErrors({
      fullDb: r.fullDb,
      probeDb: r.probeDb,
      marginDb: marginFor(r),
      ceilingDb: CEILING_DB
    });
    quietSum += quietDb;
    quietWorst = Math.max(
      quietWorst,
      quietDb
    );
  }
  console.log(
    `${label}: decoded=${decoded.toFixed(0)}s ${decoded <= BUDGET ? 'IN' : 'OVER'} | clamps=${clamped.length} avgQuiet=${(quietSum / tracks.length).toFixed(3)} worst=${quietWorst.toFixed(2)}`,
  );
}

//region Lever 1: does any heard statistic predict the residual under-read?
{
  const cTotal = ((BUDGET - shortSecs) / longSecs) - 0.0001;
  const feats = [];
  for (const t of longs) {
    if (db(t.full) <= CEILING_DB) continue;
    const {
      peak,
      decoded
    } = zoomWith(
      t,
      pass1Indices(
        t.bins
          .length,
        0.1
      ),
      Math.max(
        1,
        Math.floor(cTotal
          * t.bins
            .length)
      )
    );
    const heard = [];
    for (let i = 0; i
      < t.bins
        .length; i += 1) if (decoded[i]) heard.push(t.bins[i]);
    heard.sort((
      a,
      b
    ) => a - b);
    function q(f) { return db(heard[Math.min(
      heard.length - 1,
      Math.round((heard.length - 1) * f)
    )]); }
    const maxDb = db(peak);
    feats.push({
      ur: db(t.full) - maxDb,
      spread90: maxDb - q(0.9),
      spread99: maxDb - q(0.99),
      level: maxDb,
      logDur: Math.log10(t.dur),
    });
  }
  // Rank correlation (Spearman) of each feature against the residual under-read.
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
  const urs = feats.map((f) => f.ur);
  for (const name of [
    'spread90',
    'spread99',
    'level',
    'logDur'
  ]) {
    console.log(`lever1 spearman(ur, ${name}) = ${spearman(
      feats.map((f) => f[name]),
      urs
    )
      .toFixed(3)}`);
  }
  // The needle check: among the top-40 residual tracks, where do their features sit?
  const sorted = [...feats].sort((
    a,
    b
  ) => b.ur - a.ur);
  const tail = sorted.slice(
    0,
    40
  );
  console.log(
    `lever1 tail spread90: median=${quantile(
      tail.map((f) => f.spread90),
      0.5
    )
      .toFixed(2)} vs all median=${quantile(
        feats.map((f) => f.spread90),
        0.5
      )
        .toFixed(2)}`,
  );
}
//endregion

//region Lever 2: global reallocation with the provable-safety early stop
{
  // Pass one everywhere; tracks whose heard max is so quiet that even the corpus's worst
  // needle could not lift them above the ceiling stop early; their saved bins fund a
  // deeper climb for everyone else, allocated proportionally.
  const cTotal = ((BUDGET - shortSecs) / longSecs) - 0.0001;
  const pass1Cov = 0.1;
  let savedBins = 0;
  const risky = [];
  const rows = [];
  for (const t of longs) {
    const n = t.bins
      .length;
    const p1 = pass1Indices(
      n,
      pass1Cov
    );
    const { peak } = zoomWith(
      t,
      p1,
      p1.length
    );
    const heardDb = db(peak);
    if ((heardDb + MAX_NEEDLE_DB) <= CEILING_DB) {
      savedBins += Math.floor(cTotal * n) - p1.length;
      rows.push({
        fullDb: db(t.full),
        probeDb: heardDb,
        safe: safe.has(t.path)
      });
    } else {
      risky.push(t);
    }
  }
  const riskyBins = risky.reduce(
    (
      s,
      t
    ) => s
      + t.bins
        .length,
    0
  );
  const extra = savedBins / riskyBins;
  let decoded = shortSecs;
  for (const t of longs) {
    if (!risky.includes(t)) {
      decoded += Math.max(
        1,
        Math.round(pass1Cov
          * t.bins
            .length)
      ) * t.binSecs;
    }
  }
  for (const t of risky) {
    const n = t.bins
      .length;
    const budgetBins = Math.max(
      1,
      Math.floor((cTotal + extra) * n)
    );
    const {
      peak,
      used
    } = zoomWith(
      t,
      pass1Indices(
        n,
        pass1Cov
      ),
      budgetBins
    );
    decoded += used * t.binSecs;
    rows.push({
      fullDb: db(t.full),
      probeDb: db(peak),
      safe: safe.has(t.path)
    });
  }
  console.log(`lever2: early-stopped ${longs.length - risky.length} of ${longs.length} long tracks, saved ${(savedBins * 0.1).toFixed(0)}s, extra coverage +${extra.toFixed(4)}`);
  for (const m of [
    0.4,
    0.5
  ]) report(
    `lever2 realloc m=${m}`,
    rows,
    decoded,
    () => m
  );
}
//endregion

//region Lever 3: position-weighted pass-1 (crests skew late; open sparse, close dense)
{
  const cTotal = ((BUDGET - shortSecs) / longSecs) - 0.0001;
  // Density: half weight in the first fifth, ramping to full weight after.
  function weight(x) { return x < 0.2 ? 0.5 : 1; }
  let decoded = shortSecs;
  const rows = [];
  for (const t of longs) {
    const n = t.bins
      .length;
    const {
      peak,
      used
    } = zoomWith(
      t,
      pass1Indices(
        n,
        0.1,
        weight
      ),
      Math.max(
        1,
        Math.floor(cTotal * n)
      )
    );
    decoded += used * t.binSecs;
    rows.push({
      fullDb: db(t.full),
      probeDb: db(peak),
      safe: safe.has(t.path)
    });
  }
  for (const m of [
    0.4,
    0.5
  ]) report(
    `lever3 position-weighted m=${m}`,
    rows,
    decoded,
    () => m
  );
}
//endregion
