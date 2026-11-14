// Prints, from the second round's fit record, everything it holds about the sweep over token length, so the
// research record's figures for that sweep (slope per character, the value at twelve characters) can be checked
// against the record and not against prose.
//
// Run: node show-length-sweep.ts > keep/length-sweep.txt

import { readFileSync, } from 'node:fs';

const fit = JSON.parse(
  readFileSync(`${import.meta.dirname}/archive/results-scaling/fit.json`, 'utf8',),
) as Record<string, unknown>;

console.log(`top-level keys: ${Object.keys(fit,).join(', ',)}`,);

function walk(value: unknown, path: string,): void {
  if (value === null || typeof value !== 'object') return;

  for (const [key, child,] of Object.entries(value as Record<string, unknown>,)) {
    const here = `${path}.${key}`;

    if (/length/iu.test(here,)) {
      console.log(`${here}: ${JSON.stringify(child,).slice(0, 2500,)}`,);
      continue;
    }

    walk(child, here,);
  }
}

walk(fit, 'fit',);

// The sweep over token length is the entry of `levels` whose fixed token length is 0 (the length is what varies).
const levels = fit['levels'] as Record<string, unknown>[];

console.log(`\nlevels: ${levels.length}`,);

for (const [index, level,] of levels.entries()) {
  console.log(`\nlevels[${index}] keys: ${Object.keys(level,).join(', ',)}`,);
  console.log(JSON.stringify(level,).slice(0, index === 2 ? 9000 : 400,),);
}
