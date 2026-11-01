// The final composition: bucket-first allocation with the FLAC bones+even+zoom hybrid.
// Buckets from embedded tags only. FLAC probes are guided by frame-size profiles
// (out/flac-profiles.jsonl from flac-bones.mjs extract); lossy buckets use plain zoom.
// Searches small per-bucket (coverage, margin) grids under the quarter budget.
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
const PASS1 = 0.1;
const FLAC_EVEN = 0.05;
const FLAC_BONES_TOP = 40;

async function loadJsonl(path) {
  const rows = [];
  const rl = createInterface({
    input: createReadStream(path),
    crlfDelay: Infinity
  });
  for await (const line of rl) {
    if (line.trim()) rows.push(JSON.parse(line));
  }
  return rows;
}

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const tags = new Map((await loadJsonl(new URL(
  '../out/tags-full.jsonl',
  import.meta.url
).pathname)).map((r) => [
  r.path,
  r
]));
const profiles = new Map(
  (await loadJsonl(new URL(
    '../out/flac-profiles.jsonl',
    import.meta.url
  ).pathname)).map((r) => [
    r.path,
    r.bytes
  ]),
);
console.log(`tags ${tags.size}, flac profiles ${profiles.size}`);

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

const longs = tracks.filter((t) => t.dur > SHORT_MAX);
const shortSecs = tracks.filter((t) => t.dur <= SHORT_MAX)
  .reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
  );
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
function secsOf(b) { return (byBucket.get(b) ?? []).reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
); }

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

/** Zoom with optional bones seed slots and an even pass; returns loudest decoded bin. */
function probe(
  t,
  {
    cTotal,
    evenCov,
    bonesSlots
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
  if (bonesSlots) {
    for (const slot of bonesSlots) {
      for (let j = Math.max(
        0,
        slot - 1
      ); j <= Math.min(
        n - 1,
        slot + 1
      ); j += 1) {
        if ((!decoded[j]) && (used < budgetBins)) decode(j);
      }
    }
  }
  const count = Math.max(
    1,
    Math.round(evenCov * n)
  );
  const span = n - 1;
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
    peakDb: db(peak),
    usedSecs: used * t.binSecs
  };
}

/** Top-N byte slots for a flac track, or null when no profile exists. */
function bonesFor(t) {
  const bytes = profiles.get(t.path);
  if (!bytes) return null;
  const m = Math.min(
    bytes.length,
    t.bins
      .length
  );
  return Array.from(
    { length: m },
    (
      _,
      i
    ) => i
  )
    .sort((
      a,
      b
    ) => bytes[b] - bytes[a])
    .slice(
    0,
    FLAC_BONES_TOP
    );
}

// Precompute probes per coverage per bucket style.
const FLAC_COVS = [
  0.07,
  0.09,
  0.11,
  0.13
];
const LOSSY_COVS = [
  0.14,
  0.18,
  0.22,
  0.26,
  0.28,
  0.3,
  0.32,
  0.34
];
const probesByPath = new Map();
for (const [b, list] of byBucket) {
  for (const t of list) {
    const per = new Map();
    if (b === 'flac') {
      const bones = bonesFor(t);
      for (const c of FLAC_COVS) {
        per.set(
          c,
          probe(
            t,
            {
              cTotal: c,
              evenCov: bones ? FLAC_EVEN : Math.min(
                PASS1,
                c
              ),
              bonesSlots: bones
            }
          )
        );
      }
    } else {
      for (const c of LOSSY_COVS) per.set(
        c,
        probe(
          t,
          {
            cTotal: c,
            evenCov: Math.min(
              PASS1,
              c
            )
          }
        )
      );
    }
    probesByPath.set(
      t.path,
      per
    );
  }
}
console.log('probes precomputed');

function evaluate(assign) {
  let decoded = shortSecs;
  let quietSum = 0;
  let worst = 0;
  const clampsByBucket = {};
  for (const [b, list] of byBucket) {
    const {
      c,
      m
    } = assign[b];
    for (const t of list) {
      const {
        peakDb,
        usedSecs
      } = probesByPath.get(t.path)
        .get(c);
      decoded += usedSecs;
      const fullDb = db(t.full);
      if ((fullDb > CEILING_DB) && ((fullDb - peakDb
        - m) > TOO_LOUD)) clampsByBucket[b] = (clampsByBucket[b] ?? 0) + 1;
      const { quietDb } = gainErrors({
        fullDb,
        probeDb: peakDb,
        marginDb: m,
        ceilingDb: CEILING_DB
      });
      quietSum += quietDb;
      worst = Math.max(
        worst,
        quietDb
      );
    }
  }
  const clamps = Object.values(clampsByBucket)
    .reduce(
    (
      s,
      v
    ) => s + v,
    0
    );
  return {
    decoded,
    clamps,
    clampsByBucket,
    avg: quietSum / tracks.length,
    worst
  };
}

const results = [];
for (const cFlac of FLAC_COVS) {
  for (const mFlac of [
    0.4,
    0.45,
    0.5
  ]) {
    for (const cPurl of [
      0.14,
      0.18,
      0.22
    ]) {
      for (const mPurl of [
        0.4,
        0.5
      ]) {
        for (const cStore of [
          0.28,
          0.32
        ]) {
          for (const mStore of [
            0.3,
            0.4
          ]) {
            const fixed = shortSecs + (cFlac
              * secsOf('flac'))
              + (cPurl
                * secsOf('purl'))
              + (cStore
                * secsOf('store'));
            const cBareExact = (BUDGET - fixed) / secsOf('bare');
            const cBare = [...LOSSY_COVS].reverse()
              .find((c) => c <= cBareExact);
            if ((!cBare) || (cBare < 0.2)) continue;
            for (const mBare of [
              0.4,
              0.45,
              0.5
            ]) {
              const assign = {
                flac: {
                  c: cFlac,
                  m: mFlac
                },
                purl: {
                  c: cPurl,
                  m: mPurl
                },
                store: {
                  c: cStore,
                  m: mStore
                },
                bare: {
                  c: cBare,
                  m: mBare
                },
              };
              const r = evaluate(assign);
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
function show(r) {
  const a = Object.entries(r.assign)
    .map(([b, {
      c,
      m
    }]) => `${b}=(${c},${m})`)
    .join(' ');
  console.log(
    `  clamps=${r.clamps} ${JSON.stringify(r.clampsByBucket)} avg=${r.avg
      .toFixed(3)} worst=${r.worst
        .toFixed(2)} dec=${r.decoded
          .toFixed(0)} | ${a}`,
  );
}
results.sort((
  a,
  b
) => (a.clamps - b.clamps) || (a.avg - b.avg));
console.log('\ntop by (clamps, avg):');
for (const r of results.slice(
  0,
  6
)) show(r);
const w4 = results.filter((r) => Object.values(r.assign)
  .every(({ m }) => m <= 0.4));
w4.sort((
  a,
  b
) => (a.clamps - b.clamps) || (a.avg - b.avg));
console.log('top with all margins <= 0.4:');
for (const r of w4.slice(
  0,
  4
)) show(r);
