// Runs one command in the recheck image with no network and nothing mounted, and saves what it prints under keep/.
//
// Run: node run-in-recheck-image.ts <label> <entrypoint> <arguments...>
// Writes keep/<label>.jsonl and keep/<label>.stderr.log; the last line of the first says how the container ended.

import { spawnSync, } from 'node:child_process';
import { appendFileSync, writeFileSync, } from 'node:fs';

const [label, entrypoint, ...rest] = process.argv.slice(2,);

if (label === undefined || entrypoint === undefined) throw new Error('usage: run-in-recheck-image.ts <label> <entrypoint> <arguments...>',);

const KEEP = `${import.meta.dirname}/keep`;
const child = spawnSync('podman', [
  'run',
  '--rm',
  '--network',
  'none',
  '--cpus',
  '2',
  '--memory',
  '8g',
  '--pids-limit',
  '1024',
  '--blkio-weight',
  '10',
  '--entrypoint',
  entrypoint,
  'localhost/optique-two-remaining-copies-recheck:2026-11-13',
  ...rest,
], { encoding: 'utf8', maxBuffer: 1024 * 1024 * 1024, },);

writeFileSync(`${KEEP}/${label}.jsonl`, child.stdout,);
writeFileSync(`${KEEP}/${label}.stderr.log`, child.stderr,);
appendFileSync(`${KEEP}/${label}.jsonl`, `${JSON.stringify({ containerExit: child.status, signal: child.signal, },)}\n`,);
