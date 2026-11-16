// Runs the two behaviour probes, with their three controls, on the arm the kept patch produces and on its twin whose
// states are always views, against the base. The probe and arms folders are mounted read-only, because both changed
// after the recheck image was built; the container has no network.
//
// Run: node run-probes-on-the-kept-patch.ts
// Writes keep/behaviour-probes-on-the-kept-patch.jsonl and its stderr log.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
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
  '--security-opt',
  'label=disable',
  '--volume',
  `${HERE}/probe:/probe-now:ro`,
  '--volume',
  `${HERE}/arms:/staged:ro`,
  '--entrypoint',
  'node',
  'localhost/optique-two-remaining-copies-recheck:2026-11-13',
  '/probe-now/compare-builds-in-container.ts',
  'base=/staged/base',
  'both=/staged/both',
  'multiple-final=/staged/multiple-final',
  'multiple-final-all-views=/staged/multiple-final-all-views',
], { encoding: 'utf8', maxBuffer: 1024 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/behaviour-probes-on-the-kept-patch.jsonl`, `${child.stdout}${JSON.stringify({ containerExit: child.status, },)}\n`,);
writeFileSync(`${HERE}/keep/behaviour-probes-on-the-kept-patch.stderr.log`, child.stderr,);
