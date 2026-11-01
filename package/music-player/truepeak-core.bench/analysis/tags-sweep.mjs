// Full-library embedded-tag sweep via ffprobe (no decoding): codec, bit rate, and the
// provenance/peak tags a bucket policy could legally read (no path text). Writes
// out/tags-full.jsonl with one row per corpus track.
import { spawn } from 'node:child_process';
import { createWriteStream } from 'node:fs';
import { loadTracks } from './corpus.mjs';

const CONCURRENCY = 8;

/** Run ffprobe and return parsed JSON, or null on failure. */
function ffprobe(path) {
  return new Promise((resolve) => {
    const child = spawn(
      'ffprobe',
      [
        '-v',
        'quiet',
        '-print_format',
        'json',
        '-show_format',
        '-show_streams',
        path,
      ]
    );
    const chunks = [];
    child.stdout
      .on(
      'data',
      (c) => chunks.push(c)
      );
    child.on(
      'close',
      (code) => {
        if (code !== 0) {  resolve(null);
          return; }
        try {
          resolve(
            JSON.parse(Buffer.concat(chunks)
              .toString('utf8')),
          );
        } catch (error) {
          console.error(`parse failure for ${path}: ${error}`);
          resolve(null);
        }
      }
    );
    child.on(
      'error',
      () =>{  resolve(null); }
    );
  });
}

/** Decode iTunNORM: fields 6 and 7 (hex) are peak sample values scaled to 32768. */
function itunPeak(value) {
  const fields = value.trim()
    .split(/\s+/);
  if (fields.length < 8) return null;
  const a = Number.parseInt(
    fields[6],
    16
  );
  const b = Number.parseInt(
    fields[7],
    16
  );
  if ((!Number.isFinite(a)) || (!Number.isFinite(b))) return null;
  return Math.max(
    a,
    b
  ) / 32_768;
}

const tracks = await loadTracks(new URL(
  '../out/tracks-fine.jsonl',
  import.meta.url
).pathname);
const out = createWriteStream(new URL(
  '../out/tags-full.jsonl',
  import.meta.url
).pathname);
let done = 0;
let failed = 0;

async function worker(queue) {
  for (;;) {
    const t = queue.pop();
    if (!t) return;
    const d = await ffprobe(t.path);
    if (!d) {
      failed += 1;
      continue;
    }
    const audio = (d.streams ?? []).find((s) => s.codec_type === 'audio') ?? {};
    const tags = Object.fromEntries(
      Object.entries({
        ...audio.tags,
        ...d.format
          ?.tags
      })
        .map(([k, v]) => [
          k.toLowerCase(),
          v
        ]),
    );
    const row = {
      path: t.path,
      codec: audio.codec_name ?? null,
      bitRate: Number(audio.bit_rate
        ?? d.format
          ?.bit_rate
        ?? 0),
      tagCount: Object.keys(tags)
        .length,
      encoderTag: tags.encoder ?? null,
      hasStoreIds: Boolean(tags.isrc || tags.upc
        || tags['qbz:tid']
        || tags.content_id
        || tags.xid),
      hasItunNorm: Boolean(tags.itunnorm),
      itunPeak: tags.itunnorm ? itunPeak(tags.itunnorm) : null,
      hasReplaygain: Object.keys(tags)
        .some((k) => k.startsWith('replaygain')),
      hasPurl: Boolean(tags.purl || (tags.comment ?? '').includes('youtu')),
      date: tags.date ?? tags.year
        ?? null,
    };
    out.write(`${JSON.stringify(row)}\n`);
    done += 1;
    if ((done % 500) === 0) console.log(`probed ${done}`);
  }
}

const queue = [...tracks];
await Promise.all(Array.from(
  { length: CONCURRENCY },
  () => worker(queue)
));
out.end();
console.log(`done: ${done} probed, ${failed} failed (missing on disk or unreadable)`);
