// Runs one standalone script, exactly as written, against chosen @optique/core versions
// baked into the measurement image, over alternating rounds. Nothing is mounted.
//
// Run from the measurement folder:
//   node coordinator-check/run-script-per-arm.ts <script> <rounds> <version> [<version> ...]

import { spawnSync, } from 'node:child_process';
import { readFileSync, } from 'node:fs';

const IMAGE = 'localhost/optique-measure:2026-11-12';
const [scriptPath, roundsText, ...arms] = process.argv.slice(2,);

if (scriptPath === undefined || roundsText === undefined || arms.length === 0) {
  throw new Error('usage: run-script-per-arm.ts <script> <rounds> <version> [<version> ...]',);
}

const rounds = Number.parseInt(roundsText, 10,);
const script = readFileSync(scriptPath, 'utf8',);
const medians = new Map<string, number[]>();

for (let round = 0; round < rounds; round += 1) {
  const order = arms.map((_arm, index,) => arms[(index + round) % arms.length]!);

  for (const arm of order) {
    const child = spawnSync(
      'podman',
      ['run', '--rm', '--cpus', '2', '--memory', '4g', '--pids-limit', '512', '--blkio-weight', '10', '--workdir', `/bench/arms/${arm}`, IMAGE, '--input-type=module-typescript', '--eval', script,],
      { encoding: 'utf8', },
    );

    if (child.status !== 0) {
      console.log(JSON.stringify({ round, arm, failed: child.stderr.slice(0, 800,), },),);
      continue;
    }

    const output = JSON.parse(child.stdout.trim().split('\n',).at(-1,)!,) as { parsedPaths: number; medianMs: number; };

    console.log(JSON.stringify({ round, arm, ...output, },),);
    medians.set(arm, [...(medians.get(arm,) ?? []), output.medianMs,],);
  }
}

for (const arm of arms) {
  const sorted = [...(medians.get(arm,) ?? []),].sort((left, right,) => left - right);

  console.log(JSON.stringify({ summary: arm, script: scriptPath, roundMedians: sorted, },),);
}
