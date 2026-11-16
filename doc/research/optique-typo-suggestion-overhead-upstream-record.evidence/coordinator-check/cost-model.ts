// Cost model for one successful parse on @optique/core 1.1.1, derived from the source:
//   each positional token is offered to every option or flag parser (constructs.ts object loop);
//   each mismatch collects every option name and computes an edit distance against each
//   (primitives.ts no-match path, suggestion.ts findSimilar);
//   one edit distance fills a table of token length times name length.
// So edit-distance cells per parse = tokens x option parsers x sum over names of (token length x name length).
// This script counts those cells for the old commit parser on the measured input, divides the profiled
// edit-distance time by them, and uses the resulting time per cell to predict the cost of one search at the
// parameters upstream issue 38 names (50 options, names of about 20 characters).
//
// Run from the measurement folder: node coordinator-check/cost-model.ts

import { readFileSync, } from 'node:fs';

function prefixes(name: string, shortestStem: number,): string[] {
  const stem = name.slice(2,);
  const names: string[] = [];

  for (let length = shortestStem; length <= stem.length; length += 1) names.push(`--${stem.slice(0, length,)}`,);

  return names;
}

// One entry per option or flag parser of the old commit parser, with every name it declares.
const parsers: readonly (readonly string[])[] = [
  ['-a', '--all',],
  ['-o', '--only',],
  ['--no-only',],
  ['-i', ...prefixes('--include', 3,),],
  ['--interactive',],
  ['-p', '--patch',],
  ['--amend',],
  ['--allow-empty',],
  prefixes('--dry-run', 2,),
  prefixes('--short', 2,),
  prefixes('--porcelain', 3,),
  prefixes('--long', 1,),
  ['-z', ...prefixes('--null', 2,),],
  ['-m', '--message',],
  ['-F', '--file',],
  ['-C', '--reuse-message',],
  ['-c', '--reedit-message',],
  ['--squash',],
  ['--fixup',],
  ['--author',],
  ['--date',],
  ['--cleanup',],
  ['--trailer',],
  ['-t', '--template',],
  ['-U', '--unified',],
  ['--inter-hunk-context',],
  ['--pathspec-from-file',],
  ['--pathspec-file-nul',],
  ['--no-enforce-only',],
];

// findSimilar lower-cases and skips duplicate candidates, so names differing only in case count once per spelling.
const names = [...new Set(parsers.flat(),),];
const nameLengthSum = names.reduce((sum, name,) => sum + name.length, 0,);
const paths = JSON.parse(readFileSync('data/real-paths.json', 'utf8',),) as string[];
const tokenLengthSum = paths.reduce((sum, path,) => sum + path.length, 0,);
const probes = paths.length * parsers.length;
const cellsPerParse = parsers.length * tokenLengthSum * nameLengthSum;

const profile = readFileSync('results/profiles.jsonl', 'utf8',)
  .split('\n',)
  .filter((line,) => line.startsWith('{',))
  .map((line,) => JSON.parse(line,) as { arm: string; repeat: number; parseMs: number; topFunctions: { function: string; ms: number; }[]; })
  .find((record,) => record.arm === '1.1.1');

if (profile === undefined) throw new Error('no 1.1.1 profile record',);

const editDistanceMs = profile.topFunctions.find((entry,) => entry.function === 'levenshteinDistance')!.ms;
const nanosecondsPerCell = (editDistanceMs * 1e6) / (cellsPerParse * profile.repeat);

const noteOptions = 50;
const noteNameLength = 20;
const noteTokenLength = 20;
const noteCells = noteOptions * noteNameLength * noteTokenLength;

console.log(JSON.stringify({
  optionParsers: parsers.length,
  distinctNames: names.length,
  nameLengthSum,
  tokens: paths.length,
  meanTokenLength: Number((tokenLengthSum / paths.length).toFixed(1,),),
  mismatchProbesPerParse: probes,
  editDistanceCellsPerParse: cellsPerParse,
  profiledParses: profile.repeat,
  profiledEditDistanceMs: editDistanceMs,
  nanosecondsPerCell: Number(nanosecondsPerCell.toFixed(2,),),
  issue38: {
    cellsForOneSearch: noteCells,
    predictedMsForOneSearch: Number(((noteCells * nanosecondsPerCell) / 1e6).toFixed(3,),),
    searchesInOneThousandPathParse: 1000 * noteOptions,
    predictedMsForThatParse: Number(((1000 * noteOptions * noteCells * nanosecondsPerCell) / 1e6).toFixed(0,),),
  },
},),);
