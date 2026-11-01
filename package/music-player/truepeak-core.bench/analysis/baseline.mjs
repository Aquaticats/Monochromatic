// Reproduce the shipped proportional policy's ledger numbers offline, and add the
// honest average-quiet measure the letter asks about (bench only prints under-reads).
import {
  db,
  gainErrors,
  loadSafePaths,
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

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const safe = await loadSafePaths(new URL(
  '../out/metadata.jsonl',
  import.meta.url
).pathname);

const fullSecs = tracks.reduce(
  (
    sum,
    t
  ) => sum + t.dur,
  0
);
const target = fullSecs / 4;
console.log(`corpus: ${tracks.length} tracks, ${fullSecs.toFixed(0)}s, target ${target.toFixed(0)}s`);

// Sanity: the max over all bins should equal the full peak (same meter, same windows).
let worstBinGap = 0;
for (const t of tracks) {
  let m = 0;
  for (const b of t.bins) m = Math.max(
    m,
    b
  );
  const gap = Math.abs(db(t.full) - db(m));
  if (Number.isFinite(gap)) worstBinGap = Math.max(
    worstBinGap,
    gap
  );
}
console.log(`sanity: worst |full - max(bins)| = ${worstBinGap.toFixed(4)} dB`);

// Ledger replication: decoded seconds and per-loud-long-track under-reads.
let decoded = 0;
const underReads = [];
for (const t of tracks) {
  if (t.dur <= POLICY.shortScanMaxSecs) {
    decoded += t.dur;
    continue;
  }
  decoded += POLICY.coverage * t.dur;
  const fullDb = db(t.full);
  if (fullDb <= POLICY.ceilingDb) continue;
  const sampled = sampledMaxEven(
    t,
    POLICY.coverage,
    POLICY.windowSecs
  );
  underReads.push({
    ur: fullDb - db(sampled),
    safe: safe.has(t.path),
    path: t.path
  });
}
console.log(`decoded=${decoded.toFixed(0)}s (${((100 * decoded) / fullSecs).toFixed(1)}%) ${decoded <= target ? 'IN BUDGET' : 'OVER'}`);
const urs = underReads.map((u) => u.ur);
const percentiles = [
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
console.log(`loud long tracks=${underReads.length} | under-read dB: ${percentiles}`);
for (const margin of [
  0.5,
  0.8,
  1,
  1.2,
  1.5
]) {
  const clamped = underReads.filter((u) => (u.ur - margin) > POLICY.maxTooLoudDb);
  console.log(
    `  margin=${margin.toFixed(1)} -> clamped ${clamped.length} (${((100 * clamped.length) / underReads.length).toFixed(2)}%, safe ${clamped.filter((u) => u.safe)
      .length})`,
  );
}

// Honest three-measure ledger over ALL tracks at the shipped margin.
// Short tracks are exact (no margin); long tracks estimate probe + margin.
let quietSum = 0;
let quietWorst = 0;
let quietCount = 0;
let loudCount = 0;
let loudWorstOver = 0;
for (const t of tracks) {
  const fullDb = db(t.full);
  if (t.dur <= POLICY.shortScanMaxSecs) continue; // exact, both errors zero
  const sampled = sampledMaxEven(
    t,
    POLICY.coverage,
    POLICY.windowSecs
  );
  const {
    quietDb,
    loudDb
  } = gainErrors({
    fullDb,
    probeDb: db(sampled),
    marginDb: POLICY.marginDb,
    ceilingDb: POLICY.ceilingDb,
  });
  quietSum += quietDb;
  quietWorst = Math.max(
    quietWorst,
    quietDb
  );
  if (quietDb > 0) quietCount += 1;
  if (loudDb > POLICY.maxTooLoudDb) {
    loudCount += 1;
    loudWorstOver = Math.max(
      loudWorstOver,
      loudDb
    );
  }
}
console.log(`honest measures at margin ${POLICY.marginDb}:`);
console.log(`  average quiet across all ${tracks.length} tracks = ${(quietSum / tracks.length).toFixed(4)} dB`);
console.log(`  tracks attenuated too quiet: ${quietCount}, worst quiet = ${quietWorst.toFixed(3)} dB`);
console.log(`  clamped beyond +0.5: ${loudCount}, worst overshoot = ${loudWorstOver.toFixed(3)} dB`);
