// Validate the no-decode byte-rate profiles against the decoded corpus:
// 1. |profile duration - decoded duration| distribution across every parsed file;
// 2. Pearson correlation between per-slot payload bytes and per-slot decoded bin peaks
//    on a handful of spot-check files (loud/busy audio should cost more bits).
//
// Usage: node analysis/bones-validate.mjs
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';
import {
  db,
  loadTracks,
  quantile
} from './corpus.mjs';

/** Duration mismatch beyond this many seconds counts as a validation failure. */
const MISMATCH_LIMIT_SECS = 0.5;

/** Pearson correlation between two equally-indexed series, truncated to the shorter. */
function pearson({
  xs,
  ys
}) {
  const length = Math.min(
    xs.length,
    ys.length
  );
  if (length < 2) return Number.NaN;
  let sumX = 0;
  let sumY = 0;
  for (let index = 0; index < length; index += 1) {
    sumX += xs[index];
    sumY += ys[index];
  }
  const meanX = sumX / length;
  const meanY = sumY / length;
  let covariance = 0;
  let varianceX = 0;
  let varianceY = 0;
  for (let index = 0; index < length; index += 1) {
    const dx = xs[index] - meanX;
    const dy = ys[index] - meanY;
    covariance += dx * dy;
    varianceX += dx * dx;
    varianceY += dy * dy;
  }
  return covariance / Math.sqrt(varianceX * varianceY);
}

const profilesPath = new URL(
  '../out/byte-profiles.jsonl',
  import.meta.url
).pathname;
const corpusPath = new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname;

// Stream the profiles; keep bytes only for spot-check candidates plus slot counts for all.
const profiles = new Map();
const reader = createInterface({
  input: createReadStream(profilesPath),
  crlfDelay: Infinity
});
for await (const line of reader) {
  if (!line.trim()) continue;
  const profile = JSON.parse(line);
  profiles.set(
    profile.path,
    {
      slotSecs: profile.slotSecs,
      bytes: profile.bytes
    }
  );
}
console.log(`profiles: ${profiles.size}`);

const tracks = await loadTracks(corpusPath);
const corpus = new Map(tracks.map((track) => [
  track.path,
  track
]));
console.log(`corpus tracks: ${corpus.size}`);

// Duration comparison across every parsed profile with a corpus entry.
const diffs = [];
const offenders = [];
let noCorpus = 0;
for (const [path, profile] of profiles) {
  const track = corpus.get(path);
  if (track === undefined) {
    noCorpus += 1;
    continue;
  }
  const diff = Math.abs((profile.bytes
    .length
    * profile.slotSecs) - track.dur);
  diffs.push(diff);
  if (diff > MISMATCH_LIMIT_SECS) offenders.push({
    path,
    diff
  });
}
diffs.sort((
  a,
  b
) => a - b);
console.log(`compared: ${diffs.length}, no corpus entry: ${noCorpus}`);
console.log('duration mismatch |profileDur - corpusDur| (seconds):');
for (const fraction of [
  0.5,
  0.9,
  0.95,
  0.99,
  1
]) {
  console.log(`  p${(fraction * 100).toFixed(0)}: ${quantile(
    diffs,
    fraction
  )
    .toFixed(3)}`);
}
const withinLimit = diffs.filter((diff) => diff <= MISMATCH_LIMIT_SECS)
  .length;
console.log(
  `  within ${MISMATCH_LIMIT_SECS}s: ${withinLimit}/${diffs.length}`
    + ` (${((withinLimit / diffs.length) * 100).toFixed(2)}%), over: ${offenders.length}`,
);
offenders.sort((
  a,
  b
) => b.diff - a.diff);
for (const offender of offenders.slice(
  0,
  10
)) {
  console.log(`  OVER ${offender.diff
    .toFixed(2)}s ${offender.path}`);
}

// Spot-check alignment: per-slot bytes vs per-slot decoded bin peaks (bin_seconds must
// match slotSecs for index-for-index comparison). First/middle/last opus plus first m4a.
const opusPaths = [...profiles.keys()].filter(
  (path) => path.toLowerCase()
    .endsWith('.opus')
    && corpus.has(path),
);
const m4aPaths = [...profiles.keys()].filter(
  (path) => path.toLowerCase()
    .endsWith('.m4a')
    && corpus.has(path),
);
const spotPaths = [
  opusPaths[0],
  opusPaths[Math.floor(opusPaths.length / 2)],
  opusPaths.at(-1),
  m4aPaths[0],
];
console.log('spot-check Pearson (per-slot bytes vs per-slot bin peaks):');
for (const path of spotPaths) {
  const profile = profiles.get(path);
  const track = corpus.get(path);
  if (Math.abs(track.binSecs - profile.slotSecs) > 1e-9) {
    console.log(`  SKIP (binSecs ${track.binSecs} != slotSecs ${profile.slotSecs}) ${path}`);
    continue;
  }
  const r = pearson({
    xs: profile.bytes,
    ys: track.bins
  });
  // dB-domain peaks (floored at -60) track bit demand better on heavily limited masters,
  // whose linear peak profiles are nearly flat; report both for context.
  const dbBins = Array.from(
    track.bins,
    (value) => Math.max(
      db(value),
      -60
    )
  );
  const rDb = pearson({
    xs: profile.bytes,
    ys: dbBins
  });
  console.log(
    `  rLinear=${r.toFixed(3)} rDb=${rDb.toFixed(3)}`
      + ` slots=${profile.bytes
        .length}/${track.bins
          .length} ${path}`,
  );
}
