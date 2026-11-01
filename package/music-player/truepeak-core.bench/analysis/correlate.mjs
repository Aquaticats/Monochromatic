// Encoding-bones correlation and composed-policy evaluation. Requires out/byte-profiles.jsonl
// (per track: slotSecs 0.1, bytes[] from container framing, no decoding). Tests whether the
// crest's slot ranks high in byte-rate, then simulates the composed probe:
// bones-targeted slots + even pass-1 + frontier zoom, all within the quarter budget.
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';
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

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const safe = await loadSafePaths(new URL(
  '../out/metadata.jsonl',
  import.meta.url
).pathname);
const profiles = new Map();
{
  const rl = createInterface({
    input: createReadStream(new URL(
      '../out/byte-profiles.jsonl',
      import.meta.url
    ).pathname),
    crlfDelay: Infinity,
  });
  for await (const line of rl) {
    if (!line.trim()) continue;
    const p = JSON.parse(line);
    profiles.set(
      p.path,
      p.bytes
    );
  }
}
console.log(`profiles: ${profiles.size}`);

const shortSecs = tracks.filter((t) => t.dur <= SHORT_MAX)
  .reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
  );
const longTracks = tracks.filter((t) => t.dur > SHORT_MAX);
const loudLong = longTracks.filter((t) => db(t.full) > CEILING_DB);

//region Shared probe pieces
function sampledEvenPeak(
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
//endregion

//region Correlation: where does the crest's slot rank in the byte profile?
// For each profiled loud long track: the crest bin's byte-rank percentile, taking the best
// (largest) byte count within ±pad slots of the crest to absorb timeline skew.
for (const pad of [
  0,
  1,
  2
]) {
  const pct = [];
  const tailPct = [];
  for (const t of loudLong) {
    const bytes = profiles.get(t.path);
    if (!bytes) continue;
    let ci = 0;
    for (let j = 1; j
      < t.bins
        .length; j += 1) if (t.bins[j] > t.bins[ci]) ci = j;
    const m = Math.min(
      bytes.length,
      t.bins
        .length
    );
    if (ci >= m) continue;
    let target = 0;
    for (let j = Math.max(
      0,
      ci - pad
    ); j <= Math.min(
      m - 1,
      ci + pad
    ); j += 1) {
      target = Math.max(
        target,
        bytes[j]
      );
    }
    let higher = 0;
    for (let j = 0; j < m; j += 1) if (bytes[j] > target) higher += 1;
    const p = higher / m;
    pct.push(p);
    const ur = db(t.full) - db(sampledEvenPeak(
      t,
      0.2,
      3
    ));
    if ((ur - 0.8) > TOO_LOUD) tailPct.push(p);
  }
  function fmt(v) { return [
    0.5,
    0.9,
    0.95
  ].map((f) => `p${f * 100}=${(quantile(
    v,
    f
  ) * 100).toFixed(1)}%`)
    .join(' '); }
  console.log(`crest slot byte-rank percentile (pad ±${pad}): all(${pct.length}) ${fmt(pct)} | tail(${tailPct.length}) ${fmt(tailPct)}`);
}
//endregion

//region Shared probe pieces (continued)
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

/**
 Composed probe: bones top-B byte slots (±pad) decoded first, then even pass-1 at c1,
 then frontier zoom until the per-track bin budget is spent.
 */
function composedProbe(
  t,
  {
    bytes,
    bonesTop,
    bonesPad,
    c1,
    cTotal
  }
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
  if (bytes && (bonesTop > 0)) {
    const m = Math.min(
      bytes.length,
      n
    );
    const order = Array.from(
      { length: m },
      (
        _,
        i
      ) => i
    )
      .sort((
        a,
        b
      ) => bytes[b] - bytes[a]);
    for (const slot of order.slice(
      0,
      bonesTop
    )) {
      for (let j = Math.max(
        0,
        slot - bonesPad
      ); j <= Math.min(
        n - 1,
        slot + bonesPad
      ); j += 1) {
        if ((!decoded[j]) && (used < budgetBins)) decode(j);
      }
    }
  }
  for (const i of evenIndices(
    n,
    c1
  )) {
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
    usedSecs: used * t.binSecs
  };
}
//endregion

//region Composed evaluation across margins
function evaluate(
  label,
  params
) {
  let decoded = shortSecs;
  const rows = [];
  for (const t of longTracks) {
    const {
      peak,
      usedSecs
    } = composedProbe(
      t,
      {
        bytes: profiles.get(t.path),
        ...params
      }
    );
    decoded += usedSecs;
    rows.push({
      fullDb: db(t.full),
      probeDb: db(peak),
      safe: safe.has(t.path)
    });
  }
  console.log(`\n${label}: decoded=${decoded.toFixed(0)}s ${decoded <= BUDGET ? 'IN' : 'OVER'}`);
  const loud = rows.filter((r) => r.fullDb > CEILING_DB);
  const urs = loud.map((r) => r.fullDb - r.probeDb);
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
  console.log(`  ur ${line}`);
  for (const margin of [
    0.2,
    0.3,
    0.4,
    0.5,
    0.8
  ]) {
    const clamped = loud.filter((r) => (r.fullDb - r.probeDb
      - margin) > TOO_LOUD);
    let quietSum = 0;
    let quietWorst = 0;
    for (const r of rows) {
      const { quietDb } = gainErrors({
        fullDb: r.fullDb,
        probeDb: r.probeDb,
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
      `  m=${margin}: clamps=${clamped.length} (safe ${clamped.filter((r) => r.safe)
        .length}) avgQuiet=${(quietSum / tracks.length).toFixed(3)} worst=${quietWorst.toFixed(2)}`,
    );
  }
}

const longSecs = longTracks.reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
);
const cMax = ((BUDGET - shortSecs) / longSecs) - 0.0001;
evaluate(
  'zoom only (reference)',
  {
    bonesTop: 0,
    bonesPad: 0,
    c1: 0.1,
    cTotal: cMax
  }
);
for (const bonesTop of [
  10,
  20,
  40
]) {
  for (const bonesPad of [
    0,
    1
  ]) {
    evaluate(
      `bones top=${bonesTop} pad=${bonesPad} + zoom`,
      {
        bonesTop,
        bonesPad,
        c1: 0.1,
        cTotal: cMax
      }
    );
  }
}
//endregion
