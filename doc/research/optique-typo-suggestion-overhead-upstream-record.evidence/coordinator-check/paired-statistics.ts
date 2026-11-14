// Re-analysis of results/runs.jsonl with the process as the independent unit.
// Each arm and cell has one fresh process per round; a process's figure is the median of its
// timed parses. Two arms are compared round by round (the same round ran close together in
// time), and the exact two-sided sign test says how surprising the split of signs would be if
// the arms did not differ. With seven rounds the smallest reachable p-value is 2/128.
//
// Run from the measurement folder: node coordinator-check/paired-statistics.ts

import { readFileSync, } from 'node:fs';

type Run = { kind: string; arm: string; workload: string; pathSet: string; count: number; round: number; ms: number[]; };

function median(values: readonly number[],): number {
  const sorted = [...values,].sort((left, right,) => left - right);
  const middle = Math.floor(sorted.length / 2,);

  return sorted.length % 2 === 0 ? (sorted[middle - 1]! + sorted[middle]!) / 2 : sorted[middle]!;
}

function binomial(total: number, chosen: number,): number {
  let result = 1;

  for (let index = 1; index <= chosen; index += 1) result = (result * (total - chosen + index)) / index;

  return result;
}

function signTestTwoSided(positive: number, negative: number,): number {
  const total = positive + negative;
  const extreme = Math.max(positive, negative,);
  let tail = 0;

  for (let count = extreme; count <= total; count += 1) tail += binomial(total, count,);

  return Math.min(1, (2 * tail) / 2 ** total,);
}

const runs = readFileSync('results/runs.jsonl', 'utf8',)
  .split('\n',)
  .filter((line,) => line.startsWith('{',))
  .map((line,) => JSON.parse(line,) as Run)
  .filter((run,) => run.kind === 'run' && run.ms.length > 0);

const comparisons: readonly (readonly [string, string])[] = [
  ['1.1.1', '1.3.1',],
  ['1.3.1', '1.3.2',],
  ['1.3.2', '1.3.6',],
  ['1.3.2', '1.4.0-dev.2664',],
  ['1.4.0-dev.2664', '1.4.0-dev.2665',],
  ['1.4.0-dev.2665', '1.4.0-dev.2667',],
  ['1.4.0-dev.2667', '1.4.0-dev.2971',],
  ['1.3.6', '1.4.0-dev.2971',],
];
const cells: readonly (readonly [string, string, number])[] = [
  ['no-separator', 'real', 1000,],
  ['no-separator', 'short', 1000,],
  ['issue988', 'issue988', 1000,],
];

for (const [workload, pathSet, count,] of cells) {
  const perArm = new Map<string, Map<number, number>>();

  for (const run of runs) {
    if (run.workload !== workload || run.pathSet !== pathSet || run.count !== count) continue;
    const byRound = perArm.get(run.arm,) ?? new Map<number, number>();

    byRound.set(run.round, median(run.ms,),);
    perArm.set(run.arm, byRound,);
  }

  for (const [before, after,] of comparisons) {
    const first = perArm.get(before,);
    const second = perArm.get(after,);

    if (first === undefined || second === undefined) continue;
    const ratios: number[] = [];

    for (const [round, value,] of first) {
      const other = second.get(round,);

      if (other !== undefined) ratios.push(other / value,);
    }

    const slower = ratios.filter((ratio,) => ratio > 1).length;
    const faster = ratios.filter((ratio,) => ratio < 1).length;

    console.log(JSON.stringify({
      cell: `${workload}/${pathSet}/${count}`,
      before,
      after,
      rounds: ratios.length,
      medianRatio: Number(median(ratios,).toFixed(3,),),
      lowestRatio: Number(Math.min(...ratios,).toFixed(3,),),
      highestRatio: Number(Math.max(...ratios,).toFixed(3,),),
      roundsSlower: slower,
      roundsFaster: faster,
      signTestP: Number(signTestTwoSided(slower, faster,).toFixed(4,),),
    },),);
  }
}
