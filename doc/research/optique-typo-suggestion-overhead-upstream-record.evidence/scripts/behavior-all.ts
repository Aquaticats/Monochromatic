// Runs the behavior probe for every arm in this image, one fresh process per
// arm, and forwards every record to stdout. Keeps the container invocation to
// a single command.

import { spawnSync, } from 'node:child_process';

import { MEASURED_ARMS, } from './workloads.ts';

//region Entry point

/**
 * Probes every arm and copies its output through.
 */
function main(): void {
  for (const arm of MEASURED_ARMS) {
    /** One arm's behavior probe. */
    const child = spawnSync(
      process.execPath,
      [
        '--no-warnings',
        '/bench/scripts/behavior.ts',
        `--arm=${arm}`,
      ],
      { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },
    );

    process.stderr.write(child.stderr,);

    if (child.status !== 0) {
      throw new Error(`behavior probe failed for ${arm}: status ${child.status}`,);
    }

    process.stdout.write(child.stdout,);
  }
}

main();

//endregion Entry point
