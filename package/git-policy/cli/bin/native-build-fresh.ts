#!/usr/bin/env node
/**
 Build the native wrapper executable, and leave at the path this build names the program compiled from these sources.

 This is the command line; the work and its reasons are in `native-build-fresh-steps.ts`, and the diagnosis of what a bare
 `cargo build` leaves at that path is in
 `doc/troubleshooting/cargo-fresh-build-leaves-another-checkouts-program.md`.

 `--release` builds the optimised profile, `--bin <name>` names another executable, `--directory <path>` another package
 directory, and `--cargo <program>` another `cargo`. Everything else is read from Cargo itself.
 */
import { resolve } from 'node:path';

import { parseArguments } from './native-build-fresh-arguments.ts';
import {
  buildFresh,
  refusalText,
} from './native-build-fresh-steps.ts';

/**
 The package directory this script is written into, which is what its tasks build when they name none.

 @returns the absolute directory

 @example
 ```ts
 const own = ownPackageDirectory();
 ```
 */
function ownPackageDirectory(): string {
  return resolve(
    import.meta.dirname,
    '..',
  );
}

/**
 What one run of this command came to.

 @returns zero when the program is the one this build compiled, and one when this step refused

 @example
 ```ts
 process.exitCode = await main();
 ```
 */
async function main(): Promise<number> {
  /**
   What the command line asked for.
   */
  const asked = parseArguments({
    words: process.argv
      .slice(2,),
    ownPackage: ownPackageDirectory(),
  },);
  try {
    /**
     What this build came to.
     */
    const built = await buildFresh({
      ...asked,
      environment: process.env,
    },);
    // Cargo's own lines are this command's output: a person reads the same words a bare build printed.
    process.stdout
      .write(built.buildStdout,);
    process.stderr
      .write(built.buildStderr,);
    console.log(`${built.artifactPath} is the program this build compiled from the sources of ${asked.packageDirectory}`);
    return 0;
  }
  catch (caught) {
    // The failure is a person's to read, and the status is what a task's dependents see.
    console.error(refusalText(caught,),);
    return 1;
  }
}

process.exitCode = await main();
