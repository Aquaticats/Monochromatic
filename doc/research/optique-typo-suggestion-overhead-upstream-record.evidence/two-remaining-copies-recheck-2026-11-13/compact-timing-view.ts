// Prints keep/timing-summary.jsonl as short text lines, for reading: per step and arm the medians with their range
// and the growth per doubling, then the ratios against the base and against the multiple() fix.
//
// Run: node compact-timing-view.ts > keep/timing-summary.txt

import { readFileSync, } from 'node:fs';

const lines = readFileSync(`${import.meta.dirname}/keep/timing-summary.jsonl`, 'utf8',).split('\n',).filter((line,) => line.length > 0).map((line,) => JSON.parse(line,) as Record<string, unknown>);

for (const line of lines) {
  if ('failedRun' in line || 'runs' in line) {
    console.log(JSON.stringify(line,).slice(0, 400,),);
    continue;
  }

  if ('medians' in line) {
    const medians = line['medians'] as { size: number; rounds: number; medianMs: number; minMs: number; maxMs: number; }[];
    const growth = line['growthPerDoubling'] as { from: number; to: number; exponent: number; }[];

    console.log(
      `${line['step']} | ${line['arm']} | ${medians.map((entry,) => `${entry.size}: ${entry.medianMs} (${entry.minMs} to ${entry.maxMs}, n=${entry.rounds})`).join('; ',)}${growth.length > 0 ? ` | growth ${growth.map((entry,) => entry.exponent).join(', ',)}` : ''}`,
    );
    continue;
  }

  if ('ratioOf' in line && (line['against'] === 'base' || line['against'] === 'multiple' || line['against'] === 'multiple-final' || line['against'] === 'main')) {
    const perSize = line['perSize'] as { size: number; rounds: number; medianRatio: number; minRatio: number; maxRatio: number; slowerIn: number; }[];

    console.log(
      `${line['step']} | ${line['ratioOf']} / ${line['against']} | ${perSize.map((entry,) => `${entry.size}: ${entry.medianRatio} (${entry.minRatio} to ${entry.maxRatio}, slower in ${entry.slowerIn} of ${entry.rounds})`).join('; ',)}`,
    );
  }
}
