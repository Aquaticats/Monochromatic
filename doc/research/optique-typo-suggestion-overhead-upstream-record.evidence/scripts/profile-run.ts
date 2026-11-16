// Profiles the thousand-path no-separator workload on the versions the report
// attributes time for, using `node --cpu-prof` exactly as the brief asks, then
// analyses each written `.cpuprofile` inside the same container.

import { spawnSync, } from 'node:child_process';

//region Config

/**
 * Versions to profile and how many times to parse in each profile window.
 * Repeat counts differ so a 2.7 s parse and a sub-200 ms parse both collect a
 * usable number of samples.
 */
const PROFILE_TARGETS: readonly { arm: string; repeat: number; }[] = [
  {
    arm: '1.1.1',
    repeat: 3,
  },
  {
    arm: '1.3.6',
    repeat: 40,
  },
  {
    arm: '1.4.0-dev.2971',
    repeat: 40,
  },
  {
    arm: 'owned-4879a44e',
    repeat: 40,
  },
];

/**
 * V8 sampling interval in microseconds; the default 1000 is too coarse for a
 * parse that finishes in tens of milliseconds.
 */
const SAMPLING_INTERVAL_MICROS = 100;

//endregion Config

//region Entry point

/**
 * Runs one profiled workload, analyses the profile, and prints both records.
 */
function main(): void {
  for (const target of PROFILE_TARGETS) {
    /** Profile file name for this arm. */
    const profileName = `commit-${target.arm}.cpuprofile`;
    /** Wall-clock start of the profiled run. */
    const startedAt = performance.now();
    /** Profiled run, writing its profile into the image's profiles directory. */
    const profiled = spawnSync(
      process.execPath,
      [
        '--cpu-prof',
        '--cpu-prof-dir=/bench/profiles',
        `--cpu-prof-name=${profileName}`,
        `--cpu-prof-interval=${SAMPLING_INTERVAL_MICROS}`,
        '/bench/scripts/profile-workload.ts',
        `--arm=${target.arm}`,
        `--repeat=${target.repeat}`,
        '--pathset=real',
        '--count=1000',
      ],
      { encoding: 'utf8', },
    );
    /** Wall-clock duration of the profiled run in milliseconds. */
    const wallMs = performance.now() - startedAt;

    process.stderr.write(profiled.stderr,);

    if (profiled.status !== 0) {
      throw new Error(`profile run failed for ${target.arm}: status ${profiled.status}`,);
    }

    /** Analysis of the profile just written. */
    const analysed = spawnSync(
      process.execPath,
      [
        '/bench/scripts/analyse-profile.ts',
        `--file=/bench/profiles/${profileName}`,
      ],
      { encoding: 'utf8', },
    );

    process.stderr.write(analysed.stderr,);

    if (analysed.status !== 0) {
      throw new Error(`analysis failed for ${target.arm}: status ${analysed.status}`,);
    }

    /** Parsed analysis record. */
    const report = JSON.parse(analysed.stdout,) as Record<string, unknown>;

    process.stdout.write(`${
      JSON.stringify({
        ...report,
        kind: 'profile',
        arm: target.arm,
        repeat: target.repeat,
        wallMs: Number(wallMs.toFixed(1),),
        msPerParse: Number((wallMs / target.repeat).toFixed(3),),
      },)
    }\n`,);
  }
}

main();

//endregion Entry point
