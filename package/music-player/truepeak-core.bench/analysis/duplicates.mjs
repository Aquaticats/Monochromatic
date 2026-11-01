// Duplicate-master detection and pooled-probe simulation. The library contains duplicate
// masters (same audio ripped twice); this script quantifies how many decodable seconds are
// duplicated, whether a true-peak estimate transfers safely between copies (crest spread),
// and what redistributing the saved decode seconds buys the frontier-zoom policy.
// Detection nuance: Pearson correlation is scale-invariant, so corr-only matches can be the
// same audio at a different mastering gain; the close-bin-dB fraction is gain-sensitive, so
// frac-verified groups are the transfer-safe pooling unit and get their own simulation.
import {
  db,
  gainErrors,
  loadTracks,
  quantile
} from './corpus.mjs';

//region Constants: policy budget mirrors analysis/final.mjs; detection thresholds per task.
const SHORT_MAX = 90;
const CEILING_DB = -1;
const TOO_LOUD = 0.5;
const BUDGET = 229_215.18996182326;
const PASS1_COVERAGE = 0.1;
const COVERAGE_EPSILON = 0.0001;
/** Duration bucket width for candidate pairing (round to nearest 0.5 s). */
const BUCKET_SECS = 0.5;
/** Across adjacent buckets, only compare pairs whose durations differ by at most this. */
const ADJACENT_MAX_DIFF_SECS = 0.3;
/** Pearson correlation above which a pair counts as duplicate. */
const CORR_DUP = 0.98;
/** Fraction of close bins above which a pair counts as duplicate. */
const FRAC_DUP = 0.95;
/** Bins closer than this many dB count toward the close-bin fraction. */
const DB_DIFF_MAX = 0.25;
/** Silence floor applied to bin dB values before differencing. */
const DB_FLOOR = -60;
/** Offset-0 correlation above which a non-duplicate pair earns an offset scan. */
const CORR_NEAR = 0.9;
/** Offset-0 close-bin fraction above which a non-duplicate pair earns an offset scan. */
const FRAC_NEAR = 0.8;
/** Encoder-padding shifts to scan when offset 0 near-misses. */
const OFFSET_SCAN_BINS = 5;
/** How many of the largest duplicate groups to list. */
const TOP_GROUPS = 10;
/** Margins to evaluate, matching the final.mjs frontier-zoom rows. */
const MARGINS = [
  0.4,
  0.5
];
//endregion

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const librarySecs = tracks.reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
);
const shortSecs = tracks.filter((t) => t.dur <= SHORT_MAX)
  .reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
  );
const longSecs = tracks.filter((t) => t.dur > SHORT_MAX)
  .reduce(
  (
    s,
    t
  ) => s + t.dur,
  0
  );

//region Duplicate detection: duration-bucketed candidate pairs, bin-profile similarity,
// optional +-5-bin offset scan for encoder-padding shifts, union-find grouping.

/** Per-track bin dB profile floored at DB_FLOOR, precomputed once for the close-bin test. */
const dbBins = tracks.map((t) => Float32Array.from(
  t.bins,
  (v) => Math.max(
    db(v),
    DB_FLOOR
  )
));

/**
 Pearson correlation and close-bin fraction between two tracks' bin profiles,
 aligned at bin `offset` (bins of track ib start `offset` bins later than track ia's).
 */
function similarityAt({
  ia,
  ib,
  offset
}) {
  const a = tracks[ia]
    .bins;
  const b = tracks[ib]
    .bins;
  const aStart = Math.max(
    0,
    offset
  );
  const bStart = Math.max(
    0,
    -offset
  );
  const len = Math.min(
    a.length - aStart,
    b.length - bStart
  );
  if (len < 2) return {
    corr: 0,
    frac: 0
  };
  const dbA = dbBins[ia];
  const dbB = dbBins[ib];
  let sumA = 0;
  let sumB = 0;
  let sumAA = 0;
  let sumBB = 0;
  let sumAB = 0;
  let close = 0;
  for (let i = 0; i < len; i += 1) {
    const x = a[aStart + i];
    const y = b[bStart + i];
    sumA += x;
    sumB += y;
    sumAA += x * x;
    sumBB += y * y;
    sumAB += x * y;
    if (Math.abs(dbA[aStart + i] - dbB[bStart + i]) < DB_DIFF_MAX) close += 1;
  }
  const cov = sumAB - ((sumA * sumB)
    / len);
  const varA = sumAA - ((sumA * sumA)
    / len);
  const varB = sumBB - ((sumB * sumB)
    / len);
  const corr = (varA <= 0) || (varB <= 0) ? 0 : cov / Math.sqrt(varA * varB);
  return {
    corr,
    frac: close / len
  };
}

/** Whether a similarity result clears the duplicate thresholds. */
function isDup({
  corr,
  frac
}) { return (corr > CORR_DUP) || (frac > FRAC_DUP); }

/** Whether an offset-0 result is close enough to justify the offset scan. */
function isNearMiss({
  corr,
  frac
}) { return (corr > CORR_NEAR) || (frac > FRAC_NEAR); }

/** Array-based union-find over track indices. */
function makeUnionFind(n) {
  const parent = Array.from(
    { length: n },
    (
      _,
      i
    ) => i
  );
  function find(i) {
    let root = i;
    while (parent[root] !== root) root = parent[root];
    let cursor = i;
    while (parent[cursor] !== root) {
      const next = parent[cursor];
      parent[cursor] = root;
      cursor = next;
    }
    return root;
  }
  return {
    find,
    union: (
      a,
      b
    ) => { parent[find(a)] = find(b); }
  };
}

// ufAll merges every duplicate pair; ufFrac merges only gain-verified (close-bin) pairs.
const ufAll = makeUnionFind(tracks.length);
const ufFrac = makeUnionFind(tracks.length);
const byDur = tracks.map((
  t,
  i
) => ({
  i,
  dur: t.dur,
  bucket: Math.round(t.dur / BUCKET_SECS)
}))
  .sort((
    a,
    b
  ) => a.dur - b.dur);
let pairsCompared = 0;
let dupPairs = 0;
let offsetRescues = 0;
for (let a = 0; a < byDur.length; a += 1) {
  for (let b = a + 1; (b < byDur.length) && ((byDur[b]
    .dur
    - byDur[a]
      .dur) < BUCKET_SECS); b += 1) {
    const sameBucket = byDur[a]
      .bucket
      === byDur[b]
        .bucket;
    const adjacentClose = (Math.abs(byDur[a]
      .bucket
      - byDur[b]
        .bucket) === 1)
      && ((byDur[b]
        .dur
        - byDur[a]
          .dur) <= ADJACENT_MAX_DIFF_SECS);
    if ((!sameBucket) && (!adjacentClose)) continue;
    pairsCompared += 1;
    const pair = {
      ia: byDur[a]
        .i,
      ib: byDur[b]
        .i
    };
    const at0 = similarityAt({
      ...pair,
      offset: 0
    });
    let dup = isDup(at0);
    let fracPass = at0.frac > FRAC_DUP;
    // Offset scan: rescue near-misses shifted by encoder padding, and give corr-only
    // matches every chance to prove gain-identity before we call them unverified.
    if (((!dup) && isNearMiss(at0)) || (dup && (!fracPass))) {
      for (let offset = -OFFSET_SCAN_BINS; offset <= OFFSET_SCAN_BINS; offset += 1) {
        if (offset === 0) continue;
        const s = similarityAt({
          ...pair,
          offset
        });
        if ((!dup) && isDup(s)) {
          dup = true;
          offsetRescues += 1;
        }
        if (s.frac > FRAC_DUP) fracPass = true;
        if (dup && fracPass) break;
      }
    }
    if (dup) {
      dupPairs += 1;
      ufAll.union(
        pair.ia,
        pair.ib
      );
      if (fracPass) ufFrac.union(
        pair.ia,
        pair.ib
      );
    }
  }
}

/** Components of 2+ tracks under a union-find, members in corpus order (first = kept copy). */
function componentsOf(uf) {
  const byRoot = new Map();
  for (let i = 0; i < tracks.length; i += 1) {
    const root = uf.find(i);
    if (!byRoot.has(root)) byRoot.set(
      root,
      []
    );
    byRoot.get(root)
      .push(i);
  }
  return [...byRoot.values()].filter((m) => m.length > 1);
}

const groups = componentsOf(ufAll);
const fracGroups = componentsOf(ufFrac);
function groupSecs(members) { return members.reduce(
  (
    s,
    i
  ) => s
    + tracks[i]
      .dur,
  0
); }
const dupTracks = groups.reduce(
  (
    s,
    g
  ) => s + g.length,
  0
);
const involvedSecs = groups.reduce(
  (
    s,
    g
  ) => s + groupSecs(g),
  0
);
const dupSecs = groups.reduce(
  (
    s,
    g
  ) => (s + groupSecs(g))
    - tracks[g[0]]
      .dur,
  0
);
function basename(path) { return path.split('/')
  .at(-1); }
function extension(path) { return basename(path)
  .split('.')
  .at(-1)
  .toLowerCase(); }

console.log(`library: ${tracks.length} tracks, ${librarySecs.toFixed(0)}s`);
console.log(`candidate pairs compared: ${pairsCompared}, duplicate pairs: ${dupPairs} (offset-scan rescues: ${offsetRescues})`);
console.log(`duplicate groups: ${groups.length}, tracks involved: ${dupTracks}, seconds involved: ${involvedSecs.toFixed(0)}s`);
console.log(`duplicated seconds beyond first copies: ${dupSecs.toFixed(0)}s = ${((100 * dupSecs) / librarySecs).toFixed(2)}% of library`);
console.log(`\nlargest ${TOP_GROUPS} groups:`);
const ranked = [...groups].sort((
  a,
  b
) => (b.length - a.length) || (groupSecs(b) - groupSecs(a)));
for (const g of ranked.slice(
  0,
  TOP_GROUPS
)) {
  const exts = [...new Set(g.map((i) => extension(tracks[i]
    .path)))].join('+');
  console.log(`  n=${g.length} secs=${groupSecs(g)
    .toFixed(0)} exts=${exts} ${basename(tracks[g[0]]
      .path)
      .slice(
        0,
        70
      )}`);
}
//endregion

//region Crest spread: does a true peak measured on one copy transfer to the others?
function spreadOf(g) {
  const crests = g.map((i) => db(tracks[i]
    .full));
  return Math.max(...crests) - Math.min(...crests);
}
/** Whether every member of a duplicate group is joined by gain-verified (frac) pairs. */
function isVerified(g) { return g.every((i) => ufFrac.find(i) === ufFrac.find(g[0])); }
function splitBy(predicate) { return [
  groups.filter(predicate),
  groups.filter((g) => !predicate(g))
]; }
const [sameExt, mixedExt] = splitBy((g) => new Set(g.map((i) => extension(tracks[i]
  .path))).size === 1);
const [verified, corrOnly] = splitBy(isVerified);
function spreadLine({
  label,
  subset
}) {
  const spreads = subset.map(spreadOf);
  console.log(
    `  ${label}: groups=${subset.length} medianSpread=${quantile(
      spreads,
      1 / 2
    )
      .toFixed(3)}dB `
      + `p90=${quantile(
        spreads,
        0.9
      )
        .toFixed(3)}dB maxSpread=${(spreads.length > 0 ? Math.max(...spreads) : 0).toFixed(3)}dB`,
  );
}
console.log('\ncrest spread within groups (max-min of db(full_peak)):');
spreadLine({
  label: 'all groups        ',
  subset: groups
});
spreadLine({
  label: 'same-extension    ',
  subset: sameExt
});
spreadLine({
  label: 'mixed-extension   ',
  subset: mixedExt
});
spreadLine({
  label: 'frac-verified     ',
  subset: verified
});
spreadLine({
  label: 'corr-only         ',
  subset: corrOnly
});
//endregion

//region Zoom probe machinery, copied from analysis/final.mjs with coverage as a parameter.

/** Evenly spaced pass-1 bin indices at `coverage` of `n` bins. */
function evenIndices({
  n,
  coverage
}) {
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

/** Binary max-heap of bin indices ordered by bin value. */
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

/** Frontier-zoom probe of one track at per-track total coverage `cTotal`. */
function zoomProbe({
  track,
  cTotal
}) {
  const {bins} = track;
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
  for (const i of evenIndices({
    n,
    coverage: PASS1_COVERAGE
  })) {
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
    usedSecs: used * track.binSecs
  };
}
//endregion

//region Simulation: baseline frontier zoom vs pooled probing that shares estimates in groups.

/** Clamp count, average and worst needless-quiet, worst overshoot for estimate rows. */
function measure({
  rows,
  margin,
  label
}) {
  const clamped = rows.filter((r) => (r.fullDb > CEILING_DB) && ((r.fullDb - r.probeDb
    - margin) > TOO_LOUD));
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
      marginDb: margin,
      ceilingDb: CEILING_DB
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
    `  ${label} margin=${margin}: clamps=${clamped.length} avgQuiet=${(quietSum / tracks.length).toFixed(3)} `
      + `worstQuiet=${quietWorst.toFixed(2)} worstOver=${overWorst.toFixed(2)}`,
  );
  return clamped;
}

const cTotalBase = ((BUDGET - shortSecs) / longSecs) - COVERAGE_EPSILON;
// Baseline probe of every track: shorts decode fully (exact, no margin row), longs zoom.
const baseline = tracks.map((t) => {
  if (t.dur <= SHORT_MAX) return {
    probeDb: db(t.full),
    usedSecs: t.dur,
    isRow: false
  };
  const {
    peak,
    usedSecs
  } = zoomProbe({
    track: t,
    cTotal: cTotalBase
  });
  return {
    probeDb: db(peak),
    usedSecs,
    isRow: true
  };
});
const baselineRows = tracks
  .map((
    t,
    i
  ) => ({
    fullDb: db(t.full),
    probeDb: baseline[i]
      .probeDb,
    i
  }))
  .filter((
    _,
    i
  ) => baseline[i]
    .isRow);
const baselineDecoded = baseline.reduce(
  (
    s,
    r
  ) => s + r.usedSecs,
  0
);
console.log(`\nbaseline zoom cTotal=${cTotalBase.toFixed(4)} decoded=${baselineDecoded.toFixed(0)}s ${baselineDecoded <= BUDGET ? 'IN BUDGET' : 'OVER'}`);
const baselineClamped = new Map(MARGINS.map((m) => [
  m,
  new Set(measure({
    rows: baselineRows,
    margin: m,
    label: 'baseline'
  })
    .map((r) => r.i))
]));

/**
 Pooled policy over `poolGroups`: skip every non-first group member, share the first
 copy's estimate, and fold the seconds the skipped copies would have decoded (their
 baseline usage) back into cTotal for all probed long tracks.
 */
function simulatePooled({
  label,
  poolGroups
}) {
  const sharedFrom = new Map();
  for (const g of poolGroups) for (const i of g.slice(1)) sharedFrom.set(
    i,
    g[0]
  );
  const savedSecs = [...sharedFrom.keys()].reduce(
    (
      s,
      i
    ) => s
      + baseline[i]
        .usedSecs,
    0
  );
  const cTotalPooled = (((BUDGET - shortSecs) + savedSecs) / longSecs) - COVERAGE_EPSILON;
  const pooled = tracks.map((
    t,
    i
  ) => {
    if (sharedFrom.has(i)) return {
      probeDb: Number.NaN,
      usedSecs: 0,
      isRow: true,
      shared: true
    };
    if (t.dur <= SHORT_MAX) return {
      probeDb: db(t.full),
      usedSecs: t.dur,
      isRow: false,
      shared: false
    };
    const {
      peak,
      usedSecs
    } = zoomProbe({
      track: t,
      cTotal: cTotalPooled
    });
    return {
      probeDb: db(peak),
      usedSecs,
      isRow: true,
      shared: false
    };
  });
  for (const [copy, first] of sharedFrom) pooled[copy]
    .probeDb = pooled[first]
      .probeDb;
  const rows = tracks
    .map((
      t,
      i
    ) => ({
      fullDb: db(t.full),
      probeDb: pooled[i]
        .probeDb,
      shared: pooled[i]
        .shared,
      i
    }))
    .filter((
      _,
      i
    ) => pooled[i]
      .isRow);
  const decoded = pooled.reduce(
    (
      s,
      r
    ) => s + r.usedSecs,
    0
  );
  console.log(`\n${label}: cTotal=${cTotalPooled.toFixed(4)} (saved ${savedSecs.toFixed(0)}s over ${sharedFrom.size} skipped copies)`);
  console.log(`  decoded=${decoded.toFixed(0)}s ${decoded <= BUDGET ? 'IN BUDGET' : 'OVER'}`);
  for (const m of MARGINS) {
    const clamped = measure({
      rows,
      margin: m,
      label: 'pooled  '
    });
    // New clamps the sharing itself creates: shared-estimate copies (own true peak vs the
    // first copy's probe) that were not clamped under their own baseline probe.
    const sharedClamps = clamped.filter((r) => r.shared);
    const newFromSharing = sharedClamps.filter((r) => !baselineClamped.get(m)
      .has(r.i));
    console.log(`    clamps on shared copies: ${sharedClamps.length}, NEW vs baseline: ${newFromSharing.length}`);
    for (const r of newFromSharing) {
      console.log(`      ur=${(r.fullDb - r.probeDb).toFixed(2)} ${basename(tracks[r.i]
        .path)
        .slice(
          0,
          70
        )}`);
    }
  }
}

simulatePooled({
  label: 'pooled over ALL duplicate groups',
  poolGroups: groups
});
simulatePooled({
  label: 'pooled over frac-verified (gain-identical) groups only',
  poolGroups: fracGroups
});
//endregion
