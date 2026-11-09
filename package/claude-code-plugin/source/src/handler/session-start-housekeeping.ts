import type {
  SessionStartInput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';
import {
  glob,
  lstat,
  mkdir,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  makeHookInputParser,
  NO_STDOUT,
  type Parser,
  type WriterOutput,
} from '../runtime/handler-runtime.ts';

/**
 Stale artifact names that git metadata or Claude config can leak into nested
 `dist/final/` and committed-plugin `bundle/` directories. Removed on session
 start to prevent build contamination.
 */
const STALE_DIST_ARTIFACTS = [
  'HEAD',
  'config',
  'hooks',
  'objects',
  'refs',
  '.claude',
];

/**
 Zero-byte Claude Code sandbox sentinels that can appear at the workspace root.
 These names overlap with Git's internal metadata, so they are cleaned only
 when they are exact root-level empty regular files.
 */
const ROOT_SENTINEL_ARTIFACTS = [
  'HEAD',
  'config',
  'hooks',
  'objects',
  'refs',
] as const;

/**

 Parent directory of the sandbox roots Claude Code creates on startup.
 */
const SANDBOX_PARENT_DIRECTORY = '/tmp' as const;

/**
 Name prefix of the per-user sandbox root. Claude Code lays its sandbox out as
 `/tmp/claude-<uid>/<workspace>/<session>/scratchpad`, so the trailing number is
 the session user's UID and differs on every host.
 */
const SANDBOX_ROOT_TEMPLATE = 'claude-' as const;

/**
 Sandbox root name that predates the per-user directory, kept because installs
 and tooling written against the older layout still expect it.
 */
const LEGACY_SANDBOX_ROOT = 'claude' as const;

/**
 Sentinel for a runtime that exposes no user identifier, as Windows does. There
 is then no `/tmp/claude-<uid>` convention to mirror, so only the legacy root is
 created.
 */
const NO_USER_IDENTIFIER: unique symbol = Symbol('session-start-housekeeping/runtime-exposes-no-user-identifier',);

/**
 Creates a directory if it does not already exist (equivalent to `mkdir -p`).
 
 @param dirPath - absolute path of directory to ensure exists
 
 @example
 ```ts
 await ensureDir('/tmp/claude');
 ```
 */
async function ensureDir(dirPath: string,): Promise<void> {
  await mkdir(
    dirPath,
    { recursive: true, },
  );
}

/**
 Resolves the sandbox root Claude Code uses for one user.
 
 @param uid - user id whose directory the sandbox uses
 
 @returns absolute path of that user's sandbox root
 
 @example
 ```ts
 sandboxRootDir(1000); // '/tmp/claude-1000'
 ```
 */
function sandboxRootDir(uid: number,): string {
  return join(
    SANDBOX_PARENT_DIRECTORY,
    `${SANDBOX_ROOT_TEMPLATE}${uid}`,
  );
}

/**
 Reads the user id this process runs as.
 
 @returns the current UID, or {@link NO_USER_IDENTIFIER} when the runtime
   exposes none
 
 @example
 ```ts
 currentUserId(); // 1000 on a host whose user is uid 1000
 ```
 */
function currentUserId(): number | typeof NO_USER_IDENTIFIER {
  if ((typeof process.getuid) !== 'function')
    return NO_USER_IDENTIFIER;
  return process.getuid();
}

/**
 Lists the sandbox roots the hook must create for one user.
 
 One derivation, so a host whose user is not the uid the old hardcoded name
 assumed still gets the directory its sandbox needs.
 
 @param uid - user id whose sandbox root is wanted
 
 @param sandboxParentDirectory - directory holding the roots
 
 @returns every sandbox root path to ensure, legacy name first
 
 @example
 ```ts
 sandboxRootPaths({ sandboxParentDirectory: '/tmp', uid: 1000 });
 // ['/tmp/claude', '/tmp/claude-1000']
 ```
 */
function sandboxRootPaths(
  {
    sandboxParentDirectory,
    uid,
  }: {
    readonly sandboxParentDirectory: string;
    readonly uid: number | typeof NO_USER_IDENTIFIER;
  },
): readonly string[] {
  /**
   Legacy root every install has always had.
   */
  const legacyRoot = join(
    sandboxParentDirectory,
    LEGACY_SANDBOX_ROOT,
  );
  if (uid === NO_USER_IDENTIFIER)
    return [legacyRoot,];
  return [
    legacyRoot,
    join(
      sandboxParentDirectory,
      `${SANDBOX_ROOT_TEMPLATE}${uid}`,
    ),
  ];
}

/**
 Removes stale git metadata and Claude config directories from all nested
 `dist/final/` paths and committed-plugin `bundle/` paths under `package/`.
 Uses `node:fs/promises` glob to find matching paths and removes them
 recursively. The `bundle/` family exists because Claude Code plugin hook
 commands execute from `bundle/node/` (see
 `doc/decision/gitignore-negations.md`), so leaks land there the same way
 they landed in `dist/final/` when hooks lived under `dist/`.
 
 @param workspaceRoot - absolute path to the monorepo root
 
 @example
 ```ts
 await cleanDistArtifacts('/var/home/user/Monochromatic');
 ```
 */
async function cleanDistArtifacts(workspaceRoot: string,): Promise<void> {
  /**
   Pending `rm` operations across every matched artifact path, awaited concurrently below.
   */
  const removals: Promise<void>[] = [];

  for await (const finalDir of glob(
    [
      'package/*/*/dist/final',
      'package/*/*/bundle',
    ],
    { cwd: workspaceRoot, },
  )) {
    for (const artifact of STALE_DIST_ARTIFACTS) {
      removals.push(
        rm(
          `${workspaceRoot}/${finalDir}/${artifact}`,
          {
            recursive: true,
            force: true,
          },
        ),
      );
    }
  }

  await Promise.all(removals,);
}

/**
 Returns true when an unknown caught error is an ENOENT filesystem miss.
 
 @param error - caught exception from a filesystem operation
 
 @returns whether the error has `code === 'ENOENT'`
 */
function isMissingPathError(error: unknown,): boolean {
  if ((typeof error) !== 'object')
    return false;

  if (error === null)
    return false;

  if (!('code' in error))
    return false;

  return error.code
    === 'ENOENT';
}

/**
 Removes a single root sentinel only when it is a zero-byte regular file.
 
 @param artifactPath - absolute path to one root sentinel candidate
 */
async function cleanRootSentinelArtifact(artifactPath: string,): Promise<void> {
  try {
    /**
     Candidate stat, read with `lstat` so symlinks are preserved.
     */
    const stats = await lstat(artifactPath,);

    if ((!stats.isFile()) || (stats.size
      > 0))
      return;

    await rm(
      artifactPath,
      { force: true, },
    );
  }
  catch (error: unknown) {
    if (isMissingPathError(error,))
      return;

    throw error;
  }
}

/**
 Removes root-level zero-byte Claude Code sandbox sentinel files.
 
 @param workspaceRoot - absolute path to the monorepo root
 
 @example
 ```ts
 await cleanRootSentinelArtifacts('/var/home/user/Monochromatic');
 ```
 */
async function cleanRootSentinelArtifacts(workspaceRoot: string,): Promise<void> {
  await Promise.all(
    ROOT_SENTINEL_ARTIFACTS.map(function cleanArtifact(artifact,): Promise<void> {
      return cleanRootSentinelArtifact(`${workspaceRoot}/${artifact}`,);
    },),
  );
}

/**
 Removes the ephemeral `.mcp.json` file from the workspace root. Regenerated
 each session and must not persist across sessions.
 
 @param workspaceRoot - absolute path to the monorepo root
 
 @example
 ```ts
 await removeMcpJson('/var/home/user/Monochromatic');
 ```
 */
async function removeMcpJson(workspaceRoot: string,): Promise<void> {
  await rm(
    `${workspaceRoot}/.mcp.json`,
    { force: true, },
  );
}

/**
 Output is `void`: the housekeeping handler performs filesystem side effects
 and writes nothing to stdout. The runtime invokes {@link sessionStartHousekeepingWriter},
 which returns {@link NO_STDOUT} to preserve the legacy hook's wire behavior.
 */
type SessionStartHousekeepingOutput = void;

/**
 Performs session-start housekeeping in parallel:
 
 - Ensures the legacy `/tmp/claude` root and the per-user `/tmp/claude-<uid>`
   root exist (see {@link sandboxRootPaths})
 - Removes stale git metadata leaked into nested `dist/final/` directories
 - Removes zero-byte Claude Code sandbox sentinels from the workspace root
 - Removes the ephemeral `.mcp.json` from the workspace root
 
 Side-effecting by design; returns nothing. The runtime treats `void` outputs
 as empty stdout via the corresponding writer.
 
 @param event - parsed {@link SessionStartInput} event from Claude Code
 
 @returns nothing; runtime emits empty stdout via the writer
 
 @example
 ```ts
 await sessionStartHousekeepingHandler({ cwd: '/repo', source: 'startup', ... });
 ```
 */
function sessionStartHousekeepingHandler(
  event: ReadonlyDeep<SessionStartInput>,
): Promise<SessionStartHousekeepingOutput> {
  return performHousekeeping({ event, },);
}

/**
 Performs the housekeeping steps for one event.
 
 Separate from {@link sessionStartHousekeepingHandler} because the runtime calls
 a handler with the event alone, while the user identity and the directory the
 sandbox roots live under are environment facts a test must be able to replace
 with a disposable directory and a fixed UID.
 
 @param event - parsed {@link SessionStartInput} event from Claude Code
 
 @param sandboxParentDirectory - directory the sandbox roots are created under
 
 @param uid - user id whose sandbox root is created
 
 @returns nothing; every step is a filesystem side effect
 
 @example
 ```ts
 await performHousekeeping({ event, sandboxParentDirectory: '/tmp', uid: 1000 });
 ```
 */
async function performHousekeeping(
  {
    event,
    sandboxParentDirectory = SANDBOX_PARENT_DIRECTORY,
    uid = currentUserId(),
  }: {
    readonly event: ReadonlyDeep<SessionStartInput>;
    readonly sandboxParentDirectory?: string;
    readonly uid?: number | typeof NO_USER_IDENTIFIER;
  },
): Promise<SessionStartHousekeepingOutput> {
  /**
   Monorepo root, derived from the SessionStart event's `cwd` field.
   */
  const workspaceRoot = event.cwd;
  /**
   Sandbox roots every session needs, legacy name first.
   */
  const sandboxRoots = sandboxRootPaths({
    sandboxParentDirectory,
    uid,
  },);

  await Promise.all([
    ...sandboxRoots.map(function ensureSandboxRoot(sandboxRoot,): Promise<void> {
      return ensureDir(sandboxRoot,);
    },),
    cleanDistArtifacts(workspaceRoot,),
    cleanRootSentinelArtifacts(workspaceRoot,),
    removeMcpJson(workspaceRoot,),
  ],);
}

/**
 Parses raw stdin as a {@link SessionStartInput}.
 
 Input is trusted; it comes from Claude Code's hook dispatch system.
 
 @example
 ```ts
 const event = sessionStartHousekeepingParser(await text(process.stdin));
 ```
 */
const sessionStartHousekeepingParser: Parser<SessionStartInput> = makeHookInputParser<SessionStartInput>();

/**
 Returns {@link NO_STDOUT}; the legacy hook produced no stdout, and the
 runtime shell treats the sentinel as intentional silence.
 
 @param _output - ignored {@link SessionStartHousekeepingOutput} handler result (housekeeping has no stdout)
 
 @returns sentinel instructing the runtime to emit no stdout bytes
 
 @example
 ```ts
 sessionStartHousekeepingWriter(); // NO_STDOUT
 ```
 */
function sessionStartHousekeepingWriter(
  _output: SessionStartHousekeepingOutput,
): WriterOutput {
  return NO_STDOUT;
}

export type { SessionStartHousekeepingOutput, };

export {
  cleanRootSentinelArtifacts,
  LEGACY_SANDBOX_ROOT,
  NO_USER_IDENTIFIER,
  performHousekeeping,
  ROOT_SENTINEL_ARTIFACTS,
  SANDBOX_PARENT_DIRECTORY,
  SANDBOX_ROOT_TEMPLATE,
  sandboxRootDir,
  sandboxRootPaths,
  sessionStartHousekeepingHandler,
  sessionStartHousekeepingParser,
  sessionStartHousekeepingWriter,
};
