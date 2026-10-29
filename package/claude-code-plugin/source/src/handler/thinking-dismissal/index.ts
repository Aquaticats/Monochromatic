/**
 Claude Code Stop hook that shadow-records size dismissals in thinking.

 Scanning is local and the record carries hashes and indexes only,
 so no reasoning text reaches the log or any external provider.
 The hook never blocks a stop.

 @module
 */

import type {
  StopInput,
  StopOutput,
} from '@monochromatic-dev/claude-code-plugin-hook-type/ts';
import type {
  ReadonlyDeep,
} from 'type-fest';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  findSizeDismissal,
  type ThinkingDismissalHit,
} from './detect.ts';
import {
  appendShadowRecord,
  shadowLogPath,
  type ThinkingDismissalRecord,
} from './shadow-record.ts';
import { readThinkingBlocks, } from './transcript-read.ts';

/**
 Module logger for thinking-dismissal shadow records.
 */
const logger = tagged({ tag: 'thinking-dismissal', },);

/**
 Environment reads the handler depends on,
 injectable so tests never touch the real state directory.
 */
type ThinkingDismissalDependencies = {
  /**
   Environment to read configuration from.
   */
  readonly env?: ReadonlyDeep<NodeJS.ProcessEnv>;
};

/**
 Output of the thinking-dismissal handler.

 The hook only records, so it never sets `decision` and never blocks a stop.
 */
type ThinkingDismissalOutput = StopOutput;

/**
 Builds the Stop hook handler that shadow-records size dismissals in thinking.

 Scanning is local and the record carries hashes and indexes only,
 so no reasoning text reaches the log or any external provider.
 The handler never throws and never blocks:
 a shadow record that cannot be written must not fail the stop.

 @param dependencies - Ambient inputs, defaulting to `process.env`

 @returns Handler for one `Stop` event

 @example
 ```ts
 const handler = createThinkingDismissalHandler();
 await handler({ hook_event_name: 'Stop', stop_hook_active: false, session_id: 's', transcript_path: '/t', cwd: '/c', permission_mode: 'default' });
 ```
 */
function createThinkingDismissalHandler(
  dependencies: Readonly<ThinkingDismissalDependencies> = {},
): (event: ReadonlyDeep<StopInput>,) => Promise<ThinkingDismissalOutput> {
  /**
   Resolved ambient inputs.
   */
  const {
    env = process.env,
  } = dependencies;
  /**
   Factory logger so every handler line names this hook.
   */
  const rl = tagged({
    tag: createThinkingDismissalHandler.name,
    l: logger,
  },);

  return async function thinkingDismissalHandler(event: ReadonlyDeep<StopInput>,): Promise<ThinkingDismissalOutput> {
    /**
     Handler-scoped logger.
     */
    const l = tagged({
      tag: thinkingDismissalHandler.name,
      l: rl,
    },);
    try {
      /**
       Tail scan of the session transcript.
       */
      const scan = await readThinkingBlocks(event.transcript_path,);
      /**
       Hits of the whole tail, in block order.
       */
      const hits: ThinkingDismissalHit[] = [];
      for (const block of scan.blocks) {
        hits.push(...findSizeDismissal(block,),);
      }
      /**
       Thinking blocks of the tail scan.
       */
      const { blocks, } = scan;
      /**
       Number of thinking blocks scanned.
       */
      const scannedBlocks = blocks.length;
      /**
       Number of hits found in the tail.
       */
      const foundHits = hits.length;
      /**
       Time the record is written.
       */
      const now = new Date()
        .toISOString();
      await appendShadowRecord({
        record: {
          sessionId: event.session_id,
          time: now,
          transcriptPath: event.transcript_path,
          thinkingBlocks: scannedBlocks,
          hits,
        },
        logPath: shadowLogPath(env,),
      },);
      l.info(`thinking-dismissal shadow recorded ${String(foundHits,)} hits of ${String(scannedBlocks,)} blocks`,);
    } catch (error) {
      l.warn(`thinking-dismissal shadow skipped: ${String(error,)}`,);
    }
    return {};
  };
}

/**
 Parses raw stdin as a {@link StopInput}.

 Input is trusted; it comes from Claude Code's hook dispatch system.

 @param raw - JSON-encoded event payload received on stdin

 @returns the parsed Stop event

 @example
 ```ts
 thinkingDismissalParser('{"hook_event_name":"Stop","stop_hook_active":false}');
 ```
 */
function thinkingDismissalParser(raw: string,): StopInput {
  /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- trusted input from Claude Code hook system */
  return JSON.parse(raw,) as StopInput;
}

/**
 Serializes the thinking-dismissal output for stdout.

 No trailing newline; matches Claude Code's wire convention.

 @param output - {@link ThinkingDismissalOutput} handler result to serialize

 @returns JSON-encoded payload for Claude Code's hook reader

 @mutates output - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.

 @example
 ```ts
 thinkingDismissalWriter({});
 // => '{}'
 ```
 */
function thinkingDismissalWriter(output: ThinkingDismissalOutput,): string {
  return JSON.stringify(output,);
}

/**
 Default handler instance, wired to `process.env`.
 */
const thinkingDismissalHandler: (event: ReadonlyDeep<StopInput>,) => Promise<ThinkingDismissalOutput> =
  createThinkingDismissalHandler();

export type {
  ThinkingDismissalDependencies,
  ThinkingDismissalOutput,
};

export {
  createThinkingDismissalHandler,
  thinkingDismissalHandler,
  thinkingDismissalParser,
  thinkingDismissalWriter,
};

export {
  findSizeDismissal,
} from './detect.ts';

export type {
  ThinkingDismissalHit,
} from './detect.ts';

export type {
  ThinkingDismissalRecord,
} from './shadow-record.ts';
