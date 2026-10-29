/**
 Credential lookup for the Luna situation check.

 STOPGAP: the credential lives in the sops-encrypted `.env.local.json` and is
 decrypted by mise, so a hook launched outside a mise-provided environment has
 no `STOP_REMINDER_LLMGATEWAY_API_KEY`. This module reads the environment first
 and, only when the key is absent, asks `mise env --json` once. It exists
 until the repository has a credential mechanism that does not depend on mise,
 at which point this file is replaced and nothing else changes.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { text, } from 'node:stream/consumers';

import {
  LUNA_KEY_ENV,
  LunaKeyMissingError,
} from './luna-situation.ts';

/**
 Module logger for credential lookup; never receives a credential value.
 */
const logger = tagged({ tag: 'stop-reminder-luna-key', },);

/**
 Milliseconds the `mise env` fallback may take.

 Measured at 0.10 to 0.14 seconds; the margin covers a cold mise cache without
 letting a hung process delay a stop.
 */
const MISE_ENV_TIMEOUT_MS = 3_000;

/**
 Runs `mise env --json` in a directory and returns its parsed environment.

 @param cwd - session directory, where mise finds the project configuration

 @returns environment variables mise would set

 @throws LunaKeyMissingError when mise is absent, slow, or prints something other than a JSON object

 @example
 ```ts
 const env = await miseEnvironment('/path/to/project');
 ```
 */
async function miseEnvironment(cwd: string,): Promise<Readonly<Record<string, unknown>>> {
  /**
   Function-scoped logger so every line names this call.
   */
  const l = tagged({
    tag: miseEnvironment.name,
    l: logger,
  },);
  try {
    /**
     Child process; `mise` decrypts the sops file only inside it, and the timeout ends a hung one.
     */
    const child = spawn(
      'mise',
      [
        'env',
        '--json',
      ],
      {
        cwd,
        timeout: MISE_ENV_TIMEOUT_MS,
        stdio: [
          'ignore',
          'pipe',
          'ignore',
        ],
      },
    );
    /**
     Output and exit status, awaited together so a spawn error rejects at once.
     */
    const [stdout, closed,] = await Promise.all([
      text(child.stdout,),
      once(
        child,
        'close',
      ),
    ],);
    /**
     Exit status, the first value of the `close` event.
     */
    const exitCode: unknown = closed[0];
    if (exitCode !== 0) {
      throw new LunaKeyMissingError(`mise env exited with status ${String(exitCode,)}`,);
    }
    /**
     Parsed output, checked before any field is read.
     */
    const parsed: unknown = JSON.parse(stdout,);
    if (((typeof parsed) !== 'object') || (parsed === null)
      || Array.isArray(parsed,)) {
      throw new LunaKeyMissingError('mise env printed something other than a JSON object',);
    }
    return Object.fromEntries(Object.entries(parsed,),);
  }
  catch (error) {
    l.warn(`mise env fallback failed: ${String(error,)}`,);
    if (error instanceof LunaKeyMissingError) {
      throw error;
    }
    throw new LunaKeyMissingError(
      `${LUNA_KEY_ENV} is unavailable and mise env failed`,
      { cause: error, },
    );
  }
}

/**
 Returns the Luna credential from the environment, else from one `mise env` read.

 @param env - process environment, injected for tests

 @param cwd - session directory for the mise fallback

 @param readMiseEnvironment - mise reader, injected so tests never spawn a process

 @returns the credential; never logged

 @throws LunaKeyMissingError when neither source supplies a non-empty key

 @example
 ```ts
 const key = await readLunaKey({ env: process.env, cwd });
 ```
 */
async function readLunaKey(
  {
    env,
    cwd,
    readMiseEnvironment = miseEnvironment,
  }: {
    readonly env: NodeJS.ProcessEnv;
    readonly cwd: string;
    readonly readMiseEnvironment?: (cwd: string,) => Promise<Readonly<Record<string, unknown>>>;
  },
): Promise<string> {
  /**
   Function-scoped logger so every line names this call.
   */
  const l = tagged({
    tag: readLunaKey.name,
    l: logger,
  },);
  /**
   Credential already present in this process.
   */
  const fromEnv = env[LUNA_KEY_ENV];
  if ((fromEnv !== undefined) && (fromEnv !== '')) {
    l.debug('using the credential from the process environment',);
    return fromEnv;
  }
  // Debug, not info: console.info writes to stdout, which a hook reserves for its JSON output.
  l.debug('credential absent from the environment; reading it through mise env',);
  /**
   Credential from the one mise read.
   */
  const fromMise = (await readMiseEnvironment(cwd,))[LUNA_KEY_ENV];
  if (((typeof fromMise) !== 'string') || (fromMise === '')) {
    throw new LunaKeyMissingError(`${LUNA_KEY_ENV} is not provided by mise env`,);
  }
  return fromMise;
}

export {
  MISE_ENV_TIMEOUT_MS,
  miseEnvironment,
  readLunaKey,
};
