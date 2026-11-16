// Runs suite/repeat-one-bun-test-file.ts in the work image with no network, with only this recheck's fixture, arms
// and script mounted.
//
// Run: node run-repeat-one-bun-test-file.ts
// Writes keep/one-bun-test-file-repeated-on-two-arms.jsonl and its stderr log.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
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
  '--entrypoint',
  'node',
  'localhost/optique-prototype-work-current-bun:2026-11-13',
  '/suite/repeat-one-bun-test-file.ts',
], { encoding: 'utf8', maxBuffer: 1024 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/one-bun-test-file-repeated-on-two-arms.jsonl`, `${child.stdout}${JSON.stringify({ containerExit: child.status, },)}\n`,);
writeFileSync(`${HERE}/keep/one-bun-test-file-repeated-on-two-arms.stderr.log`, child.stderr,);
