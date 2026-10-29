#!/usr/bin/env node
/**
 `env-local-json edit <file>`: run `sops edit` with the identity resolved by
 the same precedence as the reader, instead of a hardcoded identity path.

 Exit codes follow sops: the editor/sops status is passed through, argument
 errors exit 2, and identity or tooling failures exit 1.
 */

import { homedir, } from 'node:os';

import {
  caughtValueText,
} from '@monochromatic-dev/caught-value-core/ts';

import {
  EnvLocalJsonMissingIdentityError,
  EnvLocalJsonSopsMissingError,
} from '../errors.ts';
import {
  buildSopsChildEnv,
  resolveSopsIdentity,
} from '../identity.ts';
import { resolveSopsExecutable, } from '../sops-executable.ts';
import { runSopsEdit, } from '../sops.ts';

//region Constants

/**
 Usage text shown for malformed invocations.
 */
const USAGE = `usage: env-local-json edit <file>

Edit the sops/age-encrypted environment store in <file>.
The decryption identity is resolved like mise: MISE_SOPS_AGE_KEY,
MISE_SOPS_AGE_KEY_FILE, SOPS_AGE_KEY_FILE, SOPS_AGE_KEY, then the mise
configuration directory's age.txt.`;

/**
 Exit status for malformed command-line usage.
 */
const USAGE_EXIT_CODE = 2;

/**
 Exit status for identity, tooling, or subprocess failures.
 */
const FAILURE_EXIT_CODE = 1;

//endregion Constants

//region Entry point

/**
 Runs the CLI for one argument vector.

 @param argv - arguments after the executable and script name

 @returns process exit status to pass through

 @example
 ```ts
 await main({ argv: ['edit', '.env.local.json'] });
 ```
 */
async function main({ argv, }: { readonly argv: readonly string[]; },): Promise<number> {
  /**
   Subcommand name and store path from the argument vector.
   */
  const [command, file,] = argv;
  if ((command !== 'edit') || (file === undefined)
    || (argv.length > 2)) {
    console.error(USAGE,);
    return USAGE_EXIT_CODE;
  }

  try {
    /**
     Decryption identity resolved before sops starts.
     */
    const identity = await resolveSopsIdentity({
      env: process.env,
      home: homedir(),
    },);
    /**
     Resolved sops executable.
     */
    const sopsPath = await resolveSopsExecutable({ env: process.env, },);
    /**
     Curated child environment that keeps the user's editor selection.
     */
    const childEnv = buildSopsChildEnv({
      env: process.env,
      home: homedir(),
      identity,
      interactive: true,
    },);
    /**
     Exit outcome of the interactive editor session.
     */
    const outcome = await runSopsEdit({
      childEnv,
      sopsPath,
      storePath: file,
    },);
    if (outcome.kind === 'signaled') {
      console.error(`sops edit was terminated by ${outcome.signal}`,);
      return FAILURE_EXIT_CODE;
    }
    return outcome.code;
  }
  catch (error) {
    if ((error instanceof EnvLocalJsonMissingIdentityError)
      || (error instanceof EnvLocalJsonSopsMissingError)) {
      console.error(error.message,);
      return FAILURE_EXIT_CODE;
    }
    console.error(caughtValueText(error,),);
    return FAILURE_EXIT_CODE;
  }
}

//endregion Entry point

/**
 Exit status produced by this invocation.
 */
const exitStatus = await main({ argv: process.argv
  .slice(2,), },);
process.exitCode = exitStatus;
