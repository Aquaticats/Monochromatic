#!/usr/bin/env node

/**
 CLI tool that spawns steerable child Claude Code sessions in terminal windows.
 
 Launches a child Claude session via `terminal-exec` with a pre-created spawn
 state file. Only `CLAUDE_SPAWN_ID` is passed as an env var; the child's
 SessionStart and Stop hooks use it to fill in session info and report
 completion.
 
 @module
 */

import { spawn, } from 'node:child_process';
import { randomUUID, } from 'node:crypto';
import {
  mkdir,
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
    'terminal-exec',
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

  proc.unref();

  console.log(JSON.stringify({ spawnId, },),);
}
