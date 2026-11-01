// Album prior: songs in one album share mastering, so one member's exactly-known crest
// can bound its fellows. Quantifies grouping, within-group crest spread, probe
// misidentification risk, and simulates an album-capped policy under the decode budget.
// Crest here means the track's true peak in dBTP (what a full scan learns exactly).
import { dirname } from 'node:path';
import {
  db,
  gainErrors,
  loadTracks,
  quantile,
  sampledMaxEven
} from './corpus.mjs';

const POLICY = {
  shortScanMaxSecs: 90,
  coverage: 0.2,
  windowSecs: 0.3,
  marginDb: 0.8,
  ceilingDb: -1,
  maxTooLoudDb: 0.5,
};
const ALLOWANCES_DB = [
  0,
  0.25,
  0.5
];
const MIN_GROUP_LOUD_LONG = 3;

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
for (const t of tracks) {
  t.fullDb = db(t.full);
  t.isLong = t.dur > POLICY.shortScanMaxSecs;
  t.isLoudLong = t.isLong && (t.fullDb > POLICY.ceilingDb);
  // Probe once; every long track is probed under the shipped baseline anyway.
  t.probeDb = t.isLong ? db(sampledMaxEven(
    t,
    POLICY.coverage,
    POLICY.windowSecs
  )) : t.fullDb;
}
const fullSecs = tracks.reduce(
  (
    sum,
    t
  ) => sum + t.dur,
  0
);
const budget = fullSecs / 4;
const loudLong = tracks.filter((t) => t.isLoudLong);
console.log(`corpus: ${tracks.length} tracks, loud long ${loudLong.length}, budget ${budget.toFixed(0)}s`);

// 1) Grouping: album = parent directory; membership counted over loud long tracks.
const groups = new Map();
for (const t of loudLong) {
  const key = dirname(t.path);
  if (!groups.has(key)) groups.set(
    key,
    []
  );
  groups.get(key)
    .push(t);
}
const sizes = [...groups.values()].map((g) => g.length);
for (const n of [
  2,
  3,
  5
]) {
  console.log(`groups with >= ${n} loud long members: ${sizes.filter((s) => s >= n)
    .length}`);
}
const inCore = sizes.filter((s) => s >= MIN_GROUP_LOUD_LONG)
  .reduce(
  (
    a,
    b
  ) => a + b,
  0
  );
console.log(`loud long tracks in groups of >= ${MIN_GROUP_LOUD_LONG}: ${inCore}; loose (group < ${MIN_GROUP_LOUD_LONG}): ${loudLong.length - inCore}`);

const eligible = [...groups.entries()]
  .filter(([, g]) => g.length >= MIN_GROUP_LOUD_LONG)
  .map(([key, members]) => ({
    key,
    members
  }));

// 2) Within-group crest spread for eligible groups.
const memberSpreads = [];
const medianSpreads = [];
for (const g of eligible) {
  const crests = g.members
    .map((t) => t.fullDb);
  g.maxCrest = Math.max(...crests);
  for (const c of crests) memberSpreads.push(g.maxCrest - c);
  medianSpreads.push(g.maxCrest - quantile(
    crests,
    0.5
  ));
}
function spreadLine(
  label,
  values
) {  console.log(
  `${label}: median=${quantile(
    values,
    0.5
  )
    .toFixed(2)} p90=${quantile(
      values,
      0.9
    )
      .toFixed(2)} max=${quantile(
        values,
        1
      )
        .toFixed(2)} dB (n=${values.length})`,
); }
spreadLine(
  'group max crest - member crest',
  memberSpreads
);
spreadLine(
  'group max crest - group median crest',
  medianSpreads
);

// 3) Misidentification: does the loudest 20%-even probe pick the loudest true crest?
const mismatchGaps = [];
for (const g of eligible) {
  g.scan = g.members
    .reduce((
      best,
      t
    ) => (t.probeDb > best.probeDb ? t : best));
  const gap = g.maxCrest
    - g.scan
      .fullDb;
  if (gap > 0) mismatchGaps.push(gap);
}
console.log(
  `probe-loudest member matches crest-loudest: ${eligible.length - mismatchGaps.length}/${eligible.length} groups (${((100 * (eligible.length - mismatchGaps.length)) / eligible.length).toFixed(1)}%)`,
);
spreadLine(
  'mismatch gap (true group max - probe-chosen crest)',
  mismatchGaps
);

// 4) Album-capped policy simulation. Baseline decode: short tracks exact, long probed at 20%.
const decodedBase = tracks.reduce(
  (
    sum,
    t
  ) => sum + (t.isLong ? POLICY.coverage * t.dur : t.dur),
  0
);
// Full-scanning the probe-chosen member costs the remaining (1 - coverage) of its duration.
for (const g of eligible) g.extraSecs = (1 - POLICY.coverage)
  * g.scan
    .dur;
// Largest groups first (most fellows bounded per scan); cheaper scans first among equals.
const ordered = [...eligible].sort((
  a,
  b
) =>
  (b.members
    .length
    - a.members
      .length) || (a.extraSecs - b.extraSecs)
    || (a.key < b.key ? -1 : 1)
);
let decoded = decodedBase;
const selected = [];
for (const g of ordered) {
  if ((decoded + g.extraSecs) > budget) continue;
  decoded += g.extraSecs;
  selected.push(g);
}
const extraTotal = decoded - decodedBase;
console.log(
  `scans: ${selected.length}/${eligible.length} groups fit the budget (largest first); extra decode ${extraTotal.toFixed(0)}s; decoded ${decoded.toFixed(0)}s of ${budget.toFixed(0)}s ${decoded <= budget ? 'IN BUDGET' : 'OVER'}`,
);

// Baseline clamp set for the NEW-clamp comparison. Matches the ledger's definition:
// raw shortfall fullDb - estimateDb > 0.5 counted among loud long tracks only.
const baselineClamped = new Set(
  loudLong.filter((t) => (t.fullDb - (t.probeDb + POLICY.marginDb)) > POLICY.maxTooLoudDb)
    .map((t) => t.path),
);
console.log(`baseline clamps: ${baselineClamped.size}`);

for (const allowance of ALLOWANCES_DB) {
  // estimateDb per track: exact for shorts and scanned members, capped for fellows.
  const estimate = new Map();
  for (const t of tracks) estimate.set(
    t.path,
    t.isLong ? t.probeDb + POLICY.marginDb : t.fullDb
  );
  let cappedFellows = 0;
  for (const g of selected) {
    estimate.set(
      g.scan
        .path,
      g.scan
        .fullDb
    );
    for (const t of g.members) {
      if (t === g.scan) continue;
      const cap = Math.max(
        g.scan
          .fullDb,
        t.probeDb
      ) + allowance;
      const baseline = t.probeDb + POLICY.marginDb;
      if (cap < baseline) cappedFellows += 1;
      estimate.set(
        t.path,
        Math.min(
          baseline,
          cap
        )
      );
    }
  }
  let quietSum = 0;
  let quietWorst = 0;
  const clamped = [];
  for (const t of tracks) {
    const est = estimate.get(t.path);
    const { quietDb } = gainErrors({
      fullDb: t.fullDb,
      probeDb: est,
      marginDb: 0,
      ceilingDb: POLICY.ceilingDb
    });
    quietSum += quietDb;
    quietWorst = Math.max(
      quietWorst,
      quietDb
    );
    // Ledger-style clamp: raw shortfall among loud long tracks.
    if (t.isLoudLong && ((t.fullDb - est) > POLICY.maxTooLoudDb)) clamped.push(t.path);
  }
  const newClamps = clamped.filter((p) => !baselineClamped.has(p));
  const resolved = [...baselineClamped].filter((p) => !clamped.includes(p));
  console.log(
    `allowance=${allowance.toFixed(2)}: capped fellows=${cappedFellows} | clamps=${clamped.length} (new=${newClamps.length}, resolved=${resolved.length}) | avg quiet=${(quietSum / tracks.length).toFixed(4)} dB | worst quiet=${quietWorst.toFixed(3)} dB`,
  );
  for (const p of newClamps) {
    const t = tracks.find((x) => x.path === p);
    console.log(
      `  new clamp: shortfall=${(t.fullDb - estimate.get(p)).toFixed(2)} dB (baseline shortfall ${(t.fullDb - (t.probeDb + POLICY.marginDb)).toFixed(2)}) :: ${p.split('/')
        .slice(-2)
        .join('/')
        .slice(
          0,
          70
        )}`,
    );
  }
}
