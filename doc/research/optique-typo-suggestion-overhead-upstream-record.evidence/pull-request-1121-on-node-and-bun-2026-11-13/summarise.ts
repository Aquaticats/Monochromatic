// Summarises keep/timing-rounds.jsonl: for each runtime and step, each named pair of arms as the ratio of the two
// runs' medians within one round, given as the median over the rounds with the range of the single rounds and the
// number of rounds in which the first arm was slower. Absolute medians are printed last, for scale only.
//
// Run: node summarise.ts > keep/timing-summary.txt

import { readFileSync, } from 'node:fs';

interface Run {
  round: number;
  runtime: string;
  step: string;
  arm: string;
  exit: number | null;
  result: { medianMs?: number; parsedPaths?: number; paths?: number; separator?: boolean; library?: string; } | null;
}

const PAIRS: readonly (readonly [string, string, string,])[] = [
  ['main-again', 'main', 'same package twice: the spread',],
  ['main-before-1121', 'npm-1.3.6', 'main before the merge over 1.3.6',],
  ['main', 'main-before-1121', 'pull request 1121 over main before the merge',],
  ['main', 'npm-1.3.6', 'pull request 1121 over 1.3.6',],
  ['base', 'main', 'input-trace patch on the merge over the merge',],
  ['base', 'npm-1.3.6', 'input-trace patch on the merge over 1.3.6',],
];

function medianOf(values: readonly number[],): number {
  const sorted = [...values,].sort((left, right,) => left - right);
  const middle = sorted.length >> 1;

  return sorted.length % 2 === 1 ? sorted[middle] as number : ((sorted[middle - 1] as number) + (sorted[middle] as number)) / 2;
}

const lines = readFileSync(`${import.meta.dirname}/keep/timing-rounds.jsonl`, 'utf8',).split('\n',);
const runs = lines.filter((line,) => line.startsWith('{"round"',)).map((line,) => JSON.parse(line,) as Run);
const bad = runs.filter((run,) => run.exit !== 0 || typeof run.result?.medianMs !== 'number' || run.result.parsedPaths !== 1000);

console.log(lines.find((line,) => line.startsWith('{"runtimes"',)) ?? 'no header line',);
console.log(`runs: ${runs.length}; runs that failed or parsed another number of paths: ${bad.length}`,);
console.log(`libraries resolved: ${JSON.stringify([...new Set(runs.map((run,) => `${run.arm} -> ${run.result?.library}`),),],)}`,);

for (const runtime of ['node', 'bun',]) {
  for (const step of ['separator', 'no-separator',]) {
    const here = runs.filter((run,) => run.runtime === runtime && run.step === step && !bad.includes(run,));

    console.log(`\n${runtime} | ${step}`,);

    for (const [arm, against, label,] of PAIRS) {
      const ratios: number[] = [];

      for (const run of here.filter((candidate,) => candidate.arm === arm)) {
        const other = here.find((candidate,) => candidate.arm === against && candidate.round === run.round);

        if (other !== undefined) ratios.push((run.result?.medianMs as number) / (other.result?.medianMs as number),);
      }

      if (ratios.length === 0) continue;

      console.log(
        `  ${arm} / ${against} (${label}): ${medianOf(ratios,).toFixed(3,)} (${Math.min(...ratios,).toFixed(3,)} to ${
          Math.max(...ratios,).toFixed(3,)
        }), slower in ${ratios.filter((ratio,) => ratio > 1).length} of ${ratios.length}`,
      );
    }

    const scale = [...new Set(here.map((run,) => run.arm),),].map((arm,) => {
      const values = here.filter((run,) => run.arm === arm).map((run,) => run.result?.medianMs as number);

      return `${arm} ${medianOf(values,).toFixed(1,)} ms (${Math.min(...values,).toFixed(1,)} to ${Math.max(...values,).toFixed(1,)})`;
    },);

    console.log(`  for scale only: ${scale.join('; ',)}`,);
  }
}
