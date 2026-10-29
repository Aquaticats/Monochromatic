/**
 * Stable auth token persistence across dev-mode restarts.
 *
 * During `mise watch` auto-restarts, the old server process dies and
 * a new one spawns within milliseconds. This module writes the auth
 * token to a file in `$TMPDIR` and keeps its mtime fresh (every 1s).
 * On startup, if the token file exists and its mtime is within
 * {@link FRESHNESS_THRESHOLD_MS} of now, the token is reused,
 * meaning the client's existing WebSocket URL stays valid across
 * the restart. Cold starts (stale or missing file) generate a fresh token.
 *
 * @example
 * ```ts
 * const { token, cleanup } = await resolveAuthToken({ port: 4400 });
 * // ... on shutdown:
 * cleanup();
 * ```
 */

import {
  readFile,
  stat,
  unlink,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  type Logger,
  tagged,
} from '../log.ts';

//region Constants

/**
 * Maximum age of the token file's mtime (in milliseconds) for it
 * to be considered "fresh", i.e. left by a process that was alive
 * very recently (auto-restart). Anything older is treated as stale.
 */
const FRESHNESS_THRESHOLD_MS = 3_000;

/**
 * Interval at which the running server touches the token file's mtime.
 */
const TOUCH_INTERVAL_MS = 1_000;

//endregion Constants

/**
 * Builds the token file path for a given port.
 *
 * @param port - HTTP listen port
 *
 * @returns absolute path to the token file
 */
function tokenFilePath({ port, }: { readonly port: number; },): string {
  return join(
    scratchRoot(),
    `editord-${String(port,)}.token`,
  );
}

/**
 * Checks whether the token file is fresh (mtime within threshold).
 *
 * @param path - absolute path to the token file
 *
 * @returns the token string if fresh, or null if stale/missing
 */
async function readFreshToken({ path, }: { readonly path: string; },): Promise<string | null> {
  try {
    /**
     * Stat record for the token file; throws (caught below) when the file does not exist.
     */
    const fileStat = await stat(path,);
    /**
     * Age of the token file in milliseconds; compared against {@link FRESHNESS_THRESHOLD_MS}.
     */
    const ageMs = Date.now()
      - fileStat
      .mtimeMs;
    if (ageMs > FRESHNESS_THRESHOLD_MS)
      return null;
    /**
     * Raw file contents; trimmed below to strip any trailing newline written by editors.
     */
    const content = await readFile(
      path,
      'utf8',
    );
    /**
     * Token string with surrounding whitespace removed; empty after trimming counts as no token.
     */
    const token = content.trim();
    if (token.length
      === 0)
      return null;
    return token;
  }
  catch {
    return null;
  }
}

/**
 * Re-writes the token file to keep its mtime fresh.
 * Errors are silently ignored; worst case, the next restart generates a fresh token.
 *
 * @param path - token file path
 *
 * @param token - auth token string
 */
async function touchFile({
  path,
  token,
}: {
  readonly path: string;
  readonly token: string;
},): Promise<void> {
  try {
    await writeFile(
      path,
      token,
      {
        encoding: 'utf8',
        mode: 0o600,
      },
    );
  }
  catch {
    // Touch failures are harmless.
  }
}

/**
 * Removes the token file on shutdown. Logs if already gone.
 *
 * @param path - token file path
 *
 * @param l - logger
 */
async function cleanupFile({
  path,
  l,
}: {
  readonly path: string;
  readonly l: Logger;
},): Promise<void> {
  try {
    await unlink(path,);
  }
  catch {
    l.info('token file already removed',);
  }
}

/**
 * Writes the token to disk and starts a periodic mtime touch.
 *
 * @param path - absolute path to write the token file
 *
 * @param token - auth token string to persist
 *
 * @returns `stopTouching` to stop the interval, and `deleteFile` to remove the token file
 */
async function writeAndTouch({
  path,
  token,
  l,
}: {
  readonly path: string;
  readonly token: string;
  readonly l: Logger;
},): Promise<{
  readonly stopTouching: () => void;
  readonly deleteFile: () => void;
}> {
  await writeFile(
    path,
    token,
    {
      encoding: 'utf8',
      mode: 0o600,
    },
  );

  /**
   * Timer handle for the periodic mtime-refresh; cancelled by `stopTouching`/`deleteFile`.
   */
  const interval = setInterval(
    function touchTokenFile() {
      void touchFile({
        path,
        token,
      },);
    },
    TOUCH_INTERVAL_MS,
  );

  return {
    stopTouching: function stopTouching(): void {
      clearInterval(interval,);
    },
    deleteFile: function deleteFile(): void {
      clearInterval(interval,);
      void cleanupFile({
        path,
        l,
      },);
    },
  };
}

/**
 * Resolves the auth token for this server instance.
 *
 * Precedence:
 * 1. `EDITORD_TOKEN` env var; deterministic token for external integrations
 * 2. Fresh token file (mtime within {@link FRESHNESS_THRESHOLD_MS}): reuse across dev-mode auto-restarts
 * 3. New random UUID; cold start fallback
 *
 * In all cases, starts a periodic mtime touch so the next restart
 * can detect that this process was recently alive.
 *
 * @param port - HTTP listen port (used to namespace the token file)
 *
 * @param l - parent logger
 *
 * @returns the auth token, `stopTouching` for SIGTERM, and `deleteFile` for SIGINT
 *
 * @example
 * ```ts
 * const { token, stopTouching, deleteFile } = await resolveAuthToken({ port: 3_400, l: logger, });
 * ```
 */
export async function resolveAuthToken({
  port,
  l,
}: {
  readonly port: number;
  readonly l: Logger;
},): Promise<{
  readonly token: string;
  readonly stopTouching: () => void;
  readonly deleteFile: () => void;
}> {
  /**
   * Logger scoped with the `token` tag so token lifecycle events are filterable.
   */
  const tokenLog = tagged({
    tag: 'token',
    l,
  },);
  /**
   * Absolute path to the token file for this port; isolates concurrent instances on the same machine.
   */
  const path = tokenFilePath({ port, },);

  /**
   * Override token from the environment; takes precedence over file reuse and fresh generation.
   */
  const envToken = process.env
    .EDITORD_TOKEN;
  if ((envToken !== undefined) && (envToken.length
    > 0)) {
    tokenLog.info('using token from EDITORD_TOKEN env var',);
    /**
     * Lifecycle handles returned by `writeAndTouch`; merged into the resolveAuthToken return value.
     */
    const handles = await writeAndTouch({
      path,
      token: envToken,
      l: tokenLog,
    },);
    return {
      token: envToken,
      ...handles,
    };
  }

  /**
   * Token recovered from a recently-touched file, or null when the file is stale or missing.
   */
  const existing = await readFreshToken({ path, },);

  if (existing !== null) {
    tokenLog.info('reusing token from previous instance (auto-restart detected)',);
    /**
     * Lifecycle handles for the reused token; same shape as the env-var branch.
     */
    const handles = await writeAndTouch({
      path,
      token: existing,
      l: tokenLog,
    },);
    return {
      token: existing,
      ...handles,
    };
  }

  /**
   * Newly generated UUID used as the auth token on cold start (no env override, no fresh file).
   */
  const token = crypto.randomUUID();
  tokenLog.info('generated fresh token',);
  /**
   * Lifecycle handles for the freshly generated token.
   */
  const handles = await writeAndTouch({
    path,
    token,
    l: tokenLog,
  },);
  return {
    token,
    ...handles,
  };
}
