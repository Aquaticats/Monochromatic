// Driver for the no-decode byte-rate profile extraction: reads bones-targets.txt,
// parses each container with bones-parsers.mjs, writes out/byte-profiles.jsonl
// ({ path, slotSecs: 0.1, bytes: [int, ...] } per line), and prints per-format counts.
//
// Usage: node analysis/bones-extract.mjs
import { readFile } from 'node:fs/promises';
import {
  createWriteStream,
  readFileSync
} from 'node:fs';
import { once } from 'node:events';
import {
  SLOT_SECS,
  detectFormat,
  parseMp3,
  parseMp4,
  parseOgg
} from './bones-parsers.mjs';

/** How many files are parsed concurrently; readFile dominates, so keep this small. */
const POOL_SIZE = 4;
/** Bounded worker pool: POOL_SIZE workers pull indexes off a shared cursor. */
async function runPool({
  items,
  handler
}) {
  let cursor = 0;
  const workers = Array.from(
    { length: POOL_SIZE },
    async () => {
      while (cursor < items.length) {
        const index = cursor;
        cursor += 1;
        await handler(
          items[index],
          index
        );
      }
    }
  );
  await Promise.all(workers);
}

const startedAt = Date.now();
const targetsPath = new URL(
  'bones-targets.txt',
  import.meta.url
).pathname;
const outPath = new URL(
  '../out/byte-profiles.jsonl',
  import.meta.url
).pathname;
const targets = readFileSync(
  targetsPath,
  'utf8'
)
  .split('\n')
  .filter((line) => line.length > 0);
const output = createWriteStream(outPath);

const stats = {
  missing: 0,
  parsed: {
    ogg: 0,
    mp4: 0,
    mp3: 0
  },
  failed: {
    ogg: 0,
    mp4: 0,
    mp3: 0,
    unknown: 0
  },
};
const failures = [];
let done = 0;

await runPool({
  items: targets,
  handler: async (path) => {
    let buf;
    try {
      buf = await readFile(path);
    } catch (error) {
      // Missing files are expected (library churn); count and move on.
      if (error.code === 'ENOENT') {
        stats.missing += 1;
        return;
      }
      throw error;
    }
    let format = 'unknown';
    try {
      format = detectFormat({
        buf,
        path
      });
      const parser = {
        ogg: parseOgg,
        mp4: parseMp4,
        mp3: parseMp3
      }[format];
      const { bytes } = parser(buf);
      const line = `${JSON.stringify({
        path,
        slotSecs: SLOT_SECS,
        bytes
      })}\n`;
      if (!output.write(line)) await once(
        output,
        'drain'
      );
      stats.parsed[format] += 1;
    } catch (error) {
      stats.failed[format] += 1;
      failures.push({
        path,
        message: error.message
      });
    }
    done += 1;
    if ((done % 250) === 0) console.log(`  ...${done} files processed`);
  },
});

output.end();
await once(
  output,
  'finish'
);

const elapsedSecs = ((Date.now() - startedAt) / 1_000).toFixed(1);
console.log(`targets: ${targets.length}, missing on disk: ${stats.missing}`);
console.log(`parsed:  ogg=${stats.parsed
  .ogg} mp4=${stats.parsed
    .mp4} mp3=${stats.parsed
      .mp3}`);
console.log(
  `failed:  ogg=${stats.failed
    .ogg} mp4=${stats.failed
      .mp4}`
    + ` mp3=${stats.failed
      .mp3} unknown=${stats.failed
        .unknown}`,
);
for (const failure of failures.slice(
  0,
  20
)) {
  console.log(`  FAIL ${failure.path}: ${failure.message}`);
}
if (failures.length > 20) console.log(`  ...and ${failures.length - 20} more failures`);
console.log(`wrote ${outPath} in ${elapsedSecs}s`);
