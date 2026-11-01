// The definitive comparison for the quarter-measure answer: shipped even probe vs the
// frontier-zoom probe at the full budget, across margins, plus a provenance-split margin.
// Also lists the tracks that remain clamped under the chosen policy.
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
const PASS1_COVERAGE = 0.1;

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const safe = await loadSafePaths(new URL(
  '../out/metadata.jsonl',
  import.meta.url
).pathname);
const shortSecs = tracks.filter((t) => t.dur <= SHORT_MAX)
  .reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
  );
const longTracks = tracks.filter((t) => t.dur > SHORT_MAX);
const longSecs = longTracks.reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
);
const cTotal = ((BUDGET - shortSecs) / longSecs) - 0.0001;

function evenIndices(
  n,
  coverage
) {
  const count = Math.max(
    1,
    Math.round(coverage * n)
  );
  const span = n - 1;
  const out = new Set();
  for (let index = 0; index < count; index += 1) {
    out.add(count <= 1 ? Math.floor(span / 2) : Math.round((index / (count - 1)) * span));
  }
  return out;
}

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
    const top = h[0];
    const last = h.pop();
    if (h.length > 0) {
      h[0] = last;
      let p = 0;
      for (;;) {
        const l = (2 * p) + 1;
        const r = l + 1;
        let m = p;
        if ((l < h.length) && less(
          h[m],
          h[l]
        )) m = l;
        if ((r < h.length) && less(
          h[m],
          h[r]
        )) m = r;
        if (m === p) break;
        [h[p], h[m]] = [
          h[m],
          h[p]
        ];
        p = m;
      }
    }
    return top;
  }
  return {
    push,
    pop,
    size: () => h.length
  };
}

function zoomProbe(t) {
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
  for (const i of evenIndices(
    n,
    PASS1_COVERAGE
  )) {
    if (used >= budgetBins) break;
    decode(i);
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
    usedSecs: used * t.binSecs
  };
}

function evenProbe(
  t,
  coverage,
  windowBins
) {
  const {bins} = t;
  const n = bins.length;
  const count = Math.max(
    1,
    Math.round((coverage * n) / windowBins)
  );
  const span = Math.max(
    0,
    n - windowBins
  );
  let peak = 0;
  for (let index = 0; index < count; index += 1) {
    const start = count <= 1 ? Math.floor(span / 2) : Math.round((index / (count - 1)) * span);
    for (let j = start; j < Math.min(
      start + windowBins,
      n
    ); j += 1) peak = Math.max(
      peak,
      bins[j]
    );
  }
  return {
    peak,
    usedSecs: coverage * t.dur
  };
}

/** Evaluate probe rows across margins; marginFor picks each row's margin. */
function measure(
  rows,
  marginFor,
  label
) {
  const loud = rows.filter((r) => r.fullDb > CEILING_DB);
  const clamped = loud.filter((r) => (r.fullDb - r.probeDb
    - marginFor(r)) > TOO_LOUD);
  let quietSum = 0;
  let quietWorst = 0;
  let overWorst = 0;
  for (const r of rows) {
    const {
      quietDb,
      loudDb
    } = gainErrors({
      fullDb: r.fullDb,
      probeDb: r.probeDb,
      marginDb: marginFor(r),
      ceilingDb: CEILING_DB,
    });
    quietSum += quietDb;
    quietWorst = Math.max(
      quietWorst,
      quietDb
    );
    overWorst = Math.max(
      overWorst,
      loudDb
    );
  }
  console.log(
    `  ${label}: clamps=${clamped.length} (safe ${clamped.filter((r) => r.safe)
      .length}) ` +
      `avgQuiet=${(quietSum / tracks.length).toFixed(3)} worstQuiet=${quietWorst.toFixed(2)} worstOver=${overWorst.toFixed(2)}`,
  );
  return clamped;
}

function evaluate(
  label,
  probe
) {
  let decoded = shortSecs;
  const rows = [];
  for (const t of longTracks) {
    const {
      peak,
      usedSecs
    } = probe(t);
    decoded += usedSecs;
    rows.push({
      fullDb: db(t.full),
      probeDb: db(peak),
      safe: safe.has(t.path),
      path: t.path,
      dur: t.dur
    });
  }
  const loud = rows.filter((r) => r.fullDb > CEILING_DB);
  const urs = loud.map((r) => r.fullDb - r.probeDb);
  const line = [
    0.5,
    0.9,
    0.95,
    0.99,
    0.995,
    1
  ]
    .map((f) => `p${(f * 100).toFixed(1)}=${quantile(
      urs,
      f
    )
      .toFixed(2)}`)
    .join(' ');
  console.log(`\n${label}: decoded=${decoded.toFixed(0)}s (${((100 * decoded) / (shortSecs + longSecs)).toFixed(2)}%) ${decoded <= BUDGET ? 'IN BUDGET' : 'OVER'}`);
  console.log(`  under-read: ${line}`);
  return rows;
}

const evenRows = evaluate(
  'shipped even probe c=0.2 w=0.3s',
  (t) => evenProbe(
    t,
    0.2,
    3
  )
);
for (const m of [0.8]) measure(
  evenRows,
  () => m,
  `margin=${m} (ledger)`
);

const zoomRows = evaluate(
  `frontier zoom c1=${PASS1_COVERAGE} total=${cTotal.toFixed(4)}`,
  zoomProbe
);
for (const m of [
  0.3,
  0.4,
  0.5,
  0.8
]) measure(
  zoomRows,
  () => m,
  `margin=${m}`
);
console.log('  provenance-split margins:');
const splits = [
  {
    safeM: 0.3,
    hotM: 0.5
  },
  {
    safeM: 0.35,
    hotM: 0.5
  },
  {
    safeM: 0.3,
    hotM: 0.4
  },
];
for (const {
  safeM,
  hotM
} of splits) {
  measure(
    zoomRows,
    (r) => (r.safe ? safeM : hotM),
    `safe=${safeM}/hot=${hotM}`
  );
}

console.log('\nremaining clamped tracks at zoom margin=0.5:');
const clamped = measure(
  zoomRows,
  () => 0.5,
  'margin=0.5 (repeat)'
);
for (const r of clamped.sort((
  a,
  b
) => (b.fullDb - b.probeDb) - (a.fullDb - a.probeDb))) {
  console.log(
    `  ur=${(r.fullDb - r.probeDb).toFixed(2)} dur=${r.dur
      .toFixed(0)}s ${r.path
        .split('/')
        .at(-1)
        .slice(
      0,
      70
    )}`,
  );
}
