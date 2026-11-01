// Bucket-first policies: join the corpus with the embedded-tag sweep, measure each
// bucket's zoom under-read tail across coverages, then search small per-bucket
// (coverage, margin) tables under the global budget for a composite that beats the
// uniform zoom answer.
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';
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
const tags = new Map();
{
  const rl = createInterface({
    input: createReadStream(new URL(
      '../out/tags-full.jsonl',
      import.meta.url
    ).pathname),
    crlfDelay: Infinity,
  });
  for await (const line of rl) {
    if (!line.trim()) continue;
    const r = JSON.parse(line);
    tags.set(
      r.path,
      r
    );
  }
}

/** Bucket assignment from zero-cost, production-legal observables (no path text). */
function bucketOf(t) {
  const g = tags.get(t.path);
  const ext = t.path
    .slice(t.path
      .lastIndexOf('.')
      + 1)
    .toLowerCase();
  const codec = g?.codec ?? (ext === 'flac' ? 'flac' : (ext === 'm4a') || (ext === 'mp4') ? 'aac' : ext);
  if (codec === 'flac') return 'flac';
  if (g?.hasStoreIds || g?.hasItunNorm) return 'store';
  if (g?.hasPurl) return 'purl';
  return 'bare';
}

const shorts = tracks.filter((t) => t.dur <= SHORT_MAX);
const longs = tracks.filter((t) => t.dur > SHORT_MAX);
const shortSecs = shorts.reduce(
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

function zoom(
  t,
  cTotal
) {
  const {bins} = t;
  const n = bins.length;
  const budgetBins = Math.max(
    1,
    Math.floor(cTotal * n)
  );
  const count = Math.max(
    1,
    Math.round(0.1 * n)
  );
  const span = n - 1;
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
  for (let k = 0; (k < count) && (used < budgetBins); k += 1) {
    const i = count <= 1 ? Math.floor(span / 2) : Math.round((k / (count - 1)) * span);
    if (!decoded[i]) decode(i);
  }
  while ((used < budgetBins) && (heap.size() > 0)) {
    const i = heap.pop();
    if ((i > 0) && (!decoded[i - 1])
      && (used < budgetBins)) decode(i - 1);
    if (((i + 1) < n) && (!decoded[i + 1])
      && (used < budgetBins)) decode(i + 1);
  }
  return {
    peak,
    used
  };
}

// Precompute per-track probe results at each candidate coverage (coarse grid).
const COVERAGES = [
  0.1,
  0.14,
  0.18,
  0.2398,
  0.28,
  0.32
];
const byBucket = new Map();
for (const t of longs) {
  const b = bucketOf(t);
  if (!byBucket.has(b)) byBucket.set(
    b,
    []
  );
  byBucket.get(b)
    .push(t);
}
console.log('long-track buckets (count, seconds, loud count):');
for (const [b, list] of byBucket) {
  const secs = list.reduce(
    (
      s,
      t
    ) => s + t.dur,
    0
  );
  const loud = list.filter((t) => db(t.full) > CEILING_DB)
    .length;
  console.log(`  ${b}: n=${list.length} secs=${secs.toFixed(0)} loud=${loud}`);
}

const probes = new Map();
for (const t of longs) {
  const per = new Map();
  for (const c of COVERAGES) per.set(
    c,
    db(zoom(
      t,
      c
    )
      .peak)
  );
  probes.set(
    t.path,
    per
  );
}

console.log('\nper-bucket zoom tails by coverage (loud tracks): p90/p99/max under-read; clamps at m=0.4/0.5');
for (const [b, list] of byBucket) {
  const loud = list.filter((t) => db(t.full) > CEILING_DB);
  for (const c of COVERAGES) {
    const urs = loud.map((t) => db(t.full)
      - probes.get(t.path)
        .get(c));
    const clamps4 = urs.filter((u) => (u - 0.4) > TOO_LOUD)
      .length;
    const clamps5 = urs.filter((u) => (u - 0.5) > TOO_LOUD)
      .length;
    console.log(
      `  ${b} c=${c}: p90=${quantile(
        urs,
        0.9
      )
        .toFixed(2)} p99=${quantile(
          urs,
          0.99
        )
          .toFixed(2)} max=${quantile(
            urs,
            1
          )
            .toFixed(2)} | clamps m0.4=${clamps4} m0.5=${clamps5}`,
    );
  }
}

// Composite search: assign each bucket a coverage and a margin from small grids, subject
// to the global budget; keep worst-quiet at 0.5 (margins <= 0.5) and minimize clamps
// first, then average quiet. Buckets: flac, store, purl, bare.
const MARGINS = [
  0.3,
  0.4,
  0.5
];
const bucketNames = [...byBucket.keys()];
const bucketSecs = new Map(bucketNames.map((b) => [
  b,
  byBucket.get(b)
    .reduce(
    (
      s,
      t
    ) => s + t.dur,
    0
    )
]));

/** Evaluate one assignment {bucket: {c, m}}; returns measures + decoded seconds. */
function evaluateAssignment(assign) {
  let decoded = shortSecs;
  let clamps = 0;
  let quietSum = 0;
  let worstQuiet = 0;
  for (const [b, list] of byBucket) {
    const {
      c,
      m
    } = assign[b];
    for (const t of list) {
      decoded += Math.max(
        1,
        Math.floor(c
          * t.bins
            .length)
      ) * t.binSecs;
      const probeDb = probes.get(t.path)
        .get(c);
      const fullDb = db(t.full);
      if ((fullDb > CEILING_DB) && ((fullDb - probeDb
        - m) > TOO_LOUD)) clamps += 1;
      const { quietDb } = gainErrors({
        fullDb,
        probeDb,
        marginDb: m,
        ceilingDb: CEILING_DB
      });
      quietSum += quietDb;
      worstQuiet = Math.max(
        worstQuiet,
        quietDb
      );
    }
  }
  return {
    decoded,
    clamps,
    avg: quietSum / tracks.length,
    worstQuiet
  };
}

// Enumerate: flac and store/purl take low coverages and margins; bare takes what's left.
const results = [];
for (const cFlac of COVERAGES) {
  for (const cStore of COVERAGES) {
    for (const cPurl of COVERAGES) {
      // Bare bucket coverage: the largest grid value that fits the remaining budget.
      const usedFixed = shortSecs +
        (cFlac
          * bucketSecs.get('flac'))
        + (cStore
          * (bucketSecs.get('store') ?? 0))
        + (cPurl
          * (bucketSecs.get('purl') ?? 0));
      const cBareExact = (BUDGET - usedFixed) / bucketSecs.get('bare');
      const cBare = [...COVERAGES].reverse()
        .find((c) => c <= cBareExact);
      if ((!cBare) || (cBare < 0.14)) continue;
      for (const mFlac of MARGINS) {
        for (const mStore of MARGINS) {
          for (const mPurl of MARGINS) {
            for (const mBare of MARGINS) {
              const assign = {
                flac: {
                  c: cFlac,
                  m: mFlac
                },
                store: {
                  c: cStore,
                  m: mStore
                },
                purl: {
                  c: cPurl,
                  m: mPurl
                },
                bare: {
                  c: cBare,
                  m: mBare
                },
              };
              const r = evaluateAssignment(assign);
              if (r.decoded > BUDGET) continue;
              results.push({
                assign,
                ...r
              });
            }
          }
        }
      }
    }
  }
}
results.sort((
  a,
  b
) => (a.clamps - b.clamps) || (a.avg - b.avg));
console.log(`\ncomposite search: ${results.length} in-budget assignments; best by (clamps, avg):`);
for (const r of results.slice(
  0,
  8
)) {
  const a = Object.entries(r.assign)
    .map(([b, {
      c,
      m
    }]) => `${b}=(${c},${m})`)
    .join(' ');
  console.log(`  clamps=${r.clamps} avg=${r.avg
    .toFixed(3)} worst=${r.worstQuiet
      .toFixed(2)} decoded=${r.decoded
        .toFixed(0)} | ${a}`);
}
// Also the best assignments that hold average quiet under the uniform answer's 0.371.
const better = results.filter((r) => r.avg <= (0.371
  + 1e-9));
better.sort((
  a,
  b
) => (a.clamps - b.clamps) || (a.avg - b.avg));
console.log('best with avg <= 0.371 (uniform zoom m=0.5):');
for (const r of better.slice(
  0,
  5
)) {
  const a = Object.entries(r.assign)
    .map(([b, {
      c,
      m
    }]) => `${b}=(${c},${m})`)
    .join(' ');
  console.log(`  clamps=${r.clamps} avg=${r.avg
    .toFixed(3)} worst=${r.worstQuiet
      .toFixed(2)} decoded=${r.decoded
        .toFixed(0)} | ${a}`);
}
