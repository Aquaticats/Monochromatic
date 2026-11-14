// Summarises upstream-arms-results.jsonl: for each script, the median over rounds of each arm's figure, and paired
// per-round ratios for the comparisons that matter.
//
// Run: node summarise-arms.ts upstream-arms-results.jsonl

import { readFileSync, } from 'node:fs';

type Row = {
  arm: string;
  script: string;
  args: string[];
  round: number;
  exit: number | null;
  result: { medianMs?: number; results?: { medianMs: number; }[]; fitExponent?: number; mismatchDiagnostics?: number; };
};

const path = process.argv[2];

if (path === undefined) throw new Error('usage: <script> <results.jsonl>',);

const rows = readFileSync(path, 'utf8',).trim().split('\n',).map((line,) => JSON.parse(line,) as Row);

console.log(JSON.stringify({ rows: rows.length, nonZeroExits: rows.filter((row,) => row.exit !== 0).length, },),);
console.log(JSON.stringify({ mismatchDiagnostics: Object.fromEntries(rows.filter((row,) => row.round === 0).map((row,) => [row.arm, row.result.mismatchDiagnostics,]),), },),);

const median = (values: number[],): number => [...values,].sort((left, right,) => left - right)[Math.floor(values.length / 2,)] as number;
const figure = (row: Row,): number => Number(row.result.medianMs ?? row.result.results?.at(-1,)?.medianMs,);
const PAIRS: [string, string,][] = [
  ['1.3.6', 'main',],
  ['1.3.6', 'pr1121',],
  ['main', 'pr1121',],
  ['main', 'main-patched',],
  ['pr1121', 'pr1121-patched',],
  ['1.3.6', 'pr1121-patched',],
];

for (const key of new Set(rows.filter((row,) => row.round > 0).map((row,) => `${row.script} ${row.args.join(' ',)}`.trim()),)) {
  const group = rows.filter((row,) => row.round > 0 && `${row.script} ${row.args.join(' ',)}`.trim() === key);
  const byArm = (arm: string,) => group.filter((row,) => row.arm === arm).sort((left, right,) => left.round - right.round);
  const arms = [...new Set(group.map((row,) => row.arm),),];

  console.log(JSON.stringify({
    key,
    medianMs: Object.fromEntries(arms.map((arm,) => [arm, Number(median(byArm(arm,).map(figure,),).toFixed(1,),),]),),
    exponent: key.startsWith('token-scaling',)
      ? Object.fromEntries(arms.map((arm,) => {
        const values = byArm(arm,).map((row,) => row.result.fitExponent as number);

        return [arm, `${Math.min(...values,).toFixed(2,)} to ${Math.max(...values,).toFixed(2,)}`,];
      },),)
      : undefined,
  },),);

  for (const [baseline, candidate,] of PAIRS) {
    const base = byArm(baseline,);
    const ratios = byArm(candidate,).map((row, index,) => figure(row,) / figure(base[index] as Row,));

    console.log(JSON.stringify({
      key,
      pair: `${candidate} against ${baseline}`,
      ratio: Number(median(ratios,).toFixed(2,),),
      min: Number(Math.min(...ratios,).toFixed(2,),),
      max: Number(Math.max(...ratios,).toFixed(2,),),
      roundsSlower: ratios.filter((ratio,) => ratio > 1).length,
      rounds: ratios.length,
    },),);
  }
}
