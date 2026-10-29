// Summarizes the watcher's sample log as Tctl per package-power bucket, counting only settled samples.
// Usage: node summarize-samples.ts [days-back, default 1]
import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { resolvePaths, } from './paths.ts';
import {
  BUCKET_WATTS,
  cutoffFor,
  parseSampleRows,
  SETTLED_SAMPLES,
  summarizeBuckets,
} from './sample-stats.ts';

/**
 Directory of the daily sample files.
 */
const { sampleDir, } = resolvePaths({
  env: process.env,
  uid: process.getuid?.() ?? 0,
},);

/**
 Look-back window in days, from the first argument.
 */
const daysBack = Number(process.argv[2] ?? 1,);

/**
 Daily file names in date order.
 */
const names = (await readdir(sampleDir,)).filter(function isCsv(name,) {
  return name.endsWith('.csv',);
},)
  .toSorted();

/**
 Rows of every daily file inside the look-back window.
 */
const rows = (await Promise.all(names.map(async function readDay(name,) {
  return parseSampleRows({
    csv: await readFile(
      join(
        sampleDir,
        name,
      ),
      'utf8',
    ),
    cutoff: cutoffFor({
      now: Date.now(),
      daysBack,
    },),
  },);
},),)).flat();

console.log(`samples read: ${rows.length}; settled = ${SETTLED_SAMPLES} previous samples in the same ${BUCKET_WATTS} W bucket`,);
for (const summary of summarizeBuckets(rows,)) {
  console.log(
    `${summary.bucket}-${summary.bucket + BUCKET_WATTS} W: ${summary.count} settled samples, `
      + `Tctl median ${summary.median}, p90 ${summary.p90}, max ${summary.max}`,
  );
}
if (summarizeBuckets(rows,)
  .length
  === 0) {
  console.log('no settled samples yet; keep the watcher running through some sustained load',);
}
