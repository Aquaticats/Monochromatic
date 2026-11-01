// Offline corpus loader for policy simulation. Mirrors src/corpus.rs: each JSONL line is
// one track with the full true peak and per-0.1s Catmull-Rom bin peaks from the shared meter.
import { createReadStream } from 'node:fs';
import { createInterface } from 'node:readline';

/** Convert a linear peak to dBTP; non-positive is treated as silence. */
export function db(peak) { return peak <= 0 ? -Infinity : 20 * Math.log10(peak); }

/** Load the corpus: [{ path, dur, rate, full, binSecs, bins: Float32Array }]. */
export async function loadTracks(path) {
  const tracks = [];
  const rl = createInterface({
    input: createReadStream(path),
    crlfDelay: Infinity
  });
  for await (const line of rl) {
    if (!line.trim()) continue;
    const t = JSON.parse(line);
    tracks.push({
      path: t.path,
      dur: t.duration_secs,
      rate: t.rate,
      full: t.full_peak,
      binSecs: t.bin_seconds ?? 1,
      bins: Float32Array.from(t.bin_peaks),
    });
  }
  return tracks;
}

/** Load safe-provenance paths (lossless or ytdlp) from metadata.jsonl. */
export async function loadSafePaths(path) {
  const safe = new Set();
  const rl = createInterface({
    input: createReadStream(path),
    crlfDelay: Infinity
  });
  for await (const line of rl) {
    if (!line.trim()) continue;
    const r = JSON.parse(line);
    if (r.lossless || r.ytdlp) safe.add(r.path);
  }
  return safe;
}

/** Bench-style quantile: sorted ascending, index = round((n-1) * fraction). */
export function quantile(
  values,
  fraction
) {
  const sorted = [...values].sort((
    a,
    b
  ) => a - b);
  if (sorted.length === 0) return 0;
  return sorted[Math.round((sorted.length - 1) * fraction)];
}

/**
 The loudest sampled linear peak under the shipped evenly-placed probe.
 Mirrors src/proportional.rs sampled_max exactly (f32 window fold, f64 outer fold).
 */
export function sampledMaxEven(
  track,
  coverage,
  windowSecs
) {
  const {bins} = track;
  const n = bins.length;
  const windowBins = Math.max(
    1,
    Math.round(windowSecs / track.binSecs)
  );
  const count = Math.max(
    1,
    Math.round((coverage * track.dur) / windowSecs)
  );
  const span = Math.max(
    0,
    n - windowBins
  );
  let peak = 0;
  for (let index = 0; index < count; index += 1) {
    const start = count <= 1 ? Math.floor(span / 2) : Math.round((index / (count - 1)) * span);
    const hi = Math.min(
      start + windowBins,
      n
    );
    let w = Math.fround(0);
    for (let b = start; b < hi; b += 1) w = Math.max(
      w,
      bins[b]
    );
    peak = Math.max(
      peak,
      w
    );
  }
  return peak;
}

/**
 Honest per-track gain errors for a probe estimate.
 Applied attenuation puts (probe + margin) at the ceiling, never amplifies.
 quiet = attenuation beyond what the true peak needed; loud = shortfall the clamp catches.
 */
export function gainErrors({
  fullDb,
  probeDb,
  marginDb,
  ceilingDb
}) {
  const necessary = Math.max(
    0,
    fullDb - ceilingDb
  );
  const applied = Math.max(
    0,
    (probeDb + marginDb) - ceilingDb
  );
  return {
    quietDb: Math.max(
      0,
      applied - necessary
    ),
    loudDb: Math.max(
      0,
      necessary - applied
    ),
  };
}
