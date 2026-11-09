/**
 SessionStart hook handler for the claude-spawn plugin.
 
 Handles: PID-to-session mapping, child spawn ownership claims, and
 auto-symlinking of the `spawn-claude` CLI.
 
 @module
 */

import { constants, } from 'node:fs';
import {
  access,
  chmod,
  mkdir,
  readFile,
  symlink,
  unlink,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  BY_PID_DIR,
  type PidMapping,
  parseSpawnState,
  SPAWNS_DIR,
  type SpawnState,
} from './paths.ts';

/**
 Sentinel returned by {@link autoSetupCli} when CLI setup succeeded or was
 unnecessary, so there is no warning to surface.
 
 A unique symbol rather than `null`: the caller narrows on identity
 (`=== NO_WARNING`), keeping the warning text free of a nullish union.
 */
const NO_WARNING: unique symbol = Symbol('claude-spawn/cli-setup-ok',);

/**
 Module logger for this hook.
 */
const loggerForSessionStart = tagged({ tag: 'claude-spawn-session-start', },);

/**
 Handles the SessionStart hook event.
 
 Writes PID-to-session mapping, claims spawn ownership for genuine child
 sessions, and auto-symlinks the `spawn-claude` CLI into `~/.local/bin/`
 via {@link autoSetupCli}.
 
 @param sessionId - Claude Code session identifier from hook event
 
 @param transcriptPath - absolute path to session transcript file
 
 @param hookDir - value of `import.meta.dirname` from compiled hook entry point,
   used to resolve plugin root for CLI symlinking
 
 @returns string to write to stdout (empty JSON object or CLI setup warning)
 
 @example
 ```ts
 const output = handleSessionStart({
   sessionId: event.session_id,
   transcriptPath: event.transcript_path,
   hookDir: import.meta.dirname,
 });
 process.stdout.write(output);
 ```
 */
async function handleSessionStart({
  sessionId,
  transcriptPath,
  hookDir,
}: {
  readonly sessionId: string;
  readonly transcriptPath: string;
  readonly hookDir: string;
},): Promise<string> {
  await mkdir(
    BY_PID_DIR,
    { recursive: true, },
  );

  /**
   Maps this Claude process's PID to the session identity for CLI coordination.
   */
  const mapping: PidMapping = {
    sessionId,
    transcriptPath,
  };

  await writeFile(
    join(
      BY_PID_DIR,
      String(process.ppid,),
    ),
    JSON.stringify(mapping,),
  );

  /**
   Claim ownership of the spawn file if this is a genuine child session.
   
   The CLI pre-creates `{spawnId}.json` with `sessionId: ""`. The first
   SessionStart that sees an empty `sessionId` fills it in. Sessions with
   stale `CLAUDE_SPAWN_ID` env vars (inherited from the terminal after a
   previous child exited) see a non-empty `sessionId` and skip.
   */
  const spawnId = process.env
    .CLAUDE_SPAWN_ID;

  if (spawnId !== undefined) {
    /**
     Path to the spawn-state JSON pre-created by the CLI for this child.
     */
    const jsonPath = join(
      SPAWNS_DIR,
      `${spawnId}.json`,
    );

    try {
      /**
       Existing spawn-state text on disk; parsed below before deciding to claim.
       */
      const raw = await readFile(
        jsonPath,
        'utf8',
      );
      /**
       Parsed spawn state; an empty `sessionId` field signals an unclaimed slot.
       */
      const state = parseSpawnState(raw,);

      if (state.sessionId
        === '') {
        /**
         Genuine child: claim ownership by filling in session identity.
         */
        const updated: SpawnState = {
          ...state,
          sessionId,
          transcriptPath,
        };
        await writeFile(
          jsonPath,
          JSON.stringify(updated,),
        );
      }
    }
    catch (error: unknown) {
      // A missing record is ordinary (a stale `CLAUDE_SPAWN_ID`, or a file
      // another hook already consumed); anything else means the claim never
      // happened and is worth a log line at the default level.
      tagged({
        tag: handleSessionStart.name,
        l: loggerForSessionStart,
      },)
        .debug(`spawn state not claimed: ${String(error,)}`,);
    }
  }

  /**
   Warning text from CLI auto-setup, or {@link NO_WARNING} when setup succeeded or was unnecessary.
   */
  const cliWarning = await autoSetupCli(hookDir,);
  if (cliWarning !== NO_WARNING)
    return cliWarning;

  return JSON.stringify({},);
}

/**
 Error codes that answer "nothing here can be executed".

 `ENOENT` and `ENOTDIR` say no entry stands at the path, and `EACCES` says the
 entry exists but this process may not execute or search it. Both are answers
 about the path itself, so the caller may go on looking elsewhere.
 */
const NOT_EXECUTABLE_ERROR_CODES: ReadonlySet<string> = new Set([
  'EACCES',
  'ENOENT',
  'ENOTDIR',
],);

/**
 Tests whether a path points to an executable file.
 
 A failure that says nothing about the path itself, such as an I/O error,
 reaches the caller: claiming "not installed" from it would rewrite the user's
 `~/.local/bin/spawn-claude` symlink over an answer nobody established.
 
 @param path - candidate executable path
 
 @returns true when the process can execute the path
 
 @throws The probe's own failure when executability cannot be decided
 
 @example
 ```ts
 if (await isExecutablePath(join(dir, 'spawn-claude'))) skipSetup();
 ```
 */
export async function isExecutablePath(path: string,): Promise<boolean> {
  try {
    await access(
      path,
      constants.X_OK,
    );
    return true;
  }
  catch (error) {
    if ((Error.isError(error,))
      && ('code' in error)
      && ((typeof error.code) === 'string')
      && NOT_EXECUTABLE_ERROR_CODES.has(error.code))
      return false;
    throw error;
  }
}

/**
 Detects whether `spawn-claude` is already discoverable on PATH.
 
 @returns true when an executable `spawn-claude` exists in any PATH directory
 
 @example
 ```ts
 if (await cliIsOnPath()) skipAutoSetup();
 ```
 */
async function cliIsOnPath(): Promise<boolean> {
  /**
   PATH entries searched for the spawn-claude executable.
   */
  const pathDirs = (process.env
    .PATH
    ?? '').split(':',);
  /**
   Executable checks for every PATH entry.
   */
  const checks = await Promise.all(
    pathDirs.map(function checkDir(dir,): Promise<boolean> {
      return isExecutablePath(join(
        dir,
        'spawn-claude',
      ),);
    },),
  );
  return checks.includes(true,);
}

/**
 Symlinks the CLI source into `~/.local/bin/spawn-claude` and returns a
 human-readable warning when the result is incomplete (PATH missing or
 symlink failure).
 
 @param hookDir - directory of the compiled hook entry point, used to derive plugin root
 
 @returns warning text to print to stdout, or {@link NO_WARNING} when setup either succeeded or was unnecessary
 
 @example
 ```ts
 const warning = autoSetupCli(import.meta.dirname);
 if (warning !== NO_WARNING) console.warn(warning);
 ```
 */
async function autoSetupCli(hookDir: string,): Promise<string | typeof NO_WARNING> {
  if (await cliIsOnPath())
    return NO_WARNING;

  /**
   Resolve plugin root from the compiled hook's location.
   Hook binary: `${PLUGIN_ROOT}/bundle/node/index.mjs`
   CLI source:  `${PLUGIN_ROOT}/src/cli.ts`
   */
  const pluginRoot = resolve(
    hookDir,
    '..',
    '..',
  );
  /**
   Absolute path to CLI entry point that the symlink will target.
   */
  const cliSource = join(
    pluginRoot,
    'src',
    'cli.ts',
  );

  /**
   Standard XDG user-local bin directory.
   */
  const localBin = join(
    process.env
      .HOME
      ?? '/tmp',
    '.local',
    'bin',
  );
  /**
   Destination path for the `spawn-claude` symlink in user's local bin.
   */
  const symlinkPath = join(
    localBin,
    'spawn-claude',
  );

  try {
    await mkdir(
      localBin,
      { recursive: true, },
    );

    /**
     Unix permission bits for owner rwx, group/others rx.
     */
    const EXECUTABLE_PERMISSION = 0o755;
    /**
     Ensure CLI source is executable (shebang: #!/usr/bin/env node).
     */
    await chmod(
      cliSource,
      EXECUTABLE_PERMISSION,
    );

    /**
     Remove stale symlink if it exists, then create a fresh one.
     */
    try {
      await unlink(symlinkPath,);
    }
    catch (error: unknown) {
      // Nothing to remove is the ordinary case; anything else means the stale
      // link stays and the symlink below fails, so the caught value is logged.
      tagged({
        tag: autoSetupCli.name,
        l: loggerForSessionStart,
      },)
        .debug(`no stale spawn-claude symlink to remove: ${String(error,)}`,);
    }
    await symlink(
      cliSource,
      symlinkPath,
    );

    /**
     Verify ~/.local/bin is on PATH so the symlink is discoverable.
     */
    const pathDirs = (process.env
      .PATH
      ?? '').split(':',);
    return pathDirs.includes(localBin,)
      ? NO_WARNING
      : [
        '[claude-spawn] Symlinked spawn-claude to ~/.local/bin/spawn-claude,',
        'but ~/.local/bin is not on PATH. Add it to your shell profile:',
        '  export PATH="$HOME/.local/bin:$PATH"',
      ]
        .join('\n',);
  }
  catch (error: unknown) {
    return [
      '[claude-spawn] Could not auto-setup spawn-claude CLI.',
      `Symlink target: ${cliSource}`,
      `Symlink path: ${symlinkPath}`,
      `Setup error: ${String(error,)}`,
      'Create the symlink manually or add the plugin directory to PATH.',
    ]
      .join('\n',);
  }
}

export { handleSessionStart, };
