// Tail anatomy: around each loud long track's crest bin, how fast does loudness decay?
// If the crest sits on a shoulder (neighbors nearly as loud), an adaptive zoom pass can
// find it from a sparse first pass; if it is an isolated needle, no heard trace betrays it.
import {
  db,
  loadTracks,
  quantile,
  sampledMaxEven
} from './corpus.mjs';

const CEILING_DB = -1;
const SHORT_MAX = 90;
const MARGIN = 0.8;
const TOO_LOUD = 0.5;

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const loudLong = tracks.filter((t) => (t.dur > SHORT_MAX) && (db(t.full) > CEILING_DB));

// Tag the ledger's 43 clamp-tail tracks (under-read beyond margin + too-loud at 20% even).
for (const t of loudLong) {
  const ur = db(t.full) - db(sampledMaxEven(
    t,
    0.2,
    0.3
  ));
  t.underRead = ur;
  t.isTail = (ur - MARGIN) > TOO_LOUD;
}
const tail = loudLong.filter((t) => t.isTail);
console.log(`loud long tracks: ${loudLong.length}, ledger tail (clamped at 0.8): ${tail.length}`);

/** Gap in dB between the crest and the loudest bin within d bins of it (excluding itself). */
function decayGap(
  t,
  crestIndex,
  dBins
) {
  const lo = Math.max(
    0,
    crestIndex - dBins
  );
  const hi = Math.min(
    t.bins
      .length
      - 1,
    crestIndex + dBins
  );
  let m = 0;
  for (let j = lo; j <= hi; j += 1) {
    if (j !== crestIndex) m = Math.max(
      m,
      t.bins[j]
    );
  }
  return db(t.bins[crestIndex]) - db(m);
}

for (const t of loudLong) {
  let ci = 0;
  for (let j = 1; j
    < t.bins
      .length; j += 1) if (t.bins[j] > t.bins[ci]) ci = j;
  t.crestIndex = ci;
  t.crestPos = ci
    / t.bins
      .length;
  const crestDb = db(t.bins[ci]);
  // Bins within 0.5 / 1.0 dB of the crest anywhere in the track (multiplicity of loud bins).
  let near05 = 0;
  let near10 = 0;
  for (let j = 0; j
    < t.bins
      .length; j += 1) {
    const gap = crestDb - db(t.bins[j]);
    if (gap <= 0.5) near05 += 1;
    if (gap <= 1) near10 += 1;
  }
  t.near05 = near05;
  t.near10 = near10;
  t.decay = Object.fromEntries([
    1,
    2,
    3,
    5,
    10,
    20,
    50,
    100
  ].map((d) => [
    d,
    decayGap(
      t,
      ci,
      d
    )
  ]));
}

function report(
  label,
  group
) {
  console.log(`\n${label} (${group.length} tracks)`);
  for (const d of [
    1,
    2,
    3,
    5,
    10,
    20,
    50,
    100
  ]) {
    const gaps = group.map((t) => t.decay[d]);
    console.log(
      `  decay within ±${(d * 0.1).toFixed(1)}s: median=${quantile(
        gaps,
        0.5
      )
        .toFixed(2)} p90=${quantile(
          gaps,
          0.9
        )
          .toFixed(2)} max=${quantile(
            gaps,
            1
          )
            .toFixed(2)} dB`,
    );
  }
  const n05 = group.map((t) => t.near05);
  const n10 = group.map((t) => t.near10);
  console.log(
    `  bins within 0.5 dB of crest: median=${quantile(
      n05,
      0.5
    )} p10=${quantile(
      n05,
      0.1
    )} min=${quantile(
      n05,
      0
    )}`,
  );
  console.log(
    `  bins within 1.0 dB of crest: median=${quantile(
      n10,
      0.5
    )} p10=${quantile(
      n10,
      0.1
    )} min=${quantile(
      n10,
      0
    )}`,
  );
  const pos = group.map((t) => t.crestPos);
  console.log(
    `  crest position fraction: p10=${quantile(
      pos,
      0.1
    )
      .toFixed(2)} median=${quantile(
        pos,
        0.5
      )
        .toFixed(2)} p90=${quantile(
          pos,
          0.9
        )
          .toFixed(2)}`,
  );
}

report(
  'ALL loud long',
  loudLong
);
report(
  'LEDGER TAIL (43)',
  tail
);

// For the tail: how many probe-visible hints exist? A pass-1 probe at coverage c1 samples
// every (0.1/c1) seconds; the crest is found by zoom only if some pass-1 sample within the
// zoom reach ranks near the top. Report, per tail track, the number of DISTINCT 2s-separated
// regions whose peak is within 1 dB of the crest (how many places a zoom would try first).
console.log('\nper-tail-track detail:');
for (const t of tail.slice()
  .sort((
    a,
    b
  ) => b.underRead - a.underRead)) {
  const crestDb = db(t.bins[t.crestIndex]);
  // Count 2s regions (20 bins) whose max is within 1 dB of crest.
  const regionSize = 20;
  let regions = 0;
  for (let start = 0; start
    < t.bins
      .length; start += regionSize) {
    let m = 0;
    for (let j = start; j < Math.min(
      start + regionSize,
      t.bins
        .length
    ); j += 1) m = Math.max(
      m,
      t.bins[j]
    );
    if ((crestDb - db(m)) <= 1) regions += 1;
  }
  console.log(
    `  ur=${t.underRead
      .toFixed(2)} dur=${t.dur
        .toFixed(0)}s crest@${(t.crestPos * 100).toFixed(0)}% ` +
      `decay(0.1s)=${t.decay[1]
        .toFixed(2)} (0.5s)=${t.decay[5]
          .toFixed(2)} (2s)=${t.decay[20]
            .toFixed(2)} `
      +
      `near1dB=${t.near10} hotRegions=${regions} :: ${t.path
        .split('/')
        .at(-1)
        .slice(
        0,
        60
      )}`,
  );
}
