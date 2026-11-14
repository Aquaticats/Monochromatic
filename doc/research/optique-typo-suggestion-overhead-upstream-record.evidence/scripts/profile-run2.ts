// Part 3 profiling: runs each target shape under `node --cpu-prof` and analyses
// the written profile inside the same container.

import { spawnSync, } from 'node:child_process';

//region Config

/**
 * One profiling target: which arm, which shape, and how many parses.
 */
type ProfileTarget = {
  readonly arm: string;
  readonly shape: 'commit' | 'separator';
  readonly repeat: number;
};

/**
 * Targets. Repeat counts are chosen so every window holds at least a second of
 * parsing at that arm's speed.
 */
const TARGETS: readonly ProfileTarget[] = [
  {
    arm: '1.3.2',
    shape: 'commit',
    repeat: 40,
  },
  {
    arm: '1.4.0-dev.2664',
    shape: 'commit',
    repeat: 30,
  },
  {
    arm: '1.3.6',
    shape: 'separator',
    repeat: 60,
  },
  {
    arm: '1.4.0-dev.2664',
    shape: 'separator',
    repeat: 40,
  },
  {
    arm: '1.4.0-dev.2971',
    shape: 'separator',
    repeat: 30,
  },
];

//endregion Config

//region Entry point

/**
 * Profiles every target and prints one analysis record per target.
 */
function main(): void {
  for (const target of TARGETS) {
    /** Profile file name. */
    const profileName = `${target.shape}-${target.arm}.cpuprofile`;
    /** Script to profile. */
    const script = target.shape === 'commit'
      ? '/bench/scripts/profile-workload.ts'
      : '/bench/scripts/profile-separator.ts';
    /** Extra arguments for that script. */
    const scriptArguments = target.shape === 'commit'
      ? [
        '--pathset=real',
        '--count=1000',
      ]
      : ['--tokenCount=1000',];
    /** Wall-clock start. */
    const startedAt = performance.now();
    /** Profiled run. */
    const profiled = spawnSync(
      process.execPath,
      [
        '--cpu-prof',
        '--cpu-prof-dir=/bench/profiles',
        `--cpu-prof-name=${profileName}`,
        '--cpu-prof-interval=100',
        script,
        `--arm=${target.arm}`,
        `--repeat=${target.repeat}`,
        ...scriptArguments,
      ],
      { encoding: 'utf8', },
    );
    /** Wall-clock duration of the profiled run. */
    const wallMs = performance.now() - startedAt;

    process.stderr.write(profiled.stderr,);

    if (profiled.status !== 0) {
      throw new Error(`profile run failed for ${target.arm}/${target.shape}: status ${profiled.status}`,);
    }

    /** Analysis of the profile. */
    const analysed = spawnSync(
      process.execPath,
      [
        '/bench/scripts/analyse-profile.ts',
        `--file=/bench/profiles/${profileName}`,
        `--root=${target.shape === 'commit' ? 'commit' : 'parseSync'}`,
      ],
      { encoding: 'utf8', },
    );

    process.stderr.write(analysed.stderr,);

    if (analysed.status !== 0) {
      throw new Error(`analysis failed for ${target.arm}/${target.shape}: status ${analysed.status}`,);
    }

    /** Parsed analysis record. */
    const report = JSON.parse(analysed.stdout,) as Record<string, unknown>;

    process.stdout.write(`${
      JSON.stringify({
        ...report,
        kind: 'profile',
        arm: target.arm,
        shape: target.shape,
        repeat: target.repeat,
        wallMs: Number(wallMs.toFixed(1),),
        msPerParse: Number((wallMs / target.repeat).toFixed(3),),
      },)
    }\n`,);
  }
}

main();

//endregion Entry point
