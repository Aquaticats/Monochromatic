// Coordinator's own check of the measurement agent's headline numbers.
// It shares nothing with the agent's harness except the image and its baked inputs:
// the timing loop is the inline script below, and the medians of the agent's raw
// records are recomputed here rather than read from its summary.
//
// Run from the measurement folder: node coordinator-check/independent-check.ts

import { spawnSync, } from 'node:child_process';
import { readFileSync, } from 'node:fs';

const IMAGE = 'localhost/optique-measure:2026-11-12';
const ARMS = ['1.1.1', '1.3.6', '1.4.0-dev.2971', 'owned-4879a44e',] as const;
const WARMUP_PARSES = 2;
const TIMED_PARSES = 5;

function median(values: readonly number[],): number {
  const sorted = [...values,].sort((left, right,) => left - right);
  const middle = Math.floor(sorted.length / 2,);

  return sorted.length % 2 === 0 ? (sorted[middle - 1]! + sorted[middle]!) / 2 : sorted[middle]!;
}

//region Recompute the agent's medians from its raw records

const records = readFileSync('results/runs.jsonl', 'utf8',)
  .split('\n',)
  .filter((line,) => line.startsWith('{',))
  .map((line,) => JSON.parse(line,) as { kind: string; arm: string; workload: string; pathSet: string; count: number; ms: number[]; ok: boolean; });

const realPaths = JSON.parse(readFileSync('data/real-paths.json', 'utf8',),) as string[];
const thousand = records.filter((record,) => record.kind === 'run' && record.workload === 'no-separator' && record.pathSet === 'real' && record.count === realPaths.length);

for (const arm of new Set(thousand.map((record,) => record.arm),)) {
  const samples = thousand.filter((record,) => record.arm === arm).flatMap((record,) => record.ms);

  console.log(JSON.stringify({ source: 'agent-raw-records', arm, samples: samples.length, medianMs: Number(median(samples,).toFixed(3,),), },),);
}

//endregion Recompute the agent's medians from its raw records

//region Time the same call with a separate loop

const inline = `
import { readFileSync } from 'node:fs';
const arm = process.argv[1];
const { parseCommitRegion } = await import('file:///bench/arms/' + arm + '/src/parser/commit.ts');
const paths = JSON.parse(readFileSync('/bench/data/real-paths.json', 'utf8'));
const argv = ['--message', 'x', ...paths];
for (let index = 0; index < ${WARMUP_PARSES}; index += 1) parseCommitRegion(argv);
const samples = [];
let last;
for (let index = 0; index < ${TIMED_PARSES}; index += 1) {
  const startedAt = performance.now();
  last = parseCommitRegion(argv);
  samples.push(performance.now() - startedAt);
}
console.log(JSON.stringify({ arm, paths: paths.length, pathspecs: last.pathspecs.length, hasPathspec: last.hasPathspec, ms: samples }));
`;

for (const arm of ARMS) {
  const child = spawnSync(
    'podman',
    ['run', '--rm', '--cpus', '2', '--memory', '4g', '--pids-limit', '512', '--blkio-weight', '10', IMAGE, '--input-type=module-typescript', '--eval', inline, arm,],
    { encoding: 'utf8', },
  );

  if (child.status !== 0) {
    console.log(JSON.stringify({ source: 'coordinator-loop', arm, failed: child.stderr.slice(0, 600,), },),);
    continue;
  }

  const parsed = JSON.parse(child.stdout.trim().split('\n',).at(-1,)!,) as { paths: number; pathspecs: number; hasPathspec: boolean; ms: number[]; };

  console.log(JSON.stringify({ source: 'coordinator-loop', arm, paths: parsed.paths, pathspecs: parsed.pathspecs, hasPathspec: parsed.hasPathspec, medianMs: Number(median(parsed.ms,).toFixed(3,),), minMs: Number(Math.min(...parsed.ms,).toFixed(3,),), },),);
}

//endregion Time the same call with a separate loop
