// Extra shape probe: the token sweep at one option parser, to tell whether the
// mild superlinearity in tokens is the option probing (which scales with P) or
// per-token buffer or trace work (which does not).

import { spawnSync, } from 'node:child_process';

//region Config

/**
 * Arms probed.
 */
const ARMS: readonly string[] = [
  '1.1.1',
  '1.4.0-dev.2971',
];

/**
 * Token counts probed.
 */
const TOKEN_COUNTS: readonly number[] = [
  1000,
  2000,
  4000,
  8000,
];

/**
 * Option parsers in the grammar. One keeps the probe cost out of the picture.
 */
const OPTION_COUNT = 1;

/**
 * Token length.
 */
const TOKEN_LENGTH = 12;

/**
 * Rounds, warmups and timed parses: a shape test, not a precision test.
 */
const ROUNDS = 3;

/**
 * Warmup parses per fresh process.
 */
const WARMUP = 1;

/**
 * Timed parses per fresh process.
 */
const TIMED = 3;

//endregion Config

//region Entry point

/**
 * Runs every probed point and forwards the records.
 */
function main(): void {
  for (let round = 0; round < ROUNDS; round += 1) {
    for (const tokenCount of TOKEN_COUNTS) {
      for (const arm of ARMS) {
        /** Child process outcome. */
        const child = spawnSync(
          process.execPath,
          [
            '--no-warnings',
            '/bench/scripts/scaling-run.ts',
            `--arm=${arm}`,
            '--sweep=tokens-at-one-option',
            `--level=${tokenCount}`,
            `--optionCount=${OPTION_COUNT}`,
            `--tokenLength=${TOKEN_LENGTH}`,
            `--tokenCount=${tokenCount}`,
            `--warmup=${WARMUP}`,
            `--timed=${TIMED}`,
            `--round=${round}`,
            '--mode=timed',
          ],
          {
            encoding: 'utf8',
            maxBuffer: 32 * 1024 * 1024,
          },
        );

        if (child.stderr !== '' && child.stderr !== null) {
          process.stderr.write(child.stderr,);
        }
        if (child.status !== 0) {
          throw new Error(`probe failed for ${arm} N=${tokenCount}: status ${child.status}`,);
        }

        process.stdout.write(child.stdout,);
      }
    }
  }
}

main();

//endregion Entry point
