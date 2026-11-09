#!/usr/bin/env node

/**
 CLI tool that spawns steerable child Claude Code sessions in terminal windows.
 
 Launches a child Claude session via `terminal-exec` with a pre-created spawn
 state file. Only `CLAUDE_SPAWN_ID` is passed as an env var; the child's
 SessionStart and Stop hooks use it to fill in session info and report
 completion.
 
 @module
 */

import {
  type ChildProcess,
  spawn,
} from 'node:child_process';
import { once, } from 'node:events';
import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import { message, } from '@optique/core/message';
// TODO: deprecate Optique
import { optional, } from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';

import {
  SPAWNS_DIR,
  type SpawnState,
} from '../handler/claude-spawn/paths.ts';
import {
  findCallingSession,
  SESSION_NOT_FOUND,
} from '../handler/claude-spawn/session-finder.ts';
import { splitWhitespace, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import type { ReadonlyDeep, } from 'type-fest';

/**
 TODO: deprecate Optique
 Optique parser for the spawn-claude CLI.
 
 @example
 ```
 spawn-claude "implement feature X"
 spawn-claude --cwd /some/path "fix the bug"
 spawn-claude --extra-arguments "--model sonnet" "refactor module Y"
 ```
 */
const parser = object({
  cwd: optional(option(
    '--cwd',
    string({ metavar: 'DIR', },),
    {
      description:
        message`Working directory for the child session. Defaults to current directory.`,
    },
  ),),
  extraArguments: optional(option(
    '--extra-arguments',
    string({ metavar: 'ARGS', },),
    {
      description:
        message`Additional CLI arguments passed directly to the claude command (e.g. "--model sonnet --allowedTools Edit,Bash").`,
    },
  ),),
  prompt: argument(
    string({ metavar: 'PROMPT', },),
    {
      description: message`Initial prompt for the spawned Claude session.`,
    },
  ),
},);

/**
 TODO: deprecate Optique
 Parsed CLI arguments from the spawn-claude command invocation.
 */
const args = runSync(
  parser,
  {
    programName: 'spawn-claude',
    help: 'option',
    brief:
      message`Spawn a steerable child Claude Code session in a visible terminal instance.`,
  },
);

/**
 Launcher binary the child session is started through.
 */
const LAUNCHER = 'terminal-exec' as const;

/**
 Sentinel the launch wait answers with when the launcher started.
 
 A unique symbol rather than a boolean because the failure arm carries an error:
 one value distinguishes "started" from "failed with this reason".
 */
const LAUNCHER_STARTED: unique symbol = Symbol('spawn-claude/launcher-started-the-child-process',);

/**
 Waits for the launcher to answer the spawn attempt.
 
 `spawn` reports a launch failure (a missing or non-executable binary, a sandbox
 denial) as an `'error'` event rather than a throw, and an unanswered `'error'`
 event kills the process with a raw stack trace. Waiting for either answer also
 means the state file this CLI pre-created is never left behind by a launcher
 that never ran.
 
 @param proc - child process the launcher was spawned as
 
 @returns {@link LAUNCHER_STARTED} once the launcher is running, otherwise the failure
 */
async function firstLauncherFailure(proc: ChildProcess,): Promise<ReadonlyDeep<Error> | typeof LAUNCHER_STARTED> {
  try {
    await once(
      proc,
      'spawn',
    );
    return LAUNCHER_STARTED;
  }
  catch (error: unknown) {
    return Error.isError(error,)
      ? error
      : new Error(String(error,),);
  }
}

/**
 Reports a launcher that never started and removes the state file written for it.
 
 The state file records status `running`, and nothing ever rewrites a file for a
 child that never launched, so leaving it behind means every hook event of every
 session re-reads it forever.
 
 @param spawnId - identifier of the spawn that failed
 
 @param failure - error the launcher reported
 */
async function discardFailedSpawn(
  {
    spawnId,
    failure,
  }: {
    readonly spawnId: string;
    readonly failure: ReadonlyDeep<Error>;
  },
): Promise<void> {
  process.stderr
    .write(`spawn-claude: could not start ${LAUNCHER}, so no child session was spawned; the spawn state file was removed: ${failure.message}\n`,);
  await rm(
    join(
      SPAWNS_DIR,
      `${spawnId}.json`,
    ),
    { force: true, },
  );
}

/**
 Resolved session identity of the calling Claude instance.
 */
const identity = await findCallingSession();

if (identity === SESSION_NOT_FOUND) {
  console.error(
    'Error: Could not find calling Claude session. Ensure the claude-spawn plugin hooks are active and SessionStart has fired.',
  );
  process.exitCode = 1;
}
else {
  /**
   Unique identifier for this spawn, used to coordinate state between parent and child.
   */
  const spawnId = randomUUID();
  /**
   Working directory for the child session, defaulting to the current directory.
   */
  const cwd = args.cwd
    ?? process
      .cwd();

  /**
   Extra args split on whitespace, filtering empty strings.
   */
  const extraArgs = splitWhitespace(args.extraArguments
    ?? '',);

  await mkdir(
    SPAWNS_DIR,
    { recursive: true, },
  );

  /**
   Pre-create the spawn state file before launching the child.
   
   `sessionId` starts empty; the first SessionStart that sees it empty
   claims ownership. Subsequent sessions with stale `CLAUDE_SPAWN_ID`
   env vars see a non-empty `sessionId` and skip registration.
   */
  const initialState: SpawnState = {
    spawnId,
    sessionId: '',
    transcriptPath: '',
    parentSessionId: identity.sessionId,
    status: 'running',
    lastMessage: '',
  };

  await writeFile(
    join(
      SPAWNS_DIR,
      `${spawnId}.json`,
    ),
    JSON.stringify(initialState,),
  );

  /**
   Detached child process running the spawned Claude session in a terminal instance.
   */
  const proc = spawn(
    LAUNCHER,
    [
      '--',
      'claude',
      ...extraArgs,
      args.prompt,
    ],
    {
      cwd,
      env: {
        ...process.env,
        CLAUDE_SPAWN_ID: spawnId,
      },
      detached: true,
      stdio: 'ignore',
    },
  );

  /**
   Whether the launcher started, or why it did not.
   */
  const launch = await firstLauncherFailure(proc,);

  if (launch === LAUNCHER_STARTED) {
    // A later launcher failure still reaches a listener instead of killing the
    // CLI with an unhandled 'error' event.
    proc.once(
      'error',
      function reportLateLauncherFailure(error: ReadonlyDeep<Error>,): void {
        void discardFailedSpawn({
          spawnId,
          failure: error,
        },);
      },
    );
    proc.unref();

    console.log(JSON.stringify({ spawnId, },),);
  }
  else {
    await discardFailedSpawn({
      spawnId,
      failure: launch,
    },);
    process.exitCode = 1;
  }
}
