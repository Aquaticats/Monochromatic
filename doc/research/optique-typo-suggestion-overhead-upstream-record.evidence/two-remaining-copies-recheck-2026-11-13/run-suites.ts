// Runs upstream's packages/core suite on Deno, Node and Bun for the named arms, and the deliberate breaks, in the
// work image with no network. Only this recheck's own folders are mounted: the suite fixture (a copy of the
// prototype agent's installed clone and Deno cache), the staged arms and the script.
//
// Run: node run-suites.ts <label> <arm> ...
// Writes keep/<label>.jsonl and keep/<label>.stderr.log; the suite logs land in suite-fixture/logs.

import { spawnSync, } from 'node:child_process';
import { appendFileSync, writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const [label, ...arms] = process.argv.slice(2,);

if (label === undefined || arms.length === 0) throw new Error('usage: run-suites.ts <label> <arm> ...',);

const child = spawnSync('podman', [
  'run',
  '--rm',
  '--network',
  'none',
  '--cpus',
  '4',
  '--memory',
  '8g',
  '--pids-limit',
  '1024',
  '--blkio-weight',
  '10',
  '--security-opt',
  'label=disable',
  '--volume',
  `${HERE}/suite-fixture:/work:rw`,
  '--volume',
  `${HERE}/arms:/arms:ro`,
  '--volume',
  `${HERE}/suite:/suite:ro`,
  '--env',
  'HOME=/work/container-home',
  '--env',
  'DENO_DIR=/work/.deno',
  '--entrypoint',
  'node',
  'localhost/optique-prototype-work-current-bun:2026-11-13',
  '/suite/run-suites-inside.ts',
  ...arms,
], { encoding: 'utf8', maxBuffer: 1024 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/${label}.jsonl`, child.stdout,);
writeFileSync(`${HERE}/keep/${label}.stderr.log`, child.stderr,);
appendFileSync(`${HERE}/keep/${label}.jsonl`, `${JSON.stringify({ containerExit: child.status, signal: child.signal, },)}\n`,);
