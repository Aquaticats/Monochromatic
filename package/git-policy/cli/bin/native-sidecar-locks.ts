/**
 The package's `lint` task that fails on a sidecar lock this package's dependencies moved past.

 The check itself is `native-sidecar-lock-check.ts`, which the native gate also runs before it builds anything; this file
 only reads this package's directory, prints what disagrees and sets the exit code.
 */
import { resolve, } from 'node:path';

import {
  describeProblem,
  sidecarLockProblems,
} from './native-sidecar-lock-check.ts';

/**
 Directory of the package this run checks: the one this task runs in.
 */
const packageDirectory = resolve(
  import.meta.dirname,
  '..',
);
/**
 Every sidecar that disagrees with the package's lock.
 */
const problems = await sidecarLockProblems({ packageDirectory, });
for (const problem of problems) {
  console.error(describeProblem({
    problem,
    packageDirectory,
  }));
}
// A script reads the result from the exit code: zero only when every sidecar's lock agrees with this package's.
if (problems.length > 0)
  process.exitCode = 1;
