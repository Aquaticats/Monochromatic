/**
 The lines the stand-in Pi writes about what it was given, which are names and counts and never a value.

 @module
 */

import { appendFile, } from 'node:fs/promises';
import { env, } from 'node:process';
import { NESTED_RUN_VARIABLE, } from './driver-constants.ts';
import type { SecretValue, } from './secret-value-masking.ts';

/**
 Writes what this start was given: its own arguments, whether it runs nested, the names its commands could read, and the names of the
 values it armed. A test learns what Pi could reach without any value reaching a log, and so a model provider.

 @param log - the file the test reads these lines from

 @param argumentsOfTheStart - the words this start was given, after the program

 @param values - the values the driver handed over, of which only the names are written

 @example
 ```ts
 await logTheNamesOfWhatPiCouldRead({ log: '/run/fixture.log', argumentsOfTheStart: ['--mode', 'rpc',], values: [], },);
 ```
 */
export async function logTheNamesOfWhatPiCouldRead({
  log,
  argumentsOfTheStart,
  values,
}: {
  readonly log: string;
  readonly argumentsOfTheStart: readonly string[];
  readonly values: readonly SecretValue[];
},): Promise<void> {
  await appendFile(
    log,
    `arguments ${JSON.stringify(argumentsOfTheStart,)}\n`,
  );
  await appendFile(
    log,
    `nested ${env[NESTED_RUN_VARIABLE] ?? 'unset'}\n`,
  );
  await appendFile(
    log,
    `environment-names ${JSON.stringify(Object.keys(env,)
      .toSorted(),)}\n`,
  );
  await appendFile(
    log,
    `secret-value-names ${JSON.stringify(values.map(function nameOf(value,) {
      return value.name;
    },),)}\n`,
  );
}
