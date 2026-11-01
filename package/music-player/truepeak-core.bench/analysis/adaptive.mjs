// Adaptive within-track sampling experiments: can spending part of the budget zooming
// around loud heard windows beat the evenly-placed probe? Anatomy says the worst tracks
// are isolated needles, so zoom should help the mid-tail, not the extreme tail.
import {
  db,
  gainErrors,
  loadTracks,
  quantile
} from './corpus.mjs';

const SHORT_MAX = 90;
const CEILING_DB = -1;
const TOO_LOUD = 0.5;
const BUDGET = 229_215.18996182326;

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
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
// The most coverage the budget affords when every long track gets the same fraction.
const maxCoverage = (BUDGET - shortSecs) / longSecs;
console.log(`short=${shortSecs.toFixed(0)}s long=${longSecs.toFixed(0)}s maxCoverage=${maxCoverage.toFixed(4)}`);

/** Evenly spaced single-bin sample indices at `coverage` (replicates shipped placement shape). */
function evenIndices(
  n,
  coverage
) {
  const count = Math.max(
    1,
    Math.round(coverage * n)
  );
  const span = n - 1;
  const indices = new Set();
  for (let index = 0; index < count; index += 1) {
    indices.add(count <= 1 ? Math.floor(span / 2) : Math.round((index / (count - 1)) * span));
  }
  return indices;
}

/** Even probe with contiguous windows of `windowBins` (shipped 0.3s = 3 bins). */
function evenWindowsMax(
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
  return peak;
}

/** Simple binary max-heap over bin indices keyed by bin loudness. */
function makeHeap(bins) {
  const heap = [];
  function less(
    a,
    b
  ) { return bins[a] < bins[b]; }
  function push(i) {
    heap.push(i);
    let c = heap.length - 1;
    while (c > 0) {
      const p = (c - 1) >> 1;
      if (less(
        heap[p],
        heap[c]
      )) {
        [heap[p], heap[c]] = [
          heap[c],
          heap[p]
        ];
        c = p;
      } else break;
    }
  }
  function pop() {
    const top = heap[0];
    const last = heap.pop();
    if (heap.length > 0) {
      heap[0] = last;
      let p = 0;
      for (;;) {
        const l = (2 * p) + 1;
        const r = l + 1;
        let biggest = p;
        if ((l < heap.length) && less(
          heap[biggest],
          heap[l]
        )) biggest = l;
        if ((r < heap.length) && less(
          heap[biggest],
          heap[r]
        )) biggest = r;
        if (biggest === p) break;
        [heap[p], heap[biggest]] = [
          heap[biggest],
          heap[p]
        ];
        p = biggest;
      }
    }
    return top;
  }
  return {
    push,
    pop,
    size: () => heap.length
  };
}

/**
 Frontier zoom: pass-1 even single bins at c1, then repeatedly decode the undecoded
 neighbors of the loudest decoded bin until the per-track bin budget is exhausted.
 */
function zoomMax(
  t,
  c1,
  cTotal
) {
  const {bins} = t;
  const n = bins.length;
  const budgetBins = Math.max(
    1,
    Math.round(cTotal * n)
  );
  const decoded = new Uint8Array(n);
  const heap = makeHeap(bins);
  let used = 0;
  let peak = 0;
  function decode(i) {
    decoded[i] = 1;
    used += 1;
    peak = Math.max(
      peak,
      bins[i]
    );
    heap.push(i);
  }
  for (const i of evenIndices(
    n,
    c1
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

/** Evaluate a policy (fn: track -> {peak, usedSecs}) over the corpus and print measures. */
function evaluate(
  label,
  probe
) {
  let decoded = shortSecs;
  const urs = [];
  const perTrack = [];
  for (const t of longTracks) {
    const {
      peak,
      usedSecs
    } = probe(t);
    decoded += usedSecs;
    const fullDb = db(t.full);
    const probeDb = db(peak);
    perTrack.push({
      fullDb,
      probeDb
    });
    if (fullDb > CEILING_DB) urs.push(fullDb - probeDb);
  }
  const inBudget = decoded <= (BUDGET
    + 1) ? 'IN' : 'OVER';
  const line = [
    0.5,
    0.9,
    0.99,
    1
  ].map((f) => `p${f * 100}=${quantile(
    urs,
    f
  )
    .toFixed(2)}`)
    .join(' ');
  console.log(`\n${label}: decoded=${decoded.toFixed(0)}s ${inBudget} | ur ${line}`);
  for (const margin of [
    0.3,
    0.4,
    0.5,
    0.8,
    1,
    1.2
  ]) {
    const clamped = urs.filter((u) => (u - margin) > TOO_LOUD)
      .length;
    let quietSum = 0;
    let quietWorst = 0;
    for (const {
      fullDb,
      probeDb
    } of perTrack) {
      const { quietDb } = gainErrors({
        fullDb,
        probeDb,
        marginDb: margin,
        ceilingDb: CEILING_DB
      });
      quietSum += quietDb;
      quietWorst = Math.max(
        quietWorst,
        quietDb
      );
    }
    console.log(
      `  margin=${margin.toFixed(1)}: clamped=${clamped} avgQuiet(all ${tracks.length})=${(quietSum / tracks.length).toFixed(3)} worstQuiet=${quietWorst.toFixed(2)}`,
    );
  }
}

// Baseline shape at shipped coverage, then at the full budget; window granularity effect;
// then zoom with several pass-1/total splits at the full budget.
evaluate(
  'even w=0.3s c=0.20 (shipped)',
  (t) => ({
    peak: evenWindowsMax(
      t,
      0.2,
      3
    ),
    usedSecs: 0.2 * t.dur,
  })
);
evaluate(
  `even w=0.3s c=${maxCoverage.toFixed(3)} (budget-max)`,
  (t) => ({
    peak: evenWindowsMax(
      t,
      maxCoverage,
      3
    ),
    usedSecs: maxCoverage * t.dur,
  })
);
evaluate(
  `even w=0.1s c=${maxCoverage.toFixed(3)} (budget-max, single-bin)`,
  (t) => ({
    peak: evenWindowsMax(
      t,
      maxCoverage,
      1
    ),
    usedSecs: maxCoverage * t.dur,
  })
);
for (const c1 of [
  0.05,
  0.1,
  0.15,
  0.2
]) {
  evaluate(
    `zoom c1=${c1} total=${maxCoverage.toFixed(3)}`,
    (t) => zoomMax(
      t,
      c1,
      maxCoverage
    )
  );
}
