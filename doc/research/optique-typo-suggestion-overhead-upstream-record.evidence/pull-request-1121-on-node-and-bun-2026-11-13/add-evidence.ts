// Adds the records of the Node and Bun timing of upstream's pull request 1121, and the two small scripts that
// checked the research record's rewrite against the archive's own records, to the evidence archive beside the
// research document, from an explicit list of files. The words that mark a file as local-only are read from a local
// list that is itself not archived.
//
// Run: node add-evidence.ts            (stages and scans only)
//      node add-evidence.ts --write    (also writes the archive)

import { spawnSync, } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, } from 'node:fs';
import { homedir, } from 'node:os';

const HERE = import.meta.dirname;
const HOME = homedir();
const RECAST = `${HOME}/temp/agent/optique-research-record-recast-to-ratios-2026-11-13`;
const ARCHIVE = '/var/mnt/encrypted/repository/Monochromatic/doc/research/optique-typo-suggestion-overhead-upstream-record.evidence.zip';
const STAGE = `${HERE}/evidence-stage`;
const SETS = [
  {
    root: HERE,
    folder: 'pull-request-1121-on-node-and-bun-2026-11-13',
    names: [
      'Containerfile',
      '.containerignore',
      'run-pull-request-1121-on-node-and-bun.ts',
      'run-timing-inside.ts',
      'summarise.ts',
      'add-evidence.ts',
      'keep/staging.json',
      'keep/arm-package-build.jsonl',
      'keep/timing-rounds.jsonl',
      'keep/timing-summary.txt',
    ],
  },
  {
    root: RECAST,
    folder: 'recast-to-ratios-2026-11-13',
    names: [
      'show-length-sweep.ts',
      'keep/length-sweep.txt',
      'check-live-readme.ts',
      'check-live-readme.log',
    ],
  },
] as const;

rmSync(STAGE, { recursive: true, force: true, },);

for (const { root, folder, names, } of SETS) {
  for (const name of names) {
    if (!existsSync(`${root}/${name}`,)) throw new Error(`missing: ${root}/${name}`,);

    const target = `${STAGE}/${folder}/${name}`;

    mkdirSync(target.split('/',).slice(0, -1,).join('/',), { recursive: true, },);
    cpSync(`${root}/${name}`, target,);
  }
}

// Words that would mark a file as one that must stay out of a public archive, one regular expression per line.
const WORDS = `${HOME}/temp/agent/optique-2026-11-13/words-that-mark-a-local-only-file.txt`;
const scan = spawnSync('rg', ['--line-number', '--no-heading', '--ignore-case', '--hidden', '--file', WORDS, STAGE,], { encoding: 'utf8', },);
// The home folder's path has no place in a public archive either.
const personal = spawnSync('rg', ['--line-number', '--no-heading', '--hidden', '--fixed-strings', '--regexp', HOME, STAGE,], { encoding: 'utf8', },);

console.log(JSON.stringify({
  linesNamingAWordToCheck: scan.stdout.trim().split('\n',).filter((line,) => line.length > 0).map((line,) => line.slice(STAGE.length + 1, STAGE.length + 260,)),
  linesNamingTheHomeFolder: personal.stdout.trim().split('\n',).filter((line,) => line.length > 0).map((line,) => line.slice(STAGE.length + 1, STAGE.length + 260,)),
},),);

if (process.argv.includes('--write',)) {
  for (const { folder, } of SETS) {
    const zip = spawnSync('zip', ['--recurse-paths', '--quiet', ARCHIVE, folder,], { cwd: STAGE, encoding: 'utf8', },);

    if (zip.status !== 0) throw new Error(`zip failed: ${zip.stderr.slice(0, 300,)}`,);
  }

  const listing = spawnSync('unzip', ['-l', ARCHIVE,], { encoding: 'utf8', },);
  const lines = listing.stdout.split('\n',);

  console.log(JSON.stringify({
    entriesPerFolder: SETS.map(({ folder, },) => ({ folder, entries: lines.filter((line,) => line.includes(`${folder}/`,)).length, })),
    total: lines.at(-2,)?.trim(),
    namesOfLocalOnlyFiles: lines.filter((line,) => new RegExp(readFileSync(WORDS, 'utf8',).trim().split('\n',).join('|',), 'iu',).test(line,)).length,
    namesMentioningReviewReport: lines.filter((line,) => /review-report/iu.test(line,)).length,
  },),);
}
