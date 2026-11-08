/**
 Command-line entry of the footprint measurement: `measure-footprint.ts <tree> [attempts] [seconds per attempt]`. Prints a
 note of what happened and then the `btrfs filesystem du` table, whose last row the pre-flight reads. Run as root, because
 the measurement counts files only root can read.

 @module
 */

import {
  argv,
  exit,
} from 'node:process';
import { measureFootprint, } from './footprint-measure.ts';

/**
 Attempts when none is given: a measurement takes about 12.5 minutes (plan, "Measurements") and a worktree is removed a few
 times an hour, so one retry usually suffices; three is the author's bound.
 */
const DEFAULT_ATTEMPTS = 3;

/**
 Seconds of one walk when none is given: the author's bound, as the guided runner's own measurement deadline.
 */
const DEFAULT_SECONDS_PER_ATTEMPT = 5_400;

/**
 The parts of the value this function uses, taken out of it.
 */
const [tree = '', attemptsText = '', secondsText = '',] = argv.slice(2,);
if (tree === '') {
  console.error('usage: measure-footprint.ts <tree> [attempts] [seconds per attempt]',);
  exit(2,);
}
/**
 What the measurement came to.
 */
const result = await measureFootprint({
  path: tree,
  attempts: attemptsText === '' ? DEFAULT_ATTEMPTS : Number(attemptsText,),
  secondsPerAttempt: secondsText === '' ? DEFAULT_SECONDS_PER_ATTEMPT : Number(secondsText,),
},);
if (result.succeeded) {
  console.log(result.lines
    .join('\n',),);
}
else {
  console.error(`${new Date().toISOString()} STOP: ${result.lines
    .join('\n',)}`,);
  exit(1,);
}
