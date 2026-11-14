// Checks where the quadratic token floor comes from: profiles the separator
// shape at two token counts for one arm and compares the InputTrace share.

import { spawnSync, } from 'node:child_process';

//region Config

/**
 * Arm profiled. 1.3.6 has no suggestion work on the separator shape, so the
 * profile shows the bookkeeping alone.
 */
const ARM = '1.3.6';

/**
 * Token counts profiled.
 */
const TOKEN_COUNTS: readonly number[] = [
  1000,
  4000,
];

//endregion Config

//region Entry point

/**
 * Profiles both sizes and prints the analyses.
 */
function main(): void {
  for (const tokenCount of TOKEN_COUNTS) {
    /** Parses to run; scaled down for the larger size. */
    const repeat = tokenCount === 1000 ? 40 : 10;
    /** Profile file name. */
    const profileName = `separator-${ARM}-${tokenCount}.cpuprofile`;
    /** Profiled run. */
    const profiled = spawnSync(
      process.execPath,
      [
        '--cpu-prof',
        '--cpu-prof-dir=/bench/profiles',
        `--cpu-prof-name=${profileName}`,
        '--cpu-prof-interval=100',
        '/bench/scripts/profile-separator.ts',
        `--arm=${ARM}`,
        `--repeat=${repeat}`,
        `--tokenCount=${tokenCount}`,
      ],
      { encoding: 'utf8', },
    );

    process.stderr.write(profiled.stderr,);

    if (profiled.status !== 0) {
      throw new Error(`profile failed at ${tokenCount}: status ${profiled.status}`,);
    }

    /** Analysis of this profile. */
    const analysed = spawnSync(
      process.execPath,
      [
        '/bench/scripts/analyse-profile.ts',
        `--file=/bench/profiles/${profileName}`,
        '--root=parseSync',
      ],
      { encoding: 'utf8', },
    );

    process.stderr.write(analysed.stderr,);

    if (analysed.status !== 0) {
      throw new Error(`analysis failed at ${tokenCount}: status ${analysed.status}`,);
    }

    /** Parsed analysis. */
    const report = JSON.parse(analysed.stdout,) as Record<string, unknown>;

    process.stdout.write(`${
      JSON.stringify({
        ...report,
        kind: 'profile',
        arm: ARM,
        shape: 'separator',
        tokenCount,
        repeat,
        msPerParse: Number((Number(report['parseMs'],) / repeat).toFixed(3),),
      },)
    }\n`,);
  }
}

main();

//endregion Entry point
