// Stats for a sustained-load episode: the sample rows between two UTC times, read-only.
// Usage: node analyze-episode.ts <from-utc> <to-utc> [local-date, default today]
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { resolvePaths, } from './paths.ts';
import { columnStats, } from './sample-stats.ts';

/**
 Columns worth comparing between episodes.
 */
const COLUMNS: readonly string[] = [
  'tctl_c',
  'tccd1_c',
  'tccd2_c',
  'package_w',
  'average_mhz',
  'cpu_pressure_some_avg10',
  'load1',
  'fan1_rpm',
  'fan2_rpm',
  'fan4_rpm',
  'fan5_rpm',
  'fan6_rpm',
  'corsair_probe_c',
  'cpu_fan_rpm',
  'sys_fan1_rpm',
  'sys_fan3_rpm',
  'sys_fan5_rpm',
  'sys_fan6_rpm',
  'mb_vrm_c',
  'chiplet1_w',
  'chiplet2_w',
  'board3_c',
  'board5_c',
  'nvme_c',
];

/**
 Width the column names are padded to in the output.
 */
const NAME_WIDTH = 24;

/**
 Earliest and latest UTC time, from the first two arguments.
 */
const [from = '', to = '', day = new Date().toLocaleDateString('sv',),] = process.argv
  .slice(2,);

/**
 Directory of the daily sample files.
 */
const { sampleDir, } = resolvePaths({
  env: process.env,
  uid: process.getuid?.() ?? 0,
},);

/**
 Statistics of the requested columns inside the range.
 */
const stats = columnStats({
  csv: await readFile(
    join(
      sampleDir,
      `${day}.csv`,
    ),
    'utf8',
  ),
  from,
  to,
  names: COLUMNS,
},);

console.log(
  'rows',
  stats.rows,
  from,
  to,
);
for (const column of stats.columns) {
  console.log(
    column.name
      .padEnd(NAME_WIDTH,),
    'median',
    column.median,
    'min',
    column.min,
    'max',
    column.max,
  );
}
