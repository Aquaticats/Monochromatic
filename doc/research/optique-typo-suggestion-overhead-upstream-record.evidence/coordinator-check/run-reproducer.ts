// Runs coordinator-check/reproducer.ts, exactly as written, against several @optique/core
// versions already baked into the measurement image. Nothing is mounted: the script text is
// passed on the command line and each run starts in that version's folder so the bare
// `@optique/core` imports resolve to it. Versions alternate within each round so that host
// load drifting during the run does not land on one version.
//
// Run from the measurement folder: node coordinator-check/run-reproducer.ts

import { spawnSync, } from 'node:child_process';
import { readFileSync, } from 'node:fs';

const IMAGE = 'localhost/optique-measure:2026-11-12';
const ARMS = ['1.1.1', '1.3.1', '1.3.2', '1.3.6', '1.4.0-dev.2664', '1.4.0-dev.2971',] as const;
const ROUNDS = 4;
const script = readFileSync('coordinator-check/reproducer.ts', 'utf8',);
const medians = new Map<string, number[]>();

for (let round = 0; round < ROUNDS; round += 1) {
  const order = ARMS.map((_arm, index,) => ARMS[(index + round) % ARMS.length]!);

  for (const arm of order) {
    const child = spawnSync(
      'podman',
      [
        'run',
        '--rm',
        '--cpus',
        '2',
        '--memory',
        '4g',
        '--pids-limit',
        '512',
        '--blkio-weight',
        '10',
        '--workdir',
        `/bench/arms/${arm}`,
        IMAGE,
        '--input-type=module-typescript',
        '--eval',
        script,
      ],
      { encoding: 'utf8', },
    );

    if (child.status !== 0) {
      console.log(JSON.stringify({ round, arm, failed: child.stderr.slice(0, 600,), },),);
      continue;
    }

    const output = JSON.parse(child.stdout.trim().split('\n',).at(-1,)!,) as { parsedPaths: number; medianMs: number; };

    console.log(JSON.stringify({ round, arm, ...output, },),);
    medians.set(arm, [...(medians.get(arm,) ?? []), output.medianMs,],);
  }
}

for (const arm of ARMS) {
  const sorted = [...(medians.get(arm,) ?? []),].sort((left, right,) => left - right);

  console.log(JSON.stringify({ summary: arm, roundMedians: sorted, lowest: sorted[0], highest: sorted.at(-1,), },),);
}
