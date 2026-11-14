// Builds the work image with the current Bun, makes a copy of the suite comparison script that writes its logs
// under another prefix (so the Bun 1.3.0 logs stay), and runs it in that image. Prints the build's last lines and
// the comparison's JSON lines.
//
// Run: node run-bun-comparison-on-current-bun.ts

import { spawnSync, } from 'node:child_process';
import { readFileSync, writeFileSync, } from 'node:fs';

const HOME = process.env['HOME'];

if (HOME === undefined) throw new Error('HOME is not set',);

const SCRATCH = `${HOME}/temp/agent/optique-2026-11-13`;
const PROTOTYPE = `${HOME}/temp/agent/upstream-prototype.O1iPjcqr`;
const IMAGE = 'localhost/optique-prototype-work-current-bun:2026-11-13';

const build = spawnSync('podman', ['build', '--tag', IMAGE, '--file', 'Containerfile.work-current-bun', '.',], { cwd: SCRATCH, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);

console.log(JSON.stringify({ step: 'build', exit: build.status, tail: `${build.stdout}${build.stderr}`.trim().split('\n',).slice(-4,), },),);

if (build.status !== 0) throw new Error('the image build failed',);

const original = readFileSync(`${PROTOTYPE}/logs/bun-suite-with-and-without-the-patch.ts`, 'utf8',);

if (!original.includes('17-bun-',)) throw new Error('the log prefix was not found in the comparison script',);

writeFileSync(`${PROTOTYPE}/logs/bun-suite-with-and-without-the-patch-on-current-bun.ts`, original.replaceAll('17-bun-', '19-current-bun-',),);

const run = spawnSync('podman', [
  'run',
  '--rm',
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
  `${PROTOTYPE}:/work:rw`,
  '--workdir',
  '/work/optique',
  '--env',
  'HOME=/work/container-home',
  IMAGE,
  'node',
  '/work/logs/bun-suite-with-and-without-the-patch-on-current-bun.ts',
], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);

writeFileSync(`${SCRATCH}/bun-suite-comparison-on-current-bun.jsonl`, run.stdout,);
console.log(JSON.stringify({ step: 'run', exit: run.status, stderr: run.stderr.slice(0, 400,), },),);
console.log(run.stdout.split('\n',).map((line,) => line.slice(0, 900,)).join('\n',),);
