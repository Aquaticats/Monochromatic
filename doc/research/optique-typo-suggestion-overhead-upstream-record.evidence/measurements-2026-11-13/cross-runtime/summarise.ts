// Summarises results.jsonl from run-matrix.ts: one line per runtime, arm and script for the single runs, then the
// paired ratios for the alternating rounds.
//
// Run: node summarise.ts results.jsonl

import { readFileSync, } from 'node:fs';

type Row = {
  runtime: string;
  arm: string;
  script: string;
  args: string[];
  round: number;
  exit: number | null;
  result: Record<string, unknown> | null;
  stderr: string;
};

const path = process.argv[2];

if (path === undefined) throw new Error('usage: <script> <results.jsonl>',);

const lines = readFileSync(path, 'utf8',).trim().split('\n',).map((line,) => JSON.parse(line,) as Row & { runtimes?: string[]; });

console.log(JSON.stringify({ runtimes: lines[0]?.runtimes, rows: lines.length - 1, },),);

const rows = lines.slice(1,);
const failures = rows.filter((row,) => row.exit !== 0);

console.log(JSON.stringify({ nonZeroExits: failures.map((row,) => ({ runtime: row.runtime, arm: row.arm, script: row.script, args: row.args, exit: row.exit, stderr: row.stderr.slice(0, 200,), })), },),);

for (const row of rows.filter((candidate,) => candidate.round === 0)) {
  const result = row.result ?? {};
  const keep = Object.fromEntries(
    Object.entries(result,).filter(([key, value,],) =>
      typeof value !== 'object' && !['script', 'runtime', 'library',].includes(key,)),
  );

  console.log(JSON.stringify({ runtime: row.runtime, arm: row.arm, script: row.script, ...keep, },),);
}

function median(values: number[],): number {
  const sorted = [...values,].sort((left, right,) => left - right);

  return sorted[Math.floor(sorted.length / 2,)] as number;
}

const timed = rows.filter((row,) => row.round > 0);
const groups = new Map<string, Row[]>();

for (const row of timed) {
  // npm arms are named by a version number; checkout arms by a word.
  const family = /^\d/.test(row.arm,) ? 'npm' : 'checkout';
  const key = `${row.runtime}|${family}|${row.script}|${row.args.join(' ',)}`;

  groups.set(key, [...(groups.get(key,) ?? []), row,],);
}

for (const [key, group,] of groups) {
  const arms = [...new Set(group.map((row,) => row.arm),),];
  const [baseline, candidate,] = arms as [string, string,];
  // token-scaling reports an exponent and per-count medians; commit-paths reports medianMs.
  const metric = (row: Row,): number => {
    const result = row.result as { medianMs?: number; results?: { medianMs: number; }[]; };

    // token-scaling's figure is the median at its largest token count.
    return Number(result.medianMs ?? result.results?.at(-1,)?.medianMs ?? Number.NaN,);
  };
  const of = (arm: string,) => group.filter((row,) => row.arm === arm).sort((left, right,) => left.round - right.round);
  const base = of(baseline,);
  const cand = of(candidate,);
  const ratios = cand.map((row, index,) => metric(row,) / metric(base[index] as Row,));
  const exponents = (arm: string,) =>
    of(arm,).map((row,) => Object.entries(row.result as Record<string, unknown>,).find(([name,],) => /expon/i.test(name,))?.[1]);

  console.log(JSON.stringify({
    key,
    baseline,
    candidate,
    baselineMedianMs: median(base.map(metric,),),
    candidateMedianMs: median(cand.map(metric,),),
    pairedRatioMedian: Number(median(ratios,).toFixed(3,),),
    ratioMin: Number(Math.min(...ratios,).toFixed(3,),),
    ratioMax: Number(Math.max(...ratios,).toFixed(3,),),
    baselineExponents: exponents(baseline,),
    candidateExponents: exponents(candidate,),
  },),);
}
