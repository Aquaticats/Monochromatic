/**
 Claude Code multi-event hook for the claude-spawn plugin.
 
 Dispatches across SessionStart, Stop, SessionEnd, PreToolUse, PostToolUse,
 and PostToolUseFailure. SessionStart writes PID-to-session mapping, claims
 spawn ownership, and auto-symlinks the spawn-claude CLI. Stop updates child
 `lastMessage` (child sessions) or consumes completed children by blocking
 with reason text (parent sessions). All other consuming events deliver
 completed-child results via `additionalContext`.
 
 @module
 */

import type {
  HookInput,
  HookOutputBase,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type { ReadonlyDeep, } from 'type-fest';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { handleSessionStart, } from './hook-session-start.ts';
import {
  checkCompletedChildren,
  NOTHING_TO_REPORT,
} from './inject.ts';
import {
  SPAWNS_DIR,
  type SpawnState,
} from './paths.ts';

/**
 Captured at module load. After tsdown bundles the source package into the
 per-plugin entry, this resolves to `${pluginRoot}/bundle/node/`, which
 {@link handleSessionStart} walks up two levels to recover the plugin root.
 */
const HOOK_DIR = import.meta.dirname;

/**
 Discriminated output union preserving the legacy wire convention:
 SessionStart can emit a non-JSON warning string while every other branch
 emits a JSON payload. The writer renders each variant verbatim.
 */
type ClaudeSpawnOutput =
  | {
    kind: 'raw';
    text: string;
  }
    | {
      kind: 'json';
      payload: unknown;
    };

/**
 Updates a child's spawn state file when the Stop event reports a final
 assistant message. No-op if `CLAUDE_SPAWN_ID` is unset, the file is missing,
 or this session does not own the spawn record. The caller skips this when
 the Stop event carries no final message.
 
 @param sessionId - Claude Code session identifier of the child session
 
 @param lastMessage - text of the child's last assistant message
 */
async function updateChildOnStop(
  {
    sessionId,
    lastMessage,
  }: {
    readonly sessionId: string;
    readonly lastMessage: string;
  },
): Promise<void> {
  /**
   Spawn correlation id injected by the CLI when this Claude was a child; absent in normal runs.
   */
  const spawnId = process.env
    .CLAUDE_SPAWN_ID;
  if (spawnId === undefined)
    return;

  /**
   Path to the child's spawn-state JSON file under `SPAWNS_DIR`.
   */
  const filePath = join(
    SPAWNS_DIR,
    `${spawnId}.json`,
  );

  try {
    /**
     Current on-disk state text, parsed below to confirm ownership before rewriting.
     */
    const existing = await readFile(
      filePath,
      'utf8',
    );
    /* oxlint-disable typescript/no-unsafe-type-assertion -- trusted file written by our own CLI */
    /**
     Parsed spawn state; the cast is safe because only the spawn-claude CLI writes this file.
     */
    const state = JSON.parse(existing,) as SpawnState;
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    if (state.sessionId
      === sessionId) {
      /**
       New state with `status: 'stopped'` and the final assistant message recorded.
       */
      const updated: SpawnState = {
        ...state,
        lastMessage,
        status: 'stopped',
      };
      await writeFile(
        filePath,
        JSON.stringify(updated,),
      );
    }
  }
  catch (_error: unknown) {
    /**
     File missing (already `.reported`) or unreadable: skip.
     */
  }
}

/**
 Builds the appropriate hook response for a Stop event after any child-side
 file update has been performed.
 
 - On parent sessions with completed children and no in-flight stop hook,
   checks {@link checkCompletedChildren} and returns a `block` decision
   carrying the formatted child-result text.
 - Otherwise returns an empty pass-through.
 
 @param event - parsed Stop {@link HookInput} event payload
 
 @returns block decision with child results, or empty pass-through
 */
async function stopResponse(
  event: ReadonlyDeep<Extract<HookInput, { hook_event_name: 'Stop'; }>>,
): Promise<ClaudeSpawnOutput> {
  if (!event.stop_hook_active) {
    /**
     Formatted child-result text consumed atomically; {@link NOTHING_TO_REPORT} when nothing pending.
     */
    const context = await checkCompletedChildren({
      parentSessionId: event.session_id,
      consume: true,
    },);
    if (context !== NOTHING_TO_REPORT) {
      return {
        kind: 'json',
        payload: {
          decision: 'block',
          reason: context,
        },
      };
    }
  }
  /**
   Pass-through payload returned when no children have completed yet.
   */
  const empty: HookOutputBase = {};
  return {
    kind: 'json',
    payload: empty,
  };
}

/**
 Builds the additionalContext-bearing response for non-Stop, non-Session*
 delivery hooks (PreToolUse, PostToolUse, PostToolUseFailure, etc.), using
 {@link checkCompletedChildren} to look up pending results.
 
 @param event - any {@link HookInput} event other than SessionStart, Stop, SessionEnd
 
 @returns hook response carrying child-result text, or empty pass-through
 */
async function additionalContextResponse(event: ReadonlyDeep<HookInput>,): Promise<ClaudeSpawnOutput> {
  /**
   Formatted child-result text consumed atomically; {@link NOTHING_TO_REPORT} when no completion is pending.
   */
  const context = await checkCompletedChildren({
    parentSessionId: event.session_id,
    consume: true,
  },);
  if (context === NOTHING_TO_REPORT) {
    /**
     Pass-through payload returned when there is nothing to inject.
     */
    const empty: HookOutputBase = {};
    return {
      kind: 'json',
      payload: empty,
    };
  }
  return {
    kind: 'json',
    payload: {
      hookSpecificOutput: {
        hookEventName: event.hook_event_name,
        additionalContext: context,
      },
    },
  };
}

/**
 Dispatches on `hook_event_name` to one of four behaviors:
 SessionStart (raw text response), Stop (child update plus parent
 consume), SessionEnd (no-op), or a default consuming additionalContext
 delivery.
 
 @param event - parsed {@link HookInput} event from Claude Code
 
 @returns {@link ClaudeSpawnOutput} hook response for stdout
 
 @example
 ```ts
 claudeSpawnHandler({ hook_event_name: 'SessionEnd', session_id: 'abc', ... });
 ```
 */
async function claudeSpawnHandler(event: ReadonlyDeep<HookInput>,): Promise<ClaudeSpawnOutput> {
  if (event.hook_event_name
    === 'SessionStart') {
    /**
     Raw SessionStart warning text emitted directly to stdout.
     */
    const text = await handleSessionStart({
      sessionId: event.session_id,
      transcriptPath: event.transcript_path,
      hookDir: HOOK_DIR,
    },);
    return {
      kind: 'raw',
      text,
    };
  }
  if (event.hook_event_name
    === 'Stop') {
    if (event.last_assistant_message !== undefined) {
      await updateChildOnStop({
        sessionId: event.session_id,
        lastMessage: event.last_assistant_message,
      },);
    }
    return stopResponse(event,);
  }
  if (event.hook_event_name
    === 'SessionEnd') {
    /**
     SessionEnd is a no-op for this plugin; return the empty pass-through.
     */
    const empty: HookOutputBase = {};
    return {
      kind: 'json',
      payload: empty,
    };
  }
  return additionalContextResponse(event,);
}

/**
 Parses raw stdin as a {@link HookInput} (any union member; narrowed at dispatch).
 
 @param raw - JSON payload from Claude Code stdin
 
 @returns parsed hook event union
 
 @example
 ```ts
 const event = claudeSpawnParser(await text(process.stdin));
 ```
 */
function claudeSpawnParser(raw: string,): HookInput {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted JSON contract from Claude Code hook system
  return JSON.parse(raw,) as HookInput;
}

/**
 Renders the discriminated output to stdout. Raw variants are written
 verbatim; JSON variants are stringified without trailing newline,
 matching the legacy wire format.
 
 @param output - discriminated {@link ClaudeSpawnOutput} handler result
 
 @returns text payload to write to stdout
 
 @mutates output - `JSON.stringify` may invoke hooks on JSON payload values.
 
 @example
 ```ts
 process.stdout.write(claudeSpawnWriter({ kind: 'json', payload: {} }));
 ```
 */
function claudeSpawnWriter(output: ClaudeSpawnOutput,): string {
  if (output.kind
    === 'raw')
    return output.text;
  return JSON.stringify(output.payload,);
}

export type { ClaudeSpawnOutput, };

export {
  claudeSpawnHandler,
  claudeSpawnParser,
  claudeSpawnWriter,
};
