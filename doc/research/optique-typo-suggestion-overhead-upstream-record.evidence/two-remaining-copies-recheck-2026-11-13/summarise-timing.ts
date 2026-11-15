// Summarises keep/timing-round-*.jsonl: per step, arm and size, the median over rounds of each run's own median
// with its range; the growth per doubling of the size (log2 of the ratio of neighbouring sizes, 1 is linear and 2 is
// quadratic); and, per round, the ratio of each arm to a comparison arm, as median and range over rounds.
//
// Run: node summarise-timing.ts > keep/timing-summary.jsonl

import { readdirSync, readFileSync, } from 'node:fs';

const KEEP = `${import.meta.dirname}/keep`;

interface Run {
  round: number;
  step: string;
  arm: string;
  exit: number | null;
  result: { results?: { tokens: number; medianMs: number; }[]; medianMs?: number; paths?: number; } | null;
}

function medianOf(values: readonly number[],): number {
  const sorted = [...values,].sort((left, right,) => left - right);
  const middle = sorted.length >> 1;

  return sorted.length % 2 === 1 ? sorted[middle] as number : ((sorted[middle - 1] as number) + (sorted[middle] as number)) / 2;
}

function rounded(value: number, digits = 3,): number {
  return Number(value.toFixed(digits,),);
}

const runs: Run[] = [];
let failedRuns = 0;

for (const file of readdirSync(KEEP,).filter((name,) => /^timing-rounds?-.*\.jsonl$/u.test(name,)).sort()) {
  for (const line of readFileSync(`${KEEP}/${file}`, 'utf8',).split('\n',)) {
    if (!line.startsWith('{"round"',)) continue;

    const run = JSON.parse(line,) as Run;

    if (run.exit !== 0) {
      failedRuns += 1;
      console.log(JSON.stringify({ failedRun: line.slice(0, 700,), },),);
      continue;
    }

    runs.push(run,);
  }
}

// One point per run and size: step, arm, size, round, milliseconds.
const points: { step: string; arm: string; size: number; round: number; ms: number; }[] = [];

for (const run of runs) {
  if (run.result?.results !== undefined) {
    for (const measurement of run.result.results) points.push({ step: run.step, arm: run.arm, size: measurement.tokens, round: run.round, ms: measurement.medianMs, },);
  }
  else if (typeof run.result?.medianMs === 'number') {
    points.push({ step: run.step, arm: run.arm, size: run.result.paths ?? 0, round: run.round, ms: run.result.medianMs, },);
  }
}

console.log(JSON.stringify({ runs: runs.length, failedRuns, points: points.length, rounds: [...new Set(points.map((point,) => point.round),),].sort((left, right,) => left - right), },),);

const steps = [...new Set(points.map((point,) => point.step),),];

for (const step of steps) {
  const inStep = points.filter((point,) => point.step === step);
  const arms = [...new Set(inStep.map((point,) => point.arm),),];
  const sizes = [...new Set(inStep.map((point,) => point.size),),].sort((left, right,) => left - right);

  for (const arm of arms) {
    const medians = sizes.map((size,) => {
      const values = inStep.filter((point,) => point.arm === arm && point.size === size).map((point,) => point.ms);

      return values.length === 0 ? null : { size, rounds: values.length, medianMs: rounded(medianOf(values,),), minMs: rounded(Math.min(...values,),), maxMs: rounded(Math.max(...values,),), };
    },).filter((entry,) => entry !== null);
    const growthPerDoubling = medians.slice(1,).map((entry, index,) => {
      const previous = medians[index] as { size: number; medianMs: number; };

      return { from: previous.size, to: entry.size, exponent: rounded(Math.log(entry.medianMs / previous.medianMs,) / Math.log(entry.size / previous.size,), 2,), };
    },);

    console.log(JSON.stringify({ step, arm, medians, growthPerDoubling, },),);
  }

  // Paired ratios: each arm against the comparison arms, round by round at each size.
  for (const against of ['main', 'base', 'multiple', 'multiple-final',]) {
    if (!arms.includes(against,)) continue;

    for (const arm of arms) {
      if (arm === against) continue;

      const perSize = sizes.map((size,) => {
        const ratios: number[] = [];

        for (const point of inStep.filter((candidate,) => candidate.arm === arm && candidate.size === size)) {
          const other = inStep.find((candidate,) => candidate.arm === against && candidate.size === size && candidate.round === point.round);

          if (other !== undefined) ratios.push(point.ms / other.ms,);
        }

        return ratios.length === 0
          ? null
          : { size, rounds: ratios.length, medianRatio: rounded(medianOf(ratios,),), minRatio: rounded(Math.min(...ratios,),), maxRatio: rounded(Math.max(...ratios,),), slowerIn: ratios.filter((ratio,) => ratio > 1).length, };
      },).filter((entry,) => entry !== null);

      if (perSize.length > 0) console.log(JSON.stringify({ step, ratioOf: arm, against, perSize, },),);
    }
  }
}
